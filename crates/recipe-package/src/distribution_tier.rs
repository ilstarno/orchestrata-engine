/// Self-declared distribution metadata, never an entitlement or authorization decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DistributionTier {
    Free,
    Paid,
}
