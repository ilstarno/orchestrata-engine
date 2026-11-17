use super::VisitKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteVisit {
    /// Index in the problem's shipment list.
    pub shipment: usize,
    pub kind: VisitKind,
    pub arrival: u64,
}
