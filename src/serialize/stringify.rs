use alloc::string::String;
use core::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

/// A type that can deserialize from any data to string type.
///
/// It just like [`YamlBase`](crate::YamlBase) but no null value, anchor type and containers.
///
/// Calling [`ToString::to_string`] can convert the data into string.
///
/// ```
/// use serde::Deserialize;
/// use yaml_peg::{node, serialize::Stringify};
///
/// #[derive(Deserialize)]
/// struct Content {
///     width: Stringify,
/// }
///
/// let n_value = node!({"width" => 20});
/// let n_percent = node!({"width" => "20%"});
/// let value = Content::deserialize(n_value).unwrap();
/// let percent = Content::deserialize(n_percent).unwrap();
/// assert_eq!("20", value.width.to_string());
/// assert_eq!("20%", percent.width.to_string());
/// ```
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Stringify {
    /// Boolean value.
    Bool(bool),
    /// Integer value.
    Int(i32),
    /// Float value.
    Float(f32),
    /// String value.
    Str(String),
}

impl Display for Stringify {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        match self {
            Self::Bool(true) => write!(f, "true"),
            Self::Bool(false) => write!(f, "false"),
            Self::Int(n) => write!(f, "{}", n),
            Self::Float(n) => write!(f, "{}", n),
            Self::Str(s) => write!(f, "{}", s),
        }
    }
}

impl Default for Stringify {
    fn default() -> Self {
        Self::Str(String::new())
    }
}
