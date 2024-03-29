use alloc::string::{String, ToString};
use core::fmt::{Debug, Display, Formatter, Result};

/// The error type for the serialization.
///
/// If the error is used at deserializing to a custom data,
/// the field [`SerdeError.pos`] will provide the position of the original YAML
/// document.
#[derive(Debug)]
pub struct SerdeError {
    /// Message.
    pub msg: String,
    /// The original position of the node if provided.
    ///
    /// If not provided, this field becomes zero.
    pub pos: u64,
}

impl SerdeError {
    pub(crate) fn pos(mut self, pos: u64) -> Self {
        self.pos = pos;
        self
    }
}

impl From<String> for SerdeError {
    fn from(msg: String) -> Self {
        Self { msg, pos: 0 }
    }
}

impl Display for SerdeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Debug::fmt(self, f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SerdeError {}

impl serde::ser::Error for SerdeError {
    fn custom<T: Display>(msg: T) -> Self {
        Self::from(msg.to_string())
    }
}

impl serde::de::Error for SerdeError {
    fn custom<T: Display>(msg: T) -> Self {
        Self::from(msg.to_string())
    }
}
