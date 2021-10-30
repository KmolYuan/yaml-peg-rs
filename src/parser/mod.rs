//! Parser components.
pub use self::{
    base::{TakeOpt, DEFAULT_PREFIX},
    error::PError,
};
use crate::{repr::Repr, *};
use alloc::{string::String, vec, vec::Vec};
use ritelinked::LinkedHashMap;

mod base;
mod directive;
mod error;
mod grammar;

macro_rules! err {
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
/// use yaml_peg::{node, parser::Parser};
///
/// let n = Parser::new(b"true").parse().unwrap();
/// assert_eq!(n, vec![node!(true)]);
/// ```
///
/// For matching partial grammar, each methods are the sub-parser.
/// The methods have some behaviors:
///
/// + They will move the current cursor if matched.
/// + Returned value:
///     + `Result<(), PError>` represents the sub-parser can be matched and mismatched.
///     + [`PError`] represents the sub-parser can be totally breaked when mismatched.
/// + Use `?` to match a condition.
/// + Use [`Result::unwrap_or_default`] to match an optional condition.
/// + Method [`Parser::forward`] is used to move on.
/// + Method [`Parser::text`] is used to get the matched string.
/// + Method [`Parser::backward`] is used to get back if mismatched.
pub struct Parser<'a, R: Repr> {
    doc: &'a [u8],
    indent: Vec<usize>,
    consumed: u64,
    pub(crate) version_checked: bool,
    pub(crate) tag: LinkedHashMap<String, String>,
    /// Current position.
    pub pos: usize,
    /// Read position.
    pub eaten: usize,
    /// A visitor of anchors.
    pub anchors: AnchorBase<R>,
}

/// The basic implementation.
///
/// These sub-parser returns [`PError`], and failed immediately for [`PError::Terminate`].
/// Additionally, they should eat the string by themself.
///
/// # Parameter `nest`
///
/// The `nest` parameter presents that the expression is in a **map** structure,
/// includes grand parents.
///
/// If `nest` is false, the expression might in the document root.
///
/// # Parameter `inner`
///
/// The `inner` parameter presents that the expression is in a **flow** expression.
impl<R: Repr> Parser<'_, R> {
    /// YAML entry point, return entire doc if exist.
    pub fn parse(&mut self) -> Result<Array<R>, PError> {
        loop {
            if self.context(Self::directive).is_err() {
                break;
            }
        }
        self.gap(true).unwrap_or_default();
        self.seq(b"---").unwrap_or_default();
        self.gap(true).unwrap_or_default();
        self.forward();
        let mut v = vec![self.doc()?];
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
    pub fn doc(&mut self) -> Result<NodeBase<R>, PError> {
        self.ind_define(0)?;
        self.forward();
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
    pub fn scalar(&mut self, level: usize, nest: bool, inner: bool) -> Result<NodeBase<R>, PError> {
        self.scalar_inner(|p| {
            if let Ok(s) = p.string_literal(level) {
                Ok(YamlBase::Str(s))
            } else if let Ok(s) = p.string_folded(level) {
                Ok(YamlBase::Str(s))
            } else {
                err!(
                    p.array(level, nest),
                    err!(p.map(level, nest, inner), p.scalar_term(level, inner))
                )
            }
        })
    }

    /// Match flow scalar.
    pub fn scalar_flow(&mut self, level: usize, inner: bool) -> Result<NodeBase<R>, PError> {
        self.scalar_inner(|p| p.scalar_term(level, inner))
    }

    fn scalar_inner<F>(&mut self, f: F) -> Result<NodeBase<R>, PError>
    where
        F: Fn(&mut Self) -> Result<YamlBase<R>, PError>,
    {
        let anchor = self.anchor().unwrap_or_default();
        if !anchor.is_empty() {
            self.bound()?;
        }
        self.forward();
        let tag = self.tag().unwrap_or_default();
        if !tag.is_empty() {
            self.bound()?;
        }
        self.forward();
        let pos = self.indicator();
        let yaml = f(self)?;
        self.forward();
        let node = NodeBase::new(yaml, pos, &tag, &anchor);
        if !anchor.is_empty() {
            self.anchors.insert(anchor, node.clone());
        }
        Ok(node)
    }

    /// Match flow scalar terminal.
    pub fn scalar_term(&mut self, level: usize, inner: bool) -> Result<YamlBase<R>, PError> {
        let yaml = if let Ok(s) = self.float() {
            YamlBase::Float(s)
        } else if let Ok(s) = self.sci_float() {
            YamlBase::Float(s)
        } else if let Ok(s) = self.int() {
            YamlBase::Int(s)
        } else if let Ok(s) = self.anchor_use() {
            YamlBase::Anchor(s)
        } else if let Ok(s) = self.string_quoted(b'\'', b"''") {
            YamlBase::Str(s)
        } else if let Ok(s) = self.string_quoted(b'"', b"\\\"") {
            YamlBase::Str(Self::escape(&s))
        } else if let Ok(s) = self.string_plain(level, inner) {
            match s.as_str() {
                "~" | "null" => YamlBase::Null,
                "true" => YamlBase::Bool(true),
                "false" => YamlBase::Bool(false),
                ".nan" | ".NaN" | ".NAN" => YamlBase::Float("NaN".to_string()),
                ".inf" | ".Inf" | ".INF" => YamlBase::Float("inf".to_string()),
                "-.inf" | "-.Inf" | "-.INF" => YamlBase::Float("-inf".to_string()),
                _ => YamlBase::Str(s),
            }
        } else {
            err!(
                self.array_flow(level),
                err!(self.map_flow(level), Ok(YamlBase::Null))
            )?
        };
        Ok(yaml)
    }

    /// Match flow array.
    pub fn array_flow(&mut self, level: usize) -> Result<YamlBase<R>, PError> {
        self.sym(b'[')?;
        let mut v = vec![];
        loop {
            self.inv(TakeOpt::More(0))?;
            self.forward();
            if self.sym(b']').is_ok() {
                break;
            }
            self.forward();
            v.push(err!(
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
        Ok(v.into_iter().collect())
    }

    /// Match flow map.
    pub fn map_flow(&mut self, level: usize) -> Result<YamlBase<R>, PError> {
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
                let k = err!(
                    self.scalar(level + 1, false, true),
                    self.err("flow map key")
                )?;
                if self.gap(true).is_ok() {
                    self.ind(level)?;
                }
                k
            } else {
                err!(
                    self.scalar_flow(level + 1, true),
                    self.err("flow map value")
                )?
            };
            if self.sym(b':').is_err() || self.bound().is_err() {
                return self.err("map");
            }
            self.forward();
            let v = err!(self.scalar(level + 1, false, true), self.err("map"))?;
            m.push((k, v));
            if self.sym(b',').is_err() {
                self.inv(TakeOpt::More(0))?;
                self.sym(b'}')?;
                break;
            }
        }
        self.forward();
        Ok(m.into_iter().collect())
    }

    /// Match array.
    pub fn array(&mut self, level: usize, nest: bool) -> Result<YamlBase<R>, PError> {
        let mut v = vec![];
        loop {
            self.forward();
            let mut downgrade = false;
            if v.is_empty() {
                // First item
                if nest {
                    self.gap(true)?;
                    self.ind_define(level)?;
                } else if self.gap(true).is_ok() {
                    // Root
                    self.unind(level)?;
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
            v.push(err!(
                self.scalar(if downgrade { level } else { level + 1 }, false, false),
                self.err("array item")
            )?);
        }
        // Keep last wrapping
        self.backward();
        Ok(v.into_iter().collect())
    }

    /// Match map.
    pub fn map(&mut self, level: usize, nest: bool, inner: bool) -> Result<YamlBase<R>, PError> {
        let mut m = vec![];
        loop {
            self.forward();
            let k = if m.is_empty() {
                // First item
                if nest {
                    self.gap(true)?;
                    self.ind_define(level)?;
                } else if self.gap(true).is_ok() {
                    // Root
                    self.ind(level)?;
                }
                self.forward();
                let k = if self.complex_mapping().is_ok() {
                    self.forward();
                    let k = err!(self.scalar(level + 1, true, inner), self.err("map key"))?;
                    if self.gap(true).is_ok() {
                        self.ind(level)?;
                    }
                    k
                } else {
                    self.scalar_flow(level + 1, inner)?
                };
                if self.sym(b':').is_err() || self.bound().is_err() {
                    // Return key
                    return Ok(k.yaml().clone());
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
                    let k = err!(self.scalar(level + 1, true, inner), self.err("map key"))?;
                    if self.gap(true).is_ok() {
                        self.ind(level)?;
                    }
                    k
                } else {
                    err!(self.scalar_flow(level + 1, inner), self.err("map key"))?
                };
                if self.sym(b':').is_err() || self.bound().is_err() {
                    return self.err("map splitter");
                }
                k
            };
            self.forward();
            let v = err!(self.scalar(level + 1, true, false), self.err("map value"))?;
            m.push((k, v));
        }
        // Keep last wrapping
        self.backward();
        Ok(m.into_iter().collect())
    }
}

/// Parse YAML document into [`alloc::rc::Rc`] or [`alloc::sync::Arc`] data holder.
/// Return an array of nodes and the anchors.
///
/// ```
/// use yaml_peg::{parse, node};
///
/// let doc = "
/// ---
/// name: Bob
/// married: true
/// age: 46
/// ";
/// // Node with Rc repr
/// let (n, anchors) = parse(doc).unwrap();
/// assert_eq!(anchors.len(), 0);
/// assert_eq!(n, vec![node!({
///     "name" => "Bob",
///     "married" => true,
///     "age" => 46,
/// })]);
/// // Node with Arc repr
/// let (n, anchors) = parse(doc).unwrap();
/// assert_eq!(anchors.len(), 0);
/// assert_eq!(n, vec![node!(arc{
///     "name" => "Bob",
///     "married" => true,
///     "age" => 46,
/// })]);
/// ```
pub fn parse<R: Repr>(doc: &str) -> Result<(Array<R>, AnchorBase<R>), String> {
    let mut p = Parser::new(doc.as_bytes());
    p.parse()
        .map_err(|e| e.into_error(doc))
        .map(|a| (a, p.anchors))
}
