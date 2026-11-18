use super::field_type::FieldType;
use std::collections::BTreeMap;
#[derive(Debug)]
pub(super) struct InputBinding {
    pub field: String,
    pub fields: BTreeMap<String, FieldType>,
}
