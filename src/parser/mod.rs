//! Parser components, includes parser error.
//!
//! This parser is a simple greedy algorithm that returns result when
//! matched successfully; try next or return error when mismatched.
//!
//! The [`Loader`] type can simply convert string into [`NodeBase`] type.
//!
//! Each pattern (the method of [`Parser`] type) is called "sub-parser",
//! which returns a `Result<T, PError>` type, where `T` is the return type.
//!
//! # Errors
//!
//! ## Document
//!
//! **WRONG**: Invalid tag directive will be ignored.
//!
//! + document splitter: Error about the document splitter `---` / `...`.
//! + checked version: Version directive `%YAML 1.2` is used again.
//! + version: Version directive is wrong, must be `1.2`.
//!
//! ## Structure
//!
//! ### Flow Array
//!
//! + flow sequence item: Item in `[]` bracket is invalid.
//!
//! ### Flow Map
//!
//! + flow map key: Key of map item in `{}` bracket is invalid.
//! + flow map value: Value of map item in `{}` bracket is invalid.
//! + flow map splitter: Splitter `:` of map item in `{}` bracket is invalid.
//!
//! ### Array
//!
//! + sequence item: Item behind `-` indicator is invalid.
//! + sequence terminator: The end of sequence is invalid, may caused by the last item
//!   (like wrapped string).
//!
//! ### Map
//!
//! + map key: Key of map item is invalid.
//! + map value: Value of map item is invalid.
//! + map splitter: Splitter `:` of map item is invalid.
//! + map terminator: The end of map is invalid, may caused by the last value
//!   (like wrapped string).
pub use self::{
    base::{Parser, TakeOpt},
    error::PError,
};
use crate::{repr::Repr, *};
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

mod base;
mod error;

macro_rules! tag_prefix {
    () => {
        "tag:yaml.org,2002:"
    };
}

macro_rules! err {
    ($e:expr, $then:expr) => {
        match $e {
            Ok(v) => Ok(v),
            Err(PError::Mismatch) => $then,
            Err(e) => Err(e),
        }
    };
}

pub(crate) use tag_prefix;

/// The default prefix of the YAML sub tag.
pub const DEFAULT_PREFIX: &str = tag_prefix!();

/// A parser with YAML grammar, support UTF-8 characters.
///
/// This loader will output YAML nodes with representation notation [`repr::Repr`].
/// If you just want to use sub-parser, please see [`Parser`].
///
/// A simple example for parsing YAML only:
///
/// ```
/// use yaml_peg::{node, parser::Loader};
///
/// let n = Loader::new(b"true").parse().unwrap();
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
pub struct Loader<'a, R: Repr> {
    /// Parser base.
    pub parser: Parser<'a>,
    /// A visitor of anchors.
    pub anchors: AnchorBase<R>,
}

impl<'a, R: Repr> Loader<'a, R> {
    /// Create YAML loader includes a parser.
    pub fn new(doc: &'a [u8]) -> Self {
        Self {
            parser: Parser::new(doc),
            anchors: AnchorBase::new(),
        }
    }
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
impl<R: Repr> Loader<'_, R> {
    /// YAML entry point, return entire doc if exist.
    pub fn parse(&mut self) -> Result<Seq<R>, PError> {
        loop {
            match self.parser.context(Parser::directive) {
                Ok(()) => (),
                Err(PError::Mismatch) => break,
                Err(e) => return Err(e),
            }
        }
        self.parser.gap(true).unwrap_or_default();
        self.parser.sym_seq(b"---").unwrap_or_default();
        self.parser.gap(true).unwrap_or_default();
        self.parser.forward();
        let mut v = vec![self.doc()?];
        loop {
            self.parser.gap(true).unwrap_or_default();
            if self.parser.food().is_empty() {
                break;
            }
            if self.parser.sym_seq(b"---").is_err() {
                return self.parser.err("document splitter");
            }
            self.parser.gap(true).unwrap_or_default();
            self.parser.forward();
            v.push(self.doc()?);
        }
        Ok(v)
    }

    /// Match one doc block.
    pub fn doc(&mut self) -> Result<NodeBase<R>, PError> {
        self.parser.ind_define(0)?;
        self.parser.forward();
        let ret = self.scalar(0, false, false)?;
        self.parser.gap(true).unwrap_or_default();
        self.parser.sym_seq(b"...").unwrap_or_default();
        self.parser.forward();
        Ok(ret)
    }

    /// Match doc end.
    pub fn doc_end(&mut self) -> bool {
        if self.parser.food().is_empty() {
            true
        } else {
            self.parser.context(|p| {
                let b = p.sym_seq(b"---").is_ok() || p.sym_seq(b"...").is_ok();
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
            if let Ok(s) = p.parser.string_literal(level) {
                Ok(YamlBase::Str(s))
            } else if let Ok(s) = p.parser.string_folded(level) {
                Ok(YamlBase::Str(s))
            } else {
                err!(
                    p.seq(level, nest),
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
        let anchor = self.parser.anchor().unwrap_or_default();
        if !anchor.is_empty() {
            self.parser.bound()?;
        }
        self.parser.forward();
        let tag = self.parser.tag().unwrap_or_default();
        if !tag.is_empty() {
            self.parser.bound()?;
        }
        self.parser.forward();
        let pos = self.parser.indicator();
        let yaml = f(self)?;
        self.parser.forward();
        let node = NodeBase::new(yaml, pos, &tag, &anchor);
        if !anchor.is_empty() {
            self.anchors.insert(anchor, node.clone());
        }
        Ok(node)
    }

    /// Match flow scalar terminal.
    pub fn scalar_term(&mut self, level: usize, inner: bool) -> Result<YamlBase<R>, PError> {
        let yaml = if let Ok(s) = self.parser.float() {
            YamlBase::Float(s)
        } else if let Ok(s) = self.parser.sci_float() {
            YamlBase::Float(s)
        } else if let Ok(s) = self.parser.int() {
            YamlBase::Int(s)
        } else if let Ok(s) = self.parser.anchor_use() {
            YamlBase::Anchor(s)
        } else if let Ok(s) = self.parser.string_quoted(b'\'', b"''") {
            YamlBase::Str(s)
        } else if let Ok(s) = self.parser.string_quoted(b'"', b"\\\"") {
            YamlBase::Str(Parser::escape(&s))
        } else if let Ok(s) = self.parser.string_plain(level, inner) {
            match s.as_str() {
                "~" | "null" | "Null" | "NULL" => YamlBase::Null,
                "true" | "True" | "TRUE" => YamlBase::Bool(true),
                "false" | "False" | "FALSE" => YamlBase::Bool(false),
                ".nan" | ".NaN" | ".NAN" => YamlBase::Float("NaN".to_string()),
                ".inf" | ".Inf" | ".INF" => YamlBase::Float("inf".to_string()),
                "-.inf" | "-.Inf" | "-.INF" => YamlBase::Float("-inf".to_string()),
                _ => YamlBase::Str(s),
            }
        } else {
            err!(
                self.seq_flow(level),
                err!(self.map_flow(level), Ok(YamlBase::Null))
            )?
        };
        Ok(yaml)
    }

    /// Match flow sequence.
    pub fn seq_flow(&mut self, level: usize) -> Result<YamlBase<R>, PError> {
        self.parser.sym(b'[')?;
        let mut v = vec![];
        loop {
            self.parser.inv(TakeOpt::More(0))?;
            self.parser.forward();
            if self.parser.sym(b']').is_ok() {
                break;
            }
            self.parser.forward();
            v.push(err!(
                self.scalar(level + 1, false, true),
                self.parser.err("flow sequence item")
            )?);
            self.parser.inv(TakeOpt::More(0))?;
            if self.parser.sym(b',').is_err() {
                self.parser.inv(TakeOpt::More(0))?;
                self.parser.sym(b']')?;
                break;
            }
        }
        self.parser.forward();
        Ok(v.into_iter().collect())
    }

    /// Match flow map.
    pub fn map_flow(&mut self, level: usize) -> Result<YamlBase<R>, PError> {
        self.parser.sym(b'{')?;
        let mut m = vec![];
        loop {
            self.parser.inv(TakeOpt::More(0))?;
            self.parser.forward();
            if self.parser.sym(b'}').is_ok() {
                break;
            }
            self.parser.forward();
            let k = if self.parser.complex_mapping().is_ok() {
                self.parser.forward();
                let k = err!(
                    self.scalar(level + 1, false, true),
                    self.parser.err("flow map key")
                )?;
                if self.parser.gap(true).is_ok() {
                    self.parser.ind(level)?;
                }
                k
            } else {
                err!(
                    self.scalar_flow(level + 1, true),
                    self.parser.err("flow map key")
                )?
            };
            if self.parser.sym(b':').is_err() || self.parser.bound().is_err() {
                return self.parser.err("flow map splitter");
            }
            self.parser.forward();
            let v = err!(
                self.scalar(level + 1, false, true),
                self.parser.err("flow map value")
            )?;
            m.push((k, v));
            if self.parser.sym(b',').is_err() {
                self.parser.inv(TakeOpt::More(0))?;
                self.parser.sym(b'}')?;
                break;
            }
        }
        self.parser.forward();
        Ok(m.into_iter().collect())
    }

    /// Match sequence.
    pub fn seq(&mut self, level: usize, nest: bool) -> Result<YamlBase<R>, PError> {
        let mut v = vec![];
        loop {
            self.parser.forward();
            let mut downgrade = false;
            if v.is_empty() {
                // First item
                if nest {
                    self.parser.gap(true)?;
                    self.parser.ind_define(level)?;
                } else if self.parser.gap(true).is_ok() {
                    // Root
                    self.parser.unind(level)?;
                }
                self.parser.sym(b'-')?;
                self.parser.bound()?;
            } else {
                if self.parser.gap(true).is_err() {
                    return self.parser.err("sequence terminator");
                }
                if self.doc_end() {
                    break;
                }
                if let Ok(b) = self.parser.unind(level) {
                    downgrade = b
                } else {
                    break;
                }
                if self.parser.sym(b'-').is_err() || self.parser.bound().is_err() {
                    break;
                }
            }
            self.parser.forward();
            v.push(err!(
                self.scalar(if downgrade { level } else { level + 1 }, false, false),
                self.parser.err("sequence item")
            )?);
        }
        // Keep last wrapping
        self.parser.backward();
        Ok(v.into_iter().collect())
    }

    /// Match map.
    pub fn map(&mut self, level: usize, nest: bool, inner: bool) -> Result<YamlBase<R>, PError> {
        let mut m = vec![];
        loop {
            self.parser.forward();
            let k = if m.is_empty() {
                // First item
                if nest {
                    self.parser.gap(true)?;
                    self.parser.ind_define(level)?;
                } else if self.parser.gap(true).is_ok() {
                    // Root
                    self.parser.ind(level)?;
                }
                self.parser.forward();
                let k = if self.parser.complex_mapping().is_ok() {
                    self.parser.forward();
                    let k = err!(
                        self.scalar(level + 1, true, inner),
                        self.parser.err("map key")
                    )?;
                    if self.parser.gap(true).is_ok() {
                        self.parser.ind(level)?;
                    }
                    k
                } else {
                    self.scalar_flow(level + 1, inner)?
                };
                if self.parser.sym(b':').is_err() || self.parser.bound().is_err() {
                    // Return key
                    return Ok(k.yaml().clone());
                }
                k
            } else {
                if self.parser.gap(true).is_err() {
                    return self.parser.err("map terminator");
                }
                if self.doc_end() || self.parser.ind(level).is_err() {
                    break;
                }
                self.parser.forward();
                let k = if self.parser.complex_mapping().is_ok() {
                    self.parser.forward();
                    let k = err!(
                        self.scalar(level + 1, true, inner),
                        self.parser.err("map key")
                    )?;
                    if self.parser.gap(true).is_ok() {
                        self.parser.ind(level)?;
                    }
                    k
                } else {
                    err!(
                        self.scalar_flow(level + 1, inner),
                        self.parser.err("map key")
                    )?
                };
                if self.parser.sym(b':').is_err() || self.parser.bound().is_err() {
                    return self.parser.err("map splitter");
                }
                k
            };
            self.parser.forward();
            let v = err!(
                self.scalar(level + 1, true, false),
                self.parser.err("map value")
            )?;
            m.push((k, v));
        }
        // Keep last wrapping
        self.parser.backward();
        Ok(m.into_iter().collect())
    }
}

/// Parse YAML document into [`alloc::rc::Rc`] or [`alloc::sync::Arc`] data holder.
/// Return an sequence of nodes and the anchors.
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
pub fn parse<R: Repr>(doc: &str) -> Result<(Seq<R>, AnchorBase<R>), String> {
    let mut p = Loader::new(doc.as_bytes());
    p.parse()
        .map_err(|e| e.into_error(doc))
        .map(|a| (a, p.anchors))
}
