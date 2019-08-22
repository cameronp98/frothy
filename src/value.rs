//! value.rs

use std::ops;
use std::fmt;

use crate::token::Token;

/// A `frothy` value that can be used at runtime
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Ident(String),
    Nil,
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

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => fmt::Display::fmt(n, f),
            Value::Boolean(b) => fmt::Display::fmt(b, f),
            Value::Ident(i) => write!(f, "Ident({})", i),
            Value::Nil => write!(f, "Nil"),
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
