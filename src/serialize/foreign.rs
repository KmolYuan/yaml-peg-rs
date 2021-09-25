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
