use crate::{repr::Repr, NodeBase, YamlBase};
use alloc::format;
use serde::{ser::SerializeMap, Serialize, Serializer};

impl<R: Repr> Serialize for NodeBase<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.yaml() {
            YamlBase::Null => serializer.serialize_unit(),
            YamlBase::Bool(b) => b.serialize(serializer),
            YamlBase::Int(n) => serializer.serialize_i64(n.parse().unwrap()),
            YamlBase::Float(n) => serializer.serialize_f64(n.parse().unwrap()),
            YamlBase::Str(s) => s.serialize(serializer),
            YamlBase::Array(a) => a.serialize(serializer),
            YamlBase::Map(m) => {
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            YamlBase::Anchor(s) => format!("*{}", s).serialize(serializer),
        }
    }
}
