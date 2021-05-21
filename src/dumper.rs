//! Dumper components.
use crate::*;

/// The interface for dumping data structure.
pub trait Dumper {
    /// Generate indentation.
    fn ind(level: usize) -> String {
        "  ".repeat(level)
    }

    /// Recursive dump function.
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
        let ind = Self::ind(level);
        doc += &match &self.yaml {
            Yaml::Null => "null".into(),
            Yaml::Bool(b) => b.to_string(),
            Yaml::Int(n) | Yaml::Float(n) => n.clone(),
            Yaml::Str(s) => {
                if s.contains("\n") {
                    let s = s.trim().replace("\n", &(String::from("\n") + &ind));
                    String::from("|\n") + &ind + &s
                } else {
                    s.clone()
                }
            }
            Yaml::Array(a) => {
                let mut doc = String::from(if level == 0 { "" } else { "\n" });
                for (i, node) in a.iter().enumerate() {
                    if i != 0 || level != 0 {
                        doc += &ind;
                    }
                    doc += &format!("- {}\n", node.dump(level + 1, false));
                }
                doc.pop();
                doc
            }
            Yaml::Map(m) => {
                let mut doc = String::from(if wrap { "\n" } else { "" });
                for (i, (k, v)) in m.iter().enumerate() {
                    if i != 0 || wrap {
                        doc += &ind;
                    }
                    let s = k.dump(level + 1, false);
                    if let Yaml::Map(_) | Yaml::Array(_) = k.yaml {
                        doc += "?\n";
                        doc += &(Self::ind(level + 1) + "\n" + &s + "\n" + &ind);
                    } else {
                        doc += &s;
                    }
                    doc += ": ";
                    doc += &(v.dump(level + 1, true) + "\n");
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
/// use yaml_peg::{dump, node};
/// let doc = dump(vec![node!({
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
