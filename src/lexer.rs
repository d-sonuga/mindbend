use std::str::Chars;
use std::iter::Peekable;
use crate::errors;
use self::Token::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    TripleSix,
    TripleSixEq,
    TripleSixEqM,
    TripleSixEqO,
    Tilde,
    Drill,
    Label(String),
    Jump(String),
    ConditionalJump(String),
    CellIdent(String),
    RegionIdent(String),
    PrimitiveIdent(String)
}

#[macro_export]
macro_rules! p_ident {
    ($x:expr) => {
        PrimitiveIdent(String::from($x))
    }
}

#[macro_export]
macro_rules! c_ident {
    ($x:expr) => {
        Token::CellIdent(String::from($x))
    }
}

#[macro_export]
macro_rules! r_ident {
    ($x:expr) => {
        Token::RegionIdent(String::from($x))
    }
}

impl Token {
    pub fn is_region(&self) -> bool {
        match self {
            Token::RegionIdent(_) => true,
            _ => false
        }
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens: Vec<Token> = vec![];
    let mut input = input.chars().peekable();
    let mut pos = 0;
    while input.peek().is_some(){
        let c = input.next().unwrap();
        pos += 1;
        match c {
            ' ' | '\n' => return Err(errors::err_whitespace(pos)),
            '~' => tokens.push(Token::Tilde),
            '$' => match tokenize_primitive_ident(&mut tokens, &mut input, &mut pos){
                Ok(()) => (),
                Err(err) => return Err(err)
            },
            '^' => {
                match tokenize_666(&mut tokens, &mut input, &mut pos){
                    Ok(()) => (),
                    Err(err) => return Err(err)
                };
            },
            '-' => {
                match tokenize_right_arrow_and_region_ident(&mut tokens, &mut input, &mut pos){
                    Ok(()) => (),
                    Err(err) => return Err(err)
                };
            },
            '\\' => {
                match tokenize_drill(&mut tokens, &mut input, &mut pos){
                    Ok(()) => (),
                    Err(err) => return Err(err)
                };
            },
            'l' => {
                match tokenize_label(&mut tokens, &mut input, &mut pos){
                    Ok(()) => (),
                    Err(err) => return Err(err)
                };
            },
            'i' => {
                match tokenize_ijmp(&mut tokens, &mut input, &mut pos){
                    Ok(()) => (),
                    Err(err) => return Err(err)
                };
            },
            'j' => {
                match tokenize_jmp(&mut tokens, &mut input, &mut pos){
                    Ok(()) => (),
                    Err(err) => return Err(err)
                };
            },
            '0'..='9' | 'A'..='E' => tokens.push(Token::CellIdent(c.to_string())),
            _ => return Err(errors::err_unrecognized_token(pos))
        }
    }
    if tokens.pop().unwrap() != Token::TripleSixEqO {
        Err(errors::err_org_expr_must_end_in_death())
    } else {
        Ok(tokens)
    }
}

fn tokenize_primitive_ident(tokens: &mut Vec<Token>, input: &mut Peekable<Chars>, pos: &mut i32) -> Result<(), String> {
    if input.peek().is_none(){
        return Err(errors::err_expected(*pos, "primitive identifier or index"));
    }
    *pos += 1;
    let c = input.next().unwrap();
    match c {
        '!' | '@' | '#' | '+' | '%' | '`' | '&' | '*' | '(' | ')' |
        '}' | '{' | '0'..='9' | 'A'..='D' => {
            tokens.push(p_ident!(c));
            Ok(())
        },
        '>' => {
            if input.peek().is_some(){
                *pos += 1;
                let next_char = input.next().unwrap();
                match next_char {
                    '<' => {
                        tokens.push(p_ident!("><"));
                        Ok(())
                    },
                    _ => Err(errors::err_unrecognized_token(*pos))
                }
            } else {
                Err(errors::err_unrecognized_token(*pos))
            }
        },
        '<' => {
            if input.peek().is_some(){
                *pos += 1;
                let next_char = input.next().unwrap();
                match next_char {
                    '>' => {
                        tokens.push(p_ident!("<>"));
                        Ok(())
                    },
                    _ => Err(errors::err_unrecognized_token(*pos))
                }
            } else {
                Err(errors::err_unrecognized_token(*pos))
            }
        },
        _ => Err(errors::err_invalid_primitive(*pos))
    }
}

fn tokenize_666(tokens: &mut Vec<Token>, mut input: &mut Peekable<Chars>, mut pos: &mut i32) -> Result<(), String> {
    // 5 because the first has already been checked in the main loop    
    match check_for_n_carets(&mut input, &mut pos, 5){
        Ok(()) => (),
        Err(err) => return Err(err)
    };
    match check_for_3_sixes(&mut input, &mut pos){
        Ok(()) => (),
        Err(err) => return Err(err)
    };
    match check_for_n_carets(&mut input, &mut pos, 6){
        Ok(()) => (),
        Err(err) => return Err(err)
    };
    if input.peek().is_some() && *input.peek().unwrap() == '=' {
        *pos += 1;
        input.next();
        if input.peek().is_some(){
            let next_token_ref = input.peek().unwrap();
            if *next_token_ref == 'M' {
                input.next();
                tokens.push(Token::TripleSixEqM);
            } else if *next_token_ref == 'O' {
                input.next();
                tokens.push(Token::TripleSixEqO);
            } else {
                tokens.push(Token::TripleSixEq);
            }
        } else {
            tokens.push(Token::TripleSixEq);
        }
    } else {
        tokens.push(Token::TripleSix);
    }
    Ok(())
}

fn check_for_n_carets(input: &mut &mut Peekable<Chars>, pos: &mut &mut i32, n: i32) -> Result<(), String> {
    for _ in 0..n {
        if input.peek().is_none(){
            return Err(errors::err_unrecognized_token(**pos));
        }
        **pos += 1;
        let next_char = input.next().unwrap();
        if next_char != '^' {
            return Err(errors::err_unrecognized_token(**pos));
        }
    }
    Ok(())
}

fn check_for_3_sixes(input: &mut &mut Peekable<Chars>, pos: &mut &mut i32) -> Result<(), String> {
    for _ in 0..3 {
        if input.peek().is_none(){
            return Err(errors::err_unrecognized_token(**pos));
        }
        **pos += 1;
        let next_char = input.next().unwrap();
        if next_char != '6' {
            return Err(errors::err_unrecognized_token(**pos));
        }
    }
    Ok(())
}

fn tokenize_right_arrow_and_region_ident(tokens: &mut Vec<Token>, input: &mut Peekable<Chars>, pos: &mut i32) -> Result<(), String> {
    if input.peek().is_some(){
        *pos += 1;
        match input.next(){
            Some('>') => {
                if input.peek().is_some() {
                    let next_char = input.peek().unwrap();
                    if *next_char == 'L' || *next_char == 'C' {
                        *pos += 1;
                        let next_char = input.next().unwrap();
                        tokens.push(Token::RegionIdent(next_char.to_string()));
                    }
                }
            },
            _ => return Err(errors::err_unrecognized_token(*pos))
        }
    } else {
        return Err(errors::err_unrecognized_token(*pos));
    }
    Ok(())
}

fn tokenize_drill(tokens: &mut Vec<Token>, input: &mut Peekable<Chars>, pos: &mut i32) -> Result<(), String> {
    match input.next(){
        Some('\\') => (),
        _ => return Err(errors::err_unrecognized_token(*pos))
    };
    *pos += 1;
    match input.next(){
        Some('|') => (),
        _ => return Err(errors::err_unrecognized_token(*pos))
    };
    *pos += 1;
    match input.next(){
        Some('/') => (),
        _ => return Err(errors::err_unrecognized_token(*pos))
    };
    *pos += 1;
    match input.next(){
        Some('/') => (),
        _ => return Err(errors::err_unrecognized_token(*pos))
    };
    tokens.push(Token::Drill);
    Ok(())
}

fn tokenize_jmp(tokens: &mut Vec<Token>, input: &mut Peekable<Chars>, pos: &mut i32) -> Result<(), String> {
    let expected_chars = ['m', 'p', ':'];
    for expected_char in expected_chars {
        if input.peek().is_none(){
            return Err(errors::err_expected(*pos, expected_char));
        }
        match input.next(){
            Some(c) => {
                if c != expected_char {
                    return Err(errors::err_unrecognized_token(*pos));
                }
            },
            _ => return Err(errors::err_unrecognized_token(*pos))
        };
        *pos += 1;
    }
    let mut label = String::new();
    while input.peek().is_some(){
        let c = input.next().unwrap();
        if c != ':' {
            label.push(c);
        } else {
            tokens.push(Token::Jump(label));
            return Ok(());
        }
        *pos += 1;
    }
    return Err(errors::err_expected(*pos, ":"))
}

fn tokenize_label(tokens: &mut Vec<Token>, input: &mut Peekable<Chars>, pos: &mut i32) -> Result<(), String> {
    let expected_chars = String::from("abel:");
    let expected_chars = expected_chars.chars();
    for expected_char in expected_chars {
        if input.peek().is_none(){
            return Err(errors::err_expected(*pos, expected_char));
        }
        match input.next(){
            Some(c) => {
                if c != expected_char {
                    return Err(errors::err_expected(*pos, expected_char));
                }
            },
            _ => return Err(errors::err_expected(*pos, expected_char))
        };
        *pos += 1;
    }
    let mut label = String::new();
    while input.peek().is_some(){
        let next_char = input.next().unwrap();
        if next_char == ':' {
            tokens.push(Token::Label(label));
            return Ok(());
        } else {
            label.push(next_char);
        }
        *pos += 1;
    }
    Err(errors::err_expected(*pos, ":"))
}

fn tokenize_ijmp(tokens: &mut Vec<Token>, input: &mut Peekable<Chars>, pos: &mut i32) -> Result<(), String> {
    *pos += 1;
    let expected_chars = ['j', 'm', 'p', ':'];

    for expected_char in expected_chars {
        if input.peek().is_none(){
            return Err(errors::err_expected(*pos, expected_char));
        }
        match input.next(){
            Some(c) => {
                if c != expected_char {
                    return Err(errors::err_unrecognized_token(*pos));
                }
            },
            _ => return Err(errors::err_unrecognized_token(*pos))
        };
        *pos += 1;
    }
    let mut label = String::new();
    while input.peek().is_some(){
        let c = input.next().unwrap();
        if c != ':' {
            label.push(c);
        } else {
            tokens.push(Token::ConditionalJump(label));
            return Ok(());
        }
        *pos += 1;
    }
    return Err(errors::err_expected(*pos, ":"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{r_ident, p_ident, c_ident};
    #[test]
    fn test_valid1(){
        let input = r"$!$@$#$+$%$`$&$*$($)$<>$><${$}label:hello:^^^^^^666^^^^^^^^^^^^666^^^^^^=->L~\\|//jmp:hello:ijmp:hello:^^^^^^666^^^^^^=M^^^^^^666^^^^^^=O";
        let result = tokenize(input);
        let expected_result = Ok(vec![
            p_ident!("!"), p_ident!("@"), p_ident!("#"),
            p_ident!("+"), p_ident!("%"), p_ident!("`"),
            p_ident!("&"), p_ident!("*"), p_ident!("("),
            p_ident!(")"), p_ident!("<>"), p_ident!("><"),
            p_ident!("{"), p_ident!("}"), Token::Label(format!("hello")),
            Token::TripleSix, Token::TripleSixEq, r_ident!("L"),
            Token::Tilde, Token::Drill, Token::Jump(format!("hello")),
            Token::ConditionalJump(format!("hello")), Token::TripleSixEqM
        ]);
        assert!(result.is_ok());
        assert_eq!(result, expected_result);
    }
    #[test]
    fn test_valid2(){
        let input = "0123456789ABCDE^^^^^^666^^^^^^=O";
        let result = tokenize(input);
        let expected_result = Ok(vec![
            c_ident!('0'), c_ident!('1'), c_ident!('2'), c_ident!('3'),
            c_ident!('4'), c_ident!('5'), c_ident!('6'), c_ident!('7'),
            c_ident!('8'), c_ident!('9'), c_ident!('A'), c_ident!('B'),
            c_ident!('C'), c_ident!('D'), c_ident!('E')
        ]);
        assert!(result.is_ok());
        assert_eq!(result, expected_result);
    }
    #[test]
    fn test_valid3(){
        let input = "->C->L^^^^^^666^^^^^^=O";
        let result = tokenize(input);
        let expected_result = Ok(vec![
            r_ident!('C'), r_ident!('L')
        ]);
        assert!(result.is_ok());
        assert_eq!(result, expected_result);
    }
    #[test]
    fn test_valid4(){
        let input = "C^^^^^^666^^^^^^=O";
        let result = tokenize(input);
        let expected_result = Ok(vec![
            c_ident!('C')
        ]);
        assert!(result.is_ok());
        assert_eq!(result, expected_result);
    }
    #[test]
    fn test_valid5(){
        let input = "L^^^^^^666^^^^^^=O";
        let result = tokenize(input);
        let expected_result = Err(errors::err_unrecognized_token(1));
        assert!(result.is_err());
        assert_eq!(result, expected_result);
    }
    #[test]
    fn test_valid6(){
        let input = "$0$1$2$3$4$5$6$7$8$9$A$B$C$D^^^^^^666^^^^^^=O";
        let result = tokenize(input);
        let expected_result = Ok(vec![
            p_ident!('0'), p_ident!('1'), p_ident!('2'), p_ident!('3'),
            p_ident!('4'), p_ident!('5'), p_ident!('6'), p_ident!('7'),
            p_ident!('8'), p_ident!('9'), p_ident!('A'), p_ident!('B'),
            p_ident!('C'), p_ident!('D')
        ]);
        assert!(result.is_ok());
        assert_eq!(result, expected_result);
    }
    #[test]
    fn test_valid7(){
        let input = r"->L\\|//\\|//\\|//$`->C~0->L$><->C~10~1^^^^^^666^^^^^^=O";
        let result = tokenize(input);
        let expected_result = Ok(vec![
            r_ident!("L"), Token::Drill, Token::Drill, Token::Drill, p_ident!("`"),
            r_ident!("C"), Token::Tilde, c_ident!("0"), r_ident!("L"), p_ident!("><"),
            r_ident!("C"), Token::Tilde, c_ident!("1"), c_ident!("0"), Token::Tilde, c_ident!("1")
        ]);
        assert!(result.is_ok());
        assert_eq!(result, expected_result);
    }
    #[test]
    fn test_valid8(){
        let input = r"label:hello:->L\\|//\\|//\\|//$+->C~0->L$(->C~10~2->L\\|//->C1~32~4->L\\|//->C3~54~6->L\\|//->C5~76~8->L$><->C~99~7~8^^^^^^666^^^^^^=Mjmp:hello:^^^^^^666^^^^^^=O";
        let result = tokenize(input);
        let expected_result = Ok(vec![
            Token::Label(format!("hello")), r_ident!("L"), Token::Drill, Token::Drill,
            Token::Drill, p_ident!("+"), r_ident!("C"), Token::Tilde, c_ident!("0"), r_ident!("L"),
            p_ident!("("), r_ident!("C"), Token::Tilde, c_ident!("1"), c_ident!("0"), Token::Tilde,
            c_ident!("2"), r_ident!("L"), Token::Drill, r_ident!("C"), c_ident!("1"), Token::Tilde,
            c_ident!("3"), c_ident!("2"), Token::Tilde, c_ident!("4"), r_ident!("L"), Token::Drill,
            r_ident!("C"), c_ident!("3"), Token::Tilde, c_ident!("5"), c_ident!("4"), Token::Tilde,
            c_ident!("6"), r_ident!("L"), Token::Drill, r_ident!("C"), c_ident!("5"), Token::Tilde,
            c_ident!("7"), c_ident!("6"), Token::Tilde, c_ident!("8"), r_ident!("L"), p_ident!("><"),
            r_ident!("C"), Token::Tilde, c_ident!("9"), c_ident!("9"), Token::Tilde, c_ident!("7"),
            Token::Tilde, c_ident!("8"), TripleSixEqM, Jump(format!("hello"))
        ]);
        assert!(result.is_ok());
        assert_eq!(result, expected_result);
    }
    #[test]
    fn test_err1(){
        let input = "!@l";
        let result = tokenize(input);
        let expected_result = Err(errors::err_unrecognized_token(1));
        assert!(result.is_err());
        assert_eq!(result, expected_result);
    }
    #[test]
    fn test_err2(){
        let input = " ";
        let result = tokenize(input);
        let expected_result = Err(errors::err_whitespace(1));
        assert!(result.is_err());
        assert_eq!(result, expected_result);
    }
    #[test]
    fn test_err3(){
        let input = "AB E";
        let result = tokenize(input);
        let expected_result = Err(errors::err_whitespace(3));
        assert!(result.is_err());
        assert_eq!(result, expected_result);
    }
    #[test]
    fn test_err4(){
        let input = "$E";
        let result = tokenize(input);
        let expected_result = Err(errors::err_invalid_primitive(2));
        assert!(result.is_err());
        assert_eq!(result, expected_result);
    }
}
