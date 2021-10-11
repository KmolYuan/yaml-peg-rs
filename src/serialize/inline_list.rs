use alloc::{
    slice::{from_raw_parts, Iter},
    vec,
    vec::Vec,
};
use serde::Deserialize;

/// A data type that can support listed items,
/// or inline it if there is single item.
///
/// ```
/// use serde::Deserialize;
/// use yaml_peg::{node, serialize::InlineList};
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
#[derive(Deserialize)]
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
            Self::Inline(e) => unsafe { from_raw_parts(e, 1) }.iter(),
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
