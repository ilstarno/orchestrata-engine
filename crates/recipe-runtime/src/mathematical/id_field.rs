use super::{
    ModelError, field_binding::FieldBinding, field_type::FieldType, input_binding::InputBinding,
    record_value::RecordValue, validated_record::ValidatedRecord,
};
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct IdField(FieldBinding);
impl IdField {
    pub fn bind(
        inputs: &BTreeMap<String, InputBinding>,
        table: &str,
        name: String,
    ) -> Result<Self, ModelError> {
        Ok(Self(FieldBinding::bind(
            inputs,
            table,
            name,
            FieldType::Id,
        )?))
    }
    pub fn read<'a>(
        &self,
        row: &'a ValidatedRecord,
    ) -> Result<&'a super::record_id::RecordId, ModelError> {
        match self.0.read(row)? {
            RecordValue::Id(value) => Ok(value),
            _ => Err(ModelError::InvalidInput),
        }
    }
}
