//! Parser components, includes parser error.
//!
//! This parser is a simple greedy algorithm that returns result when
//! matched successfully; try next or return error when mismatched.
//!
//! The [`Loader`] type can simply convert string into [`Node`] type.
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
    error::{PError, PResult},
};
use crate::{repr::Repr, *};
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::ops::{Deref, DerefMut};
use ritelinked::LinkedHashMap;

mod base;
mod error;

macro_rules! tag_prefix {
    () => {
        "tag:yaml.org,2002:"
    };
}

macro_rules! or {
    ($e1:expr; $e2:expr) => {
        match $e1 {
            Ok(v) => Ok(v),
            Err(PError::Mismatch) => $e2,
            Err(e) => Err(e),
        }
    };
    ($e1:expr; $($e2:expr);+) => {
        or!($e1; or!($($e2);+))
    };
}

pub(crate) use tag_prefix;

/// The type of anchor recorder in the [`Loader`].
pub type Anchors<R> = LinkedHashMap<String, Node<R>>;
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
    keep_anchors: bool,
    anchors: Anchors<R>,
}

impl<'a, R: Repr> Loader<'a, R> {
    /// Create YAML loader includes a parser.
    pub fn new(doc: &'a [u8]) -> Self {
        Self {
            parser: Parser::new(doc),
            keep_anchors: false,
            anchors: Anchors::new(),
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
    /// Keep the anchor insertion.
    ///
    /// This will make [`Yaml::Alias`] have a placeholder
    /// and adding anchor information in the [`Node`].
    pub fn keep_anchors(self, keep_anchors: bool) -> Self {
        Self {
            keep_anchors,
            ..self
        }
    }

    /// Consume this loader and return the recorded anchors.
    pub fn get_anchors(self) -> Anchors<R> {
        self.anchors
    }

    /// YAML entry point, return entire doc if exist.
    pub fn parse(&mut self) -> PResult<Vec<Node<R>>> {
        loop {
            match self.context(Parser::directive) {
                Ok(()) => (),
                Err(PError::Mismatch) => break,
                Err(e) => return Err(e),
            }
        }
        self.gap(true).unwrap_or_default();
        self.sym_seq(b"---").unwrap_or_default();
        let mut v = vec![self.doc()?];
        loop {
            self.gap(true).unwrap_or_default();
            if self.food().is_empty() {
                break;
            }
            if self.sym_seq(b"---").is_err() {
                return self.err("document splitter");
            }
            v.push(self.doc()?);
        }
        Ok(v)
    }

    /// Match one doc block.
    pub fn doc(&mut self) -> PResult<Node<R>> {
        self.context(|p| p.bound().unwrap_or_default());
        self.forward();
        let ret = self.scalar(0, false, false)?;
        self.gap(true).unwrap_or_default();
        self.sym_seq(b"...").unwrap_or_default();
        self.forward();
        Ok(ret)
    }

    /// Match doc end.
    pub fn doc_end(&mut self) -> bool {
        if self.food().is_empty() {
            true
        } else {
            self.context(|p| {
                let b = p.sym_seq(b"---").is_ok() || p.sym_seq(b"...").is_ok();
                if b {
                    p.backward();
                }
                b
            })
        }
    }

    /// Match scalar.
    pub fn scalar(&mut self, level: usize, nest: bool, inner: bool) -> PResult<Node<R>> {
        self.scalar_node(|p| {
            if let Ok(s) = p.string_literal(level) {
                Ok(R::repr(Yaml::Str(s)))
            } else if let Ok(s) = p.string_folded(level) {
                Ok(R::repr(Yaml::Str(s)))
            } else {
                or!(
                    p.seq(level, nest);
                    p.map(level, nest, inner);
                    p.scalar_term(level, inner)
                )
            }
        })
    }

    /// Match flow scalar.
    pub fn scalar_flow(&mut self, level: usize, inner: bool) -> PResult<Node<R>> {
        self.scalar_node(|p| p.scalar_term(level, inner))
    }

    fn scalar_node<F>(&mut self, f: F) -> PResult<Node<R>>
    where
        F: FnOnce(&mut Self) -> PResult<R::Ty>,
    {
        let mut anchor = self.anchor().unwrap_or_default();
        if !anchor.is_empty() {
            self.bound()?;
        }
        self.forward();
        let tag = self.tag().unwrap_or_default();
        if !tag.is_empty() {
            self.bound()?;
        }
        self.forward();
        let anchor2 = self.anchor().unwrap_or_default();
        if anchor.is_empty() && !anchor2.is_empty() {
            self.bound()?;
            anchor = anchor2;
        } else if !anchor.is_empty() && !anchor2.is_empty() {
            return self.err("duplicated anchor definition");
        }
        self.forward();
        let pos = self.indicator();
        let yaml = f(self)?;
        self.forward();
        let node = Node::new_repr(yaml, pos, &tag);
        if !anchor.is_empty() && self.anchors.insert(anchor, node.clone()).is_some() {
            self.err("duplicated anchor definition")
        } else {
            Ok(node)
        }
    }

    /// Match flow scalar terminal.
    pub fn scalar_term(&mut self, level: usize, inner: bool) -> PResult<R::Ty> {
        let yaml = if let Ok(s) = self.float() {
            R::repr(Yaml::Float(s))
        } else if let Ok(s) = self.sci_float() {
            R::repr(Yaml::Float(s))
        } else if let Ok(s) = self.int() {
            R::repr(Yaml::Int(s))
        } else if let Ok(s) = self.anchor_use() {
            if let Some(node) = self.anchors.get(&s) {
                if self.keep_anchors {
                    R::repr(Yaml::Alias(s))
                } else {
                    node.clone_yaml()
                }
            } else {
                return self.err("anchor referenced before definition");
            }
        } else if let Ok(s) = self.string_quoted(b'\'', b"''") {
            R::repr(Yaml::Str(s))
        } else if let Ok(s) = self.string_quoted(b'"', b"\\\"") {
            R::repr(Yaml::Str(Parser::escape(&s)))
        } else if let Ok(s) = self.string_plain(level, inner) {
            R::repr(match s.as_str() {
                "~" | "null" | "Null" | "NULL" => Yaml::Null,
                "true" | "True" | "TRUE" => Yaml::Bool(true),
                "false" | "False" | "FALSE" => Yaml::Bool(false),
                ".nan" | ".NaN" | ".NAN" => Yaml::Float("NaN".to_string()),
                ".inf" | ".Inf" | ".INF" => Yaml::Float("inf".to_string()),
                "-.inf" | "-.Inf" | "-.INF" => Yaml::Float("-inf".to_string()),
                _ => Yaml::Str(s),
            })
        } else {
            or!(
                self.seq_flow(level);
                self.map_flow(level);
                Ok(R::repr(Yaml::Null))
            )?
        };
        Ok(yaml)
    }

    /// Match flow sequence.
    pub fn seq_flow(&mut self, level: usize) -> PResult<R::Ty> {
        self.sym(b'[')?;
        let mut v = vec![];
        loop {
            self.inv(TakeOpt::More(0))?;
            self.forward();
            if self.sym(b']').is_ok() {
                break;
            }
            self.forward();
            v.push(or!(
                self.scalar(level + 1, false, true);
                self.err("flow sequence item")
            )?);
            self.inv(TakeOpt::More(0))?;
            if self.sym(b',').is_err() {
                self.inv(TakeOpt::More(0))?;
                self.sym(b']')?;
                break;
            }
        }
        self.forward();
        Ok(R::repr(v.into_iter().collect()))
    }

    /// Match flow map.
    pub fn map_flow(&mut self, level: usize) -> PResult<R::Ty> {
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
                let k = or!(
                    self.scalar(level + 1, false, true);
                    self.err("flow map key")
                )?;
                if self.gap(true).is_ok() {
                    self.ind(level)?;
                }
                k
            } else {
                or!(self.scalar_flow(level + 1, true); self.err("flow map key"))?
            };
            if self.sym(b':').is_err() {
                return self.err("flow map splitter");
            }
            self.context(|p| p.bound().unwrap_or_default());
            self.forward();
            let v = or!(
                self.scalar(level + 1, false, true);
                self.err("flow map value")
            )?;
            m.push((k, v));
            if self.sym(b',').is_err() {
                self.inv(TakeOpt::More(0))?;
                self.sym(b'}')?;
                break;
            }
        }
        self.forward();
        Ok(R::repr(m.into_iter().collect()))
    }

    /// Match sequence.
    pub fn seq(&mut self, level: usize, nest: bool) -> PResult<R::Ty> {
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
                    return self.err("sequence terminator");
                }
                if self.doc_end() {
                    break;
                }
                if let Ok(b) = self.unind(level) {
                    downgrade = b;
                } else {
                    break;
                }
                if self.sym(b'-').is_err() || self.bound().is_err() {
                    break;
                }
            }
            self.forward();
            v.push(or!(
                self.scalar(if downgrade { level } else { level + 1 }, false, false);
                self.err("sequence item")
            )?);
        }
        // Keep last wrapping
        self.backward();
        Ok(R::repr(v.into_iter().collect()))
    }

    /// Match map.
    pub fn map(&mut self, level: usize, nest: bool, inner: bool) -> PResult<R::Ty> {
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
                    let k = or!(self.scalar(level + 1, true, inner); self.err("map key"))?;
                    if self.gap(true).is_ok() {
                        self.ind(level)?;
                    }
                    k
                } else {
                    self.scalar_flow(level + 1, inner)?
                };
                if self.sym(b':').is_err() || self.bound().is_err() {
                    // Return key
                    return Ok(k.clone_yaml());
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
                    let k = or!(self.scalar(level + 1, true, inner); self.err("map key"))?;
                    if self.gap(true).is_ok() {
                        self.ind(level)?;
                    }
                    k
                } else {
                    or!(self.scalar_flow(level + 1, inner); self.err("map key"))?
                };
                if self.sym(b':').is_err() || self.bound().is_err() {
                    return self.err("map splitter");
                }
                k
            };
            self.forward();
            let v = or!(self.scalar(level + 1, true, false); self.err("map value"))?;
            m.push((k, v));
        }
        // Keep last wrapping
        self.backward();
        Ok(R::repr(m.into_iter().collect()))
    }
}

impl<'a, R: Repr> Deref for Loader<'a, R> {
    type Target = Parser<'a>;

    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}

impl<R: Repr> DerefMut for Loader<'_, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}

/// Parse YAML document into [`alloc::rc::Rc`] or [`alloc::sync::Arc`] data holder.
/// Return an sequence of nodes and insert the anchors automatically.
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
/// let root = parse(doc).unwrap();
/// assert_eq!(root, vec![node!({
///     "name" => "Bob",
///     "married" => true,
///     "age" => 46,
/// })]);
/// // Node with Arc repr
/// let root = parse(doc).unwrap();
/// assert_eq!(root, vec![node!(arc{
///     "name" => "Bob",
///     "married" => true,
///     "age" => 46,
/// })]);
/// ```
///
/// Please see [`Loader`] for parser customization.
pub fn parse<R: Repr>(doc: &str) -> Result<Seq<R>, PError> {
    Loader::new(doc.as_bytes()).parse()
}
