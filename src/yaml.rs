use crate::{repr::*, *};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::{
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
    iter::FromIterator,
};
use ritelinked::LinkedHashMap;

macro_rules! impl_from {
    ($(impl $($from_ty:ty),+ => $ty:ident)+) => {
        $($(impl<R: Repr> From<$from_ty> for Yaml<R> {
            fn from(s: $from_ty) -> Self {
                Self::$ty(s.to_string())
            }
        })+)+
    };
}

macro_rules! impl_iter {
    ($(impl $($item:ty),+ => $ty:ident)+) => {
        $($(impl<R: Repr> FromIterator<$item> for Yaml<R> {
            fn from_iter<T: IntoIterator<Item = $item>>(iter: T) -> Self {
                Self::$ty(iter.into_iter().collect())
            }
        })+)+
    };
}

/// A YAML data with [`alloc::rc::Rc`] holder.
pub type YamlRc = Yaml<RcRepr>;
/// A YAML data with [`alloc::sync::Arc`] holder.
pub type YamlArc = Yaml<ArcRepr>;
/// The sequence data structure of YAML.
pub type Seq<R> = Vec<Node<R>>;
/// The map data structure of YAML.
pub type Map<R> = LinkedHashMap<Node<R>, Node<R>>;

pub(crate) fn to_i64(s: &str) -> Result<i64, core::num::ParseIntError> {
    if let Some(s) = s.strip_prefix("0x") {
        i64::from_str_radix(s, 16)
    } else if let Some(s) = s.strip_prefix("0o") {
        i64::from_str_radix(s, 8)
    } else {
        s.parse()
    }
}

pub(crate) fn to_f64(s: &str) -> Result<f64, core::num::ParseFloatError> {
    s.parse()
}

/// YAML data types, but it is recommended to use [`Node`] for shorten code.
///
/// This type can convert from primitive types by `From` and `Into` traits.
///
/// ```
/// use yaml_peg::YamlRc;
///
/// assert_eq!(YamlRc::Int("20".to_string()), YamlRc::from(20));
/// assert_eq!(YamlRc::Float("0.001".to_string()), 1e-3.into());
/// ```
///
/// Also, the iterators can turned to sequence and map.
///
/// ```
/// use std::iter::FromIterator;
/// use yaml_peg::{node, YamlRc};
///
/// let v = vec![node!(1), node!(2), node!(3)];
/// assert_eq!(YamlRc::Seq(v.clone()), YamlRc::from_iter(v));
/// let m = vec![(node!(1), node!(2)), (node!(3), node!(4))];
/// assert_eq!(
///     YamlRc::Map(m.clone().into_iter().collect()),
///     YamlRc::from_iter(m)
/// );
/// ```
///
/// The digit NaN (not-a-number) will be equal in the comparison.
pub enum Yaml<R: Repr> {
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
    /// Alias (anchor insertion)
    Alias(String),
}

impl<R: Repr> Debug for Yaml<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Null => f.write_str("Null"),
            Self::Bool(b) => f.debug_tuple("Bool").field(b).finish(),
            Self::Int(s) => f.debug_tuple("Int").field(s).finish(),
            Self::Float(s) => f.debug_tuple("Float").field(s).finish(),
            Self::Str(s) => f.debug_tuple("Str").field(s).finish(),
            Self::Seq(s) => f.debug_tuple("Seq").field(s).finish(),
            Self::Map(m) => f.debug_tuple("Map").field(m).finish(),
            Self::Alias(a) => f.debug_tuple("Alias").field(a).finish(),
        }
    }
}

impl<R: Repr> Clone for Yaml<R> {
    fn clone(&self) -> Self {
        match self {
            Self::Null => Self::Null,
            Self::Bool(b) => Self::Bool(*b),
            Self::Int(s) => Self::Int(s.clone()),
            Self::Float(s) => Self::Float(s.clone()),
            Self::Str(s) => Self::Str(s.clone()),
            Self::Seq(s) => Self::Seq(s.clone()),
            Self::Map(m) => Self::Map(m.clone()),
            Self::Alias(a) => Self::Alias(a.clone()),
        }
    }
}

impl<R: Repr> Hash for Yaml<R> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Null => state.write_u8(1),
            Self::Bool(b) => {
                state.write_u8(2);
                b.hash(state)
            }
            Self::Int(s) => {
                state.write_u8(3);
                s.hash(state)
            }
            Self::Float(s) => {
                state.write_u8(4);
                s.hash(state)
            }
            Self::Str(s) => {
                state.write_u8(5);
                s.hash(state)
            }
            Self::Seq(s) => {
                state.write_u8(6);
                s.hash(state)
            }
            Self::Map(m) => {
                state.write_u8(7);
                m.hash(state)
            }
            Self::Alias(a) => {
                state.write_u8(8);
                a.hash(state)
            }
        }
    }
}

impl<R: Repr> PartialEq for Yaml<R> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Bool(b1), Self::Bool(b2)) => b1 == b2,
            (Self::Int(s1), Self::Int(s2)) => to_i64(s1).unwrap() == to_i64(s2).unwrap(),
            (Self::Float(s1), Self::Float(s2)) => {
                let f1 = to_f64(s1).unwrap();
                let f2 = to_f64(s2).unwrap();
                if f1.is_nan() && f2.is_nan() {
                    true
                } else {
                    f1 == f2
                }
            }
            (Self::Str(s1), Self::Str(s2)) => s1 == s2,
            (Self::Seq(s1), Self::Seq(s2)) => s1 == s2,
            (Self::Map(m1), Self::Map(m2)) => m1 == m2,
            (Self::Alias(a1), Self::Alias(a2)) => a1 == a2,
            _ => false,
        }
    }
}

impl<R: Repr> Eq for Yaml<R> {}

impl<R: Repr> From<()> for Yaml<R> {
    fn from(_: ()) -> Self {
        Self::Null
    }
}

impl<R: Repr> From<bool> for Yaml<R> {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl_from! {
    impl char, &str, String, &String => Str
    impl usize, u8, u16, u32, u64, u128, isize, i8, i16, i32, i64, i128 => Int
    impl f32, f64 => Float
}

impl<R: Repr> From<Seq<R>> for Yaml<R> {
    fn from(a: Seq<R>) -> Self {
        Self::Seq(a)
    }
}

impl<R: Repr> From<Map<R>> for Yaml<R> {
    fn from(m: Map<R>) -> Self {
        Self::Map(m)
    }
}

impl_iter! {
    impl Node<R> => Seq
    impl (Node<R>, Node<R>) => Map
}
