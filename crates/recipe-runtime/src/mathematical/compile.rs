use super::assignment_domain::AssignmentDomain;
use super::{
    CompiledModel, ModelError, ModelSpec, bound_assignment::BoundAssignment,
    constraint::Constraint, coverage_goal::CoverageGoal, id_field::IdField, objective::Objective,
    record_id::RecordId, validated_record::ValidatedRecord,
};
use orchestrata_solver::capacity_matching::weighted::WorkerCost;
use std::collections::BTreeSet;

pub fn compile_model(spec: &ModelSpec, json: &str) -> Result<CompiledModel, ModelError> {
    if matches!(spec.objectives.as_slice(), [Objective::MinTotalCost { .. }]) {
        return Err(ModelError::UnsupportedSemantics);
    }
    let assignment = compile_bound(spec, json)?.0;
    let goal = match spec.objectives.as_slice() {
        [Objective::MaxAssigned] => CoverageGoal::MaximumCoverage,
        [Objective::MaxAssigned, Objective::MinEdgeSum(_)] => CoverageGoal::CoverageThenEdgeCost,
        _ => return Err(ModelError::UnsupportedSemantics),
    };
    Ok(CompiledModel { assignment, goal })
}
pub(super) fn compile_bound(
    spec: &ModelSpec,
    json: &str,
) -> Result<(BoundAssignment, Vec<u32>), ModelError> {
    let tables = super::read_records::read_records(spec, json)?;
    let d = &spec.decision;
    let items = tables
        .get(&d.items)
        .ok_or(ModelError::UnsupportedSemantics)?;
    let resources = tables
        .get(&d.resources)
        .ok_or(ModelError::UnsupportedSemantics)?;
    if items.len() > 32 || resources.len() > 32 {
        return Err(ModelError::ResourceLimit);
    }
    let item_ids = ids(items, &d.item_id)?;
    let resource_ids = ids(resources, &d.resource_id)?;
    let capacities = resources
        .iter()
        .map(|row| {
            let mut cap = items.len();
            for rule in &spec.constraints {
                match rule {
                    Constraint::Capacity { field, limit } => {
                        cap = cap.min(field.read(row)? as usize);
                        if let Some(n) = limit {
                            cap = cap.min(*n as usize);
                        }
                    }
                    Constraint::Availability { field, value } => {
                        if field.read(row)? != *value {
                            cap = 0;
                        }
                    }
                }
            }
            Ok(cap)
        })
        .collect::<Result<Vec<_>, ModelError>>()?;
    let cost = match spec.objectives.as_slice() {
        [Objective::MaxAssigned] => None,
        [Objective::MaxAssigned, Objective::MinEdgeSum(f)] => Some(f),
        [Objective::MinTotalCost { service_field, .. }] => Some(service_field),
        _ => return Err(ModelError::UnsupportedSemantics),
    };
    let domains = match &d.domain {
        AssignmentDomain::Derived(domain) => domain.derive(items, resources)?,
        AssignmentDomain::Supplied {
            table,
            item: edge_item,
            resource: edge_resource,
        } => {
            let mut domains = vec![Vec::new(); items.len()];
            let mut seen = BTreeSet::new();
            for edge in tables.get(table).ok_or(ModelError::UnsupportedSemantics)? {
                let item = edge_item.read(edge)?;
                let resource = edge_resource.read(edge)?;
                let i = item_ids
                    .iter()
                    .position(|id| id == item)
                    .ok_or(ModelError::InvalidInput)?;
                let r = resource_ids
                    .iter()
                    .position(|id| id == resource)
                    .ok_or(ModelError::InvalidInput)?;
                if !seen.insert((i, r)) {
                    return Err(ModelError::InvalidInput);
                }
                domains[i].push(WorkerCost {
                    worker: r,
                    cost: cost.map(|f| f.read(edge)).transpose()?.unwrap_or(0),
                });
            }
            domains
        }
    };
    let penalties = match spec.objectives.as_slice() {
        [Objective::MinTotalCost { penalty_field, .. }] => items
            .iter()
            .map(|row| penalty_field.read(row))
            .collect::<Result<Vec<_>, _>>()?,
        _ => Vec::new(),
    };
    Ok((
        BoundAssignment {
            items: item_ids,
            resources: resource_ids,
            capacities,
            domains,
            nullable: d.nullable,
        },
        penalties,
    ))
}
fn ids(rows: &[ValidatedRecord], key: &IdField) -> Result<Vec<RecordId>, ModelError> {
    let ids = rows
        .iter()
        .map(|row| key.read(row).cloned())
        .collect::<Result<Vec<_>, _>>()?;
    if ids.iter().collect::<BTreeSet<_>>().len() != ids.len() {
        return Err(ModelError::InvalidInput);
    }
    Ok(ids)
}
