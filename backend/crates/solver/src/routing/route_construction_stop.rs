/// Why a heuristic pass ended. Neither state establishes global optimality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteConstructionStop {
    CompletedPass,
    InsertionLimit,
}
