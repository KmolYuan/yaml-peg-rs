use crate::{repr::*, *};
use alloc::string::{String, ToString};
use core::iter::FromIterator;
use ritelinked::LinkedHashMap;

/// An anchor visitor with [`alloc::rc::Rc`] holder.
pub type AnchorRc = Anchor<RcRepr>;
/// An anchor visitor with [`alloc::sync::Arc`] holder.
pub type AnchorArc = Anchor<ArcRepr>;

/// The error of using an invalid anchor.
#[derive(Debug)]
pub struct InvalidAnchor {
    /// Invalid anchor name.
    pub anchor: String,
}

/// Anchor visitor is made by a hash map that you can get the node reference inside.
///
/// Since [`Node`] type is holding a reference counter,
/// the data are just a viewer to the original memory.
/// Please use [`AnchorRc`] and [`AnchorArc`] to specify different presentation.
///
/// There is a macro [`anchors!`] can build the index tree literally.
#[derive(Clone, Default, PartialEq, Debug)]
pub struct Anchor<R: Repr = RcRepr>(LinkedHashMap<String, Node<R>>);

impl<R: Repr> Anchor<R> {
    /// Create an empty anchor list.
    pub fn new() -> Self {
        Self(LinkedHashMap::new())
    }

    /// Create with memory capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(LinkedHashMap::with_capacity(capacity))
    }

    /// Self-resolve the insertion of the visitor.
    /// Return the error [`InvalidAnchor`] if the anchor is not found.
    ///
    /// Although the original YAML specification is not support forward reference,
    /// but this algorithm achieved this with post-resolution mechanism.
    ///
    /// Do nothing if `deep` equals to zero.
    ///
    /// ```
    /// use yaml_peg::{node, parse, repr::RcRepr};
    ///
    /// let doc = "
    /// - &seq
    ///   - a: &sub b
    ///   - a: *sub
    /// - *seq
    /// ";
    ///
    /// let (mut root, mut anchor) = parse::<RcRepr>(doc).unwrap();
    /// anchor.resolve(1).unwrap();
    /// let node = root.remove(0).replace_anchor(&anchor).unwrap();
    /// std::mem::drop(anchor);
    /// assert_eq!(
    ///     node,
    ///     node!([
    ///         node!([node!({"a" => "b"}), node!({"a" => "b"})]),
    ///         node!([node!({"a" => "b"}), node!({"a" => "b"})]),
    ///     ])
    /// );
    /// ```
    pub fn resolve(&mut self, deep: usize) -> Result<(), InvalidAnchor> {
        for _ in 0..deep {
            for _ in 0..self.len() {
                let (k, node) = self.pop_front().unwrap();
                let node = node.replace_anchor(self)?;
                self.insert(k, node);
            }
        }
        Ok(())
    }
}

impl<R: Repr> From<Node<R>> for Anchor<R> {
    /// Create a visitor by visiting all nodes of the data.
    ///
    /// This method will take a lot of time to read the nodes.
    /// If you have a unparsed data, parser will give you a visitor too.
    fn from(node: Node<R>) -> Self {
        let mut visitor = Self::new();
        anchor_visit_inner(&node, &mut visitor);
        visitor
    }
}

fn anchor_visit_inner<R: Repr>(n: &Node<R>, visitor: &mut Anchor<R>) {
    if !n.anchor().is_empty() {
        visitor.insert(n.anchor().to_string(), n.clone());
    }
    match n.yaml() {
        Yaml::Seq(seq) => seq.iter().for_each(|n| anchor_visit_inner(n, visitor)),
        Yaml::Map(map) => map.iter().for_each(|(k, v)| {
            anchor_visit_inner(k, visitor);
            anchor_visit_inner(v, visitor);
        }),
        _ => (),
    }
}

impl<R: Repr> core::ops::Deref for Anchor<R> {
    type Target = LinkedHashMap<String, Node<R>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<R: Repr> core::ops::DerefMut for Anchor<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<R: Repr> FromIterator<(String, Node<R>)> for Anchor<R> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (String, Node<R>)>,
    {
        Self(iter.into_iter().collect())
    }
}
