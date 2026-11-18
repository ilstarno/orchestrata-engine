use crate::{
    ContentIdentity, ExecutionIdentity, InputIdentity, PackageError, RequestIdentity,
    RoutePackageSolution,
};
pub struct CompiledRoutePackage {
    pub(crate) model: orchestrata_recipe_runtime::route_dispatch::CompiledRoute,
    pub(crate) content_identity: ContentIdentity,
    pub(crate) execution_identity: ExecutionIdentity,
    pub(crate) input_identity: InputIdentity,
    pub(crate) request_identity: RequestIdentity,
}
impl CompiledRoutePackage {
    pub fn solve(self) -> Result<RoutePackageSolution, PackageError> {
        Ok(RoutePackageSolution {
            content_identity: self.content_identity,
            execution_identity: self.execution_identity,
            input_identity: self.input_identity,
            request_identity: self.request_identity,
            solution: orchestrata_recipe_runtime::route_dispatch::solve_route_model(self.model)
                .map_err(|_| PackageError::Unsupported)?,
        })
    }
}

impl CompiledRoutePackage {
    /// Certifies only this compiled revision/input and a separately supplied plan.
    /// No implicit solve, network operation, or hosted authorization is involved.
    pub fn certify_coverage(
        &self,
        incumbent: &orchestrata_recipe_runtime::route_dispatch::DispatchSolution,
        max_work: u32,
    ) -> Result<
        crate::RoutePackageCoverage,
        orchestrata_recipe_runtime::route_dispatch::DispatchCoverageError,
    > {
        Ok(crate::RoutePackageCoverage {
            content_identity: self.content_identity,
            execution_identity: self.execution_identity,
            input_identity: self.input_identity,
            request_identity: self.request_identity,
            coverage: orchestrata_recipe_runtime::route_dispatch::certify_route_coverage(
                &self.model,
                incumbent,
                max_work,
            )?,
        })
    }
}
