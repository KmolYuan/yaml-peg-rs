use crate::{repr::*, *};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::iter::FromIterator;
use ritelinked::LinkedHashMap;

macro_rules! impl_from {
    ($(impl $($from_ty:ty),+ => $ty:ident)+) => {
        $($(impl<R: Repr> From<$from_ty> for YamlBase<R> {
            fn from(s: $from_ty) -> Self {
                Self::$ty(s.to_string())
            }
        })+)+
    };
}

macro_rules! impl_iter {
    ($(impl $($item:ty),+ => $ty:ident)+) => {
        $($(impl<R: Repr> FromIterator<$item> for YamlBase<R> {
            fn from_iter<T: IntoIterator<Item = $item>>(iter: T) -> Self {
                Self::$ty(iter.into_iter().collect())
            }
        })+)+
    };
}

/// A YAML data with [`alloc::rc::Rc`] holder.
pub type Yaml = YamlBase<RcRepr>;
/// A YAML data with [`alloc::sync::Arc`] holder.
pub type ArcYaml = YamlBase<ArcRepr>;
/// The sequence data structure of YAML.
pub type Seq<R> = Vec<NodeBase<R>>;
/// The map data structure of YAML.
pub type Map<R> = LinkedHashMap<NodeBase<R>, NodeBase<R>>;

/// YAML data types, but it is recommended to use [`NodeBase`] for shorten code.
///
/// This type can convert from primitive types by `From` and `Into` traits.
///
/// ```
/// use yaml_peg::Yaml;
///
/// assert_eq!(Yaml::Int("20".to_string()), Yaml::from(20));
/// assert_eq!(Yaml::Float("0.001".to_string()), 1e-3.into());
/// ```
///
/// Also, the iterators can turned to sequence and map.
///
/// ```
/// use yaml_peg::{node, Yaml};
/// use std::iter::FromIterator;
///
/// let v = vec![node!(1), node!(2), node!(3)];
/// assert_eq!(Yaml::Seq(v.clone()), Yaml::from_iter(v));
/// let m = vec![(node!(1), node!(2)), (node!(3), node!(4))];
/// assert_eq!(Yaml::Map(m.clone().into_iter().collect()), Yaml::from_iter(m));
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
    /// Sequence
    Seq(Seq<R>),
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

impl<R: Repr> From<()> for YamlBase<R> {
    fn from(_: ()) -> Self {
        Self::Null
    }
}

impl<R: Repr> From<bool> for YamlBase<R> {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl_from! {
    impl char, &str, String, &String => Str
    impl usize, u8, u16, u32, u64, u128, isize, i8, i16, i32, i64, i128 => Int
    impl f32, f64 => Float
}

impl<R: Repr> From<Seq<R>> for YamlBase<R> {
    fn from(a: Seq<R>) -> Self {
        Self::Seq(a)
    }
}

impl<R: Repr> From<Map<R>> for YamlBase<R> {
    fn from(m: Map<R>) -> Self {
        Self::Map(m)
    }
}

impl_iter! {
    impl NodeBase<R> => Seq
    impl (NodeBase<R>, NodeBase<R>) => Map
}
