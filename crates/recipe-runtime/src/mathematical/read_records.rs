use super::{
    ModelError, ModelSpec, bounded_rows::BoundedRows, strict_object::StrictObject,
    validated_record::ValidatedRecord,
};
use std::collections::BTreeMap;
pub(super) fn read_records(
    spec: &ModelSpec,
    json: &str,
) -> Result<BTreeMap<String, Vec<ValidatedRecord>>, ModelError> {
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
        let BoundedRows(rows) =
            serde_json::from_str(raw.get()).map_err(|_| ModelError::InvalidInput)?;
        records += rows.len();
        if records > 128 {
            return Err(ModelError::ResourceLimit);
        }
        let typed = rows
            .into_iter()
            .map(|row| ValidatedRecord::parse(name, binding, row))
            .collect::<Result<_, _>>()?;
        tables.insert(name.clone(), typed);
    }
    Ok(tables)
}
