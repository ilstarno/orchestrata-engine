#[derive(Debug, Clone)]
pub struct RouteSpec {
    pub(super) strategy: super::strategy::Strategy,
    pub(super) max_insertions: u32,
}
