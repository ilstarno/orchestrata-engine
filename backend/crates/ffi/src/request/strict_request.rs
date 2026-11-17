use orchestrata_solver::Problem;
use serde_json::{Map, Value};

use super::{StrictStatus, checked_json::CheckedJson};

pub(crate) struct StrictRequest(Problem);

impl StrictRequest {
    pub(crate) fn decode(bytes: &[u8]) -> Result<Self, StrictStatus> {
        let CheckedJson { value, duplicate } =
            serde_json::from_slice(bytes).map_err(|_| StrictStatus::MalformedJson)?;
        if duplicate {
            return Err(StrictStatus::DuplicateField);
        }
        let problem = named(&value, &["variables", "constraints"])?;
        named_rows(problem, "variables", &["name", "domain"])?;
        for constraint in array(problem, "constraints")? {
            check_constraint(constraint)?;
        }
        serde_json::from_value(value)
            .map(Self)
            .map_err(|_| StrictStatus::InvalidShape)
    }

    pub(crate) fn into_problem(self) -> Problem {
        self.0
    }
}

fn named<'a>(value: &'a Value, fields: &[&str]) -> Result<&'a Map<String, Value>, StrictStatus> {
    let object = value.as_object().ok_or(StrictStatus::InvalidShape)?;
    if object.keys().any(|key| !fields.contains(&key.as_str())) {
        return Err(StrictStatus::UnknownField);
    }
    Ok(object)
}

fn array<'a>(object: &'a Map<String, Value>, field: &str) -> Result<&'a [Value], StrictStatus> {
    object
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or(StrictStatus::InvalidShape)
}

fn named_rows(
    object: &Map<String, Value>,
    field: &str,
    fields: &[&str],
) -> Result<(), StrictStatus> {
    for row in array(object, field)? {
        named(row, fields)?;
    }
    Ok(())
}

fn check_constraint(value: &Value) -> Result<(), StrictStatus> {
    let kind = value
        .as_object()
        .and_then(|object| object.get("kind"))
        .and_then(Value::as_str)
        .ok_or(StrictStatus::InvalidShape)?;
    let fields: &[&str] = match kind {
        "count_value" => &["kind", "variables", "value", "min", "max", "weight", "tier"],
        "all_different" => &["kind", "variables", "hard_weight"],
        "not_equal_offset" => &["kind", "left", "right", "offset", "hard_weight"],
        "precedence" => &["kind", "before", "after", "min_gap"],
        "no_overlap" => &["kind", "intervals"],
        "resource_capacity" => &["kind", "items", "limits"],
        "prefer_value" => &["kind", "variable", "value", "soft_weight"],
        _ => return Err(StrictStatus::InvalidShape),
    };
    let object = named(value, fields)?;
    match kind {
        "no_overlap" => named_rows(object, "intervals", &["start", "duration"]),
        "resource_capacity" => {
            named_rows(object, "items", &["variable", "load"])?;
            named_rows(object, "limits", &["resource", "capacity"])
        }
        _ => Ok(()),
    }
}
