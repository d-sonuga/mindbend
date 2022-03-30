#[macro_use]
extern crate clap;
use std::{fs, process, io};
use clap::{App, Arg};

mod lexer;
mod errors;
mod parser;
mod codegen;
#[cfg(test)]
mod tests;

pub const DEFAULT_OUT_FILENAME: &'static str = "out";

fn main(){
    let args = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("input file")
                .help("Source file to be compiled")
                .required(true)
                .index(1)
        )
        .arg(
            Arg::with_name("output file")
                .help("Name of output file")
                .long("output")
                .short("o")
                .required(false)
                .default_value(DEFAULT_OUT_FILENAME)
        )
        .get_matches();
    let input_file = args.value_of("input file").unwrap();
    let input: String;
    match fs::read_to_string(&input_file){
        Ok(source) => {
            if source.len() == 0 {
                eprintln!("The input file is empty");
                process::exit(1);
            }
            input = source;
        },
        Err(e) => {
            match e.kind() {
                io::ErrorKind::NotFound => eprintln!("The input file doesn't exist"),
                _ => eprintln!("An error occured. (It's most likely your fault)")
            };
            process::exit(1);
        }
    };
    let tokens: Vec<lexer::Token>;
    match lexer::tokenize(&input){
        Ok(t) => tokens = t,
        Err(err) => {
            eprintln!("{}", err.as_str());
            process::exit(1);
        }
    };
    let mut parser = parser::Parser::new(tokens);
    let ast: parser::OrganismExpression;
    let labels: Vec<String>;
    match parser.parse(){
        Ok(oe) => {
            ast = oe.0;
            labels = oe.1;
        },
        Err(err) => {
            eprintln!("{}", err.as_str());
            process::exit(1);
        }
    };
    let mut codegen = codegen::CodeGen::new(ast, labels);
    match codegen.code(){
        Ok(()) => (),
        Err(err) => {
            eprintln!("{}", err.as_str());
            process::exit(1);
        }
    };
    let out_filename = args.value_of("output file").unwrap();
    codegen.write_code_to_file(out_filename);
}
