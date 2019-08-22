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
        // literals are displayed with the default rust formatter
        match self {
            Literal::Boolean(b) => fmt::Display::fmt(b, f),
            Literal::Number(n) => fmt::Display::fmt(n, f),
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
            // lit
            Ast::Literal(lit) => fmt::Display::fmt(lit, f),
            // (a b +)
            Ast::Add(a, b) => write!(f, "({} {} +)", a, b),
            // (a b -)
            Ast::Subtract(a, b) => write!(f, "({} {} -)", a, b),
            // (a b *)
            Ast::Multiply(a, b) => write!(f, "({} {} *)", a, b),
            // (a b /)
            Ast::Divide(a, b) => write!(f, "({} {} /)", a, b),
            // {ast+}
            Ast::Block(block) => {
                f.write_str("{")?;
                for ast in block {
                    write!(f, "{}", ast)?;
                }
                f.write_str("}")
            }
            // ({ast+} fn)
            Ast::Func(block) => {
                f.write_str("({")?;
                for ast in block {
                    write!(f, "{}", ast)?;
                }
                f.write_str("} fn)")
            }
            // (ast call)
            Ast::Call(ast) => write!(f, "({} call)", ast),
            // (ident =)
            Ast::Assign(ident, value) => write!(f, "({} {} =)", ident, value),
            // ident
            Ast::Ident(ident) => f.write_str(ident),
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
            tokens: Tokens::new(program),
            stack: vec![],
        }
    }

    /// parse until EOI/error and return `Ast` stack
    pub fn parse(mut self) -> Result<Vec<Ast>> {
        loop {
            match self.parse_next() {
                // parsed successfully, no action required as the results should be on the stack
                Ok(_) => {}
                // nothing to parse, we have reached the end of the program
                Err(Error::Ast(AstError::UnexpectedEoi)) => return Ok(self.stack),
                // failed to parse for some other reason
                Err(e) => return Err(e),
            }
        }
    }

    // parse the next valid ast
    fn parse_next(&mut self) -> Result<()> {
        // macro to easily define a binary op (a b <op>) using `util::call`
        macro_rules! binary_op {
            ($variant:ident) => {{
                let _ = call(&mut self.stack, 2, |a| {
                    Ast::$variant(Box::new(a[0].clone()), Box::new(a[1].clone()))
                })?;
            }};
        }

        if let Some(token) = self.tokens.next() {
            match token? {
                // a b +
                Token::Plus => binary_op!(Add),
                // a b -
                Token::Minus => binary_op!(Subtract),
                // a b *
                Token::Multiply => binary_op!(Multiply),
                // a b /
                Token::Divide => binary_op!(Divide),
                // { <block> }
                Token::OpenBrace => self.parse_block()?,
                // identifier is either a keyword or a variable name
                Token::Ident(ident) => {
                    match ident.as_ref() {
                        // keywords
                        "fn" => self.parse_fn()?,
                        "call" => self.parse_call()?,
                        // keyword literals
                        "Nil" => self.stack.push(Ast::Literal(Literal::Nil)),
                        "true" => self.stack.push(Ast::Literal(Literal::Boolean(true))),
                        "false" => self.stack.push(Ast::Literal(Literal::Boolean(false))),
                        // default is ident
                        _ => self.stack.push(Ast::Ident(ident)),
                    }
                }
                // number
                Token::Number(num) => self.stack.push(Ast::Literal(Literal::Number(num))),
                // ident ast =
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

    // parse a block: { <ast>* }
    fn parse_block(&mut self) -> Result<()> {
        // mark the beginning of the block contents in the stack
        let start = self.stack.len();

        loop {
            // if we can read a '}' token, push the block containing all `Ast`s
            // on the stack pushed after `start`
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

            // we didn't return, so there was no '}'. parse the next Ast, and
            // if EOI is reached, return syntax error
            if let Err(Error::Ast(AstError::UnexpectedEoi)) = self.parse_next() {
                return Err(AstError::Expected(String::from("}")).into());
            }
        }
    }

    // parse a function expression: { <asts> } fn
    fn parse_fn(&mut self) -> Result<()> {
        if let Some(Ast::Block(block)) = self.stack.pop() {
            self.stack.push(Ast::Func(block));
        } else {
            return Err(AstError::Expected(String::from("block")).into());
        }
        Ok(())
    }

    // parse a call expression: <ident> call
    fn parse_call(&mut self) -> Result<()> {
        let arg = self.stack.pop().ok_or(Error::NotEnoughArguments(1, 0))?;
        self.stack.push(Ast::Call(Box::new(arg)));
        Ok(())
    }
}
