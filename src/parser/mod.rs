use self::error::*;
use crate::*;
use std::iter::FromIterator;

mod error;
mod grammar;

/// A PEG parser with YAML grammar, support UTF-8 characters.
///
/// Grammar methods will move the cursor if matched.
///
/// + For matching methods:
///     + Use `?` to match a condition.
///     + Use [`Result::unwrap_or_default`] to match an optional condition.
/// + Method [`Parser::eat`] is used to move on and get the matched string.
/// + Method [`Parser::backward`] is used to get back if mismatched.
///
/// ```
/// use yaml_peg::{Parser, node};
/// let n = Parser::new("true").parse().unwrap();
/// assert_eq!(n, vec![node!(true)])
/// ```
pub struct Parser<'a> {
    doc: &'a str,
    /// Current position.
    pub pos: usize,
    /// Read position.
    pub eaten: usize,
}

impl<'a> Parser<'a> {
    /// Create a PEG parser with the string.
    pub fn new(doc: &'a str) -> Self {
        Self {
            doc,
            pos: 0,
            eaten: 0,
        }
    }

    /// Set the starting point.
    pub fn with_cursor(mut self, pos: usize) -> Self {
        if self.doc.is_char_boundary(pos) {
            self.pos = pos;
            self.eaten = pos;
        }
        self
    }

    /// Move the eaten cursor to the current position and return the string.
    pub fn eat(&mut self) -> &'a str {
        if self.eaten < self.pos {
            let s = &self.doc[self.eaten..self.pos];
            self.eaten = self.pos;
            s
        } else {
            self.eaten = self.pos;
            ""
        }
    }

    /// Move the current position back.
    pub fn backward(&mut self) {
        self.pos = self.eaten;
    }

    /// Show the right hand side string.
    pub fn food(&self) -> &'a str {
        &self.doc[self.pos..]
    }

    /// YAML entry point, return entire doc if exist.
    pub fn parse(&mut self) -> std::io::Result<Array> {
        let mut v = vec![];
        let mut ch = self.food().char_indices();
        while let Some((i, _)) = ch.next() {
            self.pos = i;
            v.push(match self.doc() {
                Ok(n) => n,
                Err(e) => return Err(e.into_error(self.doc)),
            });
            for _ in i..self.pos {
                ch.next();
            }
        }
        Ok(v)
    }

    /// Match one doc block.
    pub fn doc(&mut self) -> Result<Node, PError> {
        self.ws_any().unwrap_or_default();
        self.seq(b"---").unwrap_or_default();
        self.ws_any().unwrap_or_default();
        self.eat();
        let ret = self.value()?;
        self.seq(b"...").unwrap_or_default();
        self.eat();
        Ok(ret)
    }

    /// Match YAML value.
    pub fn value(&mut self) -> Result<Node, PError> {
        let pos = self.pos;
        let yaml = if self.sym(b'~').is_ok() {
            Yaml::Null
        } else if self.seq(b"null").is_ok() {
            Yaml::Null
        } else if self.seq(b"true").is_ok() {
            Yaml::Bool(true)
        } else if self.seq(b"false").is_ok() {
            Yaml::Bool(false)
        } else if self.float().is_ok() {
            Yaml::Float(self.eat().into())
        } else if self.int().is_ok() {
            Yaml::Int(self.eat().into())
        } else if self.quoted_string().is_ok() {
            Yaml::Str(Self::escape(self.eat()))
        } else {
            self.backward();
            return Err(PError::new(self.pos, "value"));
        };
        Ok(node!(yaml, pos))
        // TODO
    }
}

/// Parse YAML document.
pub fn parse(doc: &str) -> std::io::Result<Array> {
    Parser::new(doc).parse()
}
