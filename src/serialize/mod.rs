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
//! The anchors are represented as a **single** key-value pair `{ "anchor": anchor }` in the serialization.
//! In actual use, this can be achieved with a `enum` type field.
//! This implementation is done by [`Foreign`] type.
//!
//! The parent field will support anchor insertion when deserialized from [`NodeBase`](crate::NodeBase).
//! In the same way, anchor insertion can also be achieved when serializing into a node.
//!
//! ```
//! use serde::{Serialize, Deserialize};
//! use yaml_peg::{node, serialize::{to_node, Foreign}};
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct Content {
//!     doc: Foreign<String>,
//! }
//!
//! let doc = Content {
//!     doc: Foreign::Data("my doc".to_string()),
//! };
//! let anchor = Content {
//!     doc: Foreign::anchor("my-anchor"),
//! };
//! let n_doc = node!({"doc" => "my doc"});
//! let n_anchor = node!({"doc" => node!(*"my-anchor")});
//! // Node -> Content (Data)
//! assert_eq!(doc, Content::deserialize(n_doc.clone()).unwrap());
//! // Content -> Node (Data)
//! assert_eq!(n_doc, to_node(doc).unwrap());
//! // Node -> Content (Anchor)
//! assert_eq!(anchor, Content::deserialize(n_anchor.clone()).unwrap());
//! // Content -> Node (Anchor)
//! assert_eq!(n_anchor, to_node(anchor).unwrap());
//! ```
//!
//! The first-step inference is fine.
//! Since there are recursive issue in the YAML data,
//! so just keep replace the `Data::Anchor` variant with another one (`Data::Doc`).
//! For anchor indexing, please see [`AnchorBase`](crate::AnchorBase) type.
pub use self::de::from_str;
pub use self::error::SerdeError;
pub use self::foreign::Foreign;
pub use self::ser::{to_arc_node, to_node, to_string};

mod de;
mod error;
mod foreign;
mod ser;
mod ser_node;
