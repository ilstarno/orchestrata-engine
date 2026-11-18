use super::{
    ModelError, field_type::FieldType, input_binding::InputBinding, record_value::RecordValue,
    validated_record::ValidatedRecord,
};
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FieldBinding {
    table: String,
    field: String,
}
impl FieldBinding {
    pub fn bind(
        inputs: &BTreeMap<String, InputBinding>,
        table: &str,
        field: String,
        expected: FieldType,
    ) -> Result<Self, ModelError> {
        if inputs.get(table).and_then(|b| b.fields.get(&field)) != Some(&expected) {
            return Err(ModelError::UnsupportedSemantics);
        }
        Ok(Self {
            table: table.to_owned(),
            field,
        })
    }
    pub fn read<'a>(&self, row: &'a ValidatedRecord) -> Result<&'a RecordValue, ModelError> {
        row.read(&self.table, &self.field)
    }
}
