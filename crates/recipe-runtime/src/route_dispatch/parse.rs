use super::{DispatchError, RouteSpec, strategy::Strategy};
use roxmltree::{Document, ParsingOptions};
pub fn parse_route_model(xml: &str) -> Result<RouteSpec, DispatchError> {
    if xml.len() > 32768 {
        return Err(DispatchError::Limit);
    }
    let doc = Document::parse_with_options(
        xml,
        ParsingOptions {
            allow_dtd: false,
            nodes_limit: 32,
            entity_resolver: None,
        },
    )
    .map_err(|_| DispatchError::InvalidModel)?;
    if xml.contains("<!DOCTYPE") || xml.contains("<!ENTITY") || doc.descendants().any(|n| n.is_pi())
    {
        return Err(DispatchError::InvalidModel);
    }
    let n = doc.root_element();
    let attrs = [
        "version",
        "id",
        "input-schema",
        "algorithm",
        "time-unit",
        "load-unit",
        "max-insertions",
    ];
    if !n.has_tag_name("route-dispatch")
        || n.tag_name().namespace().is_some()
        || n.namespaces().len() != 0
        || n.attributes().len() != attrs.len()
        || n.attributes()
            .any(|a| a.namespace().is_some() || !attrs.contains(&a.name()))
        || n.children().any(|c| {
            c.is_element() || c.is_text() && !c.text().unwrap_or_default().trim().is_empty()
        })
    {
        return Err(DispatchError::InvalidModel);
    }
    let id = n.attribute("id").unwrap_or_default();
    if id.is_empty()
        || id.len() > 64
        || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        || n.attribute("version") != Some("1")
        || n.attribute("input-schema") != Some("directed-dispatch-v1")
        || n.attribute("time-unit") != Some("minutes")
        || n.attribute("load-unit") != Some("kg")
    {
        return Err(DispatchError::InvalidModel);
    }
    let strategy = match n.attribute("algorithm") {
        Some("input-order-insertion") => Strategy::InputOrder,
        Some("two-order-insertion") => Strategy::TwoOrders,
        _ => return Err(DispatchError::InvalidModel),
    };
    let max_insertions = n
        .attribute("max-insertions")
        .and_then(|s| s.parse::<u32>().ok())
        .filter(|n| *n <= 20000)
        .ok_or(DispatchError::Limit)?;
    Ok(RouteSpec {
        strategy,
        max_insertions,
    })
}
pub fn compile_route_model(
    spec: &RouteSpec,
    records: &str,
    budget: u32,
) -> Result<super::CompiledRoute, DispatchError> {
    if records.len() > 524288 || budget > spec.max_insertions {
        return Err(DispatchError::Limit);
    }
    let snapshot: super::snapshot::Snapshot =
        serde_json::from_str(records).map_err(|_| DispatchError::InvalidInput)?;
    let (problem, travel) =
        super::compile::compile(&snapshot).map_err(|_| DispatchError::InvalidInput)?;
    Ok(super::CompiledRoute {
        problem,
        travel,
        snapshot,
        spec: spec.clone(),
        budget,
    })
}
