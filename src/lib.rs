//! A YAML 1.2 parser using greedy parsing algorithm with PEG atoms.
//!
//! The major purpose of this crate is to let the user build their own YAML reader / builder / validator.
//!
//! This parser is not ensure about YAML spec but almost functions are well-implemented.
//! (test case is [here](https://github.com/KmolYuan/yaml-peg-rs/blob/main/src/tests/test.yaml))
//!
//! The buffer reader has also not implemented, but the chunks can be read by sub-parsers.
//!
//! Function [`parse`] is used to parse YAML string into [`Node`] data structure.
//! To get back as string, please use [`dump`] function.
//!
//! There are also have some macros for building [`Node`] structure from Rust data.
//! Especially [`node!`] macro, almost data can be built by the macro literally.
//!
//! If you went to rise your own error message, [`indicated_msg`] might be a good choice.
//!
//! Please be aware that the anchor system must be done by your self to prevent recursive problem.
//! This crate is only store the anchor information in [`Yaml::Anchor`] and [`Node::anchor`].
//! A reference counter system maybe the best choice.
pub use crate::dumper::dump;
pub use crate::indicator::*;
pub use crate::node::*;
pub use crate::parser::parse;
pub use crate::yaml::*;

/// Create [`Node`] items literally.
///
/// Literals and expressions will be transformed to [`Yaml`] automatically by calling [`Into::into`].
///
/// ```
/// use yaml_peg::node;
/// let k = "a";
/// assert_eq!(node!(k), node!("a"));
/// ```
///
/// Arrays and maps can be created from this macro directly through brackets (`[]`, `{}`).
///
/// ```
/// use yaml_peg::{node, yaml_array, yaml_map};
/// assert_eq!(node!([node!(1), node!(2)]), node!(yaml_array![node!(1), node!(2)]));
/// assert_eq!(node!({node!(1) => node!(2)}), node!(yaml_map![node!(1) => node!(2)]));
/// ```
///
/// The [`Yaml::Null`] and the [`Yaml::Null`] are also supported by the syntax:
///
/// ```
/// use yaml_peg::{node, Yaml};
/// assert_eq!(node!(Yaml::Null), node!(null));
/// assert_eq!(node!(Yaml::Anchor("x".into())), node!(*"x"));
/// ```
#[macro_export]
macro_rules! node {
    ([$($token:tt)*]) => {
        $crate::node!($crate::yaml_array![$($token)*])
    };
    ({$($token:tt)*}) => {
        $crate::node!($crate::yaml_map![$($token)*])
    };
    (null) => {
        $crate::node!($crate::Yaml::Null)
    };
    (*$anchor:expr) => {
        $crate::node!($crate::Yaml::Anchor($anchor.into()))
    };
    ($yaml:expr) => {
        $crate::Node::new($yaml.into(), 0, "", "")
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
