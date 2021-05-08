use super::*;
use std::io::Error;

/// The error of parser handling.
///
/// Not recommended to use it at other times.
pub struct PError {
    pos: usize,
    msg: String,
}

impl PError {
    /// Create an error.
    pub fn new(pos: usize, msg: &str) -> Self {
        Self {
            pos,
            msg: msg.into(),
        }
    }

    /// Transform to IO error.
    pub fn into_error(self, doc: &str) -> Error {
        err!(format!(
            "{}: \n\n{}",
            self.msg,
            indicated_msg(doc, self.pos)
        ))
    }
}
