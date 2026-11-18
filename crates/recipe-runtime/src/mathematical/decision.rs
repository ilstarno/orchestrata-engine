use super::id_field::IdField;
#[derive(Debug)]
pub(super) struct Decision {
    pub name: String,
    pub items: String,
    pub resources: String,
    pub domain: String,
    pub item_id: IdField,
    pub resource_id: IdField,
    pub edge_item: IdField,
    pub edge_resource: IdField,
    pub nullable: bool,
}
