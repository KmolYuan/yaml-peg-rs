use super::*;
use alloc::format;
use core::fmt::{Display, Error, Formatter};

pub type PResult<T> = Result<T, PError>;

/// The error of parser handling, returned by [`Parser`].
///
/// Please see [module level document](super) for more error information.
#[derive(Debug)]
pub enum PError {
    /// If parser mismatched, just choose another one.
    Mismatch,
    /// The parser is the only one can be matched.
    Terminate(
        /// Name of sub-parser group.
        &'static str,
        /// Document position.
        u64,
    ),
}

impl PError {
    /// Transform to IO error.
    pub fn into_error(self, doc: &str) -> String {
        match self {
            Self::Mismatch => String::from("not matched"),
            Self::Terminate(name, pos) => {
                format!("invalid {}: \n\n{}", name, indicated_msg(doc, pos))
            }
        }
    }
}

impl Display for PError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_fmt(format_args!("{:?}", self))
    }
}
