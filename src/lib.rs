//! A YAML 1.2 parser using a greedy parsing algorithm with PEG atoms.
//!
//! The major purpose of this crate is to let the user build their own YAML
//! reader/builder/validator.
//!
//! This parser is not ensuring about YAML spec but almost functions are
//! well-implemented.
//!
//! The buffer reader has also not been implemented, but sub-parsers can read
//! the chunks.
//!
//! **WARNING: YAML 1.2 is compatible with [JSON (JavaScript Object Notation) format](https://www.json.org/),
//! but not in the strict mode.**
//!
//! # Parser
//!
//! Function [`parse`]/[`parse_cyclic`] is used to parse YAML string into
//! [`Node`] data structure, which has a data holder [`Yaml`].
//! There also has a multiple-threaded version corresponding to
//! [`NodeRc`]/[`NodeArc`] and [`YamlRc`]/[`YamlArc`]. To get back as string,
//! please use [`dump`] function.
//!
//! There are also have some macros for building [`Node`] structure from Rust
//! data. Especially [`node!`] macro, almost data can be built by the macro
//! literally.
//!
//! If you went to rise your own error message, [`indicated_msg`] might be a
//! good choice.
//!
//! ## Anchor Parsing
//!
//! + [`parse`]: The parser will replace the anchors during parsing.
//! + [`parse_cyclic`]: Cyclic data means that a parent alias is inserted at the
//! child node.   Keep the alias to avoid having undefined anchors when parsing.
//!
//! # No Standard Library
//!
//! The `std` feature is a default feature, use `--no-default-features` to build
//! in the no-std mode.
//!
//! # Serialization and Deserialization
//!
//! Enable `serde` feature to use `serde` crate,
//! which provides a set of protocol traits to convert between custom Rust data.
//! Please be aware that the additional fields will be discarded when convert to
//! a fix-sized structure. For example, the structure fields can be turned into
//! map keys as well.
//!
//! On the other hand, the primitive types are still able to transform to YAML
//! data without serialization, according to built-in `From` and `Into` traits.
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
extern crate alloc;
extern crate core;

pub use crate::{
    dumper::dump,
    indicator::*,
    node::*,
    parser::{parse, parse_cyclic},
    yaml::*,
};

/// Create [`Node`] items literally.
///
/// Literals and expressions will be transformed to [`Yaml`] automatically by
/// calling [`Into::into`].
///
/// ```
/// use yaml_peg::{node, NodeRc};
///
/// let k = "a";
/// assert_eq!(node!(k), node!("a"));
/// assert_eq!(node!(()), NodeRc::from(()));
/// ```
///
/// Arrays and maps can be created from this macro directly through brackets
/// (`[]`, `{}`).
///
/// ```
/// use yaml_peg::{node, NodeRc};
///
/// let v = vec![NodeRc::from(1), NodeRc::from(2)];
/// assert_eq!(node!([1, 2]), v.into_iter().collect());
/// let m = vec![(NodeRc::from(1), NodeRc::from(2))];
/// assert_eq!(node!({1 => 2}), m.into_iter().collect());
/// ```
#[macro_export]
macro_rules! node {
    (@[$v:expr; $n:expr]) => {{
        extern crate alloc;
        let v = alloc::vec![$crate::node!(@$v); $n];
        $crate::node!(@$crate::Yaml::Seq(v))
    }};
    (@[$($v:expr),* $(,)?]) => {{
        extern crate alloc;
        let v = alloc::vec![$($crate::node!(@$v)),*];
        $crate::node!(@$crate::Yaml::Seq(v))
    }};
    (@{$($k:expr => $v:expr),* $(,)?}) => {{
        extern crate alloc;
        let m = alloc::vec![$(($crate::node!(@$k), $crate::node!(@$v))),*];
        $crate::node!(@$crate::Yaml::Map(m.into_iter().collect()))
    }};
    (@*$anchor:expr) => {
        $crate::node!(@$crate::Yaml::Alias($anchor.into()))
    };
    (@$yaml:expr) => {
        $crate::Node::from($yaml)
    };
    (arc $($tt:tt)+) => {
        $crate::NodeArc::from($crate::node!(@$($tt)+))
    };
    (rc $($tt:tt)+) => {
        $crate::NodeRc::from($crate::node!(@$($tt)+))
    };
    ($($tt:tt)+) => {
        $crate::node!(rc $($tt)+)
    };
}

pub mod dumper;
mod indicator;
mod node;
pub mod parser;
pub mod repr;
#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
pub mod serde;
#[cfg(test)]
mod tests;
mod yaml;
