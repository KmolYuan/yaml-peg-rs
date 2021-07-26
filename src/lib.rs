//! A YAML 1.2 parser using greedy parsing algorithm with PEG atoms.
//!
//! The major purpose of this crate is to let the user build their own YAML reader / builder / validator.
//!
//! This parser is not ensure about YAML spec but almost functions are well-implemented.
//! (test case is [here](https://github.com/KmolYuan/yaml-peg-rs/blob/main/src/tests/test.yaml))
//!
//! The buffer reader has also not implemented, but the chunks can be read by sub-parsers.
//!
//! Function [`parse`] is used to parse YAML string into [`Node`] data structure,
//! which has a data holder [`Yaml`].
//! There also has multiple thread version corresponding to [`parse_arc`], [`ArcNode`], [`ArcYaml`].
//! To get back as string, please use [`dump`] function.
//!
//! There are also have some macros for building [`NodeBase`] structure from Rust data.
//! Especially [`node!`] / [`node_arc!`] macro, almost data can be built by the macro literally.
//!
//! If you went to rise your own error message, [`indicated_msg`] might be a good choice.
//!
//! The anchor system [`AnchorBase`] is implemented by using [`alloc::rc::Rc`] and [`alloc::sync::Arc`] as inner handler.
//! Additionally, [`anchors!`] macro can used to create anchor visitor by yourself.
#![no_std]
extern crate alloc;
extern crate core;
pub use crate::anchors::*;
pub use crate::dumper::dump;
pub use crate::indicator::*;
pub use crate::node::*;
pub use crate::parser::{parse, parse_arc};
pub use crate::yaml::*;

/// Create [`Node`] items literally.
///
/// Literals and expressions will be transformed to [`Yaml`] automatically by calling [`Into::into`].
///
/// ```
/// use yaml_peg::node;
///
/// let k = "a";
/// assert_eq!(node!(k), node!("a"));
/// ```
///
/// Arrays and maps can be created from this macro directly through brackets (`[]`, `{}`).
///
/// ```
/// use yaml_peg::{node, yaml_array, yaml_map};
///
/// assert_eq!(node!([node!(1), node!(2)]), node!(yaml_array![node!(1), node!(2)]));
/// assert_eq!(node!({node!(1) => node!(2)}), node!(yaml_map![node!(1) => node!(2)]));
/// ```
///
/// The [`YamlBase::Null`] and the [`YamlBase::Null`] are also supported by the syntax:
///
/// ```
/// use yaml_peg::{node, YamlBase};
///
/// assert_eq!(node!(YamlBase::Null), node!(null));
/// assert_eq!(node!(YamlBase::Anchor("x".into())), node!(*"x"));
/// ```
///
/// For [`ArcNode`], please use [`node_arc!`], which has same API.
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

/// Create [`ArcNode`] items literally.
///
/// The API is same as [`node!`] macro.
#[macro_export]
macro_rules! node_arc {
    ([$($token:tt)*]) => {
        $crate::node_arc!($crate::yaml_array![$($token)*])
    };
    ({$($token:tt)*}) => {
        $crate::node_arc!($crate::yaml_map![$($token)*])
    };
    (null) => {
        $crate::node_arc!($crate::Yaml::Null)
    };
    (*$anchor:expr) => {
        $crate::node_arc!($crate::Yaml::Anchor($anchor.into()))
    };
    ($yaml:expr) => {
        $crate::ArcNode::new($yaml.into(), 0, "", "")
    };
}

/// Create [`YamlBase::Array`] items literally.
///
/// ```
/// use yaml_peg::{node, yaml_array};
///
/// yaml_array![node!("a"), node!("b"), node!("c")];
/// ```
#[macro_export]
macro_rules! yaml_array {
    ($($token:tt)*) => {
        $crate::Yaml::Array(vec![$($token)*])
    };
}

/// Create [`YamlBase::Map`] items literally.
///
/// ```
/// use yaml_peg::{node, yaml_map};
///
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
        use core::iter::FromIterator;
        $crate::Yaml::from_iter(vec![($k1, $v1) $(, ($k2, $v2))*])
    }};
}

/// Create a custom anchor visitor.
///
/// The anchor name should implement [`alloc::string::ToString`] trait.
///
/// ```
/// use yaml_peg::{node, anchors};
///
/// let v = anchors![
///     "my-boss" => node!({node!("name") => node!("Henry")}),
/// ];
/// assert_eq!(v["my-boss"]["name"], node!("Henry"));
/// ```
#[macro_export]
macro_rules! anchors {
    () => {
        $crate::AnchorBase::new()
    };
    ($k1:expr => $v1:expr $(, $k2:expr => $v2:expr)* $(,)?) => {{
        use core::iter::FromIterator;
        $crate::AnchorBase::from_iter(vec![($k1.to_string(), $v1) $(, ($k2.to_string(), $v2))*])
    }};
}

mod anchors;
pub mod dumper;
mod indicator;
mod node;
pub mod parser;
pub mod repr;
#[cfg(test)]
mod tests;
mod yaml;
