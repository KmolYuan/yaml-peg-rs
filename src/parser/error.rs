use super::*;
use std::io::Error;

/// The error of parser handling.
///
/// Not recommended to use it at other times.
pub enum PError {
    /// If parser mismatched, just choose another one.
    Mismatch,
    /// The parser is the only one can be matched.
    Terminal(usize, String),
}

impl PError {
    /// Transform to IO error.
    pub fn into_error(self, doc: &str) -> Error {
        match self {
            Self::Mismatch => err!("not matched"),
            Self::Terminal(pos, name) => {
                err!(format!("invalid {}: \n\n{}", name, indicated_msg(doc, pos)))
            }
        }
    }
}

impl From<()> for PError {
    fn from(_: ()) -> Self {
        Self::Mismatch
    }
}
