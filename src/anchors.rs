use crate::{repr::*, *};
use alloc::string::{String, ToString};
use ritelinked::LinkedHashMap;

/// Anchor visitor is made by a hash map that you can get the node reference inside.
///
/// Since [`NodeBase`] type is holding a reference counter,
/// the data are just a viewer to the original memory.
///
/// There is a macro [`anchors!`] can build the index tree literally.
pub type AnchorBase<R> = LinkedHashMap<String, NodeBase<R>>;
/// An anchor visitor with [`alloc::rc::Rc`] holder.
pub type Anchors = AnchorBase<RcRepr>;
/// An anchor visitor with [`alloc::sync::Arc`] holder.
pub type ArcAnchors = AnchorBase<ArcRepr>;

/// Create a visitor by visiting all nodes of the data.
///
/// This method will take a lot of time to read the nodes.
/// If you have a unparsed data, parser will give you a visitor too.
pub fn anchor_visit<R: Repr>(n: &NodeBase<R>) -> AnchorBase<R> {
    let mut visitor = AnchorBase::new();
    anchor_visit_inner(n, &mut visitor);
    visitor
}

fn anchor_visit_inner<R: Repr>(n: &NodeBase<R>, visitor: &mut AnchorBase<R>) {
    if !n.anchor().is_empty() {
        visitor.insert(n.anchor().to_string(), n.clone());
    }
    match n.yaml() {
        YamlBase::Seq(seq) => seq.iter().for_each(|n| anchor_visit_inner(n, visitor)),
        YamlBase::Map(map) => map.iter().for_each(|(k, v)| {
            anchor_visit_inner(k, visitor);
            anchor_visit_inner(v, visitor);
        }),
        _ => (),
    }
}

/// Self-resolve the insertion of the visitor.
/// Return `None` if the anchor is not found.
///
/// ```
/// use yaml_peg::{anchor_resolve, node, parse, repr::RcRepr};
///
/// let doc = "
/// - &seq
///   - a: &sub b
///   - a: *sub
/// - *seq
/// ";
///
/// let (mut ans, anchor) = parse::<RcRepr>(doc).unwrap();
/// let anchor = anchor_resolve(&anchor, 1).unwrap();
/// let node = ans.remove(0).replace_anchor(&anchor).unwrap();
/// assert_eq!(
///     node,
///     node!([
///         node!([node!({"a" => "b"}), node!({"a" => "b"})]),
///         node!([node!({"a" => "b"}), node!({"a" => "b"})]),
///     ])
/// );
/// ```
pub fn anchor_resolve<R: Repr>(visitor: &AnchorBase<R>, deep: usize) -> Option<AnchorBase<R>> {
    assert_ne!(deep, 0);
    let mut tmp = visitor.clone();
    let mut visitor = visitor.clone();
    for _ in 0..deep * 2 {
        for node in visitor.values_mut() {
            *node = match node.replace_anchor(&tmp) {
                Some(node) => node,
                None => return None,
            };
        }
        std::mem::swap(&mut visitor, &mut tmp);
    }
    Some(visitor)
}
