#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ok = 0,
    InvalidProblem = 1,
    SearchLimit = 2,
    InternalError = 3,
    InvalidArgument = 4,
}
