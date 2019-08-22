type Result<T> = ::std::result::Result<T, Error>;

use crate::token::TokenError;

#[derive(Debug, Clone)]
enum Error {
    Token(TokenError),
//    Parse(ParseError),
}

