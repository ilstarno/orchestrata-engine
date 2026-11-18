use super::{
    ModelError, boolean_field::BooleanField, id_field::IdField, unsigned_field::UnsignedField,
    validated_record::ValidatedRecord,
};
#[derive(Debug)]
pub(super) enum RowEquality {
    Id(IdField, IdField),
    Unsigned(UnsignedField, UnsignedField),
    Boolean(BooleanField, BooleanField),
}
impl RowEquality {
    pub fn matches(
        &self,
        item: &ValidatedRecord,
        resource: &ValidatedRecord,
    ) -> Result<bool, ModelError> {
        Ok(match self {
            Self::Id(a, b) => a.read(item)? == b.read(resource)?,
            Self::Unsigned(a, b) => a.read(item)? == b.read(resource)?,
            Self::Boolean(a, b) => a.read(item)? == b.read(resource)?,
        })
    }
}
