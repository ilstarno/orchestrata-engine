use serde::{
    Deserialize, Deserializer,
    de::{Error, MapAccess, Visitor},
};
use serde_json::value::RawValue;
use std::{collections::BTreeMap, fmt};
pub(super) struct StrictObject(pub BTreeMap<String, Box<RawValue>>);
impl<'de> Deserialize<'de> for StrictObject {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Object;
        impl<'de> Visitor<'de> for Object {
            type Value = StrictObject;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("bounded named object with unique keys")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut fields = BTreeMap::new();
                while let Some(key) = map.next_key::<String>()? {
                    if key.len() > 64 || fields.len() >= 16 || fields.contains_key(&key) {
                        return Err(M::Error::custom("invalid object keys"));
                    }
                    fields.insert(key, map.next_value()?);
                }
                Ok(StrictObject(fields))
            }
        }
        d.deserialize_map(Object)
    }
}
