use crate::{repr::*, *};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::iter::FromIterator;
use ritelinked::LinkedHashMap;

macro_rules! impl_from {
    ($from_ty:ty => $ty:ident) => {
        impl<R: Repr> From<$from_ty> for YamlBase<R> {
            fn from(s: $from_ty) -> Self {
                Self::$ty(s.to_string())
            }
        }
    };
    ($ty1:ty $(, $ty2:ty)* => $ty:ident) => {
        impl_from! {$ty1 => $ty}
        $(impl_from! {$ty2 => $ty})*
    };
}

macro_rules! impl_iter {
    ($item:ty => $ty:ident) => {
        impl<R: Repr> FromIterator<$item> for YamlBase<R> {
            fn from_iter<T: IntoIterator<Item = $item>>(iter: T) -> Self {
                Self::$ty(iter.into_iter().collect())
            }
        }
    };
    ($item1:ty $(, $item2:ty)* => $ty:ident) => {
        impl_iter!{$item1 => $ty}
        $(impl_iter!{$item2 => $ty})*
    }
}

/// A YAML data with [`alloc::rc::Rc`] holder.
pub type Yaml = YamlBase<RcRepr>;
/// A YAML data with [`alloc::sync::Arc`] holder.
pub type ArcYaml = YamlBase<ArcRepr>;
/// The array data structure of YAML.
pub type Array<R> = Vec<NodeBase<R>>;
/// The map data structure of YAML.
pub type Map<R> = LinkedHashMap<NodeBase<R>, NodeBase<R>>;

/// YAML data types, but it is recommended to use [`NodeBase`] for shorten code.
///
/// This type can convert from primitive types by `From` and `Into` methods.
///
/// ```
/// use yaml_peg::Yaml;
///
/// assert_eq!(Yaml::Int("20".into()), 20.into());
/// assert_eq!(Yaml::Float("0.001".into()), 1e-3.into());
/// ```
///
/// Also, the iterators can turn into arrays and maps.
///
/// ```
/// use yaml_peg::{Yaml, node};
/// use yaml_peg::{yaml_array, yaml_map};
///
/// use std::iter::FromIterator;
/// let v = vec![node!(1), node!(2), node!(3)];
/// assert_eq!(Yaml::from_iter(v), yaml_array![node!(1), node!(2), node!(3)]);
/// let m = vec![(node!(1), node!(2)), (node!(3), node!(4))];
/// assert_eq!(Yaml::from_iter(m), yaml_map!{node!(1) => node!(2), node!(3) => node!(4)});
/// ```
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum YamlBase<R: Repr> {
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

impl<R: Repr> YamlBase<R> {
    /// Check the anchor is valid.
    pub fn is_valid_anchor<S: ToString>(s: S) -> bool {
        parser::Parser::<R>::new(s.to_string().as_bytes())
            .identifier()
            .is_ok()
    }
}

impl<R: Repr> From<bool> for YamlBase<R> {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl_from! {&str, String, &String => Str}
impl_from! {usize, u8, u16, u32, u64, u128, isize, i8, i16, i32, i64, i128 => Int}
impl_from! {f32, f64 => Float}
impl_iter! {NodeBase<R> => Array}
impl_iter! {(NodeBase<R>, NodeBase<R>) => Map}
