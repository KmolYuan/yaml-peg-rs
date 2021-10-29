use super::SerdeError;
use crate::{dump, node, repr::Repr, ArcNode, Array, Map, Node, NodeBase, YamlBase};
use alloc::string::{String, ToString};
use core::marker::PhantomData;
use serde::{
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    serde_if_integer128, Serialize, Serializer,
};

macro_rules! impl_serializer {
    (@) => { () };
    (@$ty:ty, $name:ident) => { $name };
    ($(fn $method:ident$(($ty:ty))?)+) => {
        $(fn $method(self$(, v: $ty)?) -> Result<Self::Ok, Self::Error> {
            Ok(impl_serializer!(@$($ty, v)?).into())
        })+
    };
}

macro_rules! impl_end {
    (@ $self:ident) => {
        $self.0.into()
    };
    (@map $self:ident) => {
        node!(@{$self.1 => $self.0})
    };
}

macro_rules! impl_seq_serializer {
    ($(impl $trait:ident for $ty:ident => $method:ident $(($tt:tt))?)+) => {
        $(impl<R: Repr> $trait for $ty<R> {
            type Ok = NodeBase<R>;
            type Error = SerdeError;

            fn $method<T>(&mut self, value: &T) -> Result<(), Self::Error>
            where
                T: Serialize + ?Sized,
            {
                self.0.push(value.serialize(NodeSerializer(PhantomData))?);
                Ok(())
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                Ok(impl_end!(@$($tt)? self))
            }
        })+
    };
}

macro_rules! impl_map_serializer {
    ($(impl $trait:ident for $ty:ident $(($tt:tt))?)+) => {
        $(impl<R: Repr> $trait for $ty<R> {
            type Ok = NodeBase<R>;
            type Error = SerdeError;

            fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
            where
                T: Serialize + ?Sized,
            {
                self.0.insert(
                    key.serialize(NodeSerializer(PhantomData))?,
                    value.serialize(NodeSerializer(PhantomData))?,
                );
                Ok(())
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                if self.0.len() == 1 {
                    if let Some(n) = self.0.get(&NodeBase::from("anchor")) {
                        if let Ok(anchor) = n.as_str() {
                            return Ok(NodeBase::from(YamlBase::Anchor(anchor.to_string())));
                        }
                    }
                }
                Ok(impl_end!(@$($tt)? self))
            }
        })+
    };
}

/// Serialize data into [`Node`].
///
/// If a serializable data is provide,
/// it should be able to transform into YAML format.
///
/// ```
/// use serde::Serialize;
/// use yaml_peg::{serialize::to_node, node};
///
/// #[derive(Serialize)]
/// struct Member<'a> {
///     name: &'a str,
///     married: bool,
///     age: u8,
/// }
///
/// let officer = Member { name: "Bob", married: true, age: 46 };
/// let officer_yaml = node!({
///     "name" => "Bob",
///     "married" => true,
///     "age" => 46,
/// });
/// assert_eq!(officer_yaml, to_node(officer).unwrap());
/// ```
///
/// There is another version for multi-thread reference counter: [`to_arc_node`].
pub fn to_node(any: impl Serialize) -> Result<Node, SerdeError> {
    any.serialize(NodeSerializer(PhantomData))
}

/// Serialize data into [`ArcNode`].
///
/// ```
/// use serde::Serialize;
/// use yaml_peg::{serialize::to_arc_node, node};
///
/// #[derive(Serialize)]
/// struct Member<'a> {
///     name: &'a str,
///     married: bool,
///     age: u8,
/// }
///
/// let officer = Member { name: "Bob", married: true, age: 46 };
/// let officer_yaml = node!(arc{
///     "name" => "Bob",
///     "married" => true,
///     "age" => 46,
/// });
/// assert_eq!(officer_yaml, to_arc_node(officer).unwrap());
/// ```
///
/// There is another version for single-thread reference counter: [`to_node`].
pub fn to_arc_node(any: impl Serialize) -> Result<ArcNode, SerdeError> {
    any.serialize(NodeSerializer(PhantomData))
}

/// Serialize data into [`Node`] then dump into string.
///
/// ```
/// use serde::Serialize;
/// use yaml_peg::serialize::to_string;
///
/// #[derive(Serialize)]
/// struct Member<'a> {
///     name: &'a str,
///     married: bool,
///     age: u8,
/// }
///
/// let officer = Member {
///     name: "Bob",
///     married: true,
///     age: 46,
/// };
/// let officer_doc = "\
/// name: Bob
/// married: true
/// age: 46
/// ";
/// assert_eq!(officer_doc, to_string(officer).unwrap());
/// ```
pub fn to_string(any: impl Serialize) -> Result<String, SerdeError> {
    Ok(dump(&[to_node(any)?]))
}

struct NodeSerializer<R: Repr>(PhantomData<R>);

impl<R: Repr> Serializer for NodeSerializer<R> {
    type Ok = NodeBase<R>;
    type Error = SerdeError;
    type SerializeSeq = SeqSerializer<R>;
    type SerializeTuple = SeqSerializer<R>;
    type SerializeTupleStruct = SeqSerializer<R>;
    type SerializeTupleVariant = TupleVariant<R>;
    type SerializeMap = MapSerializer<R>;
    type SerializeStruct = StructSerializer<R>;
    type SerializeStructVariant = StructVariant<R>;

    impl_serializer! {
        fn serialize_bool(bool)
        fn serialize_i8(i8)
        fn serialize_i16(i16)
        fn serialize_i32(i32)
        fn serialize_i64(i64)
        fn serialize_u8(u8)
        fn serialize_u16(u16)
        fn serialize_u32(u32)
        fn serialize_u64(u64)
        fn serialize_f32(f32)
        fn serialize_f64(f64)
        fn serialize_char(char)
        fn serialize_str(&str)
        fn serialize_none
        fn serialize_unit
    }

    serde_if_integer128! {
        impl_serializer! {
            fn serialize_i128(i128)
            fn serialize_u128(u128)
        }
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(v.iter().map(|b| NodeBase::from(*b)).collect())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(().into())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(variant.into())
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Ok(node!(@{variant => value.serialize(NodeSerializer(PhantomData))?}))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let array = match len {
            Some(n) => Array::with_capacity(n),
            None => Array::new(),
        };
        Ok(SeqSerializer(array))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(TupleVariant(Array::with_capacity(len), variant))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(MapSerializer(
            match len {
                Some(n) => Map::with_capacity(n),
                None => Map::new(),
            },
            None,
        ))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(StructSerializer(Map::with_capacity(len)))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(StructVariant(Map::with_capacity(len), variant))
    }

    #[cfg(not(feature = "serde-std"))]
    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: core::fmt::Display + ?Sized,
    {
        self.serialize_str(&value.to_string())
    }
}

struct SeqSerializer<R: Repr>(Array<R>);
struct TupleVariant<R: Repr>(Array<R>, &'static str);
struct MapSerializer<R: Repr>(Map<R>, Option<NodeBase<R>>);
struct StructSerializer<R: Repr>(Map<R>);
struct StructVariant<R: Repr>(Map<R>, &'static str);

impl_seq_serializer! {
    impl SerializeSeq for SeqSerializer => serialize_element
    impl SerializeTuple for SeqSerializer => serialize_element
    impl SerializeTupleStruct for SeqSerializer => serialize_field
    impl SerializeTupleVariant for TupleVariant => serialize_field (map)
}

impl_map_serializer! {
    impl SerializeStruct for StructSerializer
    impl SerializeStructVariant for StructVariant (map)
}

impl<R: Repr> SerializeMap for MapSerializer<R> {
    type Ok = NodeBase<R>;
    type Error = SerdeError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.1 = Some(key.serialize(NodeSerializer(PhantomData))?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        match self.1.take() {
            Some(k) => self
                .0
                .insert(k, value.serialize(NodeSerializer(PhantomData))?),
            None => panic!("serialize_value called before serialize_key"),
        };
        Ok(())
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: Serialize + ?Sized,
        V: Serialize + ?Sized,
    {
        self.0.insert(
            key.serialize(NodeSerializer(PhantomData))?,
            value.serialize(NodeSerializer(PhantomData))?,
        );
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.0.len() == 1 {
            if let Some(n) = self.0.get(&NodeBase::from("anchor")) {
                if let Ok(anchor) = n.as_str() {
                    return Ok(NodeBase::from(YamlBase::Anchor(anchor.to_string())));
                }
            }
        }
        Ok(self.0.into())
    }
}
