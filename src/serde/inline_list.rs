use alloc::{
    slice::{from_ref, Iter},
    vec,
    vec::Vec,
};
use serde::{Deserialize, Serialize};

/// A data type that can support listed items,
/// or inline it if there is single item.
///
/// ```
/// use serde::Deserialize;
/// use yaml_peg::{node, serde::InlineList};
///
/// #[derive(Deserialize)]
/// struct Content {
///     img: InlineList<Img>,
/// }
///
/// #[derive(Deserialize)]
/// struct Img {
///     src: String,
/// }
///
/// let n_listed = node!({"img" => node!([node!({"src" => "img/1.png"}), node!({"src" => "img/2.png"})])});
/// let n_inline = node!({"img" => node!({"src" => "img/1.png"})});
/// let listed = Content::deserialize(n_listed).unwrap();
/// let inline = Content::deserialize(n_inline).unwrap();
/// for (i, img) in listed.img.iter().enumerate() {
///     assert_eq!(format!("img/{}.png", i + 1), img.src);
/// }
/// for (i, img) in inline.img.into_iter().enumerate() {
///     assert_eq!(format!("img/{}.png", i + 1), img.src);
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum InlineList<T> {
    /// Listed representation.
    List(Vec<T>),
    /// Inline representation.
    Inline(T),
}

impl<T> InlineList<T> {
    /// Return the iterator over the items.
    pub fn iter(&self) -> Iter<T> {
        match self {
            Self::List(v) => v.iter(),
            Self::Inline(e) => from_ref(e).iter(),
        }
    }

    /// Get the length of the list.
    pub fn len(&self) -> usize {
        match self {
            Self::List(v) => v.len(),
            Self::Inline(_) => 1,
        }
    }

    /// Return true if the list is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::List(v) => v.is_empty(),
            Self::Inline(_) => false,
        }
    }

    /// Return true if the list has only one item.
    pub fn is_single(&self) -> bool {
        match self {
            Self::List(v) => v.len() == 1,
            Self::Inline(_) => true,
        }
    }
}

impl<T> IntoIterator for InlineList<T> {
    type Item = T;
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::List(v) => v.into_iter(),
            Self::Inline(e) => vec![e].into_iter(),
        }
    }
}

impl<T> Default for InlineList<T> {
    fn default() -> Self {
        Self::List(Vec::new())
    }
}
