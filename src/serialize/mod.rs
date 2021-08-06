//! The implementation of serializer.
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
//!     node!("name") => node!("Bob"),
//!     node!("married") => node!(true),
//!     node!("age") => node!(46),
//! });
//! let officer = <Member as Deserialize>::deserialize(n).unwrap();
//! assert_eq!("Bob", officer.name);
//! assert!(officer.married);
//! assert_eq!(46, officer.age);
//! ```
//!
//! At least you should enable the `serde/derive` and `serde/alloc` features to run the example.
//! The `serde/derive` feature provides derive macro for the custom data,
//! and if `serde/alloc` is not used, you cannot deserialize [`alloc::string::String`] or [`alloc::vec::Vec`] type.
//!
//! For converting custom data into YAML data, please see [`to_node`] and [`to_arc_node`].
pub use self::error::SerdeError;
pub use self::ser::{to_arc_node, to_node};

mod de;
mod error;
mod ser;
mod ser_node;
