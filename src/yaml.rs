use crate::*;
use linked_hash_map::LinkedHashMap;
use std::{fmt::Display, iter::FromIterator};

macro_rules! yaml_from_method {
    ($from_ty1:ty $(| $from_ty2:ty)* as $ty:ident) => {
        impl<R: repr::Repr> From<$from_ty1> for YamlBase<R> {
            fn from(s: $from_ty1) -> Self {
                Self::$ty(format!("{}", s))
            }
        }
        $(
        impl<R: repr::Repr> From<$from_ty2> for YamlBase<R> {
            fn from(s: $from_ty2) -> Self {
                Self::$ty(format!("{}", s))
            }
        }
        )*
    };
}

/// A YAML data with [`std::rc::Rc`] holder.
pub type Yaml = YamlBase<repr::RcRepr>;
/// A YAML data with [`std::sync::Arc`] holder.
pub type ArcYaml = YamlBase<repr::ArcRepr>;
/// The array data structure of YAML.
pub type Array<R> = Vec<NodeBase<R>>;
/// The map data structure of YAML.
pub type Map<R> = LinkedHashMap<NodeBase<R>, NodeBase<R>>;
/// Anchor visitor is made by a hash map that you can get the node reference inside.
///
/// Since [`Node`] type is holding a reference counter,
/// the data are just a viewer to the original memory.
pub type AnchorVisitor<R> = LinkedHashMap<String, NodeBase<R>>;

/// YAML data types, but it is recommended to use [`NodeBase`] for shorten code.
///
/// This type can convert from primitive types by `From` and `Into` methods.
///
/// ```
/// use yaml_peg::Yaml;
/// assert_eq!(Yaml::Int("20".into()), 20.into());
/// assert_eq!(Yaml::Float("0.001".into()), 1e-3.into());
/// ```
///
/// Also, the iterators can turn into arrays and maps.
///
/// ```
/// use yaml_peg::{Yaml, node};
/// use yaml_peg::{yaml_array, yaml_map};
/// use std::iter::FromIterator;
/// let v = vec![node!(1), node!(2), node!(3)];
/// assert_eq!(Yaml::from_iter(v), yaml_array![node!(1), node!(2), node!(3)]);
/// let m = vec![(node!(1), node!(2)), (node!(3), node!(4))];
/// assert_eq!(Yaml::from_iter(m), yaml_map!{node!(1) => node!(2), node!(3) => node!(4)});
/// ```
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum YamlBase<R: repr::Repr> {
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
    Array(Array<R>),
    /// Map
    Map(Map<R>),
    /// Anchor insertion
    Anchor(String),
}

impl<R: repr::Repr> YamlBase<R> {
    /// Check the anchor is valid.
    pub fn is_valid_anchor<T>(s: T) -> bool
    where
        T: Display,
    {
        parser::Parser::<R>::new(format!("{}", s).as_bytes())
            .identifier()
            .is_ok()
    }
}

impl<R: repr::Repr> From<bool> for YamlBase<R> {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

yaml_from_method! { &str | String | &String as Str }
yaml_from_method! { u8 | u16 | u32 | u64 | u128 | i8 | i16 | i32 | i64 | i128 as Int }
yaml_from_method! { f32 | f64 as Float }

impl<R: repr::Repr> FromIterator<NodeBase<R>> for YamlBase<R> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = NodeBase<R>>,
    {
        Self::Array(iter.into_iter().collect())
    }
}

impl<R: repr::Repr> FromIterator<(NodeBase<R>, NodeBase<R>)> for YamlBase<R> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (NodeBase<R>, NodeBase<R>)>,
    {
        Self::Map(iter.into_iter().collect())
    }
}
