use super::{ModelAssignment, ModelError, ModelSolution, bound_assignment::BoundAssignment};

pub(super) fn finish_assignment(
    model: BoundAssignment,
    workers: Vec<Option<usize>>,
    assigned: usize,
    cost: Option<u64>,
) -> Result<ModelSolution, ModelError> {
    if !model.nullable && assigned != model.items.len() {
        return Err(ModelError::Infeasible);
    }
    Ok(ModelSolution {
        assigned,
        minimized_edge_sum: cost,
        assignments: model
            .items
            .into_iter()
            .zip(workers)
            .map(|(item_id, r)| ModelAssignment {
                item_id: item_id.into_string(),
                resource_id: r.map(|index| model.resources[index].clone().into_string()),
            })
            .collect(),
    })
}
