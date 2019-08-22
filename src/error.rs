//! Error handling types and convenience methods

use std::fmt;
use std::num::ParseFloatError;
use std::str::Utf8Error;

use crate::ast::AstError;
use crate::eval::InterpreterError;
use crate::token::TokenError;

/// The crate-wide error type
pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    Token(TokenError),
    Ast(AstError),
    Interpreter(InterpreterError),
    Utf8(Utf8Error),
    ParseFloat(ParseFloatError),
    NotEnoughArguments(usize, usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Token(e) => fmt::Display::fmt(e, f),
            Error::Ast(e) => fmt::Display::fmt(e, f),
            Error::Interpreter(e) => fmt::Display::fmt(e, f),
            Error::Utf8(e) => fmt::Display::fmt(e, f),
            Error::ParseFloat(e) => fmt::Display::fmt(e, f),
            Error::NotEnoughArguments(expected, got) => {
                write!(f, "expected {} arguments but got {}", expected, got)
            }
        }
    }
}

impl From<TokenError> for Error {
    fn from(error: TokenError) -> Self {
        Error::Token(error)
    }
}

impl From<AstError> for Error {
    fn from(error: AstError) -> Self {
        Error::Ast(error)
    }
}

impl From<InterpreterError> for Error {
    fn from(error: InterpreterError) -> Self {
        Error::Interpreter(error)
    }
}

impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Error::Utf8(error)
    }
}

impl From<ParseFloatError> for Error {
    fn from(error: ParseFloatError) -> Self {
        Error::ParseFloat(error)
    }
}
