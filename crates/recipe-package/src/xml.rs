use crate::{PackageError, limits::MAX_MEMBER_BYTES};
use roxmltree::{Document, Node, ParsingOptions};
pub(crate) fn document(s: &str) -> Result<Document<'_>, PackageError> {
    if s.len() > MAX_MEMBER_BYTES {
        return Err(PackageError::Limit);
    }
    if s.contains("<!DOCTYPE") || s.contains("<!ENTITY") {
        return Err(PackageError::Xml);
    }
    let doc = Document::parse_with_options(
        s,
        ParsingOptions {
            allow_dtd: false,
            nodes_limit: 256,
            entity_resolver: None,
        },
    )
    .map_err(|_| PackageError::Xml)?;
    if doc.descendants().any(|n| {
        n.is_pi()
            || n.is_element()
                && (n.tag_name().namespace().is_some()
                    || n.namespaces().len() != 0
                    || n.attributes().any(|a| a.namespace().is_some()))
    }) {
        return Err(PackageError::Xml);
    }
    Ok(doc)
}
pub(crate) fn tag(n: Node<'_, '_>, name: &str, attrs: &[&str]) -> Result<(), PackageError> {
    if !n.has_tag_name(name)
        || n.attributes().len() != attrs.len()
        || n.attributes().any(|a| !attrs.contains(&a.name()))
    {
        return Err(PackageError::Xml);
    }
    Ok(())
}
pub(crate) fn children<'a, 'i>(n: Node<'a, 'i>) -> Result<Vec<Node<'a, 'i>>, PackageError> {
    if n.children()
        .any(|c| c.is_text() && !c.text().unwrap_or_default().trim().is_empty())
    {
        return Err(PackageError::Xml);
    }
    Ok(n.children().filter(Node::is_element).collect())
}
pub(crate) fn leaf(n: Node<'_, '_>, name: &str, attrs: &[&str]) -> Result<(), PackageError> {
    tag(n, name, attrs)?;
    if !children(n)?.is_empty() {
        return Err(PackageError::Xml);
    }
    Ok(())
}
pub(crate) fn attr<'a>(n: Node<'a, '_>, key: &str) -> Result<&'a str, PackageError> {
    n.attribute(key).ok_or(PackageError::Metadata)
}
pub(crate) fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.as_bytes()[0].is_ascii_lowercase()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
pub(crate) fn locale(s: &str) -> bool {
    let b = s.as_bytes();
    (b.len() == 2 || b.len() == 5 && b[2] == b'-' && b[3..].iter().all(u8::is_ascii_uppercase))
        && b[..2].iter().all(u8::is_ascii_lowercase)
}
pub(crate) fn text(n: Node<'_, '_>, name: &str, max: usize) -> Result<String, PackageError> {
    tag(n, name, &[])?;
    if n.children().any(|c| !c.is_text()) {
        return Err(PackageError::Metadata);
    }
    let s = n.text().unwrap_or_default();
    if s.trim() != s || s.is_empty() || s.len() > max || s.chars().any(char::is_control) {
        return Err(PackageError::Metadata);
    }
    Ok(s.to_owned())
}
