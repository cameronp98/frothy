//! Evaluate an [`Ast`](../ast/enum.Ast.html) to produce values and console output
//!
//! TODO: add support for constants (e.g. PI 3.14 const =)
//! TODO: create Const(Ast) `Ast` variant
//! TODO: change `Context.vars` value type to (Ast, is_const: bool)
//! TODO: disallow assignment where is_const is true

use std::collections::HashMap;
use std::fmt;
use std::ops;

use crate::ast::{Ast, Parser};
use crate::ast::Literal;
use crate::error::Result;

/// A frothy evaluation context (variables)
#[derive(Debug)]
pub struct Context {
    vars: HashMap<String, Value>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            vars: HashMap::new(),
        }
    }

    pub fn lookup<T: Into<String>>(&self, ident: T) -> Result<Value> {
        let ident = ident.into();
        // TODO figure out how variable values should be referenced. At the moment we just clone
        self.vars.get(&ident)
            .map(|v| v.clone())
            .ok_or(InterpreterError::VariableUndefined(ident).into())
    }

    pub fn builtin_func<T: Into<String>>(&mut self, name: T, f: BuiltinFn) {
        let name = name.into();
        self.set(name.clone(), Value::BuiltinFunc(name, f));
    }

    pub fn set<T: Into<String>>(&mut self, ident: T, value: Value) {
        self.vars.insert(ident.into(), value);
    }
}

/// A frothy interpreter
///
/// Builtins are registered when the interpreter is created
#[derive(Debug)]
pub struct Interpreter {
    ctx: Context,
}

impl Interpreter {
    /// Create a new frothy interpreter and register builtins
    pub fn new() -> Interpreter {
        // set up builtins
        let mut ctx = Context::new();

        // print function
        ctx.builtin_func("print", |ctx| {
            println!("{}", ctx.lookup("print_arg")?);
            Ok(Value::Nil)
        });

        // pi constant
        ctx.set("PI", Value::Number(::std::f64::consts::PI));

        Interpreter {
            ctx,
        }
    }

    fn eval(&mut self, ast: &Ast) -> Result<Value> {
        match ast {
            Ast::Literal(lit) => Ok(lit.clone().into()),
            Ast::Add(a, b) => Ok(self.eval(a)? + self.eval(b)?),
            Ast::Subtract(a, b) => Ok(self.eval(a)? - self.eval(b)?),
            Ast::Multiply(a, b) => Ok(self.eval(a)? * self.eval(b)?),
            Ast::Divide(a, b) => Ok(self.eval(a)? / self.eval(b)?),
            // assignment returns `Nil`
            Ast::Assign(ident, ast) => {
                let value = self.eval(ast)?;
                self.ctx.set(ident.clone(), value);
                Ok(Value::Nil)
            }
            // Block returns the result of the last `Ast` to execute successfully
            Ast::Block(asts) => self.eval_block(asts),

            Ast::Func(asts) => Ok(Value::Func(asts.clone())),
            Ast::Call(ast) => {
                let value = self.eval(ast)?;
                self.call(&value)
            },
            Ast::Ident(ident) => self.ctx.lookup(ident),
        }
    }

    pub fn interpret(mut self, program: &str) -> Result<Vec<Value>> {
        let parser = Parser::new(program);
        parser.parse()?.iter().map(|ast| self.eval(ast)).collect()
    }

    fn eval_block(&mut self, asts: &Vec<Ast>) -> Result<Value> {
        let mut value = Value::Nil;

        for ast in asts {
            value = self.eval(ast)?;
        }

        Ok(value)
    }

    fn call(&mut self, value: &Value) -> Result<Value> {
        match value {
            Value::Func(asts) => self.eval_block(asts),
            Value::BuiltinFunc(_, f) => f(&self.ctx),
            _ => Err(InterpreterError::NotCallable(format!("{}", value)).into()),
        }
    }
}

/// Errors encountered while interpreting an [`Ast`](../ast/enum.Ast.html)
#[derive(Debug, Clone)]
pub enum InterpreterError {
    VariableUndefined(String),
    NotCallable(String),
}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InterpreterError::VariableUndefined(ident) => {
                write!(f, "undefined variable '{}'", ident)
            }
            InterpreterError::NotCallable(displayed) => {
                write!(f, "value '{}' is not callable", displayed)
            }
        }
    }
}

/// A builtin function
pub type BuiltinFn = fn(&Context) -> Result<Value>;

/// A `frothy` value that can be used at runtime
#[derive(Clone)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Nil,
    Func(Vec<Ast>),
    BuiltinFunc(String, BuiltinFn),
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


// formatting impls

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => fmt::Display::fmt(n, f),
            Value::Boolean(b) => fmt::Display::fmt(b, f),
            Value::Nil => write!(f, "Nil"),
            Value::Func(_) => f.write_str("<fn>"),
            Value::BuiltinFunc(name, _) => write!(f, "<builtin-fn:{}>", name),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Boolean(b) => f.debug_tuple("Boolean").field(b).finish(),
            Value::Number(n) => f.debug_tuple("Number").field(n).finish(),
            Value::Func(asts) => f.debug_tuple("Func").field(asts).finish(),
            Value::BuiltinFunc(name, _) => f.debug_tuple("BuiltinFunc").field(name).finish(),
            Value::Nil => f.write_str("Nil"),
        }
    }
}


// conversion from literal to `Value`

impl From<Literal> for Value {
    fn from(lit: Literal) -> Self {
        match lit {
            Literal::Boolean(b) => Value::Boolean(b),
            Literal::Number(n) => Value::Number(n),
            Literal::Nil => Value::Nil,
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


// operation impls

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
