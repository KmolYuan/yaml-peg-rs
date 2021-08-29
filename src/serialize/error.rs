use alloc::string::{String, ToString};
use core::fmt::{Debug, Display, Formatter};
use serde::{de::Error as DeError, ser::Error as SerError};

/// The error type for the serialization.
#[derive(Debug)]
pub struct SerdeError(String);

impl From<String> for SerdeError {
    fn from(s: String) -> Self {
        Self(s)
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
        Self(msg.to_string())
    }
}

impl DeError for SerdeError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self(msg.to_string())
    }
}
