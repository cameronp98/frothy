//! # Frothy
//! A forth-like programming language based around postfix expressions
//!
//! TODO use generic binary op enum variant instead of `Add`, `Multiply` etc.

use eval::Interpreter;

pub mod ast;
pub mod error;
pub mod eval;
pub mod token;
pub mod util;

// execute a Frothy program
pub fn exec(program: &str) -> error::Result<()> {
    let interpreter = Interpreter::new();

    println!("{:?}", interpreter.interpret(program));

    Ok(())
}
