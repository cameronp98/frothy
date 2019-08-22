//! frothy
//!
//! TODO: upgrade error handling
//! TODO: finish parse functions
//! TODO: add 'near x' parse error messages by keeping token position and last token

use std::io::prelude::*;
use std::fs::File;
use std::env;
use std::collections::HashMap;

mod token;
mod value;
mod ast;
mod util;
mod error;

use ast::{Parser, Ast};
use token::{Token, Tokens};
use value::Value;

pub fn exec(program: &str) -> Result<(), String> {
    let nodes = Parser::new(program).parse()?;

    println!("{:?}", nodes);

    Ok(())
}

fn main() {
// read the file at the path argument
    let path = env::args().nth(1).expect("usage: frothy <path>");
    let mut file = File::open(path).expect("file not found");
    let mut program = String::new();
    file.read_to_string(&mut program).expect("failed to read file");

    match exec(&program) {
        Ok(_) => {},
        Err(e) => eprintln!("error = {:?}", e),
    }
}