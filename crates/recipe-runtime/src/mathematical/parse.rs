use super::{
    ModelError, ModelSpec, constraint::Constraint, decision::Decision, field_type::FieldType,
    input_binding::InputBinding, objective::Objective, xml_helpers::*,
};
use super::{boolean_field::BooleanField, id_field::IdField, unsigned_field::UnsignedField};
use roxmltree::{Document, ParsingOptions};
use std::collections::{BTreeMap, BTreeSet};

pub fn parse_model(xml: &str) -> Result<ModelSpec, ModelError> {
    parse_version(xml, "1")
}
pub(super) fn parse_version(xml: &str, version: &str) -> Result<ModelSpec, ModelError> {
    if xml.len() > 32768 {
        return Err(ModelError::ResourceLimit);
    }
    if xml.contains("<!DOCTYPE") || xml.contains("<!ENTITY") {
        return Err(ModelError::InvalidXml);
    }
    let doc = Document::parse_with_options(
        xml,
        ParsingOptions {
            allow_dtd: false,
            nodes_limit: 256,
            entity_resolver: None,
        },
    )
    .map_err(|_| ModelError::InvalidXml)?;
    if doc.descendants().any(|n| n.is_pi()) {
        return Err(ModelError::InvalidXml);
    }
    let root = doc.root_element();
    tag(root, "model", &["version", "id"])?;
    attr(root, "id")?;
    if root.attribute("version") != Some(version) {
        return Err(ModelError::UnsupportedSemantics);
    }
    let [inputs, parameters, decision, constraints, objectives] = children(root)?
        .try_into()
        .map_err(|_| ModelError::InvalidXml)?;
    tag(inputs, "inputs", &[])?;
    let mut bindings = BTreeMap::new();
    let mut source_fields = BTreeSet::new();
    for input in children(inputs)? {
        tag(input, "records", &["name", "field"])?;
        let name = attr(input, "name")?;
        let field = attr(input, "field")?;
        let mut fields = BTreeMap::new();
        for f in children(input)? {
            leaf(f, "field", &["name", "type"])?;
            let ty = match f.attribute("type") {
                Some("id") => FieldType::Id,
                Some("u32") => FieldType::U32,
                Some("bool") => FieldType::Bool,
                _ => return Err(ModelError::UnsupportedSemantics),
            };
            if fields.insert(attr(f, "name")?, ty).is_some() {
                return Err(ModelError::InvalidXml);
            }
        }
        if fields.is_empty()
            || fields.len() > 16
            || !source_fields.insert(field.clone())
            || bindings
                .insert(name, InputBinding { field, fields })
                .is_some()
        {
            return Err(ModelError::InvalidXml);
        }
    }
    if bindings.len() != 3 {
        return Err(ModelError::UnsupportedSemantics);
    }
    tag(parameters, "parameters", &[])?;
    let mut params = BTreeMap::new();
    let mut used = BTreeSet::new();
    for p in children(parameters)? {
        leaf(p, "parameter", &["name", "type", "value"])?;
        if p.attribute("type") != Some("u32") {
            return Err(ModelError::UnsupportedSemantics);
        }
        let value = p
            .attribute("value")
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|v| *v <= 32)
            .ok_or(ModelError::ResourceLimit)?;
        if params.insert(attr(p, "name")?, value).is_some() {
            return Err(ModelError::InvalidXml);
        }
    }
    leaf(
        decision,
        "decision",
        &[
            "name",
            "kind",
            "items",
            "resources",
            "domain",
            "item-id",
            "resource-id",
            "edge-item",
            "edge-resource",
            "nullable",
        ],
    )?;
    if decision.attribute("kind") != Some("reference") {
        return Err(ModelError::UnsupportedSemantics);
    }
    let name = attr(decision, "name")?;
    let items = attr(decision, "items")?;
    let resources = attr(decision, "resources")?;
    let domain = attr(decision, "domain")?;
    let item_id = attr(decision, "item-id")?;
    let resource_id = attr(decision, "resource-id")?;
    let edge_item = attr(decision, "edge-item")?;
    let edge_resource = attr(decision, "edge-resource")?;
    let nullable = boolean(
        decision
            .attribute("nullable")
            .ok_or(ModelError::InvalidXml)?,
    )?;
    if BTreeSet::from([&items, &resources, &domain]).len() != 3 || edge_item == edge_resource {
        return Err(ModelError::UnsupportedSemantics);
    }
    let d = Decision {
        name,
        item_id: IdField::bind(&bindings, &items, item_id)?,
        resource_id: IdField::bind(&bindings, &resources, resource_id)?,
        edge_item: IdField::bind(&bindings, &domain, edge_item)?,
        edge_resource: IdField::bind(&bindings, &domain, edge_resource)?,
        items,
        resources,
        domain,
        nullable,
    };
    tag(constraints, "constraints", &[])?;
    let mut rules = Vec::new();
    for c in children(constraints)? {
        if c.attribute("variable") != Some(&d.name) {
            return Err(ModelError::UnsupportedSemantics);
        }
        match c.tag_name().name() {
            "capacity" => {
                let has_limit = c.attribute("limit").is_some();
                leaf(
                    c,
                    "capacity",
                    if has_limit {
                        &["variable", "field", "limit"]
                    } else {
                        &["variable", "field"]
                    },
                )?;
                let field = attr(c, "field")?;
                let field = UnsignedField::bind(&bindings, &d.resources, field)?;
                let limit = if has_limit {
                    let key = attr(c, "limit")?;
                    used.insert(key.clone());
                    Some(*params.get(&key).ok_or(ModelError::UnsupportedSemantics)?)
                } else {
                    None
                };
                rules.push(Constraint::Capacity { field, limit });
            }
            "availability" => {
                leaf(c, "availability", &["variable", "field", "value"])?;
                let field = attr(c, "field")?;
                let field = BooleanField::bind(&bindings, &d.resources, field)?;
                rules.push(Constraint::Availability {
                    field,
                    value: boolean(c.attribute("value").ok_or(ModelError::InvalidXml)?)?,
                });
            }
            _ => return Err(ModelError::UnsupportedSemantics),
        }
    }
    if !rules
        .iter()
        .any(|r| matches!(r, Constraint::Capacity { .. }))
        || used.len() != params.len()
    {
        return Err(ModelError::UnsupportedSemantics);
    }
    tag(objectives, "objectives", &["order"])?;
    if objectives.attribute("order")
        != Some(if version == "1" {
            "lexicographic"
        } else {
            "single"
        })
    {
        return Err(ModelError::UnsupportedSemantics);
    }
    let mut goals = Vec::new();
    for o in children(objectives)? {
        if o.attribute("variable") != Some(&d.name) {
            return Err(ModelError::UnsupportedSemantics);
        }
        match (o.tag_name().name(), o.attribute("operator")) {
            ("minimize", Some("service-plus-unassigned-cost")) if version == "2" => {
                leaf(
                    o,
                    "minimize",
                    &[
                        "variable",
                        "operator",
                        "service-field",
                        "penalty-field",
                        "unit",
                    ],
                )?;
                let service_field = attr(o, "service-field")?;
                let penalty_field = attr(o, "penalty-field")?;
                let unit = attr(o, "unit")?;
                let service_field = UnsignedField::bind(&bindings, &d.domain, service_field)?;
                let penalty_field = UnsignedField::bind(&bindings, &d.items, penalty_field)?;
                goals.push(Objective::MinTotalCost {
                    service_field,
                    penalty_field,
                    unit,
                });
            }
            ("maximize", Some("assigned-count")) if version == "1" => {
                leaf(o, "maximize", &["variable", "operator"])?;
                goals.push(Objective::MaxAssigned);
            }
            ("minimize", Some("edge-sum")) if version == "1" => {
                leaf(o, "minimize", &["variable", "operator", "field"])?;
                let f = attr(o, "field")?;
                let f = UnsignedField::bind(&bindings, &d.domain, f)?;
                goals.push(Objective::MinEdgeSum(f));
            }
            _ => return Err(ModelError::UnsupportedSemantics),
        }
    }
    let supported = if version == "1" {
        matches!(
            goals.as_slice(),
            [Objective::MaxAssigned] | [Objective::MaxAssigned, Objective::MinEdgeSum(_)]
        )
    } else {
        d.nullable && matches!(goals.as_slice(), [Objective::MinTotalCost { .. }])
    };
    if !supported {
        return Err(ModelError::UnsupportedSemantics);
    }
    Ok(ModelSpec {
        inputs: bindings,
        decision: d,
        constraints: rules,
        objectives: goals,
    })
}
