use crate::PackageError;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}
impl PackageVersion {
    pub(crate) fn parse(s: &str) -> Result<Self, PackageError> {
        let parts = s.split('.').collect::<Vec<_>>();
        if parts.len() != 3
            || parts.iter().any(|p| {
                p.is_empty()
                    || p.len() > 1 && p.starts_with('0')
                    || !p.bytes().all(|b| b.is_ascii_digit())
            })
        {
            return Err(PackageError::Metadata);
        }
        let n = parts
            .iter()
            .map(|p| p.parse::<u32>().map_err(|_| PackageError::Metadata))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            major: n[0],
            minor: n[1],
            patch: n[2],
        })
    }
}
