pub use crate::indicator::*;
pub use crate::node::*;
pub use crate::parser::*;
pub use crate::yaml::*;

/// Build [`std::io::ErrorKind::InvalidData`] from strings.
///
/// ```
/// Err(err!("error message"));
/// ```
#[macro_export]
macro_rules! err {
    ($e:expr) => {{
        use std::io::{Error, ErrorKind};
        Error::new(ErrorKind::InvalidData, $e)
    }};
}

/// Create [`Node`] items literally.
///
/// Literals will be transformed to [`Yaml`] automatically.
///
/// ```
/// let k = "a";
/// assert_eq!(node!(k.into()), node!("a"));
/// ```
#[macro_export]
macro_rules! node {
    ($yaml:literal $(, $pos:expr $(, $anchor:literal $(, $ty:literal)?)?)?) => {
        $crate::Node::new($yaml.into())$(.pos($pos)$(.anchor($anchor.into())$(.ty($ty.into()))?)?)?
    };
    ($yaml:expr $(, $pos:expr $(, $anchor:expr $(, $ty:expr)?)?)?) => {
        $crate::Node::new($yaml)$(.pos($pos)$(.anchor($anchor)$(.ty($ty))?)?)?
    };
}

/// Create [`Yaml::Array`] items literally.
///
/// ```
/// yaml_array![node!("a"), node!("b"), node!("c")];
/// ```
#[macro_export]
macro_rules! yaml_array {
    ($v1:expr $(, $v2:expr)* $(,)?) => {
        Yaml::Array(vec![$v1 $(, $v2)*])
    };
}

/// Create [`Yaml::Map`] items literally.
///
/// ```
/// yaml_map!{
///     node!("a") => node!("b"),
///     node!("c") => node!("d"),
/// };
/// ```
#[macro_export]
macro_rules! yaml_map {
    ($k1:expr => $v1:expr $(, $k2:expr => $v2:expr)* $(,)?) => {
        Yaml::Map(vec![($k1, $v1) $(, ($k2, $v2))*].into_iter().collect())
    };
}

mod indicator;
mod node;
mod parser;
#[cfg(test)]
mod tests;
mod yaml;
