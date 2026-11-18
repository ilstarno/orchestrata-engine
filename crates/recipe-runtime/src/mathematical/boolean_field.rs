use super::{
    ModelError, field_binding::FieldBinding, field_type::FieldType, input_binding::InputBinding,
    record_value::RecordValue, validated_record::ValidatedRecord,
};
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BooleanField(FieldBinding);
impl BooleanField {
    pub fn bind(
        inputs: &BTreeMap<String, InputBinding>,
        table: &str,
        name: String,
    ) -> Result<Self, ModelError> {
        Ok(Self(FieldBinding::bind(
            inputs,
            table,
            name,
            FieldType::Bool,
        )?))
    }
    pub fn read(&self, row: &ValidatedRecord) -> Result<bool, ModelError> {
        match self.0.read(row)? {
            RecordValue::Boolean(value) => Ok(*value),
            _ => Err(ModelError::InvalidInput),
        }
    }
}
