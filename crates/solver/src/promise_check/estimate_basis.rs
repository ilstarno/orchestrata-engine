/// Estimates are caller inputs, not observed measurements of savings.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EstimateBasis {
    SuppliedEstimatesNotObservedSavings,
}
