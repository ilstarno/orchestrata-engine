use super::{ModelError, xml_helpers::identifier};
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct RecordId(String);
impl RecordId {
    pub fn parse(value: String) -> Result<Self, ModelError> {
        if !identifier(&value) {
            return Err(ModelError::InvalidInput);
        }
        Ok(Self(value))
    }
    pub fn into_string(self) -> String {
        self.0
    }
}
