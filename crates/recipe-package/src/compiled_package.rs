use crate::{ContentIdentity, ExecutionIdentity, PackageError, PackageSolution};
use orchestrata_recipe_runtime::mathematical::{CompiledModel, solve_model};
#[derive(Debug)]
pub struct CompiledPackage {
    pub(crate) model: CompiledModel,
    pub(crate) content_identity: ContentIdentity,
    pub(crate) execution_identity: ExecutionIdentity,
}
impl CompiledPackage {
    pub fn content_identity(&self) -> ContentIdentity {
        self.content_identity
    }
    pub fn execution_identity(&self) -> ExecutionIdentity {
        self.execution_identity
    }
    pub fn solve(self) -> Result<PackageSolution, PackageError> {
        Ok(PackageSolution {
            content_identity: self.content_identity,
            execution_identity: self.execution_identity,
            solution: solve_model(self.model).map_err(PackageError::Model)?,
        })
    }
}
