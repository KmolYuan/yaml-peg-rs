use super::SerdeError;
use crate::{
    parse,
    repr::{RcRepr, Repr},
    to_f64, to_i64, Map, Node, Seq, Yaml,
};
use alloc::{format, string::ToString, vec::Vec};
use core::marker::PhantomData;
use serde::{
    de::{
        DeserializeOwned, DeserializeSeed, EnumAccess, Error, Expected, MapAccess, SeqAccess,
        Unexpected, VariantAccess, Visitor,
    },
    serde_if_integer128, Deserialize, Deserializer,
};

macro_rules! impl_visitor {
    (@) => { () };
    (@$ty:ty, $name:ident) => { $name };
    ($(fn $method:ident$(($ty:ty))?)+) => {
        $(fn $method<E>(self$(, v: $ty)?) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(impl_visitor!(@$($ty, v)?).into())
        })+
    };
}

macro_rules! impl_deserializer {
    ($(fn $method:ident($ty:ident) => $visit:ident($n:ident => $value:expr))+) => {
        $(fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'a>,
        {
            match self.yaml() {
                Yaml::$ty($n) => visitor.$visit($value),
                _ => Err(unexpected(&self, visitor)),
            }
        })+
    };
}

/// Parse the document and deserialize nodes to a specific type.
///
/// Since the document can be split into multiple parts,
/// so this function will return a vector container.
///
/// ```
/// use serde::Deserialize;
/// use yaml_peg::serde::from_str;
///
/// #[derive(Deserialize)]
/// struct Member {
///     name: String,
///     married: bool,
///     age: u8,
/// }
///
/// let doc = "
/// ---
/// name: Bob
/// married: true
/// age: 46
/// ";
/// // Return Vec<Member>, use `.remove(0)` to get the first one
/// let officer = from_str::<Member>(doc).unwrap().remove(0);
/// assert_eq!("Bob", officer.name);
/// assert!(officer.married);
/// assert_eq!(46, officer.age);
/// ```
pub fn from_str<D>(doc: &str) -> Result<Vec<D>, SerdeError>
where
    D: DeserializeOwned,
{
    let root = parse::<RcRepr>(doc).map_err(|e| e.to_string())?;
    root.into_iter().map(D::deserialize).collect()
}

struct NodeVisitor<R: Repr>(PhantomData<R>);

impl<'a, R: Repr> Visitor<'a> for NodeVisitor<R> {
    type Value = Node<R>;

    fn expecting(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.write_str("YAML value")
    }

    impl_visitor! {
        fn visit_bool(bool)
        fn visit_i64(i64)
        fn visit_u64(u64)
        fn visit_f64(f64)
        fn visit_str(&str)
        fn visit_none
        fn visit_unit
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'a>,
    {
        Deserialize::deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'a>,
    {
        let mut a = Seq::new();
        while let Some(e) = seq.next_element()? {
            a.push(e);
        }
        Ok(a.into_iter().collect())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'a>,
    {
        let mut m = Map::<R>::new();
        while let Some((k, v)) = map.next_entry()? {
            m.insert(k, v);
        }
        Ok(m.into_iter().collect())
    }
}

struct SeqVisitor<R: Repr>(<Seq<R> as IntoIterator>::IntoIter);

impl<R: Repr> From<Seq<R>> for SeqVisitor<R> {
    fn from(v: Seq<R>) -> Self {
        Self(v.into_iter())
    }
}

impl<'a, R: Repr> SeqAccess<'a> for SeqVisitor<R> {
    type Error = SerdeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'a>,
    {
        match self.0.next() {
            Some(e) => seed.deserialize(e).map(Some),
            None => Ok(None),
        }
    }
}

struct MapVisitor<R: Repr>(<Map<R> as IntoIterator>::IntoIter, Option<Node<R>>);

impl<R: Repr> From<Map<R>> for MapVisitor<R> {
    fn from(m: Map<R>) -> Self {
        Self(m.into_iter(), None)
    }
}

impl<'a, R: Repr> MapAccess<'a> for MapVisitor<R> {
    type Error = SerdeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'a>,
    {
        match self.0.next() {
            Some((k, v)) => {
                self.1 = Some(v);
                seed.deserialize(k).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'a>,
    {
        match self.1.take() {
            Some(v) => seed.deserialize(v),
            None => unreachable!("visit_value called before visit_key"),
        }
    }
}

struct EnumVisitor<R: Repr>(Node<R>, Option<Node<R>>);

impl<'a, R: Repr> EnumAccess<'a> for EnumVisitor<R> {
    type Error = SerdeError;
    type Variant = VariantVisitor<R>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'a>,
    {
        let visitor = VariantVisitor(self.1);
        seed.deserialize(self.0).map(|v| (v, visitor))
    }
}

struct VariantVisitor<R: Repr>(Option<Node<R>>);

impl<'a, R: Repr> VariantAccess<'a> for VariantVisitor<R> {
    type Error = SerdeError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.0 {
            Some(v) => Deserialize::deserialize(v),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'a>,
    {
        match self.0 {
            Some(v) => seed.deserialize(v),
            None => Err(Error::invalid_type(
                Unexpected::UnitVariant,
                &"new type variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        match self.0 {
            Some(node) => match node.yaml() {
                Yaml::Seq(v) => visitor.visit_seq(SeqVisitor::from(v.clone())),
                _ => Err(unexpected(&node, "tuple variant")),
            },
            None => Err(Error::invalid_type(
                Unexpected::TupleVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        match self.0 {
            Some(node) => match node.yaml() {
                Yaml::Map(m) => visitor.visit_map(MapVisitor::from(m.clone())),
                _ => Err(unexpected(&node, "struct variant")),
            },
            None => Err(Error::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}

impl<'a, R: Repr> Deserialize<'a> for Node<R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        deserializer.deserialize_any(NodeVisitor(PhantomData))
    }
}

impl<'a, R: Repr> Deserializer<'a> for Node<R> {
    type Error = SerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        match self.yaml() {
            Yaml::Null => visitor.visit_unit(),
            Yaml::Bool(b) => visitor.visit_bool(*b),
            Yaml::Int(n) => visitor.visit_i64(to_i64(n).unwrap()),
            Yaml::Float(n) => visitor.visit_f64(to_f64(n).unwrap()),
            Yaml::Str(s) => visitor.visit_str(s),
            Yaml::Seq(v) => visitor.visit_seq(SeqVisitor::from(v.clone())),
            Yaml::Map(m) => visitor.visit_map(MapVisitor::from(m.clone())),
            Yaml::Alias(a) => Err(SerdeError::from(format!("anchor {a}")).pos(self.pos())),
        }
    }

    impl_deserializer! {
        fn deserialize_bool(Bool) => visit_bool(v => *v)
        fn deserialize_i8(Int) => visit_i8(n => to_i64(n).unwrap() as i8)
        fn deserialize_i16(Int) => visit_i16(n => to_i64(n).unwrap() as i16)
        fn deserialize_i32(Int) => visit_i32(n => to_i64(n).unwrap() as i32)
        fn deserialize_i64(Int) => visit_i64(n => to_i64(n).unwrap())
        fn deserialize_u8(Int) => visit_u8(n => to_i64(n).unwrap() as u8)
        fn deserialize_u16(Int) => visit_u16(n => to_i64(n).unwrap() as u16)
        fn deserialize_u32(Int) => visit_u32(n => to_i64(n).unwrap() as u32)
        fn deserialize_u64(Int) => visit_u64(n => to_i64(n).unwrap() as u64)
        fn deserialize_f32(Float) => visit_f32(n => to_f64(n).unwrap() as f32)
        fn deserialize_f64(Float) => visit_f64(n => to_f64(n).unwrap())
        fn deserialize_str(Str) => visit_str(s => s)
        fn deserialize_string(Str) => visit_str(s => s)
        fn deserialize_char(Str) => visit_str(s => s)
        fn deserialize_seq(Seq) => visit_seq(a => SeqVisitor::from(a.clone()))
        fn deserialize_map(Map) => visit_map(m => MapVisitor::from(m.clone()))
        fn deserialize_identifier(Str) => visit_str(s => s)
    }

    serde_if_integer128! {
        impl_deserializer! {
            fn deserialize_i128(Int) => visit_i128(n => to_i64(n).unwrap() as i128)
            fn deserialize_u128(Int) => visit_u128(n => to_i64(n).unwrap() as u128)
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        match self.yaml() {
            Yaml::Str(s) => visitor.visit_str(s),
            Yaml::Seq(v) => visitor.visit_seq(&mut SeqVisitor::from(v.clone())),
            _ => Err(unexpected(&self, visitor)),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        match self.yaml() {
            Yaml::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        if self.is_null() {
            visitor.visit_unit()
        } else {
            Err(unexpected(&self, visitor))
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        match self.yaml() {
            Yaml::Seq(v) => visitor.visit_seq(SeqVisitor::from(v.clone())),
            Yaml::Map(m) => visitor.visit_map(MapVisitor::from(m.clone())),
            _ => Err(unexpected(&self, visitor)),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        let (k, v) = match self.yaml() {
            Yaml::Map(m) => {
                if m.len() != 1 {
                    return Err(unexpected(&self, "map with single pair"));
                }
                if let Some((k, v)) = m.into_iter().next() {
                    (k.clone(), Some(v.clone()))
                } else {
                    unreachable!()
                }
            }
            Yaml::Str(_) => (self.clone(), None),
            _ => return Err(unexpected(&self, visitor)),
        };
        visitor.visit_enum(EnumVisitor(k, v))
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        visitor.visit_unit()
    }
}

#[cold]
fn unexpected<R: Repr>(node: &Node<R>, exp: impl Expected) -> SerdeError {
    let ty = match node.yaml() {
        Yaml::Null => Unexpected::Unit,
        Yaml::Bool(b) => Unexpected::Bool(*b),
        Yaml::Int(n) => Unexpected::Signed(to_i64(n).unwrap()),
        Yaml::Float(n) => Unexpected::Float(to_f64(n).unwrap()),
        Yaml::Str(s) => Unexpected::Str(s),
        Yaml::Seq(_) => Unexpected::Seq,
        Yaml::Map(_) => Unexpected::Map,
        Yaml::Alias(_) => Unexpected::Other("anchor"),
    };
    SerdeError::invalid_type(ty, &exp).pos(node.pos())
}
