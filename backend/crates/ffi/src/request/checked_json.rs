use serde::{Deserialize, Deserializer};
use serde_json::Value;

use super::json_visitor::JsonVisitor;

pub(super) struct CheckedJson {
    pub(super) value: Value,
    pub(super) duplicate: bool,
}

impl<'de> Deserialize<'de> for CheckedJson {
    fn deserialize<D: Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        decoder.deserialize_any(JsonVisitor)
    }
}
