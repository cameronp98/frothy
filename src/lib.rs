//! # Frothy
//! A postfix expression language loosely inspired by Forth
//!
//! TODO use generic binary op enum variant instead of `Add`, `Multiply` etc.

use eval::Interpreter;

use crate::error::Result;
use crate::eval::Value;

pub mod ast;
pub mod error;
pub mod eval;
pub mod token;
pub mod util;

pub fn eval(program: &str) -> Result<Vec<Value>> {
    Interpreter::new().interpret(program)
}
