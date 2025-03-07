//! # Parser
//! This is a standalone parser for reading strings into descrete values.
//!
//! It uses a main parser and derived attempts that are used to try patterns.
//!
//! # Usage
//! ```
//! use org_roamers::parser::Parser;
//!
//! let mut parser = Parser::new("(\"Test\")");
//!
//! let mut attempt = parser.attempt();
//! if let None = attempt.consume_char('(') {
//!     panic!();
//! }
//! parser.sync(attempt);
//!
//! let mut attempt = parser.attempt();
//! match attempt.consume_string() {
//!     Some(s) => println!("{s:?}"),
//!     None => panic!(),
//! }
//! parser.sync(attempt);
//!
//! let mut attempt = parser.attempt();
//! if let None = attempt.consume_char(')') {
//!     panic!();
//! }
//! parser.sync(attempt);
//!
//! assert_eq!(parser.remaining(), "".to_string());
//! ```

use std::{iter::Peekable, str::Chars};

pub struct Parser<'a> {
    code: Peekable<Chars<'a>>,
}

pub struct ParserAttempt<'a> {
    code: Peekable<Chars<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(code: &'a str) -> Parser<'a> {
        let iter = code.chars().peekable();
        Parser { code: iter }
    }

    pub fn attempt(&self) -> ParserAttempt<'a> {
        ParserAttempt {
            code: self.code.clone(),
        }
    }

    pub fn sync(&mut self, attempt: ParserAttempt<'a>) {
        self.code = attempt.code;
    }

    pub fn remaining(self) -> String {
        self.code.collect()
    }
}

impl<'a> ParserAttempt<'a> {
    pub fn consume_char(&mut self, c: char) -> Option<char> {
        if let Some(ch) = self.code.next() {
            if ch == c {
                return Some(c);
            }
        }
        None
    }

    pub fn consume_whitespace(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.code.peek() {
            match c {
                ' ' => s.push(self.code.next().unwrap()),
                '\t' => s.push(self.code.next().unwrap()),
                '\n' => s.push(self.code.next().unwrap()),
                _ => break,
            }
        }
        s
    }

    pub fn consume_string(&mut self) -> Option<String> {
        let mut string = String::new();
        let start = match self.code.next() {
            Some(c) if c == '"' => c,
            Some(c) if c == '\'' => c,
            _ => return None,
        };
        let mut flag = false;

        while let Some(c) = self.code.next() {
            if c == start && !flag {
                break;
            }
            if c == '\\' && flag {
                flag = false;
                string.push(c);
            }
            if c == '\\' && !flag {
                flag = true;
            }
            string.push(c);
        }

        Some(string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consume_whitespace() {
        let mut parser = Parser::new("  \t  to");
        let mut attempt = parser.attempt();
        let whitespace = attempt.consume_whitespace();
        parser.sync(attempt);
        assert_eq!(whitespace, "  \t  ");
        let remaining = parser.remaining();
        assert_eq!(remaining.as_str(), "to");
    }

    #[test]
    fn test_consume_whitespace_no_whitespace() {
        let mut parser = Parser::new("to");
        let mut attempt = parser.attempt();
        let whitespace = attempt.consume_whitespace();
        parser.sync(attempt);
        assert_eq!(whitespace, "");
        let remaining = parser.remaining();
        assert_eq!(remaining.as_str(), "to");
    }

    #[test]
    fn consume_string_double_quote() {
        let mut parser = Parser::new("\"Hello 'world'\"  ");
        let mut attempt = parser.attempt();
        let string = attempt.consume_string().unwrap();
        parser.sync(attempt);
        assert_eq!(string, "Hello 'world'");
        let remaining = parser.remaining();
        assert_eq!(remaining, "  ");
    }

    #[test]
    fn do_not_consume() {
        let parser = Parser::new("\"Hello\"  ");
        let mut attempt = parser.attempt();
        let string = attempt.consume_string().unwrap();
        assert_eq!(string, "Hello".to_string());
        assert_eq!(parser.remaining(), "\"Hello\"  ".to_string());
    }
}
