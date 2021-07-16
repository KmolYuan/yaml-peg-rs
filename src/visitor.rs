use super::*;

/// Create a visitor by visiting all nodes of the data.
///
/// This method will take a lot of time to read the nodes.
/// If you have a unparsed data, parser will give you a visitor too.
pub fn anchor_visit(n: &Node) -> AnchorVisitor {
    let mut visitor = AnchorVisitor::new();
    inner_anchor_visit(n, &mut visitor);
    visitor
}

fn inner_anchor_visit(n: &Node, visitor: &mut AnchorVisitor) {
    if !n.anchor().is_empty() {
        visitor.insert(n.anchor().to_owned(), n.clone());
    }
    match n.yaml() {
        Yaml::Array(a) => {
            for n in a {
                inner_anchor_visit(n, visitor);
            }
        }
        Yaml::Map(m) => {
            for (k, v) in m {
                inner_anchor_visit(k, visitor);
                inner_anchor_visit(v, visitor);
            }
        }
        _ => {}
    }
}
