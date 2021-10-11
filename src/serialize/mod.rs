//! The implementation of serialization. The technique is come from [`serde`].
//!
//! Here is an example for converting YAML data into a custom structure.
//!
//! ```
//! use serde::Deserialize;
//! use yaml_peg::node;
//!
//! #[derive(Deserialize)]
//! struct Member {
//!     name: String,
//!     married: bool,
//!     age: u8,
//! }
//!
//! let n = node!({
//!     "name" => "Bob",
//!     "married" => true,
//!     "age" => 46,
//! });
//! let officer = Member::deserialize(n).unwrap();
//! assert_eq!("Bob", officer.name);
//! assert!(officer.married);
//! assert_eq!(46, officer.age);
//! ```
//!
//! At least you should enable the `serde/derive` and `serde/alloc` features to run the example.
//! The `serde/derive` feature provides derive macro for the custom data,
//! and if `serde/alloc` is not used, you cannot deserialize [`alloc::string::String`] or [`alloc::vec::Vec`] type.
//!
//! For converting custom data into YAML data, please see [`to_node`] and [`to_arc_node`],
//! and if you went to parse / dump YAML document, use [`from_str`] and [`to_string`].
//!
//! # Anchors
//!
//! If the data supports anchor insertion, please see [`Foreign`] type.
//!
//! # Mixed String Type
//!
//! If the data needs to deserialized from any type into string, please see [`Stringify`] type.
//!
//! # Mixed Listed Map
//!
//! If the data supports listed items but allows single mapped item, please see [`InlineList`] type.
pub use self::{
    de::from_str,
    error::SerdeError,
    foreign::Foreign,
    inline_list::InlineList,
    optional::Optional,
    ser::{to_arc_node, to_node, to_string},
    stringify::Stringify,
};

mod de;
mod error;
mod foreign;
mod inline_list;
mod optional;
mod ser;
mod ser_node;
mod stringify;
