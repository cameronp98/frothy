//! Types and methods for parsing a frothy program into [`Token`](enum.Token.html)s

use std::fmt;
use std::str;

use crate::error::Result;

/// Errors produced whilst reading tokens
#[derive(Debug, Clone)]
pub enum TokenError {
    Unexpected(u8),
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenError::Unexpected(byte) => {
                write!(f, "unexpected 0x{:02x}", byte)?;
                if byte.is_ascii() {
                    write!(f, "('{}')", char::from(*byte))?;
                }
                Ok(())
            }
        }
    }
}


/// A frothy token
#[derive(Debug, Clone)]
pub enum Token {
    Ident(String),
    Number(f64),
    Plus,
    Minus,
    Multiply,
    Divide,
    Equals,
    Assign,
    Modulo,
    OpenBrace,
    CloseBrace,
    CreateFunction,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Ident(ident) => f.write_str(ident),
            Token::Number(number) => write!(f, "{}", number),
            Token::Plus => f.write_str("+"),
            Token::Minus => f.write_str("-"),
            Token::Multiply => f.write_str("*"),
            Token::Divide => f.write_str("/"),
            Token::Equals => f.write_str("=="),
            Token::Assign => f.write_str("="),
            Token::Modulo => f.write_str("%"),
            Token::OpenBrace => f.write_str("{"),
            Token::CloseBrace => f.write_str("}"),
            Token::CreateFunction => f.write_str("fn"),
        }
    }
}


/// Parse a frothy program into [`Token`](enum.Token.html)s
#[derive(Debug, Clone)]
pub struct Tokens<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Tokens<'a> {
    /// Create an `Iterator<Item = Token>` for the given input program
    pub fn new(input: &'a str) -> Tokens<'a> {
        Tokens {
            input: input.as_bytes(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<u8> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    // try to go back if the targeted position is inside the input buffer,
    // otherwise do nothing and return None
    fn back(&mut self) -> Option<u8> {
        if self.pos > 0 {
            self.pos -= 1;
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    // move to the next position and return the byte there
    fn next_byte(&mut self) -> Option<u8> {
        if let Some(byte) = self.peek() {
            self.pos += 1;
            Some(byte)
        } else {
            None
        }
    }

    fn next_byte_if<F: Fn(&u8) -> bool>(&mut self, f: F) -> Option<u8> {
        self.peek().and_then(|b| {
            if f(&b) {
                self.next_byte();
                Some(b)
            } else {
                None
            }
        })
    }

    fn next_byte_while<F: Fn(&u8) -> bool>(&mut self, f: F) -> &[u8] {
        let start = self.pos;
        while self.next_byte_if(&f).is_some() {}
        &self.input[start..self.pos]
    }

    fn next_number(&mut self) -> Result<f64> {
        let sign = if self.next_byte_if(|&b| b == b'-').is_some() {
            -1.0
        } else {
            1.0
        };

        let result: f64 = str::from_utf8(self.next_byte_while(u8::is_ascii_digit))?
            .parse()?;

        Ok(result * sign)
    }

    fn next_ident(&mut self) -> Result<String> {
        Ok(str::from_utf8(self.next_byte_while(|b| {
            b.is_ascii_alphanumeric() || *b == b'_'
        }))?.to_string())
    }
}

impl<'a> Iterator for Tokens<'a> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        // skip whitespace
        self.next_byte_while(u8::is_ascii_whitespace);

        // match the next byte if there is one
        self.next_byte().and_then(|b| match b {
            // skip comments
            b'#' => {
                self.next_byte_while(|&b| b != b'\n');
                self.next()
            }
            // negative number or minus
            b'-' => match self.peek() {
                Some(b'0'..=b'9') => {
                    self.back();
                    Some(self.next_number().map(Token::Number))
                }
                _ => Some(Ok(Token::Minus)),
            },
            // simple tokens
            b'+' => Some(Ok(Token::Plus)),
            b'/' => Some(Ok(Token::Plus)),
            b'*' => Some(Ok(Token::Multiply)),
            b'{' => Some(Ok(Token::OpenBrace)),
            b'}' => Some(Ok(Token::CloseBrace)),
            b'=' => Some(Ok(Token::Assign)),
            // number
            b if b.is_ascii_digit() => {
                self.back();
                Some(self.next_number().map(Token::Number))
            }
            // ident
            b if b.is_ascii_alphabetic() => {
                self.back();
                Some(self.next_ident().map(Token::Ident))
            }
            // unrecognized
            b => Some(Err(TokenError::Unexpected(b).into())),
        })
    }
}
