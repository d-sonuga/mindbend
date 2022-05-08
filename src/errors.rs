use std::fmt::Display;
use std::ops::Add;

pub fn err_whitespace(pos: i32) -> String {
    format!("Invalid whitespace at position n where n is around {}", pos)
}

pub fn err_unrecognized_token(pos: i32) -> String {
    format!("Unrecognized token at position n where n is around {}", pos)
}

pub fn err_expected<T>(pos: i32, expected: T) -> String where T: Display {
    format!("Expected {} at position n where n is around {}", expected, pos)
}

pub fn err_invalid_primitive(pos: i32) -> String {
    format!("Invalid primitive at position n where n is around {}", pos)
}

pub fn err_invalid_primitive_access_region<T>(pos: T) -> String where T: Add + Display {
    format!(
        "Attempting to access primitive outside the Layers Region at the \
        nth token, where n is around {}",
        pos
    )
}

pub fn err_invalid_primitive_access_gates<T>(pos: T) -> String where T: Add + Display {
    format!(
        "Attempting to access primitive when the Layers gates aren't fully open at the \
        nth token, where n is around {}", pos
    )
}

pub fn err_invalid_cell_access_region<T>(pos: T) -> String where T: Add + Display {
    format!("Attempting to access cell outside the Cells Region at the nth token, where n is around {}", pos)
}

pub fn err_unrecognized_region(pos: usize, found: String) -> String {
    format!("Use of unrecognized region {} at the nth token, where n is around {}", found.as_str(), pos)
}

pub fn err_expected_cell_expression_after(pos: usize) -> String {
    format!("Expected cell expression after the nth token, where n is around {}", pos)
}

pub fn err_expected_cell_expression(pos: usize) -> String {
    format!("Expected cell expression at the nth token, where n is around {}", pos)
}

pub fn err_unrecognized_cell(pos: usize, found: String) -> String {
    format!("Use of unrecognized cell {} at the nth token, where n is around {}", found.as_str(), pos)
}

pub fn err_drill_in_cells(pos: usize) -> String {
    format!(
        "Attempt to drill in the Cells Region at the nth token, where n is around {}", pos
    )
}

pub fn err_org_expr_must_end_in_death() -> String {
    format!("A Mindbend program must end in the death of the Organism Expression")
}

pub fn err_chained_leach_expression_must_end_in_massacre(pos: usize) -> String {
    format!("\
        Chained leach expression at the nth token, where n is around {}, does not end in \
        a massacre. A chained leach expression must end in a massacre",
        pos
    )
}

pub fn err_attempt_to_jump_to_non_existent_label(pos: usize) -> String {
    format!(
        "Attempt to jump to non existent label at the nth token, where n is around {}",
        pos
    )
}

pub fn err_duplicate_label(pos1: usize, pos2: usize) -> String {
    format!(
        "Label name at the nth token duplicated in the label name at the mth token, \
        where n is around {} and m is around {}",
        pos1, pos2
    )
}

pub fn err_leach_expression_must_start_with_primitive_or_cell(pos: usize) -> String {
    format!(
        "The leach expression at the nth token, where n is around {}, \
        does not begin with a primitive or Cell",
        pos
    )
}

pub fn err_attempt_to_leach_expr_onto_itself(pos: usize) -> String {
    format!(
        "Attempt to leach expression onto itself at the nth token, \
        where n is around {}",
        pos
    )
}

pub fn err_chain_leach_expression_ending_without_chain_leach_expression(pos: usize) -> String {
    format!(
        "Ending a chain leach expression without a chain leach expression at the nth token, \
        where n is around {}",
        pos
    )
}

pub fn err_triple_six_eq_not_expected_here(pos: usize) -> String {
    format!(
        "^^^^^^666^^^^^^= not expected at the nth token, where n is around {}",
        pos
    )
}

pub fn err_triple_six_not_expected_here(pos: usize) -> String {
    format!(
        "^^^^^^666^^^^^^ not expected at the nth token, where n is around {}",
        pos
    )
}

pub fn err_invalid_primitive_access_region_not_layers_runtime() -> String {
    format!("Attempt to access primitive when not in layers region\n")
}

pub fn err_invalid_primitive_access_gates_not_open_runtime() -> String {
    format!("Attempt to access primitive when the gates aren't open\n")
}

pub fn err_invalid_gate_access_region_not_layers_runtime() -> String {
    format!("Attempt to drill gates when not in the Layers Region\n")
}

pub fn err_attempt_to_use_non_function_primitive_to_massacre() -> String {
    format!("Attempt to use non-function primitive to massacre\n")
}

pub fn err_invalid_cell_access_region_runtime() -> String {
    format!("Attempting to access cell outside the Cells Region\n")
}

pub fn err_invalid_primitive_access_runtime() -> String {
    format!("Attempt to access primitive either when gates are closed or not in Layers Region\n")
}

pub fn err_attempt_to_leach_death_expression_onto_another_cell() -> String {
    format!("Attempt to leach death expression onto another Cell\n")
}