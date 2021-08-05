use crate::{repr::Repr, Array, Map, NodeBase};
use alloc::string::String;
use core::marker::PhantomData;
use serde::{
    de::{Error, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
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

impl<'a, R: Repr> Deserialize<'a> for NodeBase<R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        deserializer.deserialize_any(NodeVisitor(PhantomData))
    }
}
