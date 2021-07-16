//! Parser components.
pub use self::error::*;
pub use self::grammar::*;
pub use self::kernel::*;
use crate::*;
use std::iter::FromIterator;

mod error;
mod grammar;
mod kernel;

macro_rules! err_own {
    ($e:expr, $then:expr) => {
        match $e {
            Ok(v) => Ok(v),
            Err(PError::Mismatch) => $then,
            Err(e) => Err(e),
        }
    };
}

/// The basic implementation.
///
/// These sub-parser returns [`PError`], and failed immediately for [`PError::Terminate`].
/// Additionally, they should eat the string by themself.
impl Parser<'_> {
    /// YAML entry point, return entire doc if exist.
    pub fn parse(&mut self) -> Result<Array, PError> {
        self.inv(TakeOpt::More(0))?;
        self.seq(b"---").unwrap_or_default();
        self.gap(true).unwrap_or_default();
        self.forward();
        let mut v = vec![];
        v.push(self.doc()?);
        loop {
            self.gap(true).unwrap_or_default();
            if self.food().is_empty() {
                break;
            }
            if self.seq(b"---").is_err() {
                return self.err("splitter");
            }
            self.gap(true).unwrap_or_default();
            self.forward();
            v.push(self.doc()?);
        }
        Ok(v)
    }

    /// Match one doc block.
    pub fn doc(&mut self) -> Result<Node, PError> {
        let ret = self.scalar(0, false, false)?;
        self.gap(true).unwrap_or_default();
        self.seq(b"...").unwrap_or_default();
        self.forward();
        Ok(ret)
    }

    /// Match doc end.
    pub fn doc_end(&mut self) -> bool {
        if self.food().is_empty() {
            true
        } else {
            self.context(|p| {
                let b = p.seq(b"---").is_ok() || p.seq(b"...").is_ok();
                if b {
                    p.backward();
                }
                b
            })
        }
    }

    /// Match scalar.
    pub fn scalar(&mut self, level: usize, nest: bool, inner: bool) -> Result<Node, PError> {
        self.scalar_inner(|p| {
            if let Ok(s) = p.string_literal(level) {
                Ok(Yaml::Str(s))
            } else if let Ok(s) = p.string_folded(level) {
                Ok(Yaml::Str(s))
            } else {
                err_own!(
                    p.array(level, nest),
                    err_own!(p.map(level, nest, inner), p.scalar_term(level, inner))
                )
            }
        })
    }

    /// Match flow scalar.
    pub fn scalar_flow(&mut self, level: usize, inner: bool) -> Result<Node, PError> {
        self.scalar_inner(|p| p.scalar_term(level, inner))
    }

    fn scalar_inner<F>(&mut self, f: F) -> Result<Node, PError>
    where
        F: Fn(&mut Self) -> Result<Yaml, PError>,
    {
        let anchor = self.anchor().unwrap_or_default();
        if !anchor.is_empty() {
            self.bound()?;
        }
        self.forward();
        let ty = self.ty().unwrap_or_default();
        if !ty.is_empty() {
            self.bound()?;
        }
        self.forward();
        let pos = self.indicator();
        let yaml = f(self)?;
        self.forward();
        Ok(Node::new(yaml, pos, &anchor, &ty))
    }

    /// Match flow scalar terminal.
    pub fn scalar_term(&mut self, level: usize, inner: bool) -> Result<Yaml, PError> {
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
        } else if let Ok(s) = self.float() {
            Yaml::Float(s.trim_end_matches(|c| ".0".contains(c)).into())
        } else if let Ok(s) = self.sci_float() {
            Yaml::Float(s)
        } else if let Ok(s) = self.int() {
            Yaml::Int(s)
        } else if let Ok(s) = self.anchor_use() {
            Yaml::Anchor(s)
        } else if let Ok(s) = self.string_flow(level, inner) {
            Yaml::Str(s)
        } else {
            err_own!(
                self.array_flow(level),
                err_own!(self.map_flow(level), Ok(Yaml::Null))
            )?
        };
        Ok(yaml)
    }

    /// Match flow array.
    pub fn array_flow(&mut self, level: usize) -> Result<Yaml, PError> {
        self.sym(b'[')?;
        let mut v = vec![];
        loop {
            self.inv(TakeOpt::More(0))?;
            self.forward();
            if self.sym(b']').is_ok() {
                break;
            }
            self.forward();
            v.push(err_own!(
                self.scalar(level + 1, false, true),
                self.err("flow array item")
            )?);
            self.inv(TakeOpt::More(0))?;
            if self.sym(b',').is_err() {
                self.inv(TakeOpt::More(0))?;
                self.sym(b']')?;
                break;
            }
        }
        self.forward();
        Ok(Yaml::from_iter(v))
    }

    /// Match flow map.
    pub fn map_flow(&mut self, level: usize) -> Result<Yaml, PError> {
        self.sym(b'{')?;
        let mut m = vec![];
        loop {
            self.inv(TakeOpt::More(0))?;
            self.forward();
            if self.sym(b'}').is_ok() {
                break;
            }
            self.forward();
            let k = if self.complex_mapping().is_ok() {
                self.forward();
                let k = err_own!(
                    self.scalar(level + 1, false, true),
                    self.err("flow map key")
                )?;
                if self.gap(true).is_ok() {
                    self.ind(level)?;
                }
                k
            } else {
                err_own!(
                    self.scalar_flow(level + 1, true),
                    self.err("flow map value")
                )?
            };
            if self.sym(b':').is_err() || self.bound().is_err() {
                return self.err("map");
            }
            self.forward();
            let v = err_own!(self.scalar(level + 1, false, true), self.err("map"))?;
            m.push((k, v));
            if self.sym(b',').is_err() {
                self.inv(TakeOpt::More(0))?;
                self.sym(b'}')?;
                break;
            }
        }
        self.forward();
        Ok(Yaml::from_iter(m))
    }

    /// Match array.
    pub fn array(&mut self, level: usize, nest: bool) -> Result<Yaml, PError> {
        let mut v = vec![];
        loop {
            self.forward();
            let mut downgrade = false;
            if v.is_empty() {
                // Mismatch
                if nest {
                    self.gap(true)?;
                    downgrade = self.unind(level)?;
                } else if self.gap(true).is_ok() {
                    downgrade = self.unind(level)?;
                }
                self.sym(b'-')?;
                self.bound()?;
            } else {
                if self.gap(true).is_err() {
                    return self.err("array terminator");
                }
                if self.doc_end() {
                    break;
                }
                if let Ok(b) = self.unind(level) {
                    downgrade = b
                } else {
                    break;
                }
                if self.sym(b'-').is_err() || self.bound().is_err() {
                    break;
                }
            }
            self.forward();
            v.push(err_own!(
                self.scalar(if downgrade { level } else { level + 1 }, false, false),
                self.err("array item")
            )?);
        }
        // Keep last wrapping
        self.backward();
        Ok(Yaml::from_iter(v))
    }

    /// Match map.
    pub fn map(&mut self, level: usize, nest: bool, inner: bool) -> Result<Yaml, PError> {
        let mut m = vec![];
        loop {
            self.forward();
            let k = if m.is_empty() {
                // Mismatch
                if nest {
                    self.gap(true)?;
                    self.ind(level)?;
                } else if self.gap(true).is_ok() {
                    self.ind(level)?;
                }
                self.forward();
                let k = if self.complex_mapping().is_ok() {
                    self.forward();
                    let k = err_own!(self.scalar(level + 1, true, inner), self.err("map key"))?;
                    if self.gap(true).is_ok() {
                        self.ind(level)?;
                    }
                    k
                } else {
                    self.scalar_flow(level + 1, inner)?
                };
                if self.sym(b':').is_err() || self.bound().is_err() {
                    // Return key
                    return Ok(k.into_yaml());
                }
                k
            } else {
                if self.gap(true).is_err() {
                    return self.err("map terminator");
                }
                if self.doc_end() || self.ind(level).is_err() {
                    break;
                }
                self.forward();
                let k = if self.complex_mapping().is_ok() {
                    self.forward();
                    let k = err_own!(self.scalar(level + 1, true, inner), self.err("map key"))?;
                    if self.gap(true).is_ok() {
                        self.ind(level)?;
                    }
                    k
                } else {
                    err_own!(self.scalar_flow(level + 1, inner), self.err("map key"))?
                };
                if self.sym(b':').is_err() || self.bound().is_err() {
                    return self.err("map splitter");
                }
                k
            };
            self.forward();
            let v = err_own!(self.scalar(level + 1, true, false), self.err("map value"))?;
            m.push((k, v));
        }
        // Keep last wrapping
        self.backward();
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
pub fn parse(doc: &str) -> Result<Array, String> {
    Parser::new(doc.as_bytes())
        .parse()
        .map_err(|e| e.into_error(doc))
}
