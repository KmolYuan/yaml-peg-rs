//! A YAML 1.2 parser using greedy parsing algorithm with PEG atoms.
//!
//! The major purpose of this crate is to let the user build their own YAML reader / builder / validator.
//!
//! This parser is not ensure about YAML spec but almost functions are well-implemented.
//! (test case is [here](https://github.com/KmolYuan/yaml-peg-rs/blob/main/src/tests/test.yaml))
//!
//! The buffer reader has also not implemented, but the chunks can be read by sub-parsers.
//!
//! # Parser
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
//! # Anchors
//!
//! The anchor system [`AnchorBase`] is implemented by using [`alloc::rc::Rc`] and [`alloc::sync::Arc`] as inner handler.
//! Additionally, [`anchors!`] macro can used to create anchor visitor by yourself.
//!
//! # Serialization and Deserialization
//!
//! Enable `serialize` feature to use `serde` crate.
//! The crate provides a set of protocol traits to convert between custom Rust data.
//! Please be aware that the additional fields will be discard when convert to a fix-sized structure.
//! For example, the structure fields can be turned into map keys as well.
//!
//! On the other hand, the primitive types still able to transform to YAML data without serialization,
//! according to `From` and `Into` traits.
//!
//! See [`serialize`] module for more information.
#![no_std]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
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
        $crate::node!(())
    };
    (*$anchor:expr) => {
        $crate::node!($crate::YamlBase::Anchor($anchor.into()))
    };
    ($yaml:expr) => {
        $crate::Node::from($yaml)
    };
}

/// Create [`ArcNode`] items literally.
///
/// The API is same as [`node!`] macro.
///
/// ```
/// use yaml_peg::{node_arc as node, ArcNode};
///
/// assert_eq!(node!(null), ArcNode::from(()));
/// ```
#[macro_export]
macro_rules! node_arc {
    ([$($token:tt)*]) => {
        $crate::node_arc!($crate::yaml_array![$($token)*])
    };
    ({$($token:tt)*}) => {
        $crate::node_arc!($crate::yaml_map![$($token)*])
    };
    (null) => {
        $crate::node_arc!(())
    };
    (*$anchor:expr) => {
        $crate::node_arc!($crate::YamlBase::Anchor($anchor.into()))
    };
    ($yaml:expr) => {
        $crate::ArcNode::from($yaml)
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
    ($($token:tt)*) => {{
        extern crate alloc;
        $crate::YamlBase::Array(alloc::vec![$($token)*])
    }};
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
    ($($k:expr => $v:expr),* $(,)?) => {{
        extern crate alloc;
        $crate::YamlBase::Map(alloc::vec![$(($k, $v)),*].into_iter().collect())
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
    ($($k:expr => $v:expr),* $(,)?) => {{
        extern crate alloc;
        alloc::vec![$(($k.to_string(), $v)),*].into_iter().collect::<$crate::AnchorBase<_>>()
    }};
}

mod anchors;
pub mod dumper;
mod indicator;
mod node;
pub mod parser;
pub mod repr;
#[cfg(feature = "serialize")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serialize")))]
pub mod serialize;
#[cfg(test)]
mod tests;
mod yaml;
