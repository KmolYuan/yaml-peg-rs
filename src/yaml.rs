use crate::*;
use linked_hash_map::LinkedHashMap;
use std::{fmt::Display, iter::FromIterator};

macro_rules! yaml_from_method {
    ($from_ty1:ty $(| $from_ty2:ty)* as $ty:ident) => {
        impl From<$from_ty1> for Yaml {
            fn from(s: $from_ty1) -> Self {
                Yaml::$ty(format!("{}", s))
            }
        }
        $(
        impl From<$from_ty2> for Yaml {
            fn from(s: $from_ty2) -> Self {
                Yaml::$ty(format!("{}", s))
            }
        }
        )*
    };
}

/// The array data structure of YAML.
pub type Array = Vec<Node>;
/// The map data structure of YAML.
pub type Map = LinkedHashMap<Node, Node>;

/// YAML data types, can convert from primitive types by `From` and `Into` methods.
///
/// ```
/// use yaml_pom::Yaml;
/// assert_eq!(Yaml::Int("20".into()), 20.into());
/// assert_eq!(Yaml::Float("0.001".into()), 1e-3.into());
/// ```
///
/// Also, the iterators can turn into arrays and maps.
///
/// ```
/// use yaml_pom::{Yaml, yaml_array, yaml_map, node};
/// use std::iter::FromIterator;
/// let v = vec![node!(1), node!(2), node!(3)];
/// assert_eq!(Yaml::from_iter(v), yaml_array![node!(1), node!(2), node!(3)]);
/// let m = vec![(node!(1), node!(2)), (node!(3), node!(4))];
/// assert_eq!(Yaml::from_iter(m), yaml_map!{node!(1) => node!(2), node!(3) => node!(4)});
/// ```
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
    /// Check the anchor is valid.
    pub fn is_valid_anchor<T>(s: T) -> bool
    where
        T: Display,
    {
        let s = format!("{}", s);
        let ok = identifier().parse(s.as_bytes()).is_ok();
        !ok
    }
}

impl From<bool> for Yaml {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

yaml_from_method! { &str | String | &String as Str }
yaml_from_method! { u8 | u16 | u32 | u64 | u128 | i8 | i16 | i32 | i64 | i128 as Int }
yaml_from_method! { f32 | f64 as Float }

impl FromIterator<Node> for Yaml {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Node>,
    {
        Self::Array(iter.into_iter().collect())
    }
}

impl FromIterator<(Node, Node)> for Yaml {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Node, Node)>,
    {
        Self::Map(iter.into_iter().collect())
    }
}
