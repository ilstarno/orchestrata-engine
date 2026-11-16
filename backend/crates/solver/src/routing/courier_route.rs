use super::RouteVisit;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourierRoute {
    /// Index in the problem's courier list.
    pub courier: usize,
    pub departure: u64,
    pub visits: Vec<RouteVisit>,
    pub depot_return: u64,
}
