//! A YAML 1.2 parser using greedy parsing algorithm with PEG atoms.
//!
//! The major purpose of this crate is to let the user build their own YAML reader / builder / validator.
//!
//! This parser is not ensure about YAML spec but almost functions are well-implemented.
//!
//! The buffer reader has also not implemented, but the chunks can be read by sub-parsers.
//!
//! WARN: YAML 1.2 is compatible with [JSON (JavaScript Object Notation) format](https://www.json.org/),
//! but not in strict mode.
//!
//! # Parser
//!
//! Function [`parse`] is used to parse YAML string into [`Node`] data structure,
//! which has a data holder [`Yaml`].
//! There also has multiple thread version corresponding to [`ArcNode`] and [`ArcYaml`].
//! To get back as string, please use [`dump`] function.
//!
//! There are also have some macros for building [`NodeBase`] structure from Rust data.
//! Especially [`node!`] macro, almost data can be built by the macro literally.
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
//! Enable `serde` / `serde-std` feature to use `serde` crate.
//! The crate provides a set of protocol traits to convert between custom Rust data.
//! Please be aware that the additional fields will be discard when convert to a fix-sized structure.
//! For example, the structure fields can be turned into map keys as well.
//!
//! On the other hand, the primitive types still able to transform to YAML data without serialization,
//! according to `From` and `Into` traits.
#![cfg_attr(
    feature = "serde",
    doc = "See [`serialize`] module for more information."
)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![cfg_attr(not(feature = "serde-std"), no_std)]
#![warn(missing_docs)]
extern crate alloc;

pub use crate::{anchors::*, dumper::dump, indicator::*, node::*, parser::parse, yaml::*};

/// Create [`Node`] items literally.
///
/// Literals and expressions will be transformed to [`Yaml`] automatically by calling [`Into::into`].
///
/// ```
/// use yaml_peg::{node, Node};
///
/// let k = "a";
/// assert_eq!(node!(k), node!("a"));
/// assert_eq!(node!(()), Node::from(()));
/// ```
///
/// Arrays and maps can be created from this macro directly through brackets (`[]`, `{}`).
///
/// ```
/// use yaml_peg::{node, Node};
///
/// let v = vec![Node::from(1), Node::from(2)];
/// assert_eq!(node!([1, 2]), v.into_iter().collect());
/// let m = vec![(Node::from(1), Node::from(2))];
/// assert_eq!(node!({1 => 2}), m.into_iter().collect());
/// ```
///
/// The [`YamlBase::Anchor`] is also supported by the syntax:
///
/// ```
/// use yaml_peg::{node, YamlBase};
///
/// assert_eq!(node!(YamlBase::Anchor("x".to_string())), node!(*"x"));
/// ```
///
/// This macro is use [`Node`] by default,
/// to specify them, a "rc" or "arc" prefix token can choose the presentation.
///
/// ```
/// use yaml_peg::{node, ArcNode};
///
/// assert_eq!(node!(arc()), ArcNode::from(()));
/// ```
#[macro_export]
macro_rules! node {
    (@[$v:expr; $n:expr]) => {{
        extern crate alloc;
        let v = alloc::vec![$crate::node!(@$v); $n];
        $crate::node!(@$crate::YamlBase::Seq(v))
    }};
    (@[$($v:expr),* $(,)?]) => {{
        extern crate alloc;
        let v = alloc::vec![$($crate::node!(@$v)),*];
        $crate::node!(@$crate::YamlBase::Seq(v))
    }};
    (@{$($k:expr => $v:expr),* $(,)?}) => {{
        extern crate alloc;
        let m = alloc::vec![$(($crate::node!(@$k), $crate::node!(@$v))),*];
        $crate::node!(@$crate::YamlBase::Map(m.into_iter().collect()))
    }};
    (@*$anchor:expr) => {
        $crate::node!(@$crate::YamlBase::Anchor($anchor.into()))
    };
    (@$yaml:expr) => {
        $crate::NodeBase::from($yaml)
    };
    (arc $($tt:tt)+) => {
        $crate::ArcNode::from($crate::node!(@$($tt)+))
    };
    (rc $($tt:tt)+) => {
        $crate::Node::from($crate::node!(@$($tt)+))
    };
    ($($tt:tt)+) => {
        $crate::node!(rc $($tt)+)
    };
}

/// Create a custom anchor visitor.
///
/// The anchor name should implement [`alloc::string::ToString`] trait.
/// All items will convert to [`NodeBase`] automatically.
///
/// ```
/// use yaml_peg::{node, anchors};
///
/// let v = anchors![
///     "my-boss" => node!({"name" => "Henry"}),
/// ];
/// assert_eq!(v["my-boss"]["name"], node!("Henry"));
/// ```
#[macro_export]
macro_rules! anchors {
    ($($k:expr => $v:expr),* $(,)?) => {{
        extern crate alloc;
        let v = alloc::vec![$(($k.to_string(), $crate::node!(@$v))),*];
        v.into_iter().collect::<$crate::AnchorBase<_>>()
    }};
}

mod anchors;
pub mod dumper;
mod indicator;
mod node;
pub mod parser;
pub mod repr;
#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
pub mod serialize;
#[cfg(test)]
mod tests;
mod yaml;
