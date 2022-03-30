use std::fs;
use assert_cmd::Command;
use assert_cmd::prelude::*;
use crate::DEFAULT_OUT_FILENAME;

const BASE_DIR: &'static str = "sampleprog";

macro_rules! file {
    ($name:expr) => {
        format!("{}/{}", BASE_DIR, $name)
    };
}

macro_rules! default_out_filepath {
    () => {
        format!("./{}", DEFAULT_OUT_FILENAME)
    };
}

macro_rules! compile {
    (name => $filename:expr, stdout => $expected_out:expr $(,$arg:expr => $value:expr),*) => {
        Command::cargo_bin("mindbend")
            .unwrap()
            .arg(file!($filename))
            $(
                .arg(format!("{}{}", $arg, $value))
            )*
            .assert()
            .success()
            .stdout($expected_out);
    };
}

macro_rules! ecompile {
    (name => $filename: expr, stderr => $expected_out:expr) => {
        Command::cargo_bin("mindbend")
        .unwrap()
        .arg(file!($filename))
        .assert()
        .failure()
        .stderr($expected_out);
    };
    (name => $filename: expr, stdout => $expected_out:expr) => {
        Command::cargo_bin("mindbend")
        .unwrap()
        .arg(file!($filename))
        .assert()
        .failure()
        .stderr($expected_out);
    };
}

macro_rules! run {
    (stdout => $expected_out:expr) => {
        Command::new(default_out_filepath!())
            .unwrap()
            .assert()
            .success()
            .stdout($expected_out);
    };
    (input => $input:expr, stdout => $expected_out:expr) => {
        Command::new(default_out_filepath!())
            .write_stdin($input)
            .unwrap()
            .assert()
            .success()
            .stdout($expected_out);
    };
}

macro_rules! erun {
    (stdout => $expected_out:expr) => {
        let err = Command::new(default_out_filepath!())
            .unwrap_err();
        let stdout = &err.as_output().unwrap().stdout;
        let out = String::from_utf8_lossy(stdout).into_owned();
        let out = out.as_str();
        assert_eq!(out, $expected_out);
    };
}

#[test]
fn print1(){
    let filename = "print1.mb";
    compile!(name => filename, stdout => "");
    run!(stdout => "1");
}

#[test]
fn print1to5(){
    let filename = "print1to5.mb";
    compile!(name => filename, stdout => "");
    run!(stdout => "12345");
}

#[test]
fn print2(){
    let filename = "print2.mb";
    compile!(name => filename, stdout => "");
    run!(stdout => "2");
}

#[test]
fn print_capital_a(){
    let filename = "printA.mb";
    compile!(name => filename, stdout => "");
    run!(stdout => "A");
}

#[test]
fn jump_to_nonexistent_label(){
    let filename = "jump_to_non_existent_label.mb";
    ecompile!(
        name => filename,
        stderr => "Attempt to jump to non existent label at the nth token, where n is around 1\n"
    );
}

#[test]
fn chain_leach_without_ending_massacre(){
    let filename = "chain_leach_without_ending_massacre.mb";
    ecompile!(
        name => filename,
        stderr => "Chained leach expression at the nth token, \
            where n is around 5, does not end in a massacre. \
            A chained leach expression must end in a massacre\n"
    );
}

#[test]
fn non_existent_file(){
    let filename = "non_existent_file...........";
    ecompile!(
        name => filename,
        stderr => "The input file doesn't exist\n"
    );
}

#[test]
fn attempt_to_use_expr_in_arg_cell_after_massacre(){
    let filename = "attempt_to_use_expr_in_arg_cell_after_massacre.mb";
    compile!(name => filename, stdout => "");
    erun!(stdout => "2Attempt to leach death expression onto another Cell\n");
}

#[test]
fn attempt_to_leach_expr_onto_cell_after_leached_away(){
    let filename = "attempt_to_leach_expr_onto_cell_after_leached_away.mb";
    compile!(name => filename, stdout => "");
    erun!(stdout => "Attempt to leach death expression onto another Cell\n");
}

#[test]
fn attempt_to_drill_in_cells_region(){
    let filename = "attempt_to_drill_in_cells_region.mb";
    ecompile!(
        name => filename,
        stderr => "Attempt to drill in the Cells Region at the nth token, where n is around 1\n"
    );
}

#[test]
fn attempt_to_access_cell_in_layers_region(){
    let filename = "attempt_to_access_cell_in_layers_region.mb";
    compile!(name => filename, stdout => "");
    erun!(stdout => "Attempting to access cell outside the Cells Region\n");
}

#[test]
fn attempt_to_access_cell_in_layers_region2(){
    let filename = "attempt_to_access_cell_in_layers_region2.mb";
    compile!(name => filename, stdout => "");
    erun!(stdout => "Attempting to access cell outside the Cells Region\n");
}

#[test]
fn attempt_non_function_primitive_massacre(){
    let filename = "attempt_non_function_primitive_massacre.mb";
    compile!(name => filename, stdout => "");
    erun!(stdout => "Attempt to use non-function primitive to massacre\n");
}

#[test]
fn attempt_access_primitive_when_gates_closed(){
    let filename = "attempt_access_primitive_when_gates_closed.mb";
    ecompile!(
        name => filename,
        stderr => "Attempting to access primitive when the Layers gates \
        aren\'t fully open at the nth token, where n is around 2\n"
    );
}

#[test]
fn attempt_access_primitive_in_cells_region(){
    let filename = "attempt_access_primitive_in_cells_region.mb";
    ecompile!(
        name => filename,
        stderr => "Attempting to access primitive outside the Layers Region \
            at the nth token, where n is around 1\n"
    );
}

#[test]
fn attempt_access_primitive_gates_closed(){
    let filename = "attempt_access_primitive_gates_closed.mb";
    compile!(name => filename, stdout => "");
    erun!(stdout => "Attempt to access primitive when the gates aren't open\n");
}

#[test]
fn accept_a_num_and_print(){
    let filename = "accept_a_num_and_print.mb";
    let sample_input = "1";
    compile!(name => filename, stdout => "");
    run!(input => sample_input, stdout => sample_input);
}

#[test]
fn malformed_leach_expression(){
    let filename = "malformed_leach_expression.mb";
    ecompile!(
        name => filename,
        stderr => "The leach expression at the nth token, where n is around 11, \
            does not begin with a primitive or Cell\n"
    );
}

#[test]
fn attempt_to_repeat_cell_in_leach(){
    let filename = "attempt_to_repeat_cell_in_leach.mb";
    ecompile!(
        name => filename,
        stderr => "Attempt to leach expression onto itself at the nth token, where n is around 11\n"
    );
}

#[test]
fn attempt_repeat_cell_in_chain_leach(){
    let filename = "attempt_repeat_cell_in_chain_leach.mb";
    ecompile!(
        name => filename,
        stderr => "Attempt to leach expression onto itself at the nth token, where n is around 15\n"
    );
}

#[test]
fn empty_file(){
    let filename = "empty_file.mb";
    ecompile!(
        name => filename,
        stderr => "The input file is empty\n"
    );
}

#[test]
fn compile_with_alternate_output_filename(){
    let filename = "print1.mb";
    let out_filename = "alternate_out_name";
    compile!(
        name => filename,
        stdout => "",
        "-o" => out_filename
    );
    let open_file_attempt = fs::File::open(out_filename);
    assert!(open_file_attempt.is_ok());
}