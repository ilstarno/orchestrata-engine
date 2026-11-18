use crate::{ContentIdentity, ExecutionIdentity, PackageError, PenaltyPackageSolution};
use orchestrata_recipe_runtime::mathematical::{CompiledPenaltyModel, solve_penalty_model};
#[derive(Debug)]
pub struct CompiledPenaltyPackage {
    pub(crate) model: CompiledPenaltyModel,
    pub(crate) content_identity: ContentIdentity,
    pub(crate) execution_identity: ExecutionIdentity,
}
impl CompiledPenaltyPackage {
    pub fn content_identity(&self) -> ContentIdentity {
        self.content_identity
    }
    pub fn execution_identity(&self) -> ExecutionIdentity {
        self.execution_identity
    }
    pub fn solve(self) -> Result<PenaltyPackageSolution, PackageError> {
        Ok(PenaltyPackageSolution {
            content_identity: self.content_identity,
            execution_identity: self.execution_identity,
            solution: solve_penalty_model(self.model).map_err(PackageError::Model)?,
        })
    }
}
