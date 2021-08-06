//! The implementation of serializer.
pub use self::error::SerdeError;
pub use self::ser::{to_arc_node, to_node};

mod de;
mod error;
mod ser;
mod ser_node;
