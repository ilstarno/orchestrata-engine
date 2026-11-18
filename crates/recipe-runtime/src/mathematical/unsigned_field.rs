use super::{
    ModelError, field_binding::FieldBinding, field_type::FieldType, input_binding::InputBinding,
    record_value::RecordValue, validated_record::ValidatedRecord,
};
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct UnsignedField(FieldBinding);
impl UnsignedField {
    pub fn bind(
        inputs: &BTreeMap<String, InputBinding>,
        table: &str,
        name: String,
    ) -> Result<Self, ModelError> {
        Ok(Self(FieldBinding::bind(
            inputs,
            table,
            name,
            FieldType::U32,
        )?))
    }
    pub fn read(&self, row: &ValidatedRecord) -> Result<u32, ModelError> {
        match self.0.read(row)? {
            RecordValue::Unsigned(value) => Ok(*value),
            _ => Err(ModelError::InvalidInput),
        }
    }
}
