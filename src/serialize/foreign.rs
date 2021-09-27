use super::SerdeError;
use crate::{repr::Repr, AnchorBase};
use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};

/// The serializable type provide anchor insertion.
///
/// The inner type `D` should be implement one of the [`Serialize`] or [`Deserialize`] traits.
///
/// Please see the [module page about anchors](super#anchors) for more information.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Foreign<D> {
    #[doc(hidden)]
    #[serde(bound(deserialize = "D: for<'a> Deserialize<'a>", serialize = "D: Serialize"))]
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

impl<D: for<'a> Deserialize<'a> + Clone> Foreign<D> {
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
