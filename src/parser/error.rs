use super::*;
use alloc::format;
use core::fmt::{Display, Formatter, Result};

/// The error of parser handling.
///
/// Not recommended to use it at other times.
#[derive(Debug)]
pub enum PError {
    /// If parser mismatched, just choose another one.
    Mismatch,
    /// The parser is the only one can be matched.
    Terminate(&'static str, u64),
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
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}
