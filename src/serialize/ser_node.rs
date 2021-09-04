use crate::{repr::Repr, NodeBase, YamlBase};
use serde::{ser::SerializeMap, Serialize, Serializer};

impl<R: Repr> Serialize for NodeBase<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.yaml() {
            YamlBase::Null => serializer.serialize_unit(),
            YamlBase::Bool(b) => serializer.serialize_bool(*b),
            YamlBase::Int(n) => serializer.serialize_i64(n.parse().unwrap()),
            YamlBase::Float(n) => serializer.serialize_f64(n.parse().unwrap()),
            YamlBase::Str(s) => serializer.serialize_str(s),
            YamlBase::Array(a) => a.serialize(serializer),
            YamlBase::Map(m) => {
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            YamlBase::Anchor(s) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("anchor", s)?;
                map.end()
            }
        }
    }
}
