use std::fmt;

use serde::de::{Error, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};

use super::checked_json::CheckedJson;

pub(super) struct JsonVisitor;

fn scalar(value: Value) -> CheckedJson {
    CheckedJson {
        value,
        duplicate: false,
    }
}

impl<'de> Visitor<'de> for JsonVisitor {
    type Value = CheckedJson;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_bool<E: Error>(self, value: bool) -> Result<Self::Value, E> {
        Ok(scalar(Value::Bool(value)))
    }

    fn visit_i64<E: Error>(self, value: i64) -> Result<Self::Value, E> {
        Ok(scalar(Value::Number(Number::from(value))))
    }

    fn visit_u64<E: Error>(self, value: u64) -> Result<Self::Value, E> {
        Ok(scalar(Value::Number(Number::from(value))))
    }

    fn visit_f64<E: Error>(self, value: f64) -> Result<Self::Value, E> {
        Number::from_f64(value)
            .map(|number| scalar(Value::Number(number)))
            .ok_or_else(|| E::custom("nonfinite JSON number"))
    }

    fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
        self.visit_string(value.to_owned())
    }

    fn visit_string<E: Error>(self, value: String) -> Result<Self::Value, E> {
        Ok(scalar(Value::String(value)))
    }

    fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
        Ok(scalar(Value::Null))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
        let mut items = Vec::new();
        let mut duplicate = false;
        while let Some(CheckedJson {
            value,
            duplicate: nested,
        }) = access.next_element()?
        {
            duplicate |= nested;
            items.push(value);
        }
        Ok(CheckedJson {
            value: Value::Array(items),
            duplicate,
        })
    }

    fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
        let mut members = Map::new();
        let mut duplicate = false;
        while let Some((key, item)) = access.next_entry::<String, CheckedJson>()? {
            duplicate |= item.duplicate | members.insert(key, item.value).is_some();
        }
        Ok(CheckedJson {
            value: Value::Object(members),
            duplicate,
        })
    }
}
