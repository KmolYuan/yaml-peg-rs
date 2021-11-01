//! Dumper components.
use crate::{repr::Repr, *};
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

/// Newline symbol in common platforms.
///
/// There is only two symbols in the option,
/// "\r\n" in Windows platform, otherwise uses "\n".
///
/// Please be aware that your storage can be used the symbol of Windows.
pub const NL: &str = if cfg!(windows) { "\r\n" } else { "\n" };

#[derive(Eq, PartialEq)]
enum Root {
    Map,
    Array,
    Scalar,
}

impl<R: Repr> NodeBase<R> {
    fn dump(&self, level: usize, root: Root) -> String {
        let mut doc = String::new();
        let anchor = self.anchor();
        if !anchor.is_empty() {
            doc += &format!("&{} ", anchor);
        }
        let tag = self.tag();
        if !tag.is_empty() && !tag.starts_with("tag:yaml.org,2002:") {
            doc += &if tag.starts_with(parser::DEFAULT_PREFIX) {
                format!("!!{} ", tag)
            } else if parser::Parser::<R>::new(tag.as_bytes())
                .identifier()
                .is_ok()
            {
                format!("!{} ", tag)
            } else {
                format!("!<{}> ", tag)
            };
        }
        let ind = "  ".repeat(level);
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
            YamlBase::Seq(a) => {
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
                    if let YamlBase::Map(_) | YamlBase::Seq(_) = k.yaml() {
                        doc += &format!("?{}{}{}{}{}", "  ".repeat(level + 1), NL, s, NL, ind);
                    } else {
                        doc += &s;
                    }
                    doc += ":";
                    match v.yaml() {
                        YamlBase::Map(_) => {
                            doc += &v.dump(level + 1, Root::Map);
                        }
                        YamlBase::Seq(_) if root == Root::Array && i == 0 => {
                            doc += &v.dump(level, Root::Map);
                        }
                        YamlBase::Seq(_) => {
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
/// let doc = dump(&[
///     node!({
///         "a" => "b",
///         "c" => "d",
///     }),
/// ]);
/// let ans = "\
/// a: b
/// c: d
/// ";
/// assert_eq!(doc, ans.replace('\n', NL));
/// ```
///
/// When calling [`parse`] function then [`dump`] the string, the string can be reformatted.
pub fn dump<R: Repr>(nodes: &[NodeBase<R>]) -> String {
    nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            let doc = node.dump(0, Root::Scalar) + NL;
            if i == 0 {
                doc
            } else {
                format!("---{}{}", NL, doc.trim_start())
            }
        })
        .collect()
}
