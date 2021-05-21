//! Parser components.
pub use self::error::*;
pub use self::grammar::*;
use crate::*;
use std::{io::Error, iter::FromIterator};

mod error;
mod grammar;

macro_rules! err_own {
    ($e:expr, $then:expr) => {
        match $e {
            Ok(v) => Ok(v),
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
        let ret = self.scalar(0)?;
        self.seq(b"...").unwrap_or_default();
        self.eat();
        Ok(ret)
    }

    /// Match scalar.
    pub fn scalar(&mut self, level: usize) -> Result<Node, PError> {
        let anchor = self.anchor().unwrap_or_default();
        self.inv(TakeOpt::ZeroMore)?;
        let ty = self.ty().unwrap_or_default();
        self.inv(TakeOpt::ZeroMore)?;
        self.eat();
        let pos = self.pos;
        let yaml = err_own!(
            self.array(level),
            err_own!(self.map(level), self.scalar_term(level))
        )?;
        self.eat();
        Ok(node!(yaml, pos, anchor.into(), ty.into()))
    }

    /// Match flow scalar.
    pub fn scalar_flow(&mut self, level: usize) -> Result<Node, PError> {
        let anchor = self.anchor().unwrap_or_default();
        self.inv(TakeOpt::ZeroMore)?;
        let ty = self.ty().unwrap_or_default();
        self.inv(TakeOpt::ZeroMore)?;
        self.eat();
        let pos = self.pos;
        let yaml = self.scalar_term(level)?;
        self.eat();
        Ok(node!(yaml, pos, anchor.into(), ty.into()))
    }

    /// Match flow scalar terminal.
    pub fn scalar_term(&mut self, level: usize) -> Result<Yaml, PError> {
        let yaml = if self.sym(b'~').is_ok() {
            Yaml::Null
        } else if self.seq(b"null").is_ok() {
            Yaml::Null
        } else if self.seq(b"true").is_ok() {
            Yaml::Bool(true)
        } else if self.seq(b"false").is_ok() {
            Yaml::Bool(false)
        } else if self.nan().is_ok() {
            Yaml::Float("NaN".into())
        } else if let Ok(b) = self.inf() {
            Yaml::Float(if b { "inf" } else { "-inf" }.into())
        } else if self.float().is_ok() {
            Yaml::Float(self.eat().into())
        } else if self.sci_float().is_ok() {
            Yaml::Float(self.eat().into())
        } else if self.int().is_ok() {
            Yaml::Int(self.eat().into())
        } else if self.anchor_use().is_ok() {
            Yaml::Anchor(self.eat().into())
        } else if let Ok(s) = self.string_flow() {
            Yaml::Str(Self::escape(&Self::merge_ws(s)))
        } else {
            err_own!(
                self.array_flow(level),
                err_own!(self.map_flow(level), self.err("scalar"))
            )?
        };
        Ok(yaml)
    }

    /// Match flow array.
    pub fn array_flow(&mut self, level: usize) -> Result<Yaml, PError> {
        self.sym(b'[')?;
        let mut v = vec![];
        loop {
            self.inv(TakeOpt::ZeroMore)?;
            self.eat();
            if self.sym(b']').is_ok() {
                break;
            }
            self.eat();
            v.push(err_own!(
                self.scalar_flow(level + 1),
                self.err("flow array")
            )?);
            self.inv(TakeOpt::ZeroMore)?;
            if self.sym(b',').is_err() {
                self.inv(TakeOpt::ZeroMore)?;
                self.sym(b']')?;
                break;
            }
        }
        self.eat();
        Ok(Yaml::from_iter(v))
    }

    /// Match flow map.
    pub fn map_flow(&mut self, level: usize) -> Result<Yaml, PError> {
        self.sym(b'{')?;
        let mut m = vec![];
        loop {
            self.inv(TakeOpt::ZeroMore)?;
            self.eat();
            if self.sym(b'}').is_ok() {
                break;
            }
            self.eat();
            // TODO '?' key
            let k = err_own!(self.scalar_flow(level + 1), self.err("flow map"))?;
            self.inv(TakeOpt::ZeroMore)?;
            if self.sym(b':').is_err() || self.inv(TakeOpt::OneMore).is_err() {
                return self.err("map");
            }
            self.eat();
            let v = err_own!(self.scalar_flow(level + 1), self.err("map"))?;
            m.push((k, v));
            if self.sym(b',').is_err() {
                self.inv(TakeOpt::ZeroMore)?;
                self.sym(b'}')?;
                break;
            }
        }
        self.eat();
        Ok(Yaml::from_iter(m))
    }

    /// Match array.
    pub fn array(&mut self, level: usize) -> Result<Yaml, PError> {
        self.block_prefix(level).unwrap_or_default();
        let mut v = vec![];
        loop {
            self.eat();
            if v.is_empty() {
                // Mismatch
                self.sym(b'-')?;
                self.inv(TakeOpt::OneMore)?;
            } else {
                if self.gap().is_err() {
                    return self.err("array terminator");
                }
                if self.indent(level).is_err() {
                    break;
                }
                if self.sym(b'-').is_err() || self.inv(TakeOpt::OneMore).is_err() {
                    return self.err("array splitter");
                }
            }
            v.push(err_own!(self.scalar(level + 1), self.err("array value"))?);
        }
        self.eat();
        Ok(Yaml::from_iter(v))
    }

    /// Match map.
    pub fn map(&mut self, level: usize) -> Result<Yaml, PError> {
        self.block_prefix(level).unwrap_or_default();
        let mut m = vec![];
        loop {
            self.eat();
            // TODO '?' key
            let k = if m.is_empty() {
                // Mismatch
                let k = self.scalar_flow(level + 1)?;
                if self.sym(b':').is_err() || self.inv(TakeOpt::OneMore).is_err() {
                    // Return key
                    return Ok(k.yaml);
                }
                k
            } else {
                if self.gap().is_err() {
                    return self.err("map terminator");
                }
                if self.indent(level).is_err() {
                    break;
                }
                let k = err_own!(self.scalar_flow(level + 1), self.err("map key"))?;
                if self.sym(b':').is_err() || self.inv(TakeOpt::OneMore).is_err() {
                    return self.err("map splitter");
                }
                k
            };
            self.eat();
            let v = err_own!(self.scalar(level + 1), self.err("map value"))?;
            m.push((k, v));
        }
        self.eat();
        Ok(Yaml::from_iter(m))
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
