use alloc::string::{String, ToString};
use core::fmt::{Debug, Display, Formatter};
use serde::{de::Error as DeError, ser::Error as SerError};

/// The error type for the serialization.
#[derive(Debug)]
pub struct SerdeError(pub String, pub u64);

impl SerdeError {
    pub(crate) fn pos(mut self, pos: u64) -> Self {
        self.1 = pos;
        self
    }
}

impl From<String> for SerdeError {
    fn from(s: String) -> Self {
        Self(s, 0)
    }
}

impl Display for SerdeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[cfg(feature = "serde-std")]
impl std::error::Error for SerdeError {}

impl SerError for SerdeError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self(msg.to_string(), 0)
    }
}

impl DeError for SerdeError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self(msg.to_string(), 0)
    }
}
