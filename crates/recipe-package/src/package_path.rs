use crate::PackageError;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PackagePath(String);
impl PackagePath {
    pub fn parse(s: &str) -> Result<Self, PackageError> {
        if s.is_empty()
            || s.len() > 128
            || !s.ends_with(".xml")
            || s.split('/').count() > 4
            || !s.bytes().all(|b| {
                b.is_ascii_lowercase()
                    || b.is_ascii_digit()
                    || matches!(b, b'-' | b'_' | b'/' | b'.')
            })
        {
            return Err(PackageError::Path);
        }
        for part in s.split('/') {
            if part.is_empty() || part.len() > 64 || part.starts_with('.') || part.ends_with('.') {
                return Err(PackageError::Path);
            }
            let stem = part.split('.').next().unwrap_or_default();
            if ["con", "prn", "aux", "nul"].contains(&stem)
                || (stem.len() == 4
                    && (stem.starts_with("com") || stem.starts_with("lpt"))
                    && stem.as_bytes()[3].is_ascii_digit())
            {
                return Err(PackageError::Path);
            }
        }
        Ok(Self(s.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
