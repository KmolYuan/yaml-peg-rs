use self::error::*;
use crate::*;
use std::iter::FromIterator;

mod error;
mod grammar;

/// A PEG parser with YAML grammar.
///
/// Most of atom methods will move the cursor if matched.
pub struct Parser<'a> {
    pub pos: usize,
    doc: &'a str,
}

impl<'a> Parser<'a> {
    /// Create a PEG parser with the string.
    pub fn new(doc: &'a str) -> Self {
        Self { pos: 0, doc }
    }

    /// Set the start point.
    pub fn with_cursor(mut self, pos: usize) -> Self {
        if self.doc.is_char_boundary(pos) {
            self.pos = pos;
        }
        self
    }

    /// YAML entry point, return entire doc if exist.
    pub fn parse(&mut self) -> std::io::Result<Array> {
        let mut v = vec![];
        let mut ch = self.doc[self.pos..].char_indices();
        while let Some((i, _)) = ch.next() {
            self.pos = i;
            dbg!(&self.doc[self.pos..]);
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
        let mut ret = node!(Yaml::Null, self.pos);
        self.seq(b"---").unwrap_or(());
        ret = self.value()?;
        self.seq(b"...").unwrap_or(());
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
        } else if let Ok(n) = self.float() {
            Yaml::Float(self.slice(n).into())
        } else if let Ok(n) = self.int() {
            Yaml::Int(self.slice(n).into())
        } else {
            return Err(PError::new(self.pos, "invalid value"));
        };
        Ok(node!(yaml, pos))
        // TODO
    }
}

/// Parse YAML document.
pub fn parse(doc: &str) -> std::io::Result<Array> {
    Parser::new(doc).parse()
}
