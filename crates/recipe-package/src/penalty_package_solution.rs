use crate::{ContentIdentity, ExecutionIdentity};
use orchestrata_recipe_runtime::mathematical::PenaltyModelSolution;
#[derive(Debug)]
pub struct PenaltyPackageSolution {
    pub content_identity: ContentIdentity,
    pub execution_identity: ExecutionIdentity,
    pub solution: PenaltyModelSolution,
}
