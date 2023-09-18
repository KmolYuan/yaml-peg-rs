//! Dumper components.
use crate::{parser::Anchors, repr::Repr, *};
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::fmt::Write;

/// Newline symbol in common platforms.
///
/// There is only two symbols in the option,
/// "\r\n" in Windows platform, otherwise uses "\n".
///
/// Please be aware that your storage can be used the symbol of Windows.
pub const NL: &str = if cfg!(windows) { "\r\n" } else { "\n" };

#[derive(Eq, PartialEq)]
enum Root {
    Scalar,
    Map,
    Array,
}

/// Dumper for nodes.
pub struct Dumper<'a, R: Repr> {
    node: &'a Node<R>,
    root: Root,
    level: usize,
    anchors: &'a Anchors<R>,
}

impl<'a, R: Repr> Dumper<'a, R> {
    /// Create the dumper.
    pub fn new(node: &'a Node<R>, anchors: &'a Anchors<R>) -> Self {
        Self { node, root: Root::Scalar, level: 0, anchors }
    }

    fn part(&self, node: &'a Node<R>, root: Root, level: usize) -> String {
        Self { node, root, level, anchors: self.anchors }.dump()
    }

    /// Dump into string.
    pub fn dump(&self) -> String {
        let mut doc = String::new();
        if let Some(a) = self
            .anchors
            .iter()
            .find_map(|(k, v)| if v == self.node { Some(k) } else { None })
        {
            write!(doc, "&{a} ").unwrap();
        }
        let tag = self.node.tag();
        if !tag.is_empty() && !tag.starts_with(parser::tag_prefix!()) {
            if tag.starts_with(parser::tag_prefix!()) {
                write!(doc, "!!{tag} ").unwrap();
            } else if parser::Parser::new(tag.as_bytes()).identifier().is_ok() {
                write!(doc, "!{tag} ").unwrap();
            } else {
                write!(doc, "!<{tag}> ").unwrap();
            }
        }
        let ind = "  ".repeat(self.level);
        match &self.node.yaml() {
            Yaml::Null => doc += "null",
            Yaml::Bool(b) => write!(doc, "{b}").unwrap(),
            Yaml::Int(n) | Yaml::Float(n) => doc += n,
            Yaml::Str(s) => {
                if s.lines().nth(1).is_some() {
                    // Multiline string
                    let s = s
                        .lines()
                        .map(|s| {
                            if s.is_empty() {
                                String::new()
                            } else {
                                ind.to_string() + s.trim_end()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(NL);
                    write!(doc, "|{NL}{ind}{}", s.trim()).unwrap();
                } else if parser::Parser::new(s.as_bytes())
                    .string_plain(0, false)
                    .is_err()
                {
                    // Literal string, not plain string
                    write!(doc, "{s:?}").unwrap();
                } else {
                    // Single line string
                    doc += s;
                }
            }
            Yaml::Seq(v) => {
                let mut buf = NL.to_string();
                for (i, node) in v.iter().enumerate() {
                    if i != 0 || self.level != 0 {
                        buf += &ind;
                    }
                    let s = self.part(node, Root::Array, self.level + 1);
                    write!(buf, "- {s}{NL}").unwrap();
                }
                buf.truncate(buf.len() - NL.len());
                doc += &buf;
            }
            Yaml::Map(m) => {
                let mut buf = match self.root {
                    Root::Map => NL.to_string(),
                    _ => String::new(),
                };
                for (i, (k, v)) in m.iter().enumerate() {
                    if i != 0 || self.root == Root::Map {
                        buf += &ind;
                    }
                    let s = self.part(k, Root::Map, self.level + 1);
                    if matches!(k.yaml(), Yaml::Map(_) | Yaml::Seq(_)) {
                        let pre_ind = "  ".repeat(self.level + 1);
                        write!(buf, "?{pre_ind}{NL}{s}{NL}{ind}").unwrap();
                    } else {
                        buf += &s;
                    };
                    buf += ":";
                    buf += &match v.yaml() {
                        Yaml::Map(_) => self.part(v, Root::Map, self.level + 1),
                        Yaml::Seq(_) if self.root == Root::Array && i == 0 => {
                            self.part(v, Root::Map, self.level)
                        }
                        Yaml::Seq(_) => self.part(v, Root::Map, self.level + 1),
                        _ => format!(" {}", self.part(v, Root::Map, self.level + 1)),
                    };
                    buf += NL;
                }
                buf.truncate(buf.len() - NL.len());
                doc += &buf;
            }
            Yaml::Alias(a) => write!(doc, "*{a}").unwrap(),
        };
        doc
    }
}

/// Dump the YAML data in to block format.
///
/// Dumper will use plain string when the string is none-wrapped,
/// otherwise it use literal string and trim the last white spaces.
///
/// ```
/// use yaml_peg::{dump, node, dumper::NL};
///
/// let doc = dump(&[
///     node!({
///         "a" => "b",
///         "c" => "d",
///     }),
/// ], &[]);
/// let ans = "\
/// a: b
/// c: d
/// ";
/// assert_eq!(doc, ans.replace('\n', NL));
/// ```
///
/// When calling [`parse`] function then [`dump`] the string, the string can be
/// reformatted.
///
/// Anchors can pass with the result of the [`Loader`](crate::parser::Loader).
pub fn dump<R: Repr>(nodes: &[Node<R>], anchors: &[Anchors<R>]) -> String {
    let anchors_empty = Anchors::new();
    nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            let anchors = if i < anchors.len() {
                &anchors[i]
            } else {
                &anchors_empty
            };
            let doc = Dumper::new(node, anchors).dump() + NL;
            match i {
                0 => doc,
                _ => format!("---{NL}{}", doc.trim_start()),
            }
        })
        .collect()
}
