//! Parser components.
pub use self::error::*;
pub use self::grammar::*;
use crate::*;
use std::iter::FromIterator;

mod error;
mod grammar;

/// A PEG parser with YAML grammar, support UTF-8 characters.
///
/// A simple example for parsing YAML only:
///
/// ```
/// use yaml_peg::{Parser, node};
/// let n = Parser::new("true").parse().unwrap();
/// assert_eq!(n, vec![node!(true)]);
/// ```
///
/// For matching partial grammar, each methods are the sub-parser.
/// The methods have some behaviers:
///
/// + They will move the current cursor if matched.
/// + Returned value:
///     + `Result<(), ()>` represents the sub-parser can be matched and mismatched.
///     + [`PError`] represents the sub-parser can be totally breaked when mismatched.
/// + Use `?` to match a condition.
/// + Use [`Result::unwrap_or_default`] to match an optional condition.
/// + Method [`Parser::eat`] is used to move on and get the matched string.
/// + Method [`Parser::backward`] is used to get back if mismatched.
pub struct Parser<'a> {
    doc: &'a str,
    /// Current position.
    pub pos: usize,
    /// Read position.
    pub eaten: usize,
}

/// The basic implementation.
///
/// These sub-parser returns [`PError`], and failed immediately for [`PError::Terminate`].
/// Additionally, they should eat the string by themself.
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
            ch = self.food().char_indices();
        }
        Ok(v)
    }

    /// Match one doc block.
    pub fn doc(&mut self) -> Result<Node, PError> {
        self.inv(TakeOpt::If).unwrap_or_default();
        self.seq(b"---").unwrap_or_default();
        self.gap().unwrap_or_default();
        self.eat();
        let ret = self.value()?;
        self.seq(b"...").unwrap_or_default();
        self.eat();
        Ok(ret)
    }

    /// Match YAML value.
    pub fn value(&mut self) -> Result<Node, PError> {
        let anchor = self.token(Self::anchor).unwrap_or_default().into();
        let ty = self.token(Self::ty).unwrap_or_default().into();
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
        } else if let Ok(s) = self.string_flow() {
            Yaml::Str(Self::escape(s))
        } else if self.anchor_use().is_ok() {
            Yaml::Anchor(self.eat().into())
        } else {
            return Err(PError::Terminate(self.pos, "value".into()));
        };
        self.eat();
        Ok(node!(yaml, pos, anchor, ty))
    }
}

/// Parse YAML document.
pub fn parse(doc: &str) -> std::io::Result<Array> {
    Parser::new(doc).parse()
}
