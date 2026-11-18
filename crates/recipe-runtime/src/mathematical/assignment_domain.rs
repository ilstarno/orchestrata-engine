use super::{ModelError, derived_domain::DerivedDomain, id_field::IdField};
#[derive(Debug)]
pub(super) enum AssignmentDomain {
    Supplied {
        table: String,
        item: IdField,
        resource: IdField,
    },
    Derived(DerivedDomain),
}
impl AssignmentDomain {
    pub fn supplied_table(&self) -> Result<&str, ModelError> {
        match self {
            Self::Supplied { table, .. } => Ok(table),
            Self::Derived(_) => Err(ModelError::UnsupportedSemantics),
        }
    }
}
