#[derive(Debug)]
pub(super) enum Constraint {
    Capacity { field: String, limit: Option<u32> },
    Availability { field: String, value: bool },
}
