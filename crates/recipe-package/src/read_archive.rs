use crate::{PackageError, PackagePath, limits::*, zip_preflight};
use std::{
    collections::BTreeMap,
    io::{Cursor, Read},
};
use zip::{CompressionMethod, ZipArchive};
pub(crate) fn read(bytes: &[u8]) -> Result<BTreeMap<PackagePath, Vec<u8>>, PackageError> {
    let count = zip_preflight::check(bytes)?;
    let mut zip = ZipArchive::new(Cursor::new(bytes)).map_err(|_| PackageError::Archive)?;
    if zip.offset() != 0 || zip.len() != count {
        return Err(PackageError::Inventory);
    }
    zip.validate_header_consistency()
        .map_err(|_| PackageError::Archive)?;
    if zip
        .has_overlapping_entries()
        .map_err(|_| PackageError::Archive)?
    {
        return Err(PackageError::Archive);
    }
    let mut files = BTreeMap::new();
    let mut total = 0usize;
    for index in 0..zip.len() {
        let mut f = zip.by_index(index).map_err(|_| PackageError::Archive)?;
        if f.encrypted()
            || f.is_dir()
            || !matches!(
                f.compression(),
                CompressionMethod::Stored | CompressionMethod::Deflated
            )
        {
            return Err(PackageError::Archive);
        }
        if let Some(mode) = f.unix_mode() {
            let kind = mode & 0o170000;
            if ![0, 0o100000].contains(&kind) || mode & 0o111 != 0 {
                return Err(PackageError::Archive);
            }
        }
        let name = std::str::from_utf8(f.name_raw()).map_err(|_| PackageError::Path)?;
        let path = PackagePath::parse(name)?;
        let mut content = Vec::new();
        (&mut f)
            .take((MAX_MEMBER_BYTES + 1) as u64)
            .read_to_end(&mut content)
            .map_err(|_| PackageError::Archive)?;
        if content.len() > MAX_MEMBER_BYTES {
            return Err(PackageError::Limit);
        }
        total = total
            .checked_add(content.len())
            .ok_or(PackageError::Limit)?;
        if total > MAX_EXPANDED_BYTES {
            return Err(PackageError::Limit);
        }
        if content.len() as u64 != f.size() || files.insert(path, content).is_some() {
            return Err(PackageError::Inventory);
        }
    }
    Ok(files)
}
