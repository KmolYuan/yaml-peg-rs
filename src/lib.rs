pub use crate::indicator::*;
pub use crate::node::*;
pub use crate::parser::*;
pub use crate::yaml::*;

macro_rules! err {
    ($e:expr) => {{
        use std::io::{Error, ErrorKind};
        Error::new(ErrorKind::InvalidData, $e)
    }};
}

mod indicator;
mod node;
mod parser;
#[cfg(test)]
mod tests;
mod yaml;
