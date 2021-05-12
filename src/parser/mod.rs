//! Parser components.
pub use self::error::*;
pub use self::grammar::*;
use crate::*;
use std::{io::Error, iter::FromIterator};

mod error;
mod grammar;

macro_rules! err_own {
    ($e:expr, $then:expr $(, $trans:expr)?) => {
        match $e {
            Ok(v) => Ok($($trans)?(v)),
            Err(PError::Mismatch) => $then,
            Err(e) => Err(e),
        }
    };
}

/// A PEG parser with YAML grammar, support UTF-8 characters.
///
/// A simple example for parsing YAML only:
///
/// ```
/// use yaml_peg::{parser::Parser, node};
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
    pub fn start_at(mut self, pos: usize) -> Self {
        if self.doc.is_char_boundary(pos) {
            self.pos = pos;
            self.eaten = pos;
        }
        self
    }

    /// YAML entry point, return entire doc if exist.
    pub fn parse(&mut self) -> Result<Array, PError> {
        self.inv(TakeOpt::ZeroMore)?;
        self.seq(b"---").unwrap_or_default();
        self.gap().unwrap_or_default();
        self.eat();
        let mut v = vec![];
        v.push(self.doc()?);
        loop {
            self.inv(TakeOpt::ZeroMore)?;
            if self.food().is_empty() {
                break;
            }
            if self.seq(b"---").is_err() {
                return self.err("splitter");
            }
            self.gap().unwrap_or_default();
            self.eat();
            v.push(self.doc()?);
        }
        Ok(v)
    }

    /// Match one doc block.
    pub fn doc(&mut self) -> Result<Node, PError> {
        let ret = self.scalar()?;
        self.seq(b"...").unwrap_or_default();
        self.eat();
        Ok(ret)
    }

    /// Match YAML scalar.
    pub fn scalar(&mut self) -> Result<Node, PError> {
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
        } else if self.nan().is_ok() {
            Yaml::Float("NaN".into())
        } else if let Ok(b) = self.inf() {
            Yaml::Float(if b { "inf" } else { "-inf" }.into())
        } else if self.int().is_ok() {
            Yaml::Int(self.eat().into())
        } else if self.anchor_use().is_ok() {
            Yaml::Anchor(self.eat().into())
        } else if let Ok(s) = self.string_flow() {
            Yaml::Str(Self::escape(&Self::merge_ws(s)))
        } else {
            err_own!(
                self.array_flow(),
                err_own!(self.map_flow(), self.err("value"), Yaml::from_iter),
                Yaml::from_iter
            )?
        };
        self.eat();
        Ok(node!(yaml, pos, anchor, ty))
    }

    /// Match flow array.
    pub fn array_flow(&mut self) -> Result<Array, PError> {
        self.sym(b'[')?;
        let mut v = vec![];
        loop {
            self.inv(TakeOpt::ZeroMore)?;
            self.eat();
            if self.sym(b']').is_ok() {
                break;
            }
            self.eat();
            v.push(err_own!(self.scalar(), self.err("array"))?);
            self.inv(TakeOpt::ZeroMore)?;
            if self.sym(b',').is_err() {
                self.inv(TakeOpt::ZeroMore)?;
                self.sym(b']')?;
                break;
            }
        }
        self.eat();
        Ok(v)
    }

    /// Match flow map.
    pub fn map_flow(&mut self) -> Result<Vec<(Node, Node)>, PError> {
        self.sym(b'{')?;
        let mut m = vec![];
        loop {
            self.inv(TakeOpt::ZeroMore)?;
            self.eat();
            if self.sym(b'}').is_ok() {
                break;
            }
            self.eat();
            let k = err_own!(self.scalar(), self.err("map"))?;
            self.inv(TakeOpt::ZeroMore)?;
            if self.sym(b':').is_err() || self.inv(TakeOpt::OneMore).is_err() {
                return self.err("map");
            }
            self.eat();
            let v = err_own!(self.scalar(), self.err("map"))?;
            m.push((k, v));
            if self.sym(b',').is_err() {
                self.inv(TakeOpt::ZeroMore)?;
                self.sym(b'}')?;
                break;
            }
        }
        self.eat();
        Ok(m)
    }
}

/// Parse YAML document.
///
/// ```
/// use yaml_peg::{parse, node};
/// let n = parse("true").unwrap();
/// assert_eq!(n, vec![node!(true)]);
/// ```
pub fn parse(doc: &str) -> Result<Array, Error> {
    match Parser::new(doc).parse() {
        Ok(v) => Ok(v),
        Err(e) => Err(e.into_error(doc)),
    }
}
