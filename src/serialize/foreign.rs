use serde::{Deserialize, Serialize};

/// The serializable type provide anchor insertion.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Foreign<T> {
    /// A Normal data type.
    #[serde(bound(deserialize = "T: for<'a> Deserialize<'a>", serialize = "T: Serialize"))]
    Data(T),
    /// A data structure can be serialized into anchor.
    Anchor {
        /// Inner anchor field.
        anchor: String,
    },
}

impl<T: Default> Default for Foreign<T> {
    fn default() -> Self {
        Self::Data(Default::default())
    }
}

impl<T> Foreign<T> {
    /// Create a anchor insertion.
    pub fn anchor(s: impl ToString) -> Self {
        Self::Anchor {
            anchor: s.to_string(),
        }
    }
}
