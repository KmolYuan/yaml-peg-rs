use alloc::string::{String, ToString};
use serde::Deserialize;

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
#[derive(Deserialize, Debug, PartialEq)]
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

impl ToString for Stringify {
    fn to_string(&self) -> String {
        match self {
            Self::Bool(true) => "true".to_string(),
            Self::Bool(false) => "false".to_string(),
            Self::Int(n) => n.to_string(),
            Self::Float(n) => n.to_string(),
            Self::Str(s) => s.clone(),
        }
    }
}

impl Default for Stringify {
    fn default() -> Self {
        Self::Str(String::new())
    }
}
