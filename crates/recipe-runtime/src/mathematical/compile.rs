use super::{
    CompiledModel, ModelError, ModelSpec, constraint::Constraint, field_type::FieldType,
    objective::Objective, strict_object::StrictObject, xml_helpers::identifier,
};
use orchestrata_solver::capacity_matching::weighted::WorkerCost;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub fn compile_model(spec: &ModelSpec, json: &str) -> Result<CompiledModel, ModelError> {
    if json.len() > 524288 {
        return Err(ModelError::ResourceLimit);
    }
    let StrictObject(top) = serde_json::from_str(json).map_err(|_| ModelError::InvalidInput)?;
    if top.len() != spec.inputs.len() {
        return Err(ModelError::InvalidInput);
    }
    let mut tables = BTreeMap::new();
    let mut records = 0;
    for (name, binding) in &spec.inputs {
        let raw = top.get(&binding.field).ok_or(ModelError::InvalidInput)?;
        let super::bounded_rows::BoundedRows(rows) =
            serde_json::from_str(raw.get()).map_err(|_| ModelError::InvalidInput)?;
        records += rows.len();
        if records > 128 {
            return Err(ModelError::ResourceLimit);
        }
        let mut typed = Vec::new();
        for StrictObject(row) in rows {
            if row.len() != binding.fields.len() {
                return Err(ModelError::InvalidInput);
            }
            let mut values = BTreeMap::new();
            for (field, ty) in &binding.fields {
                let value: Value =
                    serde_json::from_str(row.get(field).ok_or(ModelError::InvalidInput)?.get())
                        .map_err(|_| ModelError::InvalidInput)?;
                let valid = match ty {
                    FieldType::Id => value.as_str().is_some_and(identifier),
                    FieldType::U32 => value.as_u64().is_some_and(|n| n <= 1_000_000),
                    FieldType::Bool => value.is_boolean(),
                };
                if !valid {
                    return Err(ModelError::InvalidInput);
                }
                values.insert(field.clone(), value);
            }
            typed.push(values);
        }
        tables.insert(name, typed);
    }
    let d = &spec.decision;
    let items = &tables[&d.items];
    let resources = &tables[&d.resources];
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
                        cap = cap.min(row[field].as_u64().unwrap() as usize);
                        if let Some(n) = limit {
                            cap = cap.min(*n as usize);
                        }
                    }
                    Constraint::Availability { field, value } => {
                        if row[field].as_bool() != Some(*value) {
                            cap = 0;
                        }
                    }
                }
            }
            cap
        })
        .collect();
    let cost = match spec.objectives.as_slice() {
        [Objective::MaxAssigned] => None,
        [Objective::MaxAssigned, Objective::MinEdgeSum(f)] => Some(f),
        _ => return Err(ModelError::UnsupportedSemantics),
    };
    let mut domains = vec![Vec::new(); items.len()];
    let mut seen = BTreeSet::new();
    for edge in &tables[&d.domain] {
        let i = item_ids
            .iter()
            .position(|id| Some(id.as_str()) == edge[&d.edge_item].as_str())
            .ok_or(ModelError::InvalidInput)?;
        let r = resource_ids
            .iter()
            .position(|id| Some(id.as_str()) == edge[&d.edge_resource].as_str())
            .ok_or(ModelError::InvalidInput)?;
        if !seen.insert((i, r)) {
            return Err(ModelError::InvalidInput);
        }
        domains[i].push(WorkerCost {
            worker: r,
            cost: cost.map_or(0, |f| edge[f].as_u64().unwrap() as u32),
        });
    }
    Ok(CompiledModel {
        items: item_ids,
        resources: resource_ids,
        capacities,
        domains,
        minimize_cost: cost.is_some(),
        nullable: d.nullable,
    })
}
fn ids(rows: &[BTreeMap<String, Value>], key: &str) -> Result<Vec<String>, ModelError> {
    let ids = rows
        .iter()
        .map(|r| r[key].as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    if ids.iter().collect::<BTreeSet<_>>().len() != ids.len() {
        return Err(ModelError::InvalidInput);
    }
    Ok(ids)
}
