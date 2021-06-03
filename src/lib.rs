//! A YAML 1.2 parser using LALR algorithm with PEG atoms.
//!
//! The major purpose of this crate is to let the user build their own YAML reader / builder / validator.
//!
//! Function [`parse`] is used to parse YAML string into [`Node`] data structure.
//! To get back as string, please use [`dump`] function.
//!
//! There are also have some macros for building [`Node`] structure from Rust data.
//!
//! If you went to rise your own error message, [`indicated_msg`] might be a good choice.
pub use crate::dumper::dump;
pub use crate::indicator::*;
pub use crate::node::*;
pub use crate::parser::parse;
pub use crate::yaml::*;

/// Build [`std::io::Error`] with [`std::io::ErrorKind::InvalidData`] from strings.
///
/// ```
/// use yaml_peg::err;
/// Err::<(), std::io::Error>(err!("error message"));
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
/// Literals will be transformed to [`Yaml`] automatically but variables need to convert manually.
///
/// ```
/// use yaml_peg::node;
/// let k = "a";
/// assert_eq!(node!(k.into()), node!("a"));
/// ```
///
/// The members are ordered as `node!(yaml, pos, anchor, ty)`.
///
/// Also, arrays and maps can be create from this macro directly through brackets (`[]`, `{}`).
///
/// ```
/// use yaml_peg::{node, yaml_array, yaml_map};
/// assert_eq!(node!([node!(1), node!(2)]), node!(yaml_array![node!(1), node!(2)]));
/// assert_eq!(node!({node!(1) => node!(2)}), node!(yaml_map![node!(1) => node!(2)]));
/// ```
#[macro_export]
macro_rules! node {
    ([$($token:tt)*]) => {
        $crate::node!($crate::yaml_array![$($token)*])
    };
    ({$($token:tt)*}) => {
        $crate::node!($crate::yaml_map![$($token)*])
    };
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
/// use yaml_peg::{node, yaml_array};
/// yaml_array![node!("a"), node!("b"), node!("c")];
/// ```
#[macro_export]
macro_rules! yaml_array {
    ($($token:tt)*) => {
        $crate::Yaml::Array(vec![$($token)*])
    };
}

/// Create [`Yaml::Map`] items literally.
///
/// ```
/// use yaml_peg::{node, yaml_map};
/// yaml_map!{
///     node!("a") => node!("b"),
///     node!("c") => node!("d"),
/// };
/// ```
#[macro_export]
macro_rules! yaml_map {
    () => {
        $crate::Yaml::Map($crate::Map::new())
    };
    ($k1:expr => $v1:expr $(, $k2:expr => $v2:expr)* $(,)?) => {{
        use std::iter::FromIterator;
        $crate::Yaml::from_iter(vec![($k1, $v1) $(, ($k2, $v2))*])
    }};
}

pub mod dumper;
mod indicator;
mod node;
pub mod parser;
#[cfg(test)]
mod tests;
mod yaml;
