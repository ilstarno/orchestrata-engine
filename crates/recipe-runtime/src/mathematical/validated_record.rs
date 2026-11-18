use super::{
    ModelError, input_binding::InputBinding, record_value::RecordValue, strict_object::StrictObject,
};
use std::collections::BTreeMap;
#[derive(Debug)]
pub(super) struct ValidatedRecord {
    binding: String,
    values: BTreeMap<String, RecordValue>,
}
impl ValidatedRecord {
    pub fn parse(
        name: &str,
        binding: &InputBinding,
        StrictObject(row): StrictObject<'_>,
    ) -> Result<Self, ModelError> {
        if row.len() != binding.fields.len() {
            return Err(ModelError::InvalidInput);
        }
        let mut values = BTreeMap::new();
        for (field, ty) in &binding.fields {
            let raw = row.get(field).ok_or(ModelError::InvalidInput)?;
            values.insert(field.clone(), RecordValue::parse(raw.get(), *ty)?);
        }
        Ok(Self {
            binding: name.to_owned(),
            values,
        })
    }
    pub fn read(&self, binding: &str, field: &str) -> Result<&RecordValue, ModelError> {
        if self.binding != binding {
            return Err(ModelError::InvalidInput);
        }
        self.values.get(field).ok_or(ModelError::InvalidInput)
    }
}
