use crate::*;
use std::iter::FromIterator;

/// A PEG parser with YAML grammar.
pub struct Parser<'a> {
    pos: usize,
    doc: &'a str,
}

impl<'a> Parser<'a> {
    /// Create a PEG parser with the string.
    pub fn new(doc: &'a str) -> Self {
        Self { pos: 0, doc }
    }

    /// Set the start point.
    pub fn with_cursor(mut self, pos: usize) -> Self {
        self.pos = pos;
        self
    }

    /// YAML entry point, return entire doc if exist.
    pub fn parse(&mut self) -> std::io::Result<Array> {
        let mut v = vec![];
        loop {
            dbg!(&self.doc[self.pos..]);
            v.push(match self.doc() {
                Ok(n) => n,
                Err(e) => return Err(e.into_error(self.doc)),
            });
            self.pos += 1;
            if self.pos > self.doc.len() || self.doc[self.pos..].is_empty() {
                return Ok(v);
            }
        }
    }

    pub fn doc(&mut self) -> Result<Node, PError> {
        let mut ret = node!(Yaml::Null, self.pos);
        self.seq(b"---").unwrap_or(());
        if let Ok(n) = self.value() {
            ret = n;
        }
        self.seq(b"...").unwrap_or(());
        Ok(ret)
    }

    pub fn seq(&mut self, s: &[u8]) -> Result<(), PError> {
        let len = s.len();
        let end = self.pos + len;
        if end <= self.doc.len() && self.doc[self.pos..end].as_bytes() == s {
            self.pos += len;
            Ok(())
        } else {
            Err(PError::new(
                self.pos,
                &format!("invalid {}", String::from_utf8_lossy(s)),
            ))
        }
    }

    pub fn value(&mut self) -> Result<Node, PError> {
        if self.seq(b"~").is_ok() {
            Ok(node!(Yaml::Null, self.pos))
        } else if self.seq(b"null").is_ok() {
            Ok(node!(Yaml::Null, self.pos))
        } else if self.seq(b"true").is_ok() {
            Ok(node!(Yaml::Bool(true), self.pos))
        } else if self.seq(b"false").is_ok() {
            Ok(node!(Yaml::Bool(false), self.pos))
        } else {
            Err(PError::new(self.pos, "invalid value"))
        }
        // TODO
    }

    pub fn identifier(&mut self) -> Result<&'a str, PError> {
        let mut pos = self.pos;
        for (i, c) in self.doc[self.pos..].char_indices() {
            if c.is_whitespace() {
                pos = self.pos + i;
                break;
            }
        }
        if pos == self.pos {
            Err(PError::new(self.pos, "invalid identifier"))
        } else {
            Ok(&self.doc[self.pos..pos])
        }
    }
}

/// Parse YAML document.
pub fn parse(doc: &str) -> std::io::Result<Array> {
    Parser::new(doc).parse()
}
