use crate::{ContentIdentity, ExecutionIdentity, InputIdentity, RequestIdentity};
#[derive(Debug)]
pub struct RoutePackageSolution {
    pub content_identity: ContentIdentity,
    pub execution_identity: ExecutionIdentity,
    pub input_identity: InputIdentity,
    pub request_identity: RequestIdentity,
    pub solution: orchestrata_recipe_runtime::route_dispatch::DispatchSolution,
}
