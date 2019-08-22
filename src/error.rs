//! Error handling types and convenience methods

use std::fmt;

use crate::ast::AstError;
use crate::eval::InterpreterError;
use crate::token::TokenError;

/// The crate-wide error type
pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    TokenError(TokenError),
    AstError(AstError),
    InterpreterError(InterpreterError),
    NotEnoughArguments(usize, usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::TokenError(e) => fmt::Display::fmt(e, f),
            Error::AstError(e) => fmt::Display::fmt(e, f),
            Error::InterpreterError(e) => fmt::Display::fmt(e, f),
            Error::NotEnoughArguments(expected, got) => {
                write!(f, "expected {} arguments but got {}", expected, got)
            }
        }
    }
}

impl From<TokenError> for Error {
    fn from(error: TokenError) -> Self {
        Error::TokenError(error)
    }
}

impl From<AstError> for Error {
    fn from(error: AstError) -> Self {
        Error::AstError(error)
    }
}

impl From<InterpreterError> for Error {
    fn from(error: InterpreterError) -> Self {
        Error::InterpreterError(error)
    }
}
