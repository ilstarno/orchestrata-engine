use crate::{ContentIdentity, ExecutionIdentity, InputIdentity, RequestIdentity};
/// Separate opt-in Rust result; existing solve/certificate results are unchanged.
#[derive(Debug)]
pub struct RoutePackageTravelCoverage {
    pub content_identity: ContentIdentity,
    pub execution_identity: ExecutionIdentity,
    pub input_identity: InputIdentity,
    pub request_identity: RequestIdentity,
    pub coverage: orchestrata_recipe_runtime::route_dispatch::TravelCoverageOutcome,
}
