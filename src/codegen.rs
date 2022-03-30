use std::{mem, process, fs};
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::values::{FunctionValue, IntValue, PointerValue};
use inkwell::types::{IntType, StringRadix};
use inkwell::basic_block::BasicBlock;
use inkwell::{AddressSpace, OptimizationLevel, IntPredicate};
use inkwell::targets::{CodeModel, RelocMode, InitializationConfig, Target, TargetMachine, FileType};
use crate::parser::{
    OrganismExpression, ExprType, RegionExpression, Region,
    LabelExpression, JumpExpression, LeachExpression, PrimitiveExpression, PrimitiveValue,
    CellExpression, DummyExpression
};
use crate::errors;

struct Functions<'ctx> {
    putchar: FunctionValue<'ctx>,
    primitive_access_routine: FunctionValue<'ctx>,
    state_update_routine: FunctionValue<'ctx>,
    drill_gate_routine: FunctionValue<'ctx>,
    getchar: FunctionValue<'ctx>,
    func_validation_routine: FunctionValue<'ctx>,
    expr_life_validation_routine: FunctionValue<'ctx>,
    cell_access_routine: FunctionValue<'ctx>
}

struct DataLandscape<'ctx> {
    curr_gates_state_ptr: PointerValue<'ctx>,
    gates_ttso_ptr: PointerValue<'ctx>,
    curr_region_ptr: PointerValue<'ctx>,
    ttl_table_ptr: PointerValue<'ctx>,
    cells_ptr: PointerValue<'ctx>
}

pub struct CodeGen<'ctx> {
    org_expr: Box<OrganismExpression>,
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    labels: Vec<String>
}

/// Accepts a PrimitiveValue and returns its Primitive Index as specified
/// in the Mindbend grammar.
/// Considers only function primitives because they are only used in relation to that
macro_rules! pindex {
    ($val:expr) => {
        if $val == PrimitiveValue::Subtraction {
            0
        } else if $val == PrimitiveValue::Addition {
            1
        } else if $val == PrimitiveValue::Output {
            2
        } else {
            // For PrimitiveValue::Input
            3
        }
    };
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(org_expr: OrganismExpression, labels: Vec<String>) -> CodeGen<'ctx> {
        unsafe {
            let context = Box::new(Context::create());
            let context = Box::into_raw(context) as *const Context;
            CodeGen {
                org_expr: Box::new(org_expr),
                context: &*context,
                module: (*context).create_module("main"),
                builder: (*context).create_builder(),
                labels
            }
        }
    }
    pub fn code(&mut self) -> Result<(), String> {
        let main_fn = self.init_main_fn();
        let main_block = self.context.append_basic_block(main_fn, "main");
        self.init_user_defined_blocks(&main_fn);
        let data_landscape = self.init_data_landscape(&main_fn);
        let functions = self.init_functions();
        self.builder.position_at_end(main_block);
        // Makes the right org expr of the head org expr to become
        // the new head org expr and returns the old head
        // If the right expr is none, then it replaces self's org_expr with a garbage org_expr
        // This works because this fake org_expr will never be processed
        // The if condition at the bottom of the loop will break out when it
        // encounters the none on the right expr which the fake org_expr will have
        let consume_org_expr = || unsafe {
            let s = self as *const Self as *mut Self;
            if self.org_expr.right.is_some(){
                let right = mem::replace(&mut (*s).org_expr.right, None);
                let org_expr: Box<OrganismExpression> = mem::replace(&mut (*s).org_expr, right.unwrap());
                org_expr
            } else {
                let fake_org_expr_child = Box::new(DummyExpression::new());
                let fake_org_expr = Box::new(OrganismExpression {
                    child: fake_org_expr_child,
                    right: None
                });
                mem::replace(&mut (*s).org_expr, fake_org_expr)
            }
        };
        loop {
            match self.org_expr.child.get_type(){
                ExprType::Leach => {
                    let org_expr = consume_org_expr();
                    self.code_leach_expr(org_expr, &main_fn, &data_landscape, &functions);
                }
                // A lone cell expression with no effect
                // If it was part of any leach expressions,
                // it would have been in a leach expression
                ExprType::Cell => {
                    consume_org_expr();
                    self.code_lone_cell_expression(&main_fn, &functions, &data_landscape);
                }
                // A lone primitive, just like the cell arm above
                // But the PAR still needs to be carried out
                ExprType::Primitive => {
                    consume_org_expr();
                    self.code_lone_primitive_expression(&main_fn, &functions, &data_landscape);
                }
                ExprType::Jump => {
                    let org_expr = consume_org_expr();
                    self.code_jmp(org_expr, &main_fn, &functions, &data_landscape);
                }
                ExprType::Drill => {
                    consume_org_expr();
                    self.code_drill_expr(&main_fn, &data_landscape, &functions)
                }
                ExprType::Region => {
                    let org_expr: Box<OrganismExpression> = consume_org_expr();
                    self.code_region_expr(org_expr, &data_landscape, &functions);
                }
                ExprType::Label => {
                    let org_expr = consume_org_expr();
                    self.code_label(org_expr, &main_fn);
                }
                ExprType::Dummy => break,
                x => {
                    println!("Supposed to be unreachable: {:?}", x);
                    unreachable!();
                }
            }
        }
        self.code_end_main(&main_fn);
        Ok(())
    }
    fn init_data_landscape(&self, main_fn: &FunctionValue) -> DataLandscape {
        let main_entry_block = self.get_basic_block(main_fn, "entry");
        let main_block = self.get_basic_block(main_fn, "main");
        self.builder.position_at_end(main_entry_block);
        let curr_gates_state_ptr = self.builder.build_alloca(self.context.i8_type(), "curr_gates_state");
        let gates_ttso_ptr = self.builder.build_alloca(self.context.i8_type(), "gates_ttso");
        let curr_region_ptr = self.builder.build_alloca(self.context.i8_type(), "curr_region");
        let cells_ptr = self.builder.build_array_alloca(
            self.context.i32_type(),
            self.context.i32_type().const_int(15, false),
            "cells"
        );
        let ttl_table_ptr = self.builder.build_array_alloca(
            self.context.i8_type(),
            self.context.i8_type().const_int(15, false),
            "TTL_table"
        );
        // 0 for all closed, 1 for 1 open, 2 for 2 open and 3 for 3 open
        self.builder.build_store(curr_gates_state_ptr, self.context.i8_type().int(0));
        self.builder.build_store(gates_ttso_ptr, self.context.i8_type().int(0));
        // 0 for Cells Region; 1 for Layers
        self.builder.build_store(curr_region_ptr, self.context.i8_type().int(0));
        for i in 0..15 {
            let cell_location = unsafe {
                self.builder.build_in_bounds_gep(
                    cells_ptr, &[self.context.i32_type().const_int(i, false)], "cell_location"
                )
            };
            self.builder.build_store(cell_location, self.context.i32_type().int(-1));
        }
        for i in 0..15 {
            let ttl_table_cell_location = unsafe {
                self.builder.build_in_bounds_gep(
                    ttl_table_ptr, &[self.context.i8_type().const_int(i, false)], "ttl_table_cell_location"
                )
            };
            self.builder.build_store(ttl_table_cell_location, self.context.i8_type().int(0));
        }
        self.builder.build_unconditional_branch(main_block);
        DataLandscape {
            curr_gates_state_ptr,
            gates_ttso_ptr,
            curr_region_ptr,
            ttl_table_ptr,
            cells_ptr
        }
    }
    fn init_main_fn(&self) -> FunctionValue {
        let main_fn_type = self.context.i32_type().fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_fn_type, None);
        self.context.append_basic_block(main_fn, "entry");
        self.context.append_basic_block(main_fn, main_fn_block_names::END_IN_FAIL);
        main_fn
    }
    fn init_functions(&self) -> Functions {
        let four_bytes = self.context.i32_type();
        let void = self.context.void_type();
        let putchar_type = void.fn_type(&[four_bytes.into()], false);
        let putchar = self.module.add_function("putchar", putchar_type, None);
        let getchar_type = four_bytes.fn_type(&[], false);
        let getchar = self.module.add_function("getchar", getchar_type, None);
        let primitive_access_routine = self.code_primitive_access_routine(&putchar);
        let state_update_routine = self.code_state_update_routine();
        let drill_gate_routine = self.code_drill_gate_routine(&putchar);
        let func_validation_routine = self.code_function_validation_routine(&putchar);
        let expr_life_validation_routine = self.code_expression_life_validation_routine();
        let cell_access_routine = self.code_cell_access_routine();
        Functions {
            putchar,
            getchar,
            primitive_access_routine,
            state_update_routine,
            drill_gate_routine,
            func_validation_routine,
            expr_life_validation_routine,
            cell_access_routine
        }
    }
    fn init_user_defined_blocks(&self, main_fn: &FunctionValue){
        for label in self.labels.iter(){
            let label_name = format!("{}{}", USER_DEFINED_LABEL_PREFIX, label.as_str());
            self.context.append_basic_block(*main_fn, label_name.as_str());
        }
    }
    fn code_end_main(&self, main_fn: &FunctionValue){
        let four_bytes = self.context.i32_type();
        let end_main_success = self.context.append_basic_block(*main_fn, "end_main_success");
        self.builder.build_unconditional_branch(end_main_success);
        self.builder.position_at_end(end_main_success);
        self.builder.build_return(Some(&four_bytes.int(0)));
        let end_in_fail = self.get_basic_block(main_fn, main_fn_block_names::END_IN_FAIL);
        self.builder.position_at_end(end_in_fail);
        self.builder.build_return(Some(&four_bytes.int(1)));
    }
    fn code_primitive_access_routine(&self, putchar: &FunctionValue) -> FunctionValue {
        let print_err_and_exit = ||{
            println!("Something went wrong while coding the primitive access routine");
            println!("I'm not sorry at all. Be a real man and don't whine");
            process::exit(1);
        };
        let byte = self.context.i8_type();
        let byte_ptr = byte.ptr_type(AddressSpace::Generic);
        let four_bytes = self.context.i32_type();
        // The CR pointer is the 1st parameter
        // The CGS pointer is the 2nd
        let primitive_access_routine_type = four_bytes.fn_type(&[byte_ptr.into(), byte_ptr.into()], false);
        let primitive_access_routine = self.module.add_function(
            "primitive_access_routine",
            primitive_access_routine_type,
            None
        );
        let entry_block = self.context.append_basic_block(primitive_access_routine, "entry_block");
        let curr_region_is_layers_block = self.context.append_basic_block(
            primitive_access_routine, "curr_region_is_layers_block"
        );
        let access_failed_region_not_layers_block = self.context.append_basic_block(
            primitive_access_routine, "access_failed_region_not_layers_block"
        );
        let access_failed_gates_not_open_block = self.context.append_basic_block(
            primitive_access_routine, "access_failed_gates_not_open_block"
        );
        let access_succeeded_block = self.context.append_basic_block(
            primitive_access_routine, "access_succeeded_block"
        );
        self.builder.position_at_end(entry_block);
        let curr_region_ptr = primitive_access_routine.get_first_param();
        let curr_gate_state_ptr = primitive_access_routine.get_last_param();
        if curr_region_ptr.is_none() || curr_gate_state_ptr.is_none(){
            print_err_and_exit();
        }
        let curr_region_ptr = curr_region_ptr.unwrap().into_pointer_value();
        let curr_gate_state_ptr = curr_gate_state_ptr.unwrap().into_pointer_value();
        let curr_region = self.builder.build_load(curr_region_ptr, "load_curr_region");
        let curr_gate_state = self.builder.build_load(curr_gate_state_ptr, "load_gate_state");
        let curr_region_is_layers = self.builder.build_int_compare(
            IntPredicate::EQ,
            curr_region.into_int_value(),
            byte.const_int(1, false),
            "current_region_is_layers"
        );
        self.builder.build_conditional_branch(
            curr_region_is_layers,
            curr_region_is_layers_block,
            access_failed_region_not_layers_block,
        );
        self.builder.position_at_end(curr_region_is_layers_block);
        let gates_are_open = self.builder.build_int_compare(
            IntPredicate::EQ,
            curr_gate_state.into_int_value(),
            byte.const_int(3, false),
            "gates_are_open"
        );
        self.builder.build_conditional_branch(
            gates_are_open,
            access_succeeded_block,
            access_failed_gates_not_open_block
        );
        self.builder.position_at_end(access_succeeded_block);
        self.builder.build_return(Some(&four_bytes.const_int(0, false)));
        self.builder.position_at_end(access_failed_region_not_layers_block);
        let region_not_layers_err_msg = errors::err_invalid_primitive_access_region_not_layers_runtime();
        self.code_print(&*putchar, region_not_layers_err_msg);
        self.builder.build_return(Some(&four_bytes.const_int(1, false)));
        self.builder.position_at_end(access_failed_gates_not_open_block);
        let gates_not_open_err_msg = errors::err_invalid_primitive_access_gates_not_open_runtime();
        self.code_print(&*putchar, gates_not_open_err_msg);
        self.builder.build_return(Some(&four_bytes.const_int(1, false)));
        primitive_access_routine
    }
    fn code_state_update_routine(&self) -> FunctionValue {
        let print_err_and_exit = ||{
            println!("Something went wrong while coding the state update routine");
            println!("Im' not sorry. Be a real man and don't whine about it");
            process::exit(1);
        };
        let byte = self.context.i8_type();
        let byte_ptr = byte.ptr_type(AddressSpace::Generic);
        let four_bytes = self.context.i32_type();
        let four_byte_ptr = four_bytes.ptr_type(AddressSpace::Generic);
        let void = self.context.void_type();
        // TTSO is the first arg
        // CGS is the second
        // TTL table is the third
        // reduce TTSO arg is the fourth
        // Cells Region is the fifth
        let state_update_routine_type = void.fn_type(&[
            byte_ptr.into(),
            byte_ptr.into(),
            byte_ptr.into(),
            byte.into(),
            four_byte_ptr.into()
        ], false);
        let state_update_routine = self.module.add_function(
            "state_update_routine",
            state_update_routine_type,
            None
        );
        let gate_time_to_stay_open_ptr = state_update_routine.get_first_param();
        let curr_gate_state_ptr = state_update_routine.get_nth_param(1);
        let ttl_table_ptr = state_update_routine.get_nth_param(2);
        let reduce_ttso = state_update_routine.get_nth_param(3);
        let cells_ptr = state_update_routine.get_last_param();
        if gate_time_to_stay_open_ptr.is_none() || curr_gate_state_ptr.is_none()
            || ttl_table_ptr.is_none() || reduce_ttso.is_none() || cells_ptr.is_none() {
            print_err_and_exit();
        }
        let curr_gate_state_ptr = curr_gate_state_ptr.unwrap();
        let ttl_table_ptr = ttl_table_ptr.unwrap();
        let gate_time_to_stay_open_ptr = gate_time_to_stay_open_ptr.unwrap();
        let reduce_ttso = reduce_ttso.unwrap();
        let cells_ptr = cells_ptr.unwrap().into_pointer_value();
        let entry_block = self.context.append_basic_block(state_update_routine, "entry_block");
        let update_ttl_table_block = self.context.append_basic_block(
            state_update_routine,
            "update_ttl_table_block"
        );
        let reduce_ttso_block = self.context.append_basic_block(state_update_routine, "reduce_ttso_block");
        let decrement_ttso_block = self.context.append_basic_block(state_update_routine, "decrement_ttso_block");
        let close_gates_block = self.context.append_basic_block(state_update_routine, "close_gates_block");
        self.builder.position_at_end(entry_block);
        let update_ttso = self.builder.build_int_compare(
            IntPredicate::NE,
            reduce_ttso.into_int_value(),
            byte.int(0),
            "update_ttso"
        );
        self.builder.build_conditional_branch(
            update_ttso,
            reduce_ttso_block,
            update_ttl_table_block
        );
        self.builder.position_at_end(reduce_ttso_block);
        let ttso_val = self.builder.build_load(gate_time_to_stay_open_ptr.into_pointer_value(), "ttso_value");
        let ttso_is_0 = self.builder.build_int_compare(
            IntPredicate::EQ,
            ttso_val.into_int_value(),
            byte.int(0), "ttso_is_0"
        );
        self.builder.build_conditional_branch(
            ttso_is_0,
            update_ttl_table_block,
            decrement_ttso_block
        );
        self.builder.position_at_end(decrement_ttso_block);
        let new_ttso_val = self.builder.build_int_sub(
            ttso_val.into_int_value(),
            byte.int(1),
            "new_ttso_value"
        );
        self.builder.build_store(gate_time_to_stay_open_ptr.into_pointer_value(), new_ttso_val);
        let new_ttso_val_is_0 = self.builder.build_int_compare(
            IntPredicate::EQ,
            new_ttso_val,
            byte.int(0),
            "new_ttso_val_is_0"
        );
        self.builder.build_conditional_branch(
            new_ttso_val_is_0,
            close_gates_block,
            update_ttl_table_block
        );
        self.builder.position_at_end(close_gates_block);
        self.builder.build_store(curr_gate_state_ptr.into_pointer_value(), byte.int(0));
        self.builder.build_unconditional_branch(update_ttl_table_block);
        self.builder.position_at_end(update_ttl_table_block);
        let mut next_update_ttl_cell_block: BasicBlock<'_>;
        for i in 0..15 {
            let ttl_cell_ptr = unsafe {
                self.builder.build_in_bounds_gep(
                    ttl_table_ptr.into_pointer_value(),
                    &[byte.int(i)],
                    "ttl_cell_ptr"
                )
            };
            let ttl_cell_val = self.builder.build_load(ttl_cell_ptr, "ttl_cell_val")
                .into_int_value();
            let ttl_cell_val_is_0 = self.builder.build_int_compare(
                IntPredicate::EQ, byte.int(0), ttl_cell_val, "ttl_cell_val_is_0"
            );
            next_update_ttl_cell_block = self.context.append_basic_block(
                state_update_routine,
                "next_update_ttl_cell_block"
            );
            let reduce_ttl_val_block = self.context.append_basic_block(
                state_update_routine,
                "reduce_ttl_val_block"
            );
            self.builder.build_conditional_branch(
                ttl_cell_val_is_0,
                next_update_ttl_cell_block,
                reduce_ttl_val_block
            );
            self.builder.position_at_end(reduce_ttl_val_block);
            let new_ttl_cell_val = self.builder.build_int_sub(ttl_cell_val, byte.int(1), "new_ttl_cell_val");
            self.builder.build_store(ttl_cell_ptr, new_ttl_cell_val);
            let new_ttl_cell_val_is_0 = self.builder.build_int_compare(
                IntPredicate::EQ,
                new_ttl_cell_val,
                byte.int(0),
                "new_ttl_cell_val_is_0"
            );
            let clear_cells_cell_block = self.context.append_basic_block(state_update_routine, "clear_cells_cell_block");
            self.builder.build_conditional_branch(
                new_ttl_cell_val_is_0,
                clear_cells_cell_block,
                next_update_ttl_cell_block
            );
            self.builder.position_at_end(clear_cells_cell_block);
            let cells_cell_ptr = unsafe {
                self.builder.build_in_bounds_gep(
                    cells_ptr,
                    &[four_bytes.int(i)],
                    "cells_cell_ptr"
                )
            };
            self.builder.build_store(cells_cell_ptr, four_bytes.int(-1));
            self.builder.build_unconditional_branch(next_update_ttl_cell_block);
            self.builder.position_at_end(next_update_ttl_cell_block);
        }
        self.builder.build_return(None);
        state_update_routine
    }
    fn code_drill_gate_routine(&self, putchar: &FunctionValue) -> FunctionValue {
        let print_err_and_exit = ||{
            println!("Something went wrong while coding the drill gate routine");
            println!("I'm not sorry. Be a real man and don't whine about it");
            process::exit(1);
        };
        let byte = self.context.i8_type();
        let byte_ptr = byte.ptr_type(AddressSpace::Generic);
        let four_bytes = self.context.i32_type();
        // The CR is the first param
        // The CGS is the second
        // The TTSO is the third
        let drill_gate_routine_type = four_bytes.fn_type(&[
            byte_ptr.into(),
            byte_ptr.into(),
            byte_ptr.into()
        ], false);
        let drill_gate_routine = self.module.add_function(
            "drill_gate_routine",
            drill_gate_routine_type,
            None
        );
        let entry_block = self.context.append_basic_block(drill_gate_routine, "entry");
        let update_cgs_block = self.context.append_basic_block(drill_gate_routine, "update_cgs_block");
        let open_a_gate_block = self.context.append_basic_block(drill_gate_routine, "open_a_gate_block");
        let update_ttso_block = self.context.append_basic_block(drill_gate_routine, "update_ttso_block");
        let curr_region_is_not_layers_err_block = self.context.append_basic_block(
            drill_gate_routine, "curr_region_is_not_layers_err_block"
        );
        let end_block = self.context.append_basic_block(drill_gate_routine, "end_block");
        self.builder.position_at_end(entry_block);
        let curr_region_ptr = drill_gate_routine.get_first_param();
        let curr_gate_state_ptr = drill_gate_routine.get_nth_param(1);
        let ttso_ptr = drill_gate_routine.get_last_param();
        if curr_gate_state_ptr.is_none() || curr_region_ptr.is_none() || ttso_ptr.is_none(){
            print_err_and_exit();
        }
        let curr_region_ptr = curr_region_ptr.unwrap().into_pointer_value();
        let curr_gate_state_ptr = curr_gate_state_ptr.unwrap().into_pointer_value();
        let ttso_ptr = ttso_ptr.unwrap().into_pointer_value();
        let curr_region = self.builder.build_load(curr_region_ptr, "curr_region");
        let curr_region_is_layers = self.builder.build_int_compare(
            IntPredicate::EQ,
            curr_region.into_int_value(),
            byte.int(1),
            "curr_region_is_layers"
        );
        self.builder.build_conditional_branch(
            curr_region_is_layers,
            update_cgs_block,
            curr_region_is_not_layers_err_block
        );
        self.builder.position_at_end(update_cgs_block);
        let curr_gate_state = self.builder.build_load(curr_gate_state_ptr, "curr_gate_state");
        let all_gates_are_open = self.builder.build_int_compare(
            IntPredicate::EQ,
            curr_gate_state.into_int_value(),
            byte.int(3),
            "all_gates_are_open"
        );
        self.builder.build_conditional_branch(
            all_gates_are_open,
            end_block,
            open_a_gate_block
        );
        self.builder.position_at_end(open_a_gate_block);
        let new_curr_gate_state = self.builder.build_int_add(
            curr_gate_state.into_int_value(),
            byte.int(1),
            "new_curr_gate_state"
        );
        self.builder.build_store(curr_gate_state_ptr, new_curr_gate_state);
        let new_curr_gate_state_is_three = self.builder.build_int_compare(
            IntPredicate::EQ,
            new_curr_gate_state,
            byte.int(3),
            "new_curr_gate_state_is_three"
        );
        self.builder.build_conditional_branch(
            new_curr_gate_state_is_three,
            update_ttso_block,
            end_block
        );
        self.builder.position_at_end(update_ttso_block);
        self.builder.build_store(ttso_ptr, byte.int(5));
        self.builder.build_unconditional_branch(end_block);
        self.builder.position_at_end(curr_region_is_not_layers_err_block);
        let region_not_layers_err_msg = errors::err_invalid_gate_access_region_not_layers_runtime();
        self.code_print(putchar, region_not_layers_err_msg);
        self.builder.build_return(Some(&four_bytes.int(1)));
        self.builder.position_at_end(end_block);
        self.builder.build_return(Some(&four_bytes.int(0)));
        drill_gate_routine
    }
    fn code_expression_life_validation_routine(&self) -> FunctionValue {
        let print_err_and_exit = ||{
            println!("Something went wrong while coding the Expression Life Validatin routine");
            println!("I'm not sorry. Be a real man and don't whine about it");
            process::exit(1);
        };
        let byte = self.context.i8_type();
        let byte_ptr = byte.ptr_type(AddressSpace::Generic);
        let four_bytes = self.context.i32_type();
        // First args is TTL table pointer
        // Second arg is Cell number
        let expr_life_validation_routine_type = four_bytes.fn_type(&[
            byte_ptr.into(),
            byte.into()
        ], false);
        let expr_life_validation_routine = self.module.add_function(
            "expression_life_validation_routine",
            expr_life_validation_routine_type,
            None
        );
        let ttl_ptr = expr_life_validation_routine.get_first_param();
        let cell_num = expr_life_validation_routine.get_last_param();
        if ttl_ptr.is_none() || cell_num.is_none(){
            print_err_and_exit();
        }
        let ttl_ptr = ttl_ptr.unwrap().into_pointer_value();
        let cell_num = cell_num.unwrap().into_int_value();
        let entry_block = self.context.append_basic_block(expr_life_validation_routine, "entry_block");
        let expression_life_validation_success = self.context.append_basic_block(
            expr_life_validation_routine,
            "expression_life_validation_success"
        );
        let expression_life_validation_fail = self.context.append_basic_block(
            expr_life_validation_routine,
            "expression_life_validation_fail"
        );
        self.builder.position_at_end(entry_block);
        let ttl_cell_ptr = unsafe {
            self.builder.build_in_bounds_gep(
                ttl_ptr,
                &[cell_num],
                "ttl_cell_ptr"
            )
        };
        let ttl_cell_value = self.builder.build_load(ttl_cell_ptr, "ttl_cell_value");
        let expression_is_dead = self.builder.build_int_compare(
            IntPredicate::EQ,
            ttl_cell_value.into_int_value(),
            byte.int(0),
            "expression_is_dead"
        );
        self.builder.build_conditional_branch(
            expression_is_dead,
            expression_life_validation_fail,
            expression_life_validation_success
        );
        self.builder.position_at_end(expression_life_validation_fail);
        self.builder.build_return(Some(&four_bytes.int(1)));
        self.builder.position_at_end(expression_life_validation_success);
        self.builder.build_return(Some(&four_bytes.int(0)));
        expr_life_validation_routine
    }
    fn code_function_validation_routine(&self, putchar: &FunctionValue) -> FunctionValue {
        let print_err_and_exit = ||{
            println!("Something went wrong while coding the Function Validation Routine");
            println!("I'm not sorry. Be a real man and don't whine about it");
            process::exit(1);
        };
        let byte = self.context.i8_type();
        let byte_ptr = byte.ptr_type(AddressSpace::Generic);
        let four_bytes = self.context.i32_type();
        let four_bytes_ptr = four_bytes.ptr_type(AddressSpace::Generic);
        // The first arg is the Cell number
        // The second arg is a pointer to the Cells region
        // The third is the TTL table pointer
        let func_validation_routine_type = four_bytes.fn_type(&[
            byte.into(),
            four_bytes_ptr.into(),
            byte_ptr.into()
        ], false);
        let func_validation_routine = self.module.add_function(
            "func_validation_routine",
            func_validation_routine_type,
            None
        );
        let cell_number = func_validation_routine.get_first_param();
        let cells_ptr = func_validation_routine.get_nth_param(1);
        let ttl_table_ptr = func_validation_routine.get_last_param();
        if cell_number.is_none() || cells_ptr.is_none() || ttl_table_ptr.is_none() {
            print_err_and_exit();
        }
        let cell_number = cell_number.unwrap().into_int_value();
        let cells_ptr = cells_ptr.unwrap().into_pointer_value();
        let ttl_table_ptr = ttl_table_ptr.unwrap().into_pointer_value();
        let entry_block = self.context.append_basic_block(func_validation_routine, "entry_block");
        let validate_primitive_index_block = self.context.append_basic_block(
            func_validation_routine,
            "validate_primitive_index_block"
        );
        let func_validation_fail_dead_expression = self.context.append_basic_block(
            func_validation_routine,
            "function_validation_fail_dead_expression"
        );
        let func_validation_fail_invalid_primitive_index_block = self.context.append_basic_block(
            func_validation_routine,
            "function_validation_fail_invalid_primitive_index_block"
        );
        let func_validation_success_block = self.context.append_basic_block(
            func_validation_routine,
            "function_validation_success"
        );
        self.builder.position_at_end(entry_block);
        let ttl_cell_ptr = unsafe {
            self.builder.build_in_bounds_gep(
                ttl_table_ptr,
                &[cell_number.into()],
                "ttl_cell_ptr"
            )
        };
        let ttl_cell_value = self.builder.build_load(ttl_cell_ptr, "ttl_cell_value");
        let expression_is_dead = self.builder.build_int_compare(
            IntPredicate::EQ,
            ttl_cell_value.into_int_value(),
            byte.int(0),
            "expression_is_dead"
        );
        self.builder.build_conditional_branch(
            expression_is_dead,
            func_validation_fail_dead_expression,
            validate_primitive_index_block
        );
        self.builder.position_at_end(func_validation_fail_dead_expression);
        self.builder.build_return(Some(&four_bytes.int(1)));
        self.builder.position_at_end(validate_primitive_index_block);
        let cell_ptr = unsafe {
            self.builder.build_in_bounds_gep(
                cells_ptr,
                &[cell_number.into()],
                "cell_ptr"
            )
        };
        let cell_value = self.builder.build_load(cell_ptr, "cell_value").into_int_value();
        self.builder.build_switch(
            cell_value,
            func_validation_fail_invalid_primitive_index_block,
            &[
                (four_bytes.int(pindex!(PrimitiveValue::Input)), func_validation_success_block),
                (four_bytes.int(pindex!(PrimitiveValue::Output)), func_validation_success_block),
                (four_bytes.int(pindex!(PrimitiveValue::Addition)), func_validation_success_block),
                (four_bytes.int(pindex!(PrimitiveValue::Subtraction)), func_validation_success_block),
            ]
        );
        self.builder.position_at_end(func_validation_fail_invalid_primitive_index_block);
        let err_msg = errors::err_attempt_to_use_non_function_primitive_to_massacre();
        self.code_print(putchar, err_msg);
        self.builder.build_return(Some(&four_bytes.int(1)));
        self.builder.position_at_end(func_validation_success_block);
        self.builder.build_return(Some(&four_bytes.int(0)));
        func_validation_routine
    }
    fn code_cell_access_routine(&self) -> FunctionValue {
        let print_err_and_exit = ||{
            println!("Something went wrong while coding the Cell Access Routine");
            println!("I'm not sorry. Be a real man and don't whine about it");
            process::exit(1);
        };
        let byte = self.context.i8_type();
        let byte_ptr = byte.ptr_type(AddressSpace::Generic);
        let four_bytes = self.context.i32_type();
        // The arg is the CR
        let cell_access_routine_type = four_bytes.fn_type(&[byte_ptr.into()], false);
        let cell_access_routine = self.module.add_function(
            "cell_access_routine",
            cell_access_routine_type,
            None
        );
        let curr_region_ptr = cell_access_routine.get_first_param();
        if curr_region_ptr.is_none(){
            print_err_and_exit();
        }
        let curr_region_ptr = curr_region_ptr.unwrap().into_pointer_value();
        let entry_block = self.context.append_basic_block(cell_access_routine, "entry_block");
        let success_block = self.context.append_basic_block(cell_access_routine, "success_block");
        let fail_block = self.context.append_basic_block(cell_access_routine, "fail_block");
        self.builder.position_at_end(entry_block);
        let curr_region_value = self.builder.build_load(curr_region_ptr, "curr_region_value");
        let curr_region_is_cells = self.builder.build_int_compare(
            IntPredicate::EQ,
            curr_region_value.into_int_value(),
            byte.int(0),
            "curr_region_is_cells"
        );
        self.builder.build_conditional_branch(
            curr_region_is_cells,
            success_block,
            fail_block
        );
        self.builder.position_at_end(success_block);
        self.builder.build_return(Some(&four_bytes.int(0)));
        self.builder.position_at_end(fail_block);
        self.builder.build_return(Some(&four_bytes.int(1)));
        cell_access_routine
    }
    fn code_leach_expr(
        &self,
        org_expr: Box<OrganismExpression>,
        main_fn: &FunctionValue,
        data_landscape: &DataLandscape,
        functions: &Functions
    ){
        let leach_expr = org_expr.child.as_any().downcast_ref::<LeachExpression>().unwrap();
        if leach_expr.left().get_type() == ExprType::Primitive {
            let left_expr = leach_expr.left().as_any().downcast_ref::<PrimitiveExpression>().unwrap();
            let right_expr = leach_expr.right().as_ref().unwrap().left();
            let right_expr = right_expr.as_any().downcast_ref::<CellExpression>().unwrap();
            let region_changes = leach_expr.region_change.clone();
            self.code_store_primitive(
                left_expr.pval(),
                right_expr.ident(),
                region_changes,
                &*main_fn,
                &*data_landscape,
                &*functions
            );
        } else {
            // Then it is a cell
            if leach_expr.is_chain {
                // Then it's a function call
                // Get the args for the function
                // However the args will be interpreted depends on the kind of function it is
                let mut args: Vec<&CellExpression> = vec![];
                let fake_expr = CellExpression {ident: 255};
                let leach_expr_ptr = leach_expr as *const LeachExpression as *mut LeachExpression;
                loop {
                    unsafe {
                        let right = {
                            let right = mem::replace(&mut (*leach_expr_ptr).right, None);
                            if right.is_none(){
                                let fake_leach_expr = LeachExpression::new(
                                    Box::new(fake_expr.clone()), None, false, None
                                );
                                Box::new(fake_leach_expr)
                            } else {
                                right.unwrap()
                            }
                        };
                        let mut old_leach_expr = mem::replace(&mut *leach_expr_ptr, *right);
                        let left = mem::replace(&mut old_leach_expr.left, Box::new(fake_expr.clone()));
                        let left_ptr = Box::into_raw(left);
                        let left = (*left_ptr).as_any().downcast_ref::<CellExpression>().unwrap();
                        if *left == fake_expr {
                            break;
                        }
                        args.push(left);
                    }
                }
                // The first cell isn't supposed to be an arg
                let mut args = args.into_iter();
                let cell_holding_primitive_func = args.next().unwrap().ident();
                let args: Vec<&CellExpression> = args.collect();
                self.code_function_call(
                    cell_holding_primitive_func,
                    args,
                    &*main_fn,
                    &*data_landscape,
                    &*functions
                );
            } else {
                // Then it's a copy operation
                // The cell must either be in the TTL table
                // So check if it's there first
                let left_expr = leach_expr.left().as_any().downcast_ref::<CellExpression>().unwrap();
                let right_expr = leach_expr.right().as_ref().unwrap().left();
                let right_expr = right_expr.as_any().downcast_ref::<CellExpression>().unwrap();
                self.code_cell_copy(
                    left_expr.ident(),
                    right_expr.ident(),
                    &*main_fn,
                    &*data_landscape,
                    &*functions
                );
            }
        }
    }
    fn code_function_call(
        &self,
        pf_cell_ident: u8,
        args: Vec<&CellExpression>,
        main_fn: &FunctionValue,
        data_landscape: &DataLandscape,
        functions: &Functions
    ){
        let print_err_and_exit = ||{
            println!("Something went wrong while coding a function call");
            println!("I'm not sorry. Be a real man and don't whine about it");
            process::exit(1);
        };
        let byte = self.context.i8_type();
        let four_bytes = self.context.i32_type();
        // Verify that the function is actually a function
        let func_validation_routine_result = self.builder.build_call(
            functions.func_validation_routine,
            &[
                byte.int(pf_cell_ident as i64).into(),
                data_landscape.cells_ptr.into(),
                data_landscape.ttl_table_ptr.into()
            ],
            "carry_out_function_validation_routine"
        ).try_as_basic_value().left();
        if func_validation_routine_result.is_none(){
            print_err_and_exit();
        }
        let func_validation_routine_result = func_validation_routine_result.unwrap().into_int_value();
        let func_is_valid = self.builder.build_int_compare(
            IntPredicate::EQ,
            func_validation_routine_result,
            four_bytes.int(0),
            "func_is_valid"
        );
        let func_validation_successful_block = self.context.append_basic_block(
            *main_fn,
            "func_validation_routine_block"
        );
        let func_validation_failed_block = self.context.append_basic_block(
            *main_fn,
            "func_validation_failed_block"
        );
        let end_in_fail_block = self.get_basic_block(main_fn, main_fn_block_names::END_IN_FAIL);
        self.builder.build_conditional_branch(
            func_is_valid,
            func_validation_successful_block,
            func_validation_failed_block
        );
        self.builder.position_at_end(func_validation_failed_block);
        let err_msg = errors::err_attempt_to_use_non_function_primitive_to_massacre();
        self.code_print(&functions.putchar, err_msg);
        self.builder.build_unconditional_branch(end_in_fail_block);
        self.builder.position_at_end(func_validation_successful_block);
        let cell_ptr = unsafe {
            self.builder.build_in_bounds_gep(
                data_landscape.cells_ptr,
                &[four_bytes.int(pf_cell_ident as i64)],
                "cell_ptr"
            )
        };
        let primitive_index = self.builder.build_load(cell_ptr, "func_primitive_index");
        // Block to branch to after the execution of a function
        // The main line of code building continues from here
        let continue_main_block = self.context.append_basic_block(*main_fn, "continue_main");
        let addition_block = self.context.append_basic_block(*main_fn, "addition_block");
        let subtraction_block = self.context.append_basic_block(*main_fn, "subtraction_block");
        let input_block = self.context.append_basic_block(*main_fn, "input_block");
        let output_block = self.context.append_basic_block(*main_fn, "output_block");
        self.builder.build_switch(
            primitive_index.into_int_value(),
            output_block,
            &[
                (byte.int(pindex!(PrimitiveValue::Addition)), addition_block),
                (byte.int(pindex!(PrimitiveValue::Subtraction)), subtraction_block),
                (byte.int(pindex!(PrimitiveValue::Input)), input_block),
                (byte.int(pindex!(PrimitiveValue::Output)), output_block)
            ]
        );
        // The value always gets stored in the last cell
        let target_cell_ident = args[args.len() - 1].ident();
        let first_arg = args[0];
        self.builder.position_at_end(addition_block);
        let first_arg_cell_ptr = unsafe {
            self.builder.build_in_bounds_gep(
                data_landscape.cells_ptr,
                &[four_bytes.int(first_arg.ident() as i64)],
                "first_arg_cell_ptr"
            )
        };
        let mut addition_result = self.builder.build_load(first_arg_cell_ptr, "first_arg_value")
            .into_int_value();
        for arg in args.iter() {
            let arg_cell_ptr = unsafe {
                self.builder.build_in_bounds_gep(
                    data_landscape.cells_ptr,
                    &[four_bytes.int(arg.ident() as i64)],
                    "arg_cell_ptr"
                )
            };
            let arg_value = self.builder.build_load(arg_cell_ptr, "arg_value")
                .into_int_value();
            addition_result = self.builder.build_int_add(
                addition_result,
                arg_value,
                "add_another_arg"
            );
        }
        // Store the result in the last cell
        let last_arg_cell_ptr = unsafe {
            self.builder.build_in_bounds_gep(
                data_landscape.cells_ptr,
                &[four_bytes.int(target_cell_ident as i64)],
                "last_arg_cell_ptr"
            )
        };
        self.builder.build_store(last_arg_cell_ptr, addition_result);
        self.code_post_func_exec_routine(pf_cell_ident, &args, &data_landscape, &functions);
        self.builder.build_unconditional_branch(continue_main_block);
        self.builder.position_at_end(subtraction_block);
        let first_arg_cell_ptr = unsafe {
            self.builder.build_in_bounds_gep(
                data_landscape.cells_ptr,
                &[four_bytes.int(first_arg.ident() as i64)],
                "first_arg_cell_ptr"
            )
        };
        let mut subtraction_result = self.builder.build_load(first_arg_cell_ptr, "first_arg_value")
            .into_int_value();
        for arg in args.iter() {
            let arg_cell_ptr = unsafe {
                self.builder.build_in_bounds_gep(
                    data_landscape.cells_ptr,
                    &[four_bytes.int(arg.ident() as i64)],
                    "arg_cell_ptr"
                )
            };
            let arg_value = self.builder.build_load(arg_cell_ptr, "arg_value")
                .into_int_value();
            subtraction_result = self.builder.build_int_sub(
                subtraction_result,
                arg_value,
                "add_another_arg"
            );
        }
        self.builder.build_store(last_arg_cell_ptr, subtraction_result);
        self.code_post_func_exec_routine(pf_cell_ident, &args, &data_landscape, &functions);
        self.builder.build_unconditional_branch(continue_main_block);
        self.builder.position_at_end(output_block);
        for i in (0..args.len()).step_by(2) {
            let num_value: IntValue;
            if i + 1 == args.len(){
                let num_cell_offset = args[i].ident();
                let num_cell_ptr = unsafe {
                    self.builder.build_in_bounds_gep(
                        data_landscape.cells_ptr,
                        &[four_bytes.int(num_cell_offset as i64)],
                        "num_cell_ptr"
                    )
                };
                num_value = self.builder.build_load(num_cell_ptr, "num_value")
                    .into_int_value();
            } else {
                let first_digit_cell_offset = args[i].ident();
                let second_digit_cell_offset = args[i + 1].ident();
                let first_digit_cell_ptr = unsafe {
                    self.builder.build_in_bounds_gep(
                        data_landscape.cells_ptr,
                        &[four_bytes.int(first_digit_cell_offset as i64)],
                        "first_digit_cell_ptr"
                    )
                };
                let second_digit_cell_ptr = unsafe {
                    self.builder.build_in_bounds_gep(
                        data_landscape.cells_ptr,
                        &[four_bytes.int(second_digit_cell_offset as i64)],
                        "second_digit_cell_ptr"
                    )
                };
                let first_digit_value = self.builder.build_load(first_digit_cell_ptr, "first_digit")
                    .into_int_value();
                let second_digit_value = self.builder.build_load(second_digit_cell_ptr, "second_digit")
                    .into_int_value();
                let first_digit_tens_value = self.builder.build_int_mul(
                    first_digit_value,
                    four_bytes.int(10),
                    "first_digit_tens_value"
                );
                let final_ascii_value = self.builder.build_int_add(
                    first_digit_tens_value,
                    second_digit_value,
                    "final_ascii_value"
                );
                num_value = final_ascii_value;
            }
            self.builder.build_call(
                functions.putchar,
                &[num_value.into()],
                "print_value"
            );
        }
        self.code_post_func_exec_routine(pf_cell_ident, &args, &data_landscape, &functions);
        self.builder.build_unconditional_branch(continue_main_block);
        self.builder.position_at_end(input_block);
        // The input primitive can have only one argument, that is the
        // cell location where the input should be stored
        let target_cell_ptr = unsafe {
            self.builder.build_in_bounds_gep(
                data_landscape.cells_ptr,
                &[four_bytes.int(first_arg.ident() as i64)],
                "first_arg_cell_ptr"
            )
        };
        let input = self.builder.build_call(functions.getchar, &[], "input")
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();
        self.builder.build_store(target_cell_ptr, input);
        self.code_post_func_exec_routine(pf_cell_ident, &args, &data_landscape, &functions);
        self.builder.build_unconditional_branch(continue_main_block);
        // Other code that comes after this should be in the continue_main_block
        self.builder.position_at_end(continue_main_block);
    }
    fn code_cell_copy(
        &self,
        left_cell_ident: u8,
        right_cell_ident: u8,
        main_fn: &FunctionValue,
        data_landscape: &DataLandscape,
        functions: &Functions
    ){
        let print_err_and_exit = ||{
            println!("Something went wrong while coding a leach expression");
            println!("I'm not sorry. Be a real man and don't whine about it");
            process::exit(1);
        };
        let byte = self.context.i8_type();
        let four_bytes = self.context.i32_type();
        let expr_life_validation_result = self.builder.build_call(
            functions.expr_life_validation_routine,
            &[
                data_landscape.ttl_table_ptr.into(),
                byte.int(left_cell_ident as i64).into()
            ],
            "expr_life_validation_result"
        ).try_as_basic_value().left();
        if expr_life_validation_result.is_none(){
            print_err_and_exit();
        }
        let expr_life_validation_result = expr_life_validation_result.unwrap().into_int_value();
        let expr_is_alive = self.builder.build_int_compare(
            IntPredicate::EQ,
            expr_life_validation_result,
            four_bytes.int(0),
            "expr_is_alive"
        );
        let expr_life_validation_successful = self.context.append_basic_block(
            *main_fn,
            "expr_life_validation_successful"
        );
        let expr_life_validation_failed = self.context.append_basic_block(
            *main_fn,
            "expr_life_validation_failed"
        );
        let end_in_fail = self.get_basic_block(&main_fn, main_fn_block_names::END_IN_FAIL);
        self.builder.build_conditional_branch(
            expr_is_alive,
            expr_life_validation_successful,
            expr_life_validation_failed
        );
        let continue_main_block = self.context.append_basic_block(*main_fn, "continue_main_block");
        self.builder.position_at_end(expr_life_validation_failed);
        let err_msg = errors::err_attempt_to_leach_death_expression_onto_another_cell();
        self.code_print(&functions.putchar, err_msg);
        self.builder.build_unconditional_branch(end_in_fail);
        self.builder.position_at_end(expr_life_validation_successful);
        
        let src_cell_addr_offset = left_cell_ident;
        let src_cell_ptr = unsafe {
            self.builder.build_in_bounds_gep(
                data_landscape.cells_ptr,
                &[four_bytes.int(src_cell_addr_offset as i64)],
                "src_cell_ptr"
            )
        };
        let src_cell_value = self.builder.build_load(src_cell_ptr, "src_cell_value")
            .into_int_value();
        let dest_cell_addr_offset = right_cell_ident;
        let dest_cell_ptr = unsafe {
            self.builder.build_in_bounds_gep(
                data_landscape.cells_ptr,
                &[four_bytes.int(dest_cell_addr_offset as i64)],
                "dest_cell_ptr"
            )
        };
        self.builder.build_store(dest_cell_ptr, src_cell_value);
        self.builder.build_call(
            functions.state_update_routine,
            &[
                data_landscape.gates_ttso_ptr.into(),
                data_landscape.curr_gates_state_ptr.into(),
                data_landscape.ttl_table_ptr.into(),
                byte.int(1).into(),
                data_landscape.cells_ptr.into()
            ], "carry_out_state_update_routine");
        let dest_ttl_cell_offset = right_cell_ident;
        let dest_ttl_cell_ptr = unsafe {
            self.builder.build_in_bounds_gep(
                data_landscape.ttl_table_ptr,
                &[byte.int(dest_ttl_cell_offset as i64)],
                "dest_ttl_cell_ptr"
            )
        };
        let src_ttl_cell_ptr = unsafe {
            self.builder.build_in_bounds_gep(
                data_landscape.ttl_table_ptr,
                &[byte.int(src_cell_addr_offset as i64)],
                "src_ttl_cell_ptr"
            )
        };
        self.builder.build_store(dest_ttl_cell_ptr, byte.int(5));
        // Kill the source expression
        self.builder.build_store(src_ttl_cell_ptr, byte.int(0));
        self.builder.build_unconditional_branch(continue_main_block);
        self.builder.position_at_end(continue_main_block);
    }
    fn code_store_primitive(
        &self,
        pval: PrimitiveValue,
        target_cell_ident: u8,
        region_changes: Option<Vec<RegionExpression>>,
        main_fn: &FunctionValue,
        data_landscape: &DataLandscape,
        functions: &Functions
    ){
        let print_err_and_exit = ||{
            println!("Something went wrong while coding a store primitive operation");
            println!("I'm not sorry. Be a real man and don't whine about it");
            process::exit(1);
        };
        let byte = self.context.i8_type();
        let four_bytes = self.context.i32_type();
        let primitive_access_routine_result = self.builder.build_call(
            functions.primitive_access_routine,
            &[
                data_landscape.curr_region_ptr.into(),
                data_landscape.curr_gates_state_ptr.into()
            ],
            "primitive_access_routine_result"
        ).try_as_basic_value().left();
        if primitive_access_routine_result.is_none(){
            print_err_and_exit();
        }
        let primitive_access_routine_result = primitive_access_routine_result.unwrap();
        let primitive_access_routine_successful_block = self.context.append_basic_block(
            *main_fn,
            "primitive_access_routine_successful_block"
        );
        let primitive_access_routine_failed_block = self.context.append_basic_block(
            *main_fn,
            "primitive_access_routine_failed_block"
        );
        let end_in_fail_block = self.get_basic_block(main_fn, main_fn_block_names::END_IN_FAIL);
        let continue_main_block = self.context.append_basic_block(*main_fn, "continue_main_block");
        let primitive_access_is_successful = self.builder.build_int_compare(
            IntPredicate::EQ,
            primitive_access_routine_result.into_int_value(),
            four_bytes.int(0),
            "primitive_access_is_successful"
        );
        self.builder.build_conditional_branch(
            primitive_access_is_successful,
            primitive_access_routine_successful_block,
            primitive_access_routine_failed_block
        );
        self.builder.position_at_end(primitive_access_routine_failed_block);
        let err_msg = errors::err_invalid_primitive_access_runtime();
        self.code_print(main_fn, err_msg);
        self.builder.build_unconditional_branch(end_in_fail_block);
        self.builder.position_at_end(primitive_access_routine_successful_block);
        let value_to_store_in_target_cell: i32;
        match pval {
            // Functions
            PrimitiveValue::Input | PrimitiveValue::Output |
            PrimitiveValue::Addition | PrimitiveValue::Subtraction => {
                let primitive_index_no = pindex!(pval);
                value_to_store_in_target_cell = primitive_index_no;
            }
            // Integers and other values that can be stored
            other => {
                match other {
                    PrimitiveValue::Zero => value_to_store_in_target_cell = 0,
                    PrimitiveValue::One => value_to_store_in_target_cell = 1,
                    PrimitiveValue::Two => value_to_store_in_target_cell = 2,
                    PrimitiveValue::Three => value_to_store_in_target_cell = 3,
                    PrimitiveValue::Four => value_to_store_in_target_cell = 4,
                    PrimitiveValue::Five => value_to_store_in_target_cell = 5,
                    PrimitiveValue::Six => value_to_store_in_target_cell = 6,
                    PrimitiveValue::Seven => value_to_store_in_target_cell = 7,
                    PrimitiveValue::Eight => value_to_store_in_target_cell = 8,
                    PrimitiveValue::Nine => value_to_store_in_target_cell = 9,
                    _ => unreachable!()
                };
            }
        }
        if region_changes.is_some(){
            for region_expr in region_changes.unwrap(){
                let organismed_region_expr = Box::new(OrganismExpression::new(Box::new(region_expr), None));
                self.code_region_expr(organismed_region_expr, &*data_landscape, &*functions);
            }
        }
        let cell_access_routine_result = self.builder.build_call(
            functions.cell_access_routine,
            &[data_landscape.curr_region_ptr.into()],
            "carry_out_cell_access_routine"
        ).try_as_basic_value().left();
        if cell_access_routine_result.is_none(){
            print_err_and_exit();
        }
        let cell_access_routine_result = cell_access_routine_result.unwrap();
        let cell_access_routine_successful = self.builder.build_int_compare(
            IntPredicate::EQ,
            cell_access_routine_result.into_int_value(),
            four_bytes.int(0),
            "cell_access_routine_successful"
        );
        let cell_access_routine_successful_block = self.context.append_basic_block(
            *main_fn,
            "cell_access_routine_successful_block"
        );
        let cell_access_routine_failed_block = self.context.append_basic_block(
            *main_fn,
            "cell_access_routine_failed_block"
        );
        self.builder.build_conditional_branch(
            cell_access_routine_successful,
            cell_access_routine_successful_block,
            cell_access_routine_failed_block
        );
        self.builder.position_at_end(cell_access_routine_failed_block);
        let err_msg = errors::err_invalid_cell_access_region_runtime();
        self.code_print(&functions.putchar, err_msg);
        self.builder.build_unconditional_branch(end_in_fail_block);
        self.builder.position_at_end(cell_access_routine_successful_block);
        let target_cell_ptr = unsafe {
            self.builder.build_in_bounds_gep(
                data_landscape.cells_ptr,
                &[four_bytes.int(target_cell_ident as i64)],
                "target_cell_ptr"
            )
        };
        self.builder.build_store(target_cell_ptr, four_bytes.int(value_to_store_in_target_cell as i64));
        self.builder.build_call(
            functions.state_update_routine,
            &[
                data_landscape.gates_ttso_ptr.into(),
                data_landscape.curr_gates_state_ptr.into(),
                data_landscape.ttl_table_ptr.into(),
                byte.int(1).into(),
                data_landscape.cells_ptr.into()
            ],
            "carry_out_state_update_routine"
        );
        let ttl_cell_offset = target_cell_ident;
        let ttl_cell_ptr = unsafe {
            self.builder.build_in_bounds_gep(
                data_landscape.ttl_table_ptr,
                &[byte.int(ttl_cell_offset as i64)],
                "ttl_cell_ptr"
            )
        };
        self.builder.build_store(ttl_cell_ptr, byte.int(5));
        self.builder.build_unconditional_branch(continue_main_block);
        self.builder.position_at_end(continue_main_block);
    }
    fn code_region_expr(
        &self,
        org_expr: Box<OrganismExpression>,
        data_landscape: &DataLandscape,
        functions: &Functions
    ) -> () {
        let region_expr = (*org_expr).child.as_any().downcast_ref::<RegionExpression>().unwrap();
        let byte = self.context.i8_type();
        let target_region_num: IntValue;
        match region_expr.to() {
            Region::Cells => target_region_num = byte.int(0),
            Region::Layers => target_region_num = byte.int(1)
        };
        self.builder.build_store(data_landscape.curr_region_ptr, target_region_num);
        self.builder.build_call(
            functions.state_update_routine,
            &[
                data_landscape.gates_ttso_ptr.into(),
                data_landscape.curr_gates_state_ptr.into(),
                data_landscape.ttl_table_ptr.into(),
                byte.int(1).into(),
                data_landscape.cells_ptr.into()
            ],
            "carry_out_state_update_routine"
        );
    }
    fn code_drill_expr(&self, main_fn: &FunctionValue, data_landscape: &DataLandscape, functions: &Functions){
        let print_err_and_exit = ||{
            println!("Something went wrong while coding the drill gate routine");
            println!("I'm not sorry. Be a real man and don't whine about it");
            process::exit(1);
        };
        let byte = self.context.i8_type();
        let four_bytes = self.context.i32_type();
        let drill_result = self.builder.build_call(
            functions.drill_gate_routine,
            &[
                data_landscape.curr_region_ptr.into(),
                data_landscape.curr_gates_state_ptr.into(),
                data_landscape.gates_ttso_ptr.into()
            ],
            "carry_out_drill_routine"
        ).try_as_basic_value().left();
        if drill_result.is_none(){
            print_err_and_exit();
        }
        let drill_result = drill_result.unwrap().into_int_value();
        let drill_success_block = self.context.append_basic_block(*main_fn, "drill_success_block");
        let drill_fail_block = self.context.append_basic_block(*main_fn, "drill_fail_block");
        let continue_main_block = self.context.append_basic_block(*main_fn, "continue_main_block");
        let end_main_fail = self.get_basic_block(main_fn, main_fn_block_names::END_IN_FAIL);
        let drill_fail =  self.builder.build_int_compare(
            IntPredicate::NE,
            drill_result,
            four_bytes.int(0),
            "drill_fail"
        );
        self.builder.build_conditional_branch(
            drill_fail,
            drill_fail_block,
            drill_success_block
        );
        self.builder.position_at_end(drill_fail_block);
        self.builder.build_unconditional_branch(end_main_fail);
        self.builder.position_at_end(drill_success_block);
        self.builder.build_call(
            functions.state_update_routine,
            &[
                data_landscape.gates_ttso_ptr.into(),
                data_landscape.curr_gates_state_ptr.into(),
                data_landscape.ttl_table_ptr.into(),
                byte.int(0).into(),
                data_landscape.cells_ptr.into()
            ],
            "carry_out_state_update_routine"
        );
        self.builder.build_unconditional_branch(continue_main_block);
        self.builder.position_at_end(continue_main_block);
    }
    fn code_label(&self, org_expr: Box<OrganismExpression>, main_fn: &FunctionValue){
        let label_expr = org_expr.child.as_any().downcast_ref::<LabelExpression>().unwrap();
        let label = format!("{}{}", USER_DEFINED_LABEL_PREFIX, label_expr.label());
        let labelled_block = self.get_basic_block(main_fn, label.as_str());
        self.builder.build_unconditional_branch(labelled_block);
        self.builder.position_at_end(labelled_block);
    }
    fn code_jmp(&self, org_expr: Box<OrganismExpression>, main_fn: &FunctionValue, functions: &Functions, data_landscape: &DataLandscape){
        let jmp_expr = org_expr.child.as_any().downcast_ref::<JumpExpression>().unwrap();
        let to = jmp_expr.to();
        let target_block = main_fn.get_basic_blocks()
            .into_iter()
            .filter(|block|{
                let dest_block_name = format!("{}{}", USER_DEFINED_LABEL_PREFIX, to);
                let block_name = block.get_name().to_str().unwrap();
                block_name == dest_block_name
            }).next().unwrap();
        if jmp_expr.conditional(){
            self.code_conditional_jmp(&target_block, &*main_fn, &*functions, &*data_landscape);
        } else {
            self.code_unconditional_jmp(&target_block, &*main_fn, &*functions, &*data_landscape);
        }
    }
    fn code_unconditional_jmp(&self, target_block: &BasicBlock<'_>, main_fn: &FunctionValue, functions: &Functions, data_landscape: &DataLandscape){
        let byte = self.context.i8_type();
        self.builder.build_call(
            functions.state_update_routine,
            &[
                data_landscape.gates_ttso_ptr.into(),
                data_landscape.curr_gates_state_ptr.into(),
                data_landscape.ttl_table_ptr.into(),
                byte.int(1).into(),
                data_landscape.cells_ptr.into()
            ],
            "carry_out_state_update_routine"
        );
        self.builder.build_unconditional_branch(*target_block);
        let continue_main_block = self.context.append_basic_block(*main_fn, "continue_main_block");
        self.builder.position_at_end(continue_main_block);
    }
    fn code_conditional_jmp(&self, target_block: &BasicBlock<'_>, main_fn: &FunctionValue, functions: &Functions, data_landscape: &DataLandscape){
        let byte = self.context.i8_type();
        let four_bytes = self.context.i32_type();
        let cell_0_ptr = unsafe {
            self.builder.build_in_bounds_gep(
                data_landscape.cells_ptr,
                &[four_bytes.int(0)],
                "cell_0_ptr"
            )
        };
        let cell_0_val = self.builder.build_load(cell_0_ptr, "cell_0_val");
        let cell_0_val_is_0 = self.builder.build_int_compare(
            IntPredicate::EQ,
            cell_0_val.into_int_value(),
            four_bytes.int(0),
            "cell_0_val_is_0"
        );
        self.builder.build_call(
            functions.state_update_routine,
            &[
                data_landscape.gates_ttso_ptr.into(),
                data_landscape.curr_gates_state_ptr.into(),
                data_landscape.ttl_table_ptr.into(),
                byte.int(1).into(),
                data_landscape.cells_ptr.into()
            ],
            "carry_out_state_update_routine"
        );
        let continue_main_block = self.context.append_basic_block(*main_fn, "continue_main_block");
        self.builder.build_conditional_branch(
            cell_0_val_is_0,
            *target_block,
            continue_main_block
        );
        self.builder.position_at_end(continue_main_block);
    }
    fn code_lone_cell_expression(
        &self,
        main_fn: &FunctionValue,
        functions: &Functions,
        data_landscape: &DataLandscape
    ){
        let print_err_and_exit = ||{
            println!("Something went wrong while coding a leach expression");
            println!("I'm not sorry at all. Be a real man and don't whine");
            process::exit(1);
        };
        let four_bytes = self.context.i32_type();
        let cell_access_routine_result = self.builder.build_call(
            functions.cell_access_routine,
            &[data_landscape.curr_region_ptr.into()],
            "cell_access_routine_result"
        ).try_as_basic_value().left();
        if cell_access_routine_result.is_none(){
            print_err_and_exit();
        }
        let cell_access_routine_result = cell_access_routine_result.unwrap().into_int_value();
        let cell_access_failed_block = self.context.append_basic_block(*main_fn, "cell_access_failed_block");
        let cell_access_successful_block = self.context.append_basic_block(*main_fn, "cell_access_successful_block");
        let continue_main_block = self.context.append_basic_block(*main_fn, "continue_main_block");
        let end_in_fail_block = self.get_basic_block(main_fn, main_fn_block_names::END_IN_FAIL);
        let cell_access_routine_successful = self.builder.build_int_compare(
            IntPredicate::EQ,
            cell_access_routine_result,
            four_bytes.int(0),
            "cell_access_routine_successful"
        );
        self.builder.build_conditional_branch(
            cell_access_routine_successful,
            cell_access_successful_block,
            cell_access_failed_block,
        );
        self.builder.position_at_end(cell_access_failed_block);
        let err_msg = errors::err_invalid_cell_access_region_runtime();
        self.code_print(&functions.putchar, err_msg);
        self.builder.build_unconditional_branch(end_in_fail_block);
        self.builder.position_at_end(cell_access_successful_block);
        self.builder.build_unconditional_branch(continue_main_block);
        self.builder.position_at_end(continue_main_block);
    }
    fn code_lone_primitive_expression(
        &self,
        main_fn: &FunctionValue,
        functions: &Functions,
        data_landscape: &DataLandscape
    ){
        let print_err_and_exit = ||{
            println!("Something went wrong while coding a primitive expression");
            println!("I'm not sorry at all. Be a real man and don't whine");
            process::exit(1);
        };
        let four_bytes = self.context.i32_type();
        let primitive_access_routine_result = self.builder.build_call(
            functions.primitive_access_routine,
            &[
                data_landscape.curr_region_ptr.into(),
                data_landscape.curr_gates_state_ptr.into()
            ],
            "primitive_access_routine_result"
        ).try_as_basic_value().left();
        if primitive_access_routine_result.is_none(){
            print_err_and_exit();
        }
        let primitive_access_routine_result = primitive_access_routine_result.unwrap().into_int_value();
        let primitive_access_failed_block = self.context.append_basic_block(
            *main_fn,
            "primitive_access_failed_block"
        );
        let primitive_access_successful_block = self.context.append_basic_block(
            *main_fn,
            "primitive_access_successful_block"
        );
        let continue_main_block = self.context.append_basic_block(*main_fn, "continue_main_block");
        let end_in_fail_block = self.get_basic_block(main_fn, main_fn_block_names::END_IN_FAIL);
        let primitive_access_routine_successful = self.builder.build_int_compare(
            IntPredicate::EQ,
            primitive_access_routine_result,
            four_bytes.int(0),
            "primitive_access_routine_successful"
        );
        self.builder.build_conditional_branch(
            primitive_access_routine_successful,
            primitive_access_successful_block,
            primitive_access_failed_block,
        );
        self.builder.position_at_end(primitive_access_failed_block);
        self.builder.build_unconditional_branch(end_in_fail_block);
        self.builder.position_at_end(primitive_access_successful_block);
        self.builder.build_unconditional_branch(continue_main_block);
        self.builder.position_at_end(continue_main_block);
    }
    fn code_post_func_exec_routine(
        &self,
        pf_cell_ident: u8,
        args: &Vec<&CellExpression>,
        data_landscape: &DataLandscape,
        functions: &Functions
    ){
        let byte = self.context.i8_type();
        let kill_expr = |cell_ttl_addr_offset|{
            let cell_ttl_ptr = unsafe {
                self.builder.build_in_bounds_gep(
                    data_landscape.ttl_table_ptr,
                    &[byte.int(cell_ttl_addr_offset as i64)],
                    "cell_ttl_ptr"
                )
            };
            self.builder.build_store(cell_ttl_ptr, byte.int(0));
        };
        // Consider a scenario: 0~1~2^^^^^^666^^^^^^=M
        // There are 2 args
        // The SUR must be carried out twice
        // The same goes for all number of args
        for _ in 0..args.len(){
            self.builder.build_call(
                functions.state_update_routine,
                &[
                    data_landscape.gates_ttso_ptr.into(),
                    data_landscape.curr_gates_state_ptr.into(),
                    data_landscape.ttl_table_ptr.into(),
                    byte.int(1).into(),
                    data_landscape.cells_ptr.into()
                ],
                "carry_out_state_update_routine"
            );
        }
        // Kill all args except the last
        for i in 0..args.len() - 1 {
            let arg = args[i];
            let cell_ttl_addr_offset = arg.ident();
            kill_expr(cell_ttl_addr_offset);
        }
        // Kill the expression that went on the massacre
        kill_expr(pf_cell_ident);
        // The new expression in the last cell has to have a new TTL of 5
        let target_cell_ident = args[args.len() - 1].ident();
        let target_cell_ptr = unsafe {
            self.builder.build_gep(
                data_landscape.ttl_table_ptr,
                &[byte.int(target_cell_ident as i64)],
                "target_cell_ptr"
            )
        };
        self.builder.build_store(target_cell_ptr, byte.int(5));
    }
    fn code_print(&self, putchar: &FunctionValue, msg: String){
        for c in msg.chars(){
            self.builder.build_call(
                *putchar,
                &[self.context.i32_type().const_int(c as u64, false).into()],
                "printerror"
            );
        }
    }
    fn get_basic_block<'a>(&self, func: &'a FunctionValue, name: &str) -> BasicBlock<'a> {
        func.get_basic_blocks()
            .into_iter()
            .filter(|basic_block| {
                basic_block.get_name().to_str().unwrap() == name
            }).next().unwrap()
    }
    pub fn write_code_to_file(&self, out_filename: &str){
        let print_err_and_exit = ||{
            println!("Something went wrong while writing code to a file");
            println!("Be a real man and don't whine");
            process::exit(1);
        };
        Target::initialize_all(&InitializationConfig::default());
        let target_triple = TargetMachine::get_default_triple();
        let cpu = TargetMachine::get_host_cpu_name().to_string();
        let features = TargetMachine::get_host_cpu_features().to_string();
        let target = Target::from_triple(&target_triple);
        if target.is_err(){
            print_err_and_exit();
        }
        let target = target.unwrap();
        let target_machine = target.create_target_machine(
            &target_triple,
            &cpu,
            &features,
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default
        );
        if target_machine.is_none(){
            print_err_and_exit();
        }
        let target_machine = target_machine.unwrap();
        let temp_out_filename = format!("{}.tmp", out_filename);
        let write_to_file_result = target_machine.write_to_file(
            &self.module,
            FileType::Object,
            temp_out_filename.as_ref()
        );
        if write_to_file_result.is_err(){
            print_err_and_exit();
        }
        let save_code_result = process::Command::new("gcc")
            .arg(&temp_out_filename)
            .arg(format!("-o{}", out_filename))
            .arg("-no-pie")
            .output();
        if save_code_result.is_err(){
            print_err_and_exit();
        }
        if fs::remove_file(&temp_out_filename).is_err(){
            print_err_and_exit();
        }
    }
}

trait CreateNegNum {
    fn int<'ctx>(&self, n: i64) -> IntValue<'_>;
}

impl CreateNegNum for IntType<'_> {
    fn int<'ctx>(&self, n: i64) -> IntValue<'_> {
        if n < 0 {
            self.const_int_from_string(format!("{}", n).as_str(), StringRadix::Decimal).unwrap()
        } else {
            self.const_int(n as u64, false)
        }
    }
}

mod main_fn_block_names {
    pub const END_IN_FAIL: &'static str = "end_main_fail";
}

const USER_DEFINED_LABEL_PREFIX: &'static str = "user_defined_label";