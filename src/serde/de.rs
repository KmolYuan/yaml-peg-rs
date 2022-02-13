use super::SerdeError;
use crate::{
    parse,
    repr::{RcRepr, Repr},
    Map, NodeBase, Seq, YamlBase,
};
use alloc::{string::ToString, vec::Vec};
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
                YamlBase::$ty($n) => visitor.$visit($value),
                _ => Err(self.unexpected(visitor)),
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
    let (nodes, _) = parse::<RcRepr>(doc)?;
    nodes.into_iter().map(D::deserialize).collect()
}

struct NodeVisitor<R: Repr>(PhantomData<R>);

impl<'a, R: Repr> Visitor<'a> for NodeVisitor<R> {
    type Value = NodeBase<R>;

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
        if m.len() == 1 {
            if let Some(n) = m.get(&NodeBase::from("anchor")) {
                if let Ok(anchor) = n.as_str() {
                    return Ok(NodeBase::from(YamlBase::Anchor(anchor.to_string())));
                }
            }
        }
        Ok(m.into_iter().collect())
    }
}

struct SeqVisitor<R: Repr>(<Seq<R> as IntoIterator>::IntoIter);

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

struct MapVisitor<R: Repr>(<Map<R> as IntoIterator>::IntoIter, Option<NodeBase<R>>);

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
            None => panic!("visit_value called before visit_key"),
        }
    }
}

struct EnumVisitor<R: Repr>(NodeBase<R>, Option<NodeBase<R>>);

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

struct VariantVisitor<R: Repr>(Option<NodeBase<R>>);

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
                YamlBase::Seq(a) => visitor.visit_seq(SeqVisitor(a.clone().into_iter())),
                _ => Err(node.unexpected("tuple variant")),
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
                YamlBase::Map(m) => visitor.visit_map(MapVisitor(m.clone().into_iter(), None)),
                _ => Err(node.unexpected("struct variant")),
            },
            None => Err(Error::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}

impl<'a, R: Repr> Deserialize<'a> for NodeBase<R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        deserializer.deserialize_any(NodeVisitor(PhantomData))
    }
}

impl<'a, R: Repr> Deserializer<'a> for NodeBase<R> {
    type Error = SerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        match self.yaml() {
            YamlBase::Null => visitor.visit_unit(),
            YamlBase::Bool(v) => visitor.visit_bool(*v),
            YamlBase::Int(n) => visitor.visit_i64(n.parse().unwrap()),
            YamlBase::Float(n) => visitor.visit_f64(n.parse().unwrap()),
            YamlBase::Str(s) => visitor.visit_str(s),
            YamlBase::Seq(a) => visitor.visit_seq(SeqVisitor(a.clone().into_iter())),
            YamlBase::Map(m) => visitor.visit_map(MapVisitor(m.clone().into_iter(), None)),
            YamlBase::Anchor(s) => {
                let mut m = Map::<R>::new();
                m.insert(NodeBase::from("anchor"), NodeBase::from(s));
                visitor.visit_map(MapVisitor(m.into_iter(), None))
            }
        }
    }

    impl_deserializer! {
        fn deserialize_bool(Bool) => visit_bool(v => *v)
        fn deserialize_i8(Int) => visit_i8(n => n.parse().unwrap())
        fn deserialize_i16(Int) => visit_i16(n => n.parse().unwrap())
        fn deserialize_i32(Int) => visit_i32(n => n.parse().unwrap())
        fn deserialize_i64(Int) => visit_i64(n => n.parse().unwrap())
        fn deserialize_u8(Int) => visit_u8(n => n.parse().unwrap())
        fn deserialize_u16(Int) => visit_u16(n => n.parse().unwrap())
        fn deserialize_u32(Int) => visit_u32(n => n.parse().unwrap())
        fn deserialize_u64(Int) => visit_u64(n => n.parse().unwrap())
        fn deserialize_f32(Float) => visit_f32(n => n.parse().unwrap())
        fn deserialize_f64(Float) => visit_f64(n => n.parse().unwrap())
        fn deserialize_str(Str) => visit_str(s => s)
        fn deserialize_string(Str) => visit_str(s => s)
        fn deserialize_char(Str) => visit_str(s => s)
        fn deserialize_seq(Seq) => visit_seq(a => SeqVisitor(a.clone().into_iter()))
        fn deserialize_map(Map) => visit_map(m => MapVisitor(m.clone().into_iter(), None))
        fn deserialize_identifier(Str) => visit_str(s => s)
    }

    serde_if_integer128! {
        impl_deserializer! {
            fn deserialize_i128(Int) => visit_i128(n => n.parse().unwrap())
            fn deserialize_u128(Int) => visit_u128(n => n.parse().unwrap())
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
            YamlBase::Str(s) => visitor.visit_str(s),
            YamlBase::Seq(a) => visitor.visit_seq(&mut SeqVisitor(a.clone().into_iter())),
            _ => Err(self.unexpected(visitor)),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        match self.yaml() {
            YamlBase::Null => visitor.visit_none(),
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
            Err(self.unexpected(visitor))
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
            YamlBase::Seq(a) => visitor.visit_seq(SeqVisitor(a.clone().into_iter())),
            YamlBase::Map(m) => visitor.visit_map(MapVisitor(m.clone().into_iter(), None)),
            _ => Err(self.unexpected(visitor)),
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
            YamlBase::Map(m) => {
                if m.len() != 1 {
                    return Err(self.unexpected("map with single pair"));
                }
                if let Some((k, v)) = m.into_iter().next() {
                    (k.clone(), Some(v.clone()))
                } else {
                    unreachable!()
                }
            }
            YamlBase::Str(_) => (self.clone(), None),
            _ => return Err(self.unexpected(visitor)),
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

impl<R: Repr> NodeBase<R> {
    #[cold]
    fn unexpected(&self, exp: impl Expected) -> SerdeError {
        let ty = match self.yaml() {
            YamlBase::Null => Unexpected::Unit,
            YamlBase::Bool(b) => Unexpected::Bool(*b),
            YamlBase::Int(n) => Unexpected::Signed(n.parse().unwrap()),
            YamlBase::Float(n) => Unexpected::Float(n.parse().unwrap()),
            YamlBase::Str(s) => Unexpected::Str(s),
            YamlBase::Seq(_) => Unexpected::Seq,
            YamlBase::Map(_) => Unexpected::Map,
            YamlBase::Anchor(_) => Unexpected::Other("anchor"),
        };
        SerdeError::invalid_type(ty, &exp).pos(self.pos())
    }
}
