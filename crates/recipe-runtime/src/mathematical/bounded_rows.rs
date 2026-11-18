use super::strict_object::StrictObject;
use serde::{
    Deserialize, Deserializer,
    de::{Error, SeqAccess, Visitor},
};
use std::fmt;
pub(super) struct BoundedRows(pub Vec<StrictObject>);
impl<'de> Deserialize<'de> for BoundedRows {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Rows;
        impl<'de> Visitor<'de> for Rows {
            type Value = BoundedRows;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("at most 128 named records")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut rows = Vec::new();
                while let Some(row) = seq.next_element::<StrictObject>()? {
                    if rows.len() == 128 {
                        return Err(A::Error::custom("record limit exceeded"));
                    }
                    rows.push(row);
                }
                Ok(BoundedRows(rows))
            }
        }
        d.deserialize_seq(Rows)
    }
}
