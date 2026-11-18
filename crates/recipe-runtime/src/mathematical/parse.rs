use super::{
    ModelError, ModelSpec, constraint::Constraint, decision::Decision, field_type::FieldType,
    input_binding::InputBinding, objective::Objective, xml_helpers::*,
};
use roxmltree::{Document, ParsingOptions};
use std::collections::{BTreeMap, BTreeSet};

pub fn parse_model(xml: &str) -> Result<ModelSpec, ModelError> {
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
    if root.attribute("version") != Some("1") {
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
    let d = Decision {
        name: attr(decision, "name")?,
        items: attr(decision, "items")?,
        resources: attr(decision, "resources")?,
        domain: attr(decision, "domain")?,
        item_id: attr(decision, "item-id")?,
        resource_id: attr(decision, "resource-id")?,
        edge_item: attr(decision, "edge-item")?,
        edge_resource: attr(decision, "edge-resource")?,
        nullable: boolean(
            decision
                .attribute("nullable")
                .ok_or(ModelError::InvalidXml)?,
        )?,
    };
    if BTreeSet::from([&d.items, &d.resources, &d.domain]).len() != 3
        || d.edge_item == d.edge_resource
    {
        return Err(ModelError::UnsupportedSemantics);
    }
    for (table, field, ty) in [
        (&d.items, &d.item_id, FieldType::Id),
        (&d.resources, &d.resource_id, FieldType::Id),
        (&d.domain, &d.edge_item, FieldType::Id),
        (&d.domain, &d.edge_resource, FieldType::Id),
    ] {
        field_type(&bindings, table, field, ty)?;
    }
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
                field_type(&bindings, &d.resources, &field, FieldType::U32)?;
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
                field_type(&bindings, &d.resources, &field, FieldType::Bool)?;
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
    if objectives.attribute("order") != Some("lexicographic") {
        return Err(ModelError::UnsupportedSemantics);
    }
    let mut goals = Vec::new();
    for o in children(objectives)? {
        if o.attribute("variable") != Some(&d.name) {
            return Err(ModelError::UnsupportedSemantics);
        }
        match (o.tag_name().name(), o.attribute("operator")) {
            ("maximize", Some("assigned-count")) => {
                leaf(o, "maximize", &["variable", "operator"])?;
                goals.push(Objective::MaxAssigned);
            }
            ("minimize", Some("edge-sum")) => {
                leaf(o, "minimize", &["variable", "operator", "field"])?;
                let f = attr(o, "field")?;
                field_type(&bindings, &d.domain, &f, FieldType::U32)?;
                goals.push(Objective::MinEdgeSum(f));
            }
            _ => return Err(ModelError::UnsupportedSemantics),
        }
    }
    if !matches!(
        goals.as_slice(),
        [Objective::MaxAssigned] | [Objective::MaxAssigned, Objective::MinEdgeSum(_)]
    ) {
        return Err(ModelError::UnsupportedSemantics);
    }
    Ok(ModelSpec {
        inputs: bindings,
        decision: d,
        constraints: rules,
        objectives: goals,
    })
}
fn field_type(
    inputs: &BTreeMap<String, InputBinding>,
    table: &str,
    field: &str,
    ty: FieldType,
) -> Result<(), ModelError> {
    if inputs.get(table).and_then(|b| b.fields.get(field)) != Some(&ty) {
        return Err(ModelError::UnsupportedSemantics);
    }
    Ok(())
}
