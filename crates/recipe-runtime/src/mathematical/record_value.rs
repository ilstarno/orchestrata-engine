use super::{ModelError, field_type::FieldType, record_id::RecordId};
#[derive(Debug)]
pub(super) enum RecordValue {
    Id(RecordId),
    Unsigned(u32),
    Boolean(bool),
}
impl RecordValue {
    pub fn parse(raw: &str, ty: FieldType) -> Result<Self, ModelError> {
        match ty {
            FieldType::Id => Ok(Self::Id(RecordId::parse(
                serde_json::from_str(raw).map_err(|_| ModelError::InvalidInput)?,
            )?)),
            FieldType::U32 => {
                let value: u32 = serde_json::from_str(raw).map_err(|_| ModelError::InvalidInput)?;
                if value > 1_000_000 {
                    return Err(ModelError::InvalidInput);
                }
                Ok(Self::Unsigned(value))
            }
            FieldType::Bool => Ok(Self::Boolean(
                serde_json::from_str(raw).map_err(|_| ModelError::InvalidInput)?,
            )),
        }
    }
}
