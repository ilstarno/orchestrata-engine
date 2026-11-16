/// Facts that can be established from the two verified plans and their models.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentChangeReason {
    /// The previous value is absent from this variable's updated eligible domain.
    PreviousValueOutsideUpdatedDomain,
    /// The assignment changed, but the comparison did not establish why.
    ReasonNotEstablished,
}
