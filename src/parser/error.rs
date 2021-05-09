use super::*;
use std::io::Error;

/// The error of parser handling.
///
/// Not recommended to use it at other times.
pub struct PError {
    pos: usize,
    name: String,
}

impl PError {
    /// Create an error.
    pub fn new(pos: usize, name: &str) -> Self {
        Self {
            pos,
            name: name.into(),
        }
    }

    /// Transform to IO error.
    pub fn into_error(self, doc: &str) -> Error {
        err!(format!(
            "invalid {}: \n\n{}",
            self.name,
            indicated_msg(doc, self.pos)
        ))
    }
}
