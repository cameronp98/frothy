//! Evaluate an [`Ast`][Ast] to produce values and console output

use std::collections::HashMap;
use std::fmt;
use std::ops;

use crate::ast::{Ast, Parser};
use crate::ast::Literal;
use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct Interpreter {
    stack: Vec<Value>,
    vars: HashMap<String, Value>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            stack: vec![],
            vars: HashMap::new(),
        }
    }

    fn eval(&mut self, ast: &Ast) -> Result<Value> {
        match ast {
            Ast::Literal(lit) => Ok(lit.clone().into()),
            Ast::Add(a, b) => Ok(self.eval(a)? + self.eval(b)?),
            Ast::Subtract(a, b) => Ok(self.eval(a)? - self.eval(b)?),
            Ast::Multiply(a, b) => Ok(self.eval(a)? * self.eval(b)?),
            Ast::Divide(a, b) => Ok(self.eval(a)? / self.eval(b)?),
            Ast::Assign(ident, ast) => {
                let value = self.eval(ast)?;
                self.vars.insert(ident.clone(), value);
                Ok(Value::Nil)
            }
            Ast::Block(asts) => {
                let mut value = Value::Nil;

                for ast in asts {
                    value = self.eval(ast)?;
                }

                Ok(value)
            }
            Ast::Func(asts) => Ok(Value::Func(asts.clone())),
            Ast::Ident(ident) => {
                // TODO figure out how variable values should be referenced. At the moment we just clone
                self.vars
                    .get(ident)
                    .ok_or(InterpreterError::VariableUndefined(ident.clone()).into())
                    .map(|v| v.clone())
            }
        }
    }

    pub fn interpret(mut self, program: &str) -> Result<Vec<Value>> {
        let parser = Parser::new(program);

        for ast in parser.parse()? {
            self.eval(&ast)?;
        }

        Ok(self.stack)
    }
}

#[derive(Debug, Clone)]
pub enum InterpreterError {
    VariableUndefined(String),
}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InterpreterError::VariableUndefined(ident) => {
                write!(f, "undefined variable '{}'", ident)
            }
        }
    }
}

/// A `frothy` value that can be used at runtime
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Nil,
    Func(Vec<Ast>),
}

impl Value {
    /// Determine if two values are equal to each other
    pub fn eq(&self, other: &Self) -> Value {
        match (self, other) {
            (Value::Number(lhs), Value::Number(rhs)) => Value::Boolean(lhs == rhs),
            (Value::Boolean(lhs), Value::Boolean(rhs)) => Value::Boolean(lhs == rhs),
            _ => Value::Nil,
        }
    }

    /// Determine if two values are not equal to each other
    pub fn neq(&self, other: &Self) -> Value {
        if let Value::Boolean(b) = self.eq(other) {
            Value::Boolean(!b)
        } else {
            Value::Nil
        }
    }
}

impl From<Literal> for Value {
    fn from(lit: Literal) -> Self {
        match lit {
            Literal::Boolean(b) => Value::Boolean(b),
            Literal::Number(n) => Value::Number(n),
            Literal::Nil => Value::Nil,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => fmt::Display::fmt(n, f),
            Value::Boolean(b) => fmt::Display::fmt(b, f),
            Value::Nil => write!(f, "Nil"),
            Value::Func(_) => write!(f, "Func(?)"),
        }
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Value {
        Value::Number(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Value {
        Value::Boolean(value)
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(value: Option<T>) -> Value {
        if let Some(v) = value {
            v.into()
        } else {
            Value::Nil
        }
    }
}

impl ops::Add for Value {
    type Output = Value;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs + rhs),
            _ => Value::Nil,
        }
    }
}

impl ops::Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs - rhs),
            _ => Value::Nil,
        }
    }
}

impl ops::Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs * rhs),
            _ => Value::Nil,
        }
    }
}

impl ops::Div for Value {
    type Output = Value;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs / rhs),
            _ => Value::Nil,
        }
    }
}
