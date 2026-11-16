/// Invalid scope or unsafe load bounds, rather than a proof of infeasibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceCapacityError {
    EmptyItems,
    DuplicateItem(usize),
    DuplicateLimit(i64),
    DuplicateDomainValue { variable: usize, resource: i64 },
    MissingLimit(i64),
    UnusedLimit(i64),
    PotentialLoadOverflow(i64),
}
