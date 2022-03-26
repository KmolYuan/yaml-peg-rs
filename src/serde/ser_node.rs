use crate::{repr::Repr, Node, Yaml};
use serde::{
    ser::{Error as _, SerializeMap as _},
    Serialize, Serializer,
};

impl<R: Repr> Serialize for Node<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.yaml() {
            Yaml::Null => serializer.serialize_unit(),
            Yaml::Bool(b) => serializer.serialize_bool(*b),
            Yaml::Int(n) => serializer.serialize_i64(n.parse().unwrap()),
            Yaml::Float(n) => serializer.serialize_f64(n.parse().unwrap()),
            Yaml::Str(s) => serializer.serialize_str(s),
            Yaml::Seq(a) => a.serialize(serializer),
            Yaml::Map(m) => {
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            Yaml::Alias(a) => Err(S::Error::custom(format!("anchor {}", a))),
        }
    }
}
