use crate::{ContentIdentity, ExecutionIdentity};
use orchestrata_recipe_runtime::mathematical::ModelSolution;
#[derive(Debug)]
pub struct PackageSolution {
    pub content_identity: ContentIdentity,
    pub execution_identity: ExecutionIdentity,
    pub solution: ModelSolution,
}
