/// Independent certificate status, not solver termination or infeasibility.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum VerificationOutcome {
    WorkLimit,
    Verified { work: u64 },
}
