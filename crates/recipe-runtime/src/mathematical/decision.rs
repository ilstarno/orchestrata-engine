use super::{assignment_domain::AssignmentDomain, id_field::IdField};
#[derive(Debug)]
pub(super) struct Decision {
    pub name: String,
    pub items: String,
    pub resources: String,
    pub domain: AssignmentDomain,
    pub item_id: IdField,
    pub resource_id: IdField,
    pub nullable: bool,
}
