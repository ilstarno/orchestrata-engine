use orchestrata_solver::Solution;
use serde::Serialize;

#[derive(Serialize)]
pub struct ResultDocument {
    pub solution: Option<Solution>,
}
