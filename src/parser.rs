use std::collections::HashMap;
use std::{vec, fmt};
use std::cmp::PartialEq;
use std::any::Any;
use std::iter::{Peekable, Enumerate, Iterator};
use lazy_static::lazy_static;
use crate::lexer::Token;
use crate::{errors, r_ident};


macro_rules! p_value {
    ($key:ident) => {
        (*PRIMITIVE_TABLE.get(&$key).unwrap()).clone()
    };
    ($key:expr) => {
        (*PRIMITIVE_TABLE.get(&$key).unwrap()).clone()
    };
}

#[derive(Debug, Clone)]
struct TokenIterator {
    tokens: Peekable<Enumerate<vec::IntoIter<Token>>>,
    offset: usize
}

impl TokenIterator {
    fn new(tokens: Vec<Token>, offset: usize) -> TokenIterator {
        TokenIterator {
            tokens: tokens.into_iter().enumerate().peekable(),
            offset
        }
    }
    fn peek(&mut self) -> Option<&Token> {
        if self.tokens.peek().is_none(){
            None
        } else {
            Some(&(*self.tokens.peek().unwrap()).1)
        }
    }
}

impl Iterator for TokenIterator {
    type Item = (usize, Token);
    
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.tokens.next();
        if next.is_none(){
            return next;
        }
        let (pos, token) = next.unwrap();
        Some((pos + self.offset, token))
    }
}

pub struct Parser {
    tokens: TokenIterator,
    curr_region: Region,
    layers_gates_state: LayersGatesState,
    encountered_jumps: Vec<(usize, String)>,
    encountered_labels: Vec<String>
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: TokenIterator::new(tokens, 1),
            curr_region: Region::Cells,
            layers_gates_state: LayersGatesState::AllClose,
            encountered_jumps: vec![],
            encountered_labels: vec![]
        }
    }
    pub fn parse(&mut self) -> Result<(OrganismExpression, Vec<String>), String> {
        let parse_result = self.parse_expressions();
        for (pos, label) in self.encountered_jumps.iter() {
            if !self.encountered_labels.contains(label){
                return Err(errors::err_attempt_to_jump_to_non_existent_label(*pos))
            }
        }
        match parse_result {
            Ok(oe) => Ok((oe, self.encountered_labels.clone())),
            Err(err) => Err(err)
        }
    }
    fn parse_expressions(&mut self) -> Result<OrganismExpression, String> {
        let child: Box<dyn Expression>;
        let mut next_org_expr: Option<Box<OrganismExpression>> = None;
        match self.tokens.next().unwrap() {
            (pos, Token::PrimitiveIdent(p_ident)) => {
                match self.validate_primitive_access(pos){
                    Ok(()) => (),
                    Err(err) => return Err(err)
                };
                let curr_primitive = p_ident.clone();
                
                let mut region_changes: Vec<(usize, Token)> = vec![];
                while self.tokens.peek().is_some() && (*self.tokens.peek().unwrap()).is_region(){
                    let (pos, token) = self.tokens.next().unwrap();
                    let r_ident: String;
                    match token {
                        Token::RegionIdent(r) => r_ident = r,
                        _ => unreachable!()
                    };
                    match self.change_region(pos, r_ident.clone()){
                        Ok(()) => (),
                        Err(err) => return Err(err)
                    };
                    region_changes.push((pos, r_ident!(r_ident)));
                }
                // If this condition is true, all the region changes that occur after
                // the occurence of the primitive should end up being part of the leach
                // expression
                if self.tokens.peek().is_some() && *self.tokens.peek().unwrap() == Token::Tilde {
                    let left = Box::new(PrimitiveExpression::new(curr_primitive));
                    match self.parse_primitive_leach_expression(pos, left, Some(region_changes)){
                        Ok(leach_expr) => {
                            child = leach_expr;
                        },
                        Err(err) => return Err(err)
                    };
                } else {
                    // put back the consumed region tokens
                    if region_changes.len() != 0 {
                        let new_tokens = region_changes.into_iter().chain(self.tokens.clone());
                        let new_tokens: Vec<Token> = new_tokens
                            .map(|(_, token)| token)
                            .collect();
                        self.tokens = TokenIterator::new(new_tokens, pos);
                    }
                    child = Box::new(PrimitiveExpression::new(p_ident));
                }
            },
            // The Cell Identifier being encountered here means that it must
            // either be the left operand in a leach expression or it is an
            // expression of no effect
            (pos, Token::CellIdent(c_ident)) => {
                match self.validate_cell_access(pos){
                    Ok(()) => (),
                    Err(err) => return Err(err)
                };
                if *self.tokens.peek().unwrap() == Token::Tilde {
                    let left = Box::new(CellExpression::new(c_ident.clone()));
                    let (pos, _) = self.tokens.next().unwrap();
                    match self.parse_leach_expression(pos, left, None, false, vec![c_ident]){
                        Ok(leach_expr) => child = leach_expr,
                        Err(err) => return Err(err)
                    };
                } else {
                    child = Box::new(CellExpression::new(c_ident));
                }
            },
            // A standalone region change
            (pos, Token::RegionIdent(r_ident)) => {
                match self.change_region(pos, r_ident.clone()){
                    Ok(()) => (),
                    Err(err) => return Err(err)
                };
                match self.parse_region(pos, r_ident){
                    Ok(r_expr) => child = Box::new(r_expr),
                    Err(err) => return Err(err)
                };
            },
            (pos, Token::Drill) => {
                match self.validate_drill(pos){
                    Ok(()) => self.drill(),
                    Err(err) => return Err(err)
                };
                child = Box::new(DrillExpression::new());
            },
            (_, Token::Label(label)) => {
                self.encountered_labels.push(label.clone());
                child = Box::new(LabelExpression::new(label));
            }
            (pos, Token::Jump(label)) => {
                self.encountered_jumps.push((pos, label.clone()));
                child = Box::new(JumpExpression::new(label));
            }
            (pos, Token::ConditionalJump(label)) => {
                self.encountered_jumps.push((pos, label.clone()));
                child = Box::new(JumpExpression::new_conditional(label))
            }
            (pos, Token::Tilde) => {
                return Err(errors::err_leach_expression_must_start_with_primitive_or_cell(pos))
            }
            (pos, Token::TripleSixEqM) => {
                return Err(errors::err_chain_leach_expression_ending_without_chain_leach_expression(pos))
            }
            (pos, Token::TripleSixEq) => {
                return Err(errors::err_triple_six_eq_not_expected_here(pos))
            }
            (pos, Token::TripleSix) => {
                return Err(errors::err_triple_six_not_expected_here(pos))
            }
            (_, Token::TripleSixEqO) => {
                let mut new_next_org_expr;
                if self.tokens.peek().is_some(){
                    match self.parse_expressions(){
                        Ok(org_expr) => new_next_org_expr = org_expr,
                        Err(err) => return Err(err)
                    };
                    use std::mem;
                    let dummy_expr = Box::new(DummyExpression::new());
                    let dummy_expr1 = Box::new(DummyExpression::new());
                    let dummy_org_expr = Some(Box::new(OrganismExpression::new(dummy_expr1, None)));
                    child = mem::replace(&mut new_next_org_expr.child, dummy_expr);
                    next_org_expr = mem::replace(&mut new_next_org_expr.right, dummy_org_expr);
                } else {
                    child = Box::new(DummyExpression::new());
                }
            }
        }
        if next_org_expr.is_some(){
            return Ok(OrganismExpression::new(child, next_org_expr))
        }
        if self.tokens.peek().is_some(){
            match self.parse_expressions(){
                Ok(org_expr) => Ok(OrganismExpression::new(
                    child,
                    Some(Box::new(org_expr))
                )),
                Err(err) => Err(err)
            }
        } else {
            Ok(OrganismExpression::new(child, None))
        }
    }
    fn validate_primitive_access(&self, pos: usize) -> Result<(), String> {
        if self.curr_region != Region::Layers {
            return Err(errors::err_invalid_primitive_access_region(pos));
        }
        if self.layers_gates_state != LayersGatesState::ThreeOpen {
            return Err(errors::err_invalid_primitive_access_gates(pos));
        }
        Ok(())
    }
    fn validate_cell_access(&self, pos: usize) -> Result<(), String> {
        if self.curr_region != Region::Cells {
            return Err(errors::err_invalid_cell_access_region(pos));
        }
        Ok(())
    }
    fn validate_drill(&self, pos: usize) -> Result<(), String> {
        match self.curr_region {
            Region::Layers => Ok(()),
            Region::Cells => Err(errors::err_drill_in_cells(pos))
        }
    }
    fn drill(&mut self){
        match self.layers_gates_state {
            LayersGatesState::AllClose => self.layers_gates_state = LayersGatesState::OneOpen,
            LayersGatesState::OneOpen => self.layers_gates_state = LayersGatesState::TwoOpen,
            LayersGatesState::TwoOpen => self.layers_gates_state = LayersGatesState::ThreeOpen,
            LayersGatesState::ThreeOpen => ()
        };
    }
    fn parse_primitive_leach_expression(
        &mut self,
        pos: usize,
        left: Box<dyn PassiveExpression>,
        region_changes: Option<Vec<(usize, Token)>>
    ) -> Result<Box<LeachExpression>, String>{
        // Get rid of the Tilde
        self.tokens.next();
        if self.tokens.peek().is_none(){
            return Err(errors::err_expected_cell_expression_after(pos));
        }
        match self.tokens.next().unwrap() {
            (_, Token::CellIdent(c_ident)) => {
                if !self.cell_ident_is_valid(c_ident.clone()){
                    return Err(errors::err_unrecognized_cell(pos, c_ident))
                }
                let region_changes = self.parse_region_changes(region_changes);
                let right = Some(Box::new(LeachExpression::new(
                    Box::new(CellExpression::new(c_ident)),
                    None, false, None
                )));
                Ok(Box::new(LeachExpression::new(
                    left, right, false, region_changes
                )))
            },
            (pos, _) => Err(errors::err_expected_cell_expression(pos))
        }
    }
    fn parse_leach_expression(
        &mut self,
        pos: usize,
        left: Box<dyn PassiveExpression>,
        region_changes: Option<Vec<(usize, Token)>>,
        predecessor_is_chain: bool,
        cells_encountered: Vec<String>
    ) -> Result<Box<LeachExpression>, String> {
        if self.tokens.peek().is_none(){
            return Err(errors::err_expected_cell_expression_after(pos));
        }
        // The Tilde has already been taken care of by the caller, so no need to bother about it
        let token = self.tokens.next();
        let c_ident: String;
        let pos: usize;
        match token.unwrap(){
            (p, Token::CellIdent(ident)) => {
                c_ident = ident;
                pos = p;
            },
            (pos, _) => return Err(errors::err_expected_cell_expression(pos))
        };
        if !self.cell_ident_is_valid(c_ident.clone()){
            return Err(errors::err_unrecognized_cell(pos, c_ident));
        }
        if cells_encountered.contains(&c_ident){
            return Err(errors::err_attempt_to_leach_expr_onto_itself(pos));
        }
        let right: Option<Box<LeachExpression>>;
        let mut is_chain = false;
        let curr_cell_expr = Box::new(CellExpression::new(c_ident));
        let region_changes: Option<Vec<RegionExpression>> = self.parse_region_changes(region_changes);
        if self.tokens.peek().is_some(){
            let next_token_ref = self.tokens.peek().unwrap();
            match next_token_ref {
                // If a tilde comes next, must be a chained leach expression
                Token::Tilde => {
                    // consume the tilde so it won't reach the next recursion
                    let (pos, _) = self.tokens.next().unwrap();
                    is_chain = true;
                    match self.parse_leach_expression(pos, curr_cell_expr, None, true, cells_encountered){
                        Ok(leach_expr) => right = Some(leach_expr),
                        Err(err) => return Err(err)
                    };
                },
                // ^^^^^^666^^^^^^=M comes next, must be the end of a chained leach
                // expression
                Token::TripleSixEqM => {
                    // Get rid of the token
                    // It's not needed for code generation
                    self.tokens.next();
                    // Consider the expression: 0~1^^^^^^666^^^^^^=M
                    // The above is a chain leach expression without predecessor
                    // So it will be called with predecessor_is_chain = false
                    // So the expression will be labelled as chain, so it will be coded
                    // as a function call
                    // But consider the expression: 0~1~2^^^^^^666^^^^^^=M
                    // The last expression in the chain is 1~2 and should not be tagged as chain
                    // That is taken care of here because when the ~ is encountered in 0~1,
                    // predecessor_is_chain will be true, so when the last 1~2 is reached,
                    // it won't be tagged as chain
                    is_chain = !predecessor_is_chain;
                    right = Some(Box::new(LeachExpression::new(curr_cell_expr, None, false, None)));
                    return Ok(Box::new(
                        LeachExpression::new(left, right, is_chain, region_changes)
                    ));
                },
                // anything else comes next, must be the start of another unrelated
                // expression, so right must be this current cell expression
                _ => {
                    right = Some(Box::new(LeachExpression::new(curr_cell_expr, None, false, None)))
                }
            };
        } else {
            right = Some(Box::new(LeachExpression::new(
                curr_cell_expr, None, false, None
            )));
        }
        if predecessor_is_chain {
            return Err(errors::err_chained_leach_expression_must_end_in_massacre(pos))
        }
        Ok(Box::new(
            LeachExpression::new(left, right, is_chain, region_changes)
        ))
    }
    fn parse_region_changes(
        &mut self,
        region_changes: Option<Vec<(usize, Token)>>
    ) -> Option<Vec<RegionExpression>> {
        if region_changes.is_some(){
            Some(region_changes.unwrap()
                .into_iter()
                .map(|(pos, token)| {
                    match token {
                        Token::RegionIdent(r_ident) => {
                            match self.parse_region(pos, r_ident.clone()){
                                Ok(region_expr) => {
                                    match self.change_region(pos, r_ident){
                                        Ok(()) => (),
                                        Err(err) => return Err(err)
                                    };
                                    Ok(region_expr)
                                },
                                Err(err) => return Err(err)
                            }
                        },
                        _ => unreachable!()
                    }
                })
                .map(|expr| expr.unwrap())
                .collect())
        } else {
            None
        }
    }
    fn cell_ident_is_valid(&self, c_ident: String) -> bool {
        if c_ident.len() != 1 {
            return false
        }
        match c_ident.chars().next().unwrap(){
            '0'..='9' | 'A'..='E' => true,
            _ => false
        }
    }
    fn change_region(&mut self, pos: usize, new_region: String) -> Result<(), String> {
        match new_region.as_str() {
            "C" => self.curr_region = Region::Cells,
            "L" => self.curr_region = Region::Layers,
            _ => return Err(errors::err_unrecognized_region(pos, new_region))
        };
        Ok(())
    }
    fn parse_region(&mut self, pos: usize, r_ident: String) -> Result<RegionExpression, String> {
        match r_ident.as_str(){
            "C" => Ok(RegionExpression::new(Region::Cells)),
            "L" => Ok(RegionExpression::new(Region::Layers)),
            _ => Err(errors::err_unrecognized_region(pos, r_ident))
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Region {
    Cells,
    Layers
}

#[derive(Debug, PartialEq, Eq)]
enum LayersGatesState {
    AllClose,
    OneOpen,
    TwoOpen,
    ThreeOpen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveValue {
    One, Two, Three, Four, Five, Six,
    Seven, Eight, Nine, Zero, Input, Output,
    Addition, Subtraction
}

pub trait Expression: fmt::Debug + Any {
    fn get_type(&self) -> ExprType;
    fn get_repr(&self) -> String;
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

pub trait PassiveExpression: Expression {}

trait ActiveExpression: Expression {}

#[derive(Debug, Clone, PartialEq)]
pub struct CellExpression {
    pub ident: u8
}

impl Expression for CellExpression {
    fn get_type(&self) -> ExprType {
        ExprType::Cell
    }
    fn get_repr(&self) -> String {
        format!("{}", self.ident)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl CellExpression {
    pub fn new(c_ident: String) -> CellExpression {
        let c_ident = c_ident.chars().next().unwrap();
        CellExpression {
            ident: c_ident.to_digit(16).unwrap() as u8
        }
    }
    pub fn ident(&self) -> u8 {
        self.ident
    }
}

impl PassiveExpression for CellExpression {}

#[derive(Debug)]
pub struct PrimitiveExpression {
    pval: PrimitiveValue
}

impl Expression for PrimitiveExpression {
    fn get_type(&self) -> ExprType {
        ExprType::Primitive
    }
    fn get_repr(&self) -> String {
        format!("{:?}", self.pval)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl PassiveExpression for PrimitiveExpression {}

impl PrimitiveExpression {
    fn new(p_ident: String) -> PrimitiveExpression {
        PrimitiveExpression {
            pval: p_value!(p_ident)
        }
    }
    pub fn pval(&self) -> PrimitiveValue {
        self.pval.clone()
    }
}

#[derive(Debug)]
pub struct LeachExpression {
    pub left: Box<dyn PassiveExpression>,
    pub right: Option<Box<LeachExpression>>,
    pub is_chain: bool,
    pub region_change: Option<Vec<RegionExpression>>
}

impl Expression for LeachExpression {
    fn get_type(&self) -> ExprType {
        ExprType::Leach
    }
    fn get_repr(&self) -> String {
        format!(
            "(left: {:?}, right: {:?}, is_chain: {:?})",
            self.left.get_repr(), self.right, self.is_chain
        )
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl LeachExpression {
    pub fn new(
        left: Box<dyn PassiveExpression>,
        right: Option<Box<LeachExpression>>,
        is_chain: bool,
        region_change: Option<Vec<RegionExpression>>
    ) -> LeachExpression {
        LeachExpression {
            left,
            right,
            is_chain,
            region_change
        }
    }
    pub fn left(&self) -> &Box<dyn PassiveExpression> {
        &self.left
    }
    pub fn right(&self) -> &Option<Box<LeachExpression>> {
        &self.right
    }
}

#[derive(Debug, Clone)]
pub struct RegionExpression {
    to: Region
}

impl Expression for RegionExpression {
    fn get_type(&self) -> ExprType {
        ExprType::Region
    }
    fn get_repr(&self) -> String {
        format!("(to: {:?})", self.to)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl RegionExpression {
    fn new(to: Region) -> RegionExpression {
        RegionExpression {
            to
        }
    }
    pub fn to(&self) -> Region {
        self.to.clone()
    }
}

#[derive(Debug)]
struct DrillExpression {}

impl Expression for DrillExpression {
    fn get_type(&self) -> ExprType {
        ExprType::Drill
    }
    fn get_repr(&self) -> String {
        format!("{:?}", Token::Drill)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl DrillExpression {
    fn new() -> DrillExpression {
        DrillExpression {}
    }
}

#[derive(Debug)]
pub struct LabelExpression {
    label: String
}

impl Expression for LabelExpression {
    fn get_type(&self) -> ExprType {
        ExprType::Label
    }
    fn get_repr(&self) -> String {
        format!("(label: {:?})", self.label)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl LabelExpression {
    pub fn new(label: String) -> LabelExpression {
        LabelExpression {
            label
        }
    }
    pub fn label(&self) -> String {
        self.label.clone()
    }
}

#[derive(Debug)]
pub struct JumpExpression {
    to: String,
    conditional: bool
}

impl Expression for JumpExpression {
    fn get_type(&self) -> ExprType {
        ExprType::Jump
    }
    fn get_repr(&self) -> String {
        format!("(jump: to:{:?})", self.to)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl JumpExpression {
    fn new(label: String) -> JumpExpression {
        JumpExpression {
            to: label,
            conditional: false
        }
    }
    fn new_conditional(label: String) -> JumpExpression {
        JumpExpression {
            to: label,
            conditional: true
        }
    }
    pub fn to(&self) -> String {
        self.to.clone()
    }
    pub fn conditional(&self) -> bool {
        self.conditional
    }
}

#[derive(Debug)]
pub struct DummyExpression {}

impl DummyExpression {
    pub fn new() -> DummyExpression {
        DummyExpression {}
    }
}

impl Expression for DummyExpression {
    fn get_type(&self) -> ExprType {
        ExprType::Dummy
    }
    fn get_repr(&self) -> String {
        format!("Dummy Expression")
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct OrganismExpression {
    pub child: OrgExprChild,
    pub right: OrgExprRight
}

pub type OrgExprChild = Box<dyn Expression>;
pub type OrgExprRight = Option<Box<OrganismExpression>>;

impl Expression for OrganismExpression {
    fn get_type(&self) -> ExprType {
        ExprType::Organism
    }
    fn get_repr(&self) -> String {
        format!("(child: ({:?}), right: ({:?}))", self.child.get_repr(), self.right)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl OrganismExpression {
    pub fn new(child: Box<dyn Expression>, right: Option<Box<OrganismExpression>>) -> OrganismExpression {
        OrganismExpression {
            child,
            right
        }
    }
}

impl PartialEq for OrganismExpression {
    fn eq(&self, other: &Self) -> bool {
        self.child.get_type() == other.child.get_type() &&
        self.child.get_repr() == other.child.get_repr() &&
        self.right == other.right
    }
    fn ne(&self, other: &Self) -> bool {
        self.child.get_type() != other.child.get_type() ||
        self.child.get_repr() != other.child.get_repr() ||
        self.right != other.right
    }
}

lazy_static! {
    static ref PRIMITIVE_TABLE: HashMap<String, PrimitiveValue> = [
        // Symbols
        (format!("!"), PrimitiveValue::One),
        (format!("@"), PrimitiveValue::Two),
        (format!("#"), PrimitiveValue::Three),
        (format!("+"), PrimitiveValue::Four),
        (format!("%"), PrimitiveValue::Five),
        (format!("`"), PrimitiveValue::Six),
        (format!("&"), PrimitiveValue::Seven),
        (format!("*"), PrimitiveValue::Eight),
        (format!("("), PrimitiveValue::Nine),
        (format!(")"), PrimitiveValue::Zero),
        (format!("><"), PrimitiveValue::Output),
        (format!("<>"), PrimitiveValue::Input),
        (format!("}}"), PrimitiveValue::Addition),
        (format!("{{"), PrimitiveValue::Subtraction),
        
        // Indexes
        (format!("D"), PrimitiveValue::One),
        (format!("C"), PrimitiveValue::Two),
        (format!("B"), PrimitiveValue::Three),
        (format!("A"), PrimitiveValue::Four),
        (format!("9"), PrimitiveValue::Five),
        (format!("8"), PrimitiveValue::Six),
        (format!("7"), PrimitiveValue::Seven),
        (format!("6"), PrimitiveValue::Eight),
        (format!("5"), PrimitiveValue::Nine),
        (format!("4"), PrimitiveValue::Zero),
        (format!("2"), PrimitiveValue::Output),
        (format!("3"), PrimitiveValue::Input),
        (format!("1"), PrimitiveValue::Addition),
        (format!("0"), PrimitiveValue::Subtraction)
    ].iter().cloned().collect();
}

#[derive(Debug, PartialEq)]
pub enum ExprType {
    Cell,
    Leach,
    Organism,
    Label,
    Jump,
    Primitive,
    Region,
    Drill,
    Dummy
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Token::*;
    use crate::{c_ident, r_ident, p_ident};
    #[test]
    fn test_valid1(){
        let tokens = vec![
            r_ident!("L"), Token::Drill, Token::Drill, Token::Drill, p_ident!("`"),
            r_ident!("C"), Token::Tilde, c_ident!("0"), r_ident!("L"), p_ident!("><"),
            r_ident!("C"), Token::Tilde, c_ident!("1"), c_ident!("0"), Token::Tilde, c_ident!("1")
        ];
        let result = Parser::new(tokens).parse();
        let expected_result = OrganismExpression {
            child: Box::new(RegionExpression {
                to: Region::Layers
            }),
            right: Some(Box::new(OrganismExpression {
                child: Box::new(DrillExpression::new()),
                right: Some(Box::new(OrganismExpression {
                    child: Box::new(DrillExpression::new()),
                    right: Some(Box::new(OrganismExpression {
                        child: Box::new(DrillExpression::new()),
                        right: Some(Box::new(OrganismExpression {
                            child: Box::new(LeachExpression {
                                left: Box::new(PrimitiveExpression {
                                    pval: PrimitiveValue::Six
                                }),
                                right: Some(Box::new(LeachExpression {
                                    left: Box::new(CellExpression {
                                        ident: 0
                                    }),
                                    right: None,
                                    is_chain: false,
                                    region_change: None
                                })),
                                is_chain: false,
                                region_change: Some(vec![RegionExpression::new(Region::Cells)])
                            }),
                            right: Some(Box::new(OrganismExpression {
                                child: Box::new(RegionExpression::new(Region::Layers)),
                                right: Some(Box::new(OrganismExpression {
                                    child: Box::new(LeachExpression {
                                        left: Box::new(PrimitiveExpression {
                                            pval: PrimitiveValue::Output
                                        }),
                                        right: Some(Box::new(LeachExpression {
                                            left: Box::new(CellExpression {
                                                ident: 1
                                            }),
                                            right: None,
                                            is_chain: false,
                                            region_change: None
                                        })),
                                        is_chain: false,
                                        region_change: Some(vec![RegionExpression::new(Region::Cells)])
                                    }),
                                    right: Some(Box::new(OrganismExpression {
                                        child: Box::new(LeachExpression {
                                            left: Box::new(CellExpression {
                                                ident: 0
                                            }),
                                            right: Some(Box::new(LeachExpression {
                                                left: Box::new(CellExpression {
                                                    ident: 1
                                                }),
                                                right: None,
                                                is_chain: false,
                                                region_change: None
                                            })),
                                            is_chain: false,
                                            region_change: None
                                        }),
                                        right: None
                                    }))
                                }))
                            }))
                        }))
                    }))
                }))
            }))
        };
        assert!(result.is_ok());
        let result = result.unwrap().0;
        assert_eq!(result, expected_result);
    }
    #[test]
    fn test_valid2(){
        let tokens = vec![
            r_ident!("L"), Drill, Drill, Drill, p_ident!("<>"), r_ident!("C"), Tilde, c_ident!("0"),
            c_ident!("0"), Tilde, c_ident!("1"), TripleSixEqM, r_ident!("L"), p_ident!("><"),
            r_ident!("C"), Tilde, c_ident!("2"), c_ident!("2"), Tilde, c_ident!("1"), TripleSixEqM
        ];
        let result = Parser::new(tokens).parse();
        let expected_result = OrganismExpression {
            child: Box::new(RegionExpression {to: Region::Layers}),
            right: Some(Box::new(OrganismExpression {
                child: Box::new(DrillExpression::new()),
                right: Some(Box::new(OrganismExpression {
                    child: Box::new(DrillExpression::new()),
                    right: Some(Box::new(OrganismExpression {
                        child: Box::new(DrillExpression::new()),
                        right: Some(Box::new(OrganismExpression {
                            child: Box::new(LeachExpression {
                                left: Box::new(PrimitiveExpression {pval: PrimitiveValue::Input}),
                                right: Some(Box::new(LeachExpression {
                                    left: Box::new(CellExpression {ident: 0}),
                                    right: None,
                                    is_chain: false,
                                    region_change: None
                                })),
                                is_chain: false,
                                region_change: Some(vec![RegionExpression::new(Region::Cells)]),
                            }),
                            right: Some(Box::new(OrganismExpression {
                                child: Box::new(LeachExpression {
                                    left: Box::new(CellExpression {ident: 0}),
                                    right: Some(Box::new(LeachExpression {
                                        left: Box::new(CellExpression {ident: 1}),
                                        right: None, is_chain: false, region_change: None 
                                    })),
                                    is_chain: true,
                                    region_change: None
                                }),
                                right: Some(Box::new(OrganismExpression {
                                    child: Box::new(RegionExpression {to: Region::Layers}),
                                    right: Some(Box::new(OrganismExpression {
                                        child: Box::new(LeachExpression {
                                            left: Box::new(PrimitiveExpression {pval: PrimitiveValue::Output}),
                                            right: Some(Box::new(LeachExpression {
                                                left: Box::new(CellExpression {ident: 2}),
                                                right: None,
                                                is_chain: false,
                                                region_change: None
                                            })),
                                            is_chain: false,
                                            region_change: Some(vec![RegionExpression::new(Region::Cells)])
                                        }),
                                        right: Some(Box::new(OrganismExpression {
                                            child: Box::new(LeachExpression {
                                                left: Box::new(CellExpression {ident: 2}),
                                                right: Some(Box::new(LeachExpression {
                                                    left: Box::new(CellExpression {ident: 1}),
                                                    right: None,
                                                    is_chain: false,
                                                    region_change: None
                                                })),
                                                is_chain: true,
                                                region_change: None
                                            }),
                                            right: None
                                        }))
                                    }))
                                }))
                            }))
                        }))
                    }))
                })) 
            })) 
        };
        assert!(result.is_ok());
        let result = result.unwrap().0;
        assert_eq!(result, expected_result);

    }
}