//! ast.rs

use crate::value::Value;
use crate::token::{Tokens, Token};
use crate::util::pop_n;
use core::fmt;

#[derive(Debug, Clone)]
pub enum Literal {
    Boolean(bool),
    Number(f64),
    Nil,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Literal::Boolean(boolean) => write!(f, "{}", boolean),
            Literal::Number(num) => write!(f, "{}", num),
            Nil => f.write_str("Nil"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Ast {
    Literal(Literal),

    // operations
    Add(Box<Ast>, Box<Ast>),
    Subtract(Box<Ast>, Box<Ast>),
    Multiply(Box<Ast>, Box<Ast>),
    Divide(Box<Ast>, Box<Ast>),

    Func(Vec<Ast>),

    // variables
    Ident(String),
    Assign(String, Box<Ast>),

    Block(Vec<Ast>),
}

impl fmt::Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Ast::Literal(lit) => fmt::Display::fmt(lit, f),
            Ast::Add(a, b) => write!(f, "({} {} +)", a, b),
            Ast::Subtract(a, b) => write!(f, "({} {} -)", a, b),
            Ast::Multiply(a, b) => write!(f, "({} {} *)", a, b),
            Ast::Divide(a, b) => write!(f, "({} {} /)", a, b),
            Ast::Block(block) => {
                f.write_str("{");
                for ast in block {
                    write!(f, "{}", ast)?;
                }
                f.write_str("}")
            }
            Ast::Func(block) => {
                f.write_str("({");
                for ast in block {
                    write!(f, "{}", ast)?;
                }
                f.write_str("} fn)")
            }
            Ast::Assign(ident, value) => write!(f, "({} {} =)", ident, value),
            Ast::Ident(ident) => write!(f, "<{}>", ident),
        }
    }
}

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

    pub fn parse(mut self) -> Result<Vec<Ast>, String> {
        loop {
            self.parse_next()?;
        }

        Ok(self.stack)
    }

    fn parse_next(&mut self) -> Result<(), String> {
        macro_rules! binary_op {
            ($ast:ident) => {
                {
                    let _ = pop_n(&mut self.stack, 2, |a| Ast::$ast(Box::new(a[0].clone()), Box::new(a[1].clone())))?;
                }
            }
        }

        if let Some(token) = self.tokens.next() {
            match token.unwrap() {
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
                        "Nil" => self.stack.push(Ast::Literal(Literal::Nil)),
                        "true" => self.stack.push(Ast::Literal(Literal::Boolean(true))),
                        "false" => self.stack.push(Ast::Literal(Literal::Boolean(false))),
                        // default is ident
                        _ => self.stack.push(Ast::Ident(ident)),
                    }
                }
                Token::Number(num) => self.stack.push(Ast::Literal(Literal::Number(num))),
                Token::Assign => {
                    if let (Some(ast), Some(Ast::Ident(ident))) = (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(Ast::Assign(ident, Box::new(ast)));
                    } else {
                        return Err(format!("Expected ident and ast"));
                    }
                }
                // unexpected token
                token => return Err(format!("Unexpected token {:?}", token))
            }
        } else {
            return Err(String::from("eof"));
        }

        for ast in self.stack.iter() {
            println!("{}", ast);
        }

        Ok(())
    }

    fn parse_block(&mut self) -> Result<(), String> {
        let start= self.stack.len();

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

            self.parse_next()?;
        }

        Err(format!("Expected }}"))
    }

    fn parse_fn(&mut self) -> Result<(), String> {
        if let Some(Ast::Block(block)) = self.stack.pop() {
            self.stack.push(Ast::Func(block));
        } else {
            return Err(String::from("expected block"));
        }
        Ok(())
    }
}
