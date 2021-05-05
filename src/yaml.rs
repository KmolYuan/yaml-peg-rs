use crate::*;
use linked_hash_map::LinkedHashMap;
use std::fmt::Display;

macro_rules! yaml_from_method {
    ($(#[$meta:meta])* fn $id:ident = $ty:ident) => {
        $(#[$meta])*
        pub fn $id<T>(s: T) -> Self
        where
            T: Display,
        {
            Self::$ty(format!("{}", s))
        }
    };
}

/// The array data structure of YAML.
pub type Array = Vec<Node>;
/// The map data structure of YAML.
pub type Map = LinkedHashMap<Node, Node>;

/// Yaml data types.
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Yaml {
    /// Null
    Null,
    /// Boolean
    Bool(bool),
    /// Integer
    Int(String),
    /// Float
    Float(String),
    /// String
    Str(String),
    /// Array
    Array(Array),
    /// Map
    Map(Map),
    /// Anchor insertion
    Anchor(String),
}

impl Yaml {
    yaml_from_method! {
        /// Create from integer.
        fn int = Int
    }
    yaml_from_method! {
        /// Create from float.
        fn float = Float
    }
    yaml_from_method! {
        /// Create from string.
        fn string = Str
    }
    yaml_from_method! {
        /// Create an inserted anchor, won't check.
        fn anchor = Anchor
    }

    /// Check the anchor is valid.
    pub fn is_valid_anchor<T>(s: T) -> bool
    where
        T: Display,
    {
        let s = format!("{}", s);
        !s.contains(" ")
    }
}

impl From<&str> for Yaml {
    fn from(s: &str) -> Self {
        Yaml::Str(s.into())
    }
}
