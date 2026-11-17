/// Model validation failures specific to a `count_value` rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CountValueError {
    EmptyScope,
    InvalidBounds,
    InvalidWeight,
    UnknownVariable(usize),
    DuplicateVariable(usize),
}
