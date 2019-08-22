// frothy/src/ast.rs

//! Functions and types for parsing frothy programs into [`Ast`](enum.Ast.html)s

use std::fmt;

use crate::error::{Error, Result};
use crate::token::{Token, Tokens};
use crate::util::call;

/// Errors in AST building or evaluation
#[derive(Debug, Clone)]
pub enum AstError {
    Expected(String),
    ExpectedButGot(String, Token),
    Unexpected(Token),
    UnexpectedEoi,
}

impl fmt::Display for AstError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AstError::Expected(pattern) => write!(f, "expected {}", pattern),
            AstError::ExpectedButGot(pattern, token) => {
                write!(f, "expected {} but got '{}'", pattern, token)
            }
            AstError::Unexpected(token) => write!(f, "unexpected token '{}'", token),
            AstError::UnexpectedEoi => f.write_str("unexpected EOI"),
        }
    }
}

/// A frothy literal
#[derive(Debug, Clone)]
pub enum Literal {
    Boolean(bool),
    Number(f64),
    Nil,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Literal::Boolean(boolean) => write!(f, "{}", boolean),
            Literal::Number(num) => write!(f, "{}", num),
            Literal::Nil => f.write_str("Nil"),
        }
    }
}

/// Frothy AST node types
#[derive(Debug, Clone)]
pub enum Ast {
    Literal(Literal),

    // operations
    Add(Box<Ast>, Box<Ast>),
    Subtract(Box<Ast>, Box<Ast>),
    Multiply(Box<Ast>, Box<Ast>),
    Divide(Box<Ast>, Box<Ast>),

    Func(Vec<Ast>),
    Call(Box<Ast>),

    // variables
    Ident(String),
    Assign(String, Box<Ast>),

    Block(Vec<Ast>),
}

impl fmt::Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Ast::Literal(lit) => fmt::Display::fmt(lit, f),
            Ast::Add(a, b) => write!(f, "({} {} +)", a, b),
            Ast::Subtract(a, b) => write!(f, "({} {} -)", a, b),
            Ast::Multiply(a, b) => write!(f, "({} {} *)", a, b),
            Ast::Divide(a, b) => write!(f, "({} {} /)", a, b),
            Ast::Block(block) => {
                f.write_str("{")?;
                for ast in block {
                    write!(f, "{}", ast)?;
                }
                f.write_str("}")
            }
            Ast::Func(block) => {
                f.write_str("({")?;
                for ast in block {
                    write!(f, "{}", ast)?;
                }
                f.write_str("} fn)")
            }
            Ast::Call(ast) => write!(f, "({} call)", ast),
            Ast::Assign(ident, value) => write!(f, "({} {} =)", ident, value),
            Ast::Ident(ident) => write!(f, "<{}>", ident),
        }
    }
}

/// Parse a frothy program into [`ast::Ast`]
pub struct Parser<'a> {
    tokens: Tokens<'a>,
    stack: Vec<Ast>,
}

impl<'a> Parser<'a> {
    pub fn new(program: &'a str) -> Parser<'a> {
        Parser {
            tokens: Tokens::new(program.as_ref()),
            stack: vec![],
        }
    }

    /// parse until EOI/error and return `Ast` stack
    pub fn parse(mut self) -> Result<Vec<Ast>> {
        loop {
            match self.parse_next() {
                Ok(_) => {}
                Err(Error::Ast(AstError::UnexpectedEoi)) => return Ok(self.stack),
                Err(e) => return Err(e),
            }
        }
    }

    fn parse_next(&mut self) -> Result<()> {
        macro_rules! binary_op {
            ($ast:ident) => {{
                let _ = call(&mut self.stack, 2, |a| {
                    Ast::$ast(Box::new(a[0].clone()), Box::new(a[1].clone()))
                })?;
            }};
        }

        if let Some(token) = self.tokens.next() {
            match token? {
                // operators
                Token::Plus => binary_op!(Add),
                Token::Minus => binary_op!(Subtract),
                Token::Multiply => binary_op!(Multiply),
                Token::Divide => binary_op!(Divide),
                // simple signs
                Token::OpenBrace => self.parse_block()?,
                // identifier
                Token::Ident(ident) => {
                    match ident.as_ref() {
                        // keywords
                        "fn" => self.parse_fn()?,
                        "call" => self.parse_call()?,
                        "Nil" => self.stack.push(Ast::Literal(Literal::Nil)),
                        "true" => self.stack.push(Ast::Literal(Literal::Boolean(true))),
                        "false" => self.stack.push(Ast::Literal(Literal::Boolean(false))),
                        // default is ident
                        _ => self.stack.push(Ast::Ident(ident)),
                    }
                }
                Token::Number(num) => self.stack.push(Ast::Literal(Literal::Number(num))),
                Token::Assign => {
                    // expect an assign: ident + ast
                    if let (Some(ast), Some(Ast::Ident(ident))) =
                    (self.stack.pop(), self.stack.pop())
                    {
                        self.stack.push(Ast::Assign(ident, Box::new(ast)));
                    } else {
                        return Err(AstError::Expected(String::from("ident + ast")).into());
                    }
                }
                // unexpected token
                token => return Err(AstError::Unexpected(token).into()),
            }
        } else {
            return Err(AstError::UnexpectedEoi.into());
        }

        Ok(())
    }

    fn parse_block(&mut self) -> Result<()> {
        let start = self.stack.len();

        loop {
            if let Ok(Token::CloseBrace) = self.tokens.clone().peekable().peek().unwrap() {
                // pop the asts added since `start` from the stack
                let block = (&self.stack[start..]).to_vec();
                self.stack.drain(start..);

                // push the block to the stack
                self.stack.push(Ast::Block(block));

                // skip the '}'
                self.tokens.next();

                return Ok(());
            }

            if let Err(Error::Ast(AstError::UnexpectedEoi)) = self.parse_next() {
                return Err(AstError::Expected(String::from("}")).into());
            }
        }
    }

    fn parse_fn(&mut self) -> Result<()> {
        if let Some(Ast::Block(block)) = self.stack.pop() {
            self.stack.push(Ast::Func(block));
        } else {
            return Err(AstError::Expected(String::from("block")).into());
        }
        Ok(())
    }

    fn parse_call(&mut self) -> Result<()> {
        let arg = self.stack.pop().ok_or(Error::NotEnoughArguments(1, 0))?;
        self.stack.push(Ast::Call(Box::new(arg)));
        Ok(())
    }
}
