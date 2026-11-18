use super::{boolean_field::BooleanField, unsigned_field::UnsignedField};
#[derive(Debug)]
pub(super) enum Constraint {
    Capacity {
        field: UnsignedField,
        limit: Option<u32>,
    },
    Availability {
        field: BooleanField,
        value: bool,
    },
}
