use alloc::string::String;
use core::fmt::{Display, Error, Formatter};

/// Type of the parser result.
pub type PResult<T> = Result<T, PError>;

/// The error of parser handling, returned by [`Parser`](super::Parser).
///
/// Please see [module level document](super) for more error information.
#[derive(Debug)]
pub enum PError {
    /// If parser mismatched, just choose another one.
    Mismatch,
    /// The parser is the only one can be matched.
    Terminate {
        /// Name of sub-parser group.
        name: &'static str,
        /// Document position.
        msg: String,
    },
}

impl PError {
    /// A "than" function for the error handling, execute the given function
    /// when mismatched.
    pub fn or<R, F>(self, f: F) -> Result<R, Self>
    where
        F: FnOnce() -> Result<R, Self>,
    {
        match self {
            Self::Mismatch => f(),
            Self::Terminate { .. } => Err(self),
        }
    }
}

impl Display for PError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::Mismatch => write!(f, "not matched"),
            Self::Terminate { name, msg } => {
                write!(f, "invalid {}: \n\n{}", name, msg)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PError {}
