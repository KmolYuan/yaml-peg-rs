use super::SerdeError;
use crate::{repr::Repr, Array, Map, NodeBase, YamlBase};
use alloc::{format, string::String};
use core::marker::PhantomData;
use serde::{
    de::{DeserializeSeed, Error, MapAccess, SeqAccess, Unexpected, Visitor},
    serde_if_integer128, Deserialize, Deserializer,
};

macro_rules! impl_visitor {
    (fn $method:ident) => {
        fn $method<E>(self) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(().into())
        }
    };
    (fn $method:ident($ty:ty)) => {
        fn $method<E>(self, v: $ty) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(v.into())
        }
    };
    (fn $method1:ident$(($ty1:ty))? $(fn $method2:ident$(($ty2:ty))?)+) => {
        impl_visitor! { fn $method1$(($ty1))? }
        $(impl_visitor! { fn $method2$(($ty2))? })+
    };
}

macro_rules! impl_deserializer {
    (fn $method:ident($ty:ident) => $visit:ident($n:ident => $value:expr)) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'a>,
        {
            match self.yaml() {
                YamlBase::$ty($n) => visitor.$visit($value),
                _ => Err(Error::invalid_type(self.unexpected(), &visitor)),
            }
        }
    };
    (fn $method1:ident($ty1:ident) => $visit1:ident($n1:ident => $value1:expr)
    $(fn $method2:ident($ty2:ident) => $visit2:ident($n2:ident => $value2:expr))+) => {
        impl_deserializer! { fn $method1($ty1) => $visit1($n1 => $value1) }
        $(impl_deserializer! { fn $method2($ty2) => $visit2($n2 => $value2) })+
    }
}

struct NodeVisitor<R: Repr>(PhantomData<R>);

impl<'a, R: Repr> Visitor<'a> for NodeVisitor<R> {
    type Value = NodeBase<R>;

    fn expecting(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.write_str("any YAML value")
    }

    impl_visitor! {
        fn visit_bool(bool)
        fn visit_i64(i64)
        fn visit_u64(u64)
        fn visit_f64(f64)
        fn visit_str(&str)
        fn visit_string(String)
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
        let mut a = Array::new();
        while let Some(e) = seq.next_element()? {
            a.push(e);
        }
        Ok(a.into_iter().collect())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'a>,
    {
        let mut m = Map::new();
        while let Some((k, v)) = map.next_entry()? {
            m.insert(k, v);
        }
        Ok(m.into_iter().collect())
    }
}

struct SeqVisitor<R: Repr>(<Array<R> as IntoIterator>::IntoIter);

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
        match self.into_yaml() {
            YamlBase::Null => visitor.visit_unit(),
            YamlBase::Bool(v) => visitor.visit_bool(v),
            YamlBase::Int(n) => visitor.visit_i64(n.parse().unwrap()),
            YamlBase::Float(n) => visitor.visit_f64(n.parse().unwrap()),
            YamlBase::Str(s) => visitor.visit_string(s),
            YamlBase::Array(a) => visitor.visit_seq(&mut SeqVisitor(a.into_iter())),
            YamlBase::Map(m) => visitor.visit_map(&mut MapVisitor(m.into_iter(), None)),
            YamlBase::Anchor(s) => visitor.visit_string(format!("*{}", s)),
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
        fn deserialize_string(Str) => visit_string(s => s.clone())
    }

    serde_if_integer128! {
        impl_deserializer! {
            fn deserialize_i128(Int) => visit_i128(n => n.parse().unwrap())
            fn deserialize_u128(Int) => visit_u128(n => n.parse().unwrap())
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        todo!()
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        todo!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        todo!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        if self.is_null() {
            visitor.visit_unit()
        } else {
            Err(Error::invalid_type(self.unexpected(), &visitor))
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        todo!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        todo!()
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        todo!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        todo!()
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        todo!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        todo!()
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        todo!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'a>,
    {
        todo!()
    }
}

impl<R: Repr> NodeBase<R> {
    #[cold]
    fn unexpected(&self) -> Unexpected {
        match self.yaml() {
            YamlBase::Null => Unexpected::Unit,
            YamlBase::Bool(b) => Unexpected::Bool(*b),
            YamlBase::Int(n) => Unexpected::Signed(n.parse().unwrap()),
            YamlBase::Float(n) => Unexpected::Float(n.parse().unwrap()),
            YamlBase::Str(s) => Unexpected::Str(s),
            YamlBase::Array(_) => Unexpected::Seq,
            YamlBase::Map(_) => Unexpected::Map,
            YamlBase::Anchor(_) => Unexpected::Other("anchor"),
        }
    }
}
