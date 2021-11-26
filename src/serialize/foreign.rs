use super::SerdeError;
use crate::{repr::Repr, AnchorBase};
use alloc::{
    borrow::Cow,
    string::{String, ToString},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// The serializable type provide anchor insertion.
///
/// The inner type `D` should be implement one of the [`Serialize`] or [`Deserialize`] traits.
///
/// The anchors are represented as a **single** key-value pair `{ "anchor": anchor }` in the serialization.
/// In actual use, this can be achieved with a `enum` type field.
/// This implementation is done by [`Foreign`] type.
///
/// The parent field will support anchor insertion when deserialized from [`NodeBase`](crate::NodeBase).
/// In the same way, anchor insertion can also be achieved when serializing into a node.
///
/// ```
/// use serde::{Serialize, Deserialize};
/// use yaml_peg::{node, serialize::{to_node, Foreign}};
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq)]
/// struct Content {
///     doc: Foreign<String>,
/// }
///
/// let doc = Content {
///     doc: Foreign::data("my doc".to_string()),
/// };
/// let anchor = Content {
///     doc: Foreign::anchor("my-anchor"),
/// };
/// let n_doc = node!({"doc" => "my doc"});
/// let n_anchor = node!({"doc" => node!(*"my-anchor")});
/// // Node -> Content (Data)
/// assert_eq!(doc, Content::deserialize(n_doc.clone()).unwrap());
/// // Content -> Node (Data)
/// assert_eq!(n_doc, to_node(doc).unwrap());
/// // Node -> Content (Anchor)
/// assert_eq!(anchor, Content::deserialize(n_anchor.clone()).unwrap());
/// // Content -> Node (Anchor)
/// assert_eq!(n_anchor, to_node(anchor).unwrap());
/// ```
///
/// The first-step inference is fine.
/// Since there are recursive issue in the YAML data,
/// please see the method [`Foreign::visit`].
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Foreign<D> {
    #[doc(hidden)]
    Data(D),
    #[doc(hidden)]
    Anchor { anchor: String },
}

impl<D: Default> Default for Foreign<D> {
    fn default() -> Self {
        Self::Data(Default::default())
    }
}

impl<D> Foreign<D> {
    /// Create a data.
    pub fn data(data: D) -> Self {
        Self::Data(data)
    }

    /// Create an anchor insertion.
    pub fn anchor(s: impl ToString) -> Self {
        Self::Anchor {
            anchor: s.to_string(),
        }
    }
}

impl<D: DeserializeOwned + Clone> Foreign<D> {
    /// Get the deserializable value from exist anchor.
    ///
    /// Where returned type is [`Cow`], a reference container that can also save the actual data.
    /// If the value is saved in the anchor visitor, it will be deserialized and saved in [`Cow::Owned`],
    /// otherwise it is already deserialized, which will be borrowed as [`Cow::Borrowed`].
    ///
    /// The borrowed reference will be copied by [`Cow::into_owned`], but the owned data will just move itself.
    ///
    /// ```
    /// use serde::Deserialize;
    /// use yaml_peg::{anchors, node, Node, serialize::Foreign};
    ///
    /// #[derive(Deserialize, Debug, PartialEq)]
    /// struct Content {
    ///     doc: Foreign<String>,
    /// }
    ///
    /// let visitor = anchors!["my-anchor" => "doc in anchor"];
    /// let n_doc = node!({"doc" => "my doc"});
    /// let n_anchor = node!({"doc" => node!(*"my-anchor")});
    /// assert_eq!("my doc", n_doc.with(&visitor, "doc", "error!", Node::as_str).unwrap());
    /// assert_eq!("doc in anchor", n_anchor.with(&visitor, "doc", "error!", Node::as_str).unwrap());
    /// let doc = Content::deserialize(n_doc).unwrap();
    /// let anchor = Content::deserialize(n_anchor).unwrap();
    /// assert_eq!("my doc", doc.doc.visit(&visitor).unwrap().into_owned());
    /// assert_eq!("doc in anchor", anchor.doc.visit(&visitor).unwrap().into_owned());
    /// ```
    pub fn visit<R: Repr>(&self, anchor: &AnchorBase<R>) -> Result<Cow<D>, SerdeError> {
        match self {
            Self::Data(data) => Ok(Cow::Borrowed(data)),
            Self::Anchor { anchor: tag } => match anchor.get(tag) {
                Some(n) => Ok(Cow::Owned(D::deserialize(n.clone())?)),
                None => Err(SerdeError::from("missing anchor".to_string())),
            },
        }
    }
}
