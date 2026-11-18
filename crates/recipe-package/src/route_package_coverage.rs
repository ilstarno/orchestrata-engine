use crate::{ContentIdentity, ExecutionIdentity, InputIdentity, RequestIdentity};
/// Opt-in Rust result; not an additional field in existing solve output.
#[derive(Debug)]
pub struct RoutePackageCoverage {
    pub content_identity: ContentIdentity,
    pub execution_identity: ExecutionIdentity,
    pub input_identity: InputIdentity,
    pub request_identity: RequestIdentity,
    pub coverage: orchestrata_recipe_runtime::route_dispatch::CoverageOutcome,
}
