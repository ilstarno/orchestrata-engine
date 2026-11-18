use super::{
    ModelError, assignment_domain::AssignmentDomain, boolean_field::BooleanField,
    derived_domain::DerivedDomain, field_type::FieldType, id_field::IdField,
    input_binding::InputBinding, row_equality::RowEquality, unsigned_field::UnsignedField,
    xml_helpers::*,
};
use std::collections::BTreeMap;
pub(super) fn parse_derived_domain(
    node: roxmltree::Node<'_, '_>,
    inputs: &BTreeMap<String, InputBinding>,
    items: &str,
    resources: &str,
) -> Result<AssignmentDomain, ModelError> {
    if node.attribute("domain") != Some("derived") || items == resources {
        return Err(ModelError::UnsupportedSemantics);
    }
    let pair_limit = limit(node, "pair-limit", 1024)?;
    let comparison_limit = limit(node, "comparison-limit", 8192)?;
    let children = children(node)?;
    if children.is_empty() || children.len() > 8 {
        return Err(ModelError::ResourceLimit);
    }
    let mut predicates = Vec::new();
    for predicate in children {
        leaf(predicate, "equal", &["item-field", "resource-field"])?;
        let a = attr(predicate, "item-field")?;
        let b = attr(predicate, "resource-field")?;
        let left = inputs
            .get(items)
            .and_then(|t| t.fields.get(&a))
            .ok_or(ModelError::UnsupportedSemantics)?;
        let right = inputs
            .get(resources)
            .and_then(|t| t.fields.get(&b))
            .ok_or(ModelError::UnsupportedSemantics)?;
        if left != right {
            return Err(ModelError::UnsupportedSemantics);
        }
        predicates.push(match left {
            FieldType::Id => RowEquality::Id(
                IdField::bind(inputs, items, a)?,
                IdField::bind(inputs, resources, b)?,
            ),
            FieldType::U32 => RowEquality::Unsigned(
                UnsignedField::bind(inputs, items, a)?,
                UnsignedField::bind(inputs, resources, b)?,
            ),
            FieldType::Bool => RowEquality::Boolean(
                BooleanField::bind(inputs, items, a)?,
                BooleanField::bind(inputs, resources, b)?,
            ),
        });
    }
    Ok(AssignmentDomain::Derived(DerivedDomain {
        predicates,
        pair_limit,
        comparison_limit,
    }))
}
fn limit(node: roxmltree::Node<'_, '_>, name: &str, maximum: usize) -> Result<usize, ModelError> {
    let raw = node.attribute(name).ok_or(ModelError::InvalidXml)?;
    if raw.is_empty() || !raw.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ModelError::InvalidXml);
    }
    let value = raw
        .parse::<usize>()
        .map_err(|_| ModelError::ResourceLimit)?;
    if value > maximum {
        return Err(ModelError::ResourceLimit);
    }
    Ok(value)
}
