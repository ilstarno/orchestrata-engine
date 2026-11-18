use super::{ModelError, row_equality::RowEquality, validated_record::ValidatedRecord};
use orchestrata_solver::capacity_matching::weighted::WorkerCost;
#[derive(Debug)]
pub(super) struct DerivedDomain {
    pub predicates: Vec<RowEquality>,
    pub pair_limit: usize,
    pub comparison_limit: usize,
}
impl DerivedDomain {
    pub fn derive(
        &self,
        items: &[ValidatedRecord],
        resources: &[ValidatedRecord],
    ) -> Result<Vec<Vec<WorkerCost>>, ModelError> {
        let pairs = items
            .len()
            .checked_mul(resources.len())
            .ok_or(ModelError::ResourceLimit)?;
        let comparisons = pairs
            .checked_mul(self.predicates.len())
            .ok_or(ModelError::ResourceLimit)?;
        if pairs > self.pair_limit || comparisons > self.comparison_limit {
            return Err(ModelError::ResourceLimit);
        }
        items
            .iter()
            .map(|item| {
                let mut edges = Vec::new();
                for (worker, resource) in resources.iter().enumerate() {
                    let mut eligible = true;
                    for predicate in &self.predicates {
                        eligible &= predicate.matches(item, resource)?;
                    }
                    if eligible {
                        edges.push(WorkerCost { worker, cost: 0 });
                    }
                }
                Ok(edges)
            })
            .collect()
    }
}
