use super::ModelError;
use roxmltree::Node;
pub(super) fn tag(n: Node<'_, '_>, name: &str, attrs: &[&str]) -> Result<(), ModelError> {
    if !n.has_tag_name(name)
        || n.tag_name().namespace().is_some()
        || n.namespaces().len() != 0
        || n.attributes().len() != attrs.len()
        || n.attributes()
            .any(|a| a.namespace().is_some() || !attrs.contains(&a.name()))
    {
        return Err(ModelError::InvalidXml);
    }
    Ok(())
}
pub(super) fn attr(n: Node<'_, '_>, key: &str) -> Result<String, ModelError> {
    n.attribute(key)
        .filter(|s| identifier(s))
        .map(str::to_owned)
        .ok_or(ModelError::InvalidXml)
}
pub(super) fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
pub(super) fn children<'a, 'input>(
    n: Node<'a, 'input>,
) -> Result<Vec<Node<'a, 'input>>, ModelError> {
    if n.children()
        .any(|c| c.is_text() && !c.text().unwrap_or_default().trim().is_empty())
    {
        return Err(ModelError::InvalidXml);
    }
    Ok(n.children().filter(Node::is_element).collect())
}
pub(super) fn leaf(n: Node<'_, '_>, name: &str, attrs: &[&str]) -> Result<(), ModelError> {
    tag(n, name, attrs)?;
    if !children(n)?.is_empty() {
        return Err(ModelError::InvalidXml);
    }
    Ok(())
}
pub(super) fn boolean(s: &str) -> Result<bool, ModelError> {
    match s {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(ModelError::InvalidXml),
    }
}
