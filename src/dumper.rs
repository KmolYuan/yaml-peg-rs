//! Dumper components.
use crate::*;
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

#[cfg(windows)]
pub const NL: &str = "\r\n";
#[cfg(not(windows))]
pub const NL: &str = "\n";

/// The root type.
#[derive(Eq, PartialEq)]
pub enum Root {
    Map,
    Array,
    Scalar,
}

/// The interface for dumping data structure.
pub trait Dumper {
    /// Generate indentation.
    fn ind(level: usize) -> String {
        "  ".repeat(level)
    }

    /// Recursive dump function.
    fn dump(&self, level: usize, root: Root) -> String;
}

impl<R: repr::Repr> Dumper for NodeBase<R> {
    fn dump(&self, level: usize, root: Root) -> String {
        let mut doc = String::new();
        if !self.anchor().is_empty() {
            doc += &format!("&{} ", self.anchor());
        }
        if !self.ty().is_empty() {
            doc += &format!("!!{} ", self.ty());
        }
        let ind = Self::ind(level);
        doc += &match self.yaml() {
            YamlBase::Null => "null".to_string(),
            YamlBase::Bool(b) => b.to_string(),
            YamlBase::Int(n) | YamlBase::Float(n) => n.clone(),
            YamlBase::Str(s) => {
                if s.contains(NL) {
                    let s = s
                        .split(NL)
                        .map(|s| {
                            if s.is_empty() {
                                "".to_string()
                            } else {
                                ind.to_string() + s.trim_end()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(NL);
                    format!("|{}{}{}", NL, ind, s.trim())
                } else {
                    s.clone()
                }
            }
            YamlBase::Array(a) => {
                let mut doc = NL.to_string();
                for (i, node) in a.iter().enumerate() {
                    if i != 0 || level != 0 {
                        doc += &ind;
                    }
                    doc += &format!("- {}{}", node.dump(level + 1, Root::Array), NL);
                }
                doc.truncate(doc.len() - NL.len());
                doc
            }
            YamlBase::Map(m) => {
                let mut doc = if root == Root::Map { NL } else { "" }.to_string();
                for (i, (k, v)) in m.iter().enumerate() {
                    if i != 0 || root == Root::Map {
                        doc += &ind;
                    }
                    let s = k.dump(level + 1, Root::Map);
                    if let YamlBase::Map(_) | YamlBase::Array(_) = k.yaml() {
                        doc += &format!("?{}{}{}{}{}", Self::ind(level + 1), NL, s, NL, ind);
                    } else {
                        doc += &s;
                    }
                    doc += ":";
                    match v.yaml() {
                        YamlBase::Map(_) => {
                            doc += &v.dump(level + 1, Root::Map);
                        }
                        YamlBase::Array(_) if root == Root::Array && i == 0 => {
                            doc += &v.dump(level, Root::Map);
                        }
                        YamlBase::Array(_) => {
                            doc += &v.dump(level + 1, Root::Map);
                        }
                        _ => {
                            doc += " ";
                            doc += &v.dump(level + 1, Root::Map);
                        }
                    }
                    doc += NL;
                }
                doc.truncate(doc.len() - NL.len());
                doc
            }
            YamlBase::Anchor(anchor) => format!("*{}", anchor),
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
/// let doc = dump(vec![
///     node!({
///         node!("a") => node!("b"),
///         node!("c") => node!("d"),
///     }),
/// ]);
/// assert_eq!(doc, format!("a: b{0}c: d{0}", NL));
/// ```
///
/// When calling [`parse`] function then [`dump`] the string, the string can be reformatted.
pub fn dump<I>(nodes: I) -> String
where
    I: IntoIterator,
    I::Item: Dumper,
{
    nodes
        .into_iter()
        .enumerate()
        .map(|(i, node)| {
            if i == 0 {
                format!("{}{}", node.dump(0, Root::Scalar).trim_start(), NL)
            } else {
                format!("---{}{}{}", NL, node.dump(0, Root::Scalar).trim_start(), NL)
            }
        })
        .collect()
}
