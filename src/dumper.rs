//! Dumper components.
use crate::*;

/// The interface for dumping data structure.
pub trait Dumper {
    fn dump(&self, level: usize, wrap: bool) -> String;
}

impl Dumper for Node {
    fn dump(&self, level: usize, wrap: bool) -> String {
        let mut doc = String::new();
        if !self.anchor.is_empty() {
            doc += &format!("&{} ", self.anchor);
        }
        if !self.ty.is_empty() {
            doc += &format!("!!{} ", self.ty);
        }
        doc += &match &self.yaml {
            Yaml::Null => "null".into(),
            Yaml::Bool(b) => b.to_string(),
            Yaml::Int(n) | Yaml::Float(n) => n.clone(),
            Yaml::Str(s) => {
                if s.contains("\n") {
                    let s = s
                        .trim()
                        .replace("\n", &(String::from("\n") + &"  ".repeat(level)));
                    String::from("|\n") + &"  ".repeat(level) + &s
                } else {
                    s.clone()
                }
            }
            Yaml::Array(a) => {
                let mut doc = String::from(if level == 0 { "" } else { "\n" });
                for (i, node) in a.iter().enumerate() {
                    let mut s = format!("- {}\n", node.dump(level + 1, false));
                    if i != 0 || level != 0 {
                        s = "  ".repeat(level) + &s;
                    }
                    doc += &s;
                }
                doc.pop();
                doc
            }
            Yaml::Map(m) => {
                let mut doc = String::from(if wrap { "\n" } else { "" });
                for (i, (k, v)) in m.iter().enumerate() {
                    let mut s = k.dump(level + 1, false) + ": " + &v.dump(level + 1, true) + "\n";
                    if i != 0 || wrap {
                        s = "  ".repeat(level) + &s;
                    }
                    doc += &s;
                }
                doc.pop();
                doc
            }
            Yaml::Anchor(anchor) => format!("*{}", anchor),
        };
        doc
    }
}

/// Dump the YAML data in to block format.
///
/// ```
/// use yaml_peg::{dump, yaml_map, node};
/// let doc = dump(vec![node!(yaml_map!{
///     node!("a") => node!("b"),
///     node!("c") => node!("d"),
/// })]);
/// assert_eq!(doc, "a: b\nc: d\n");
/// ```
pub fn dump<I>(nodes: I) -> String
where
    I: IntoIterator<Item = Node>,
{
    nodes
        .into_iter()
        .enumerate()
        .map(|(i, node)| {
            String::from(if i == 0 { "" } else { "---\n" })
                + &node
                    .dump(0, false)
                    .split('\n')
                    .map(|s| s.trim_end())
                    .collect::<Vec<_>>()
                    .join("\n")
                + "\n"
        })
        .collect()
}
