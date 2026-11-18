#[derive(Debug)]
pub(super) struct Decision {
    pub name: String,
    pub items: String,
    pub resources: String,
    pub domain: String,
    pub item_id: String,
    pub resource_id: String,
    pub edge_item: String,
    pub edge_resource: String,
    pub nullable: bool,
}
