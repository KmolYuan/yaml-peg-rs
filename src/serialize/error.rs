use alloc::string::{String, ToString};
use core::fmt::{Debug, Display, Formatter};
use serde::de::{Error, StdError};

#[derive(Debug)]
pub struct SerdeError(String);

impl Display for SerdeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl StdError for SerdeError {}

impl Error for SerdeError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self(msg.to_string())
    }
}
