/// Additional return codes for the strict JSON entry point.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrictStatus {
    MalformedJson = 5,
    UnknownField = 6,
    DuplicateField = 7,
    InvalidShape = 8,
}
