use crate::{PackageError, PackagePath, limits::*};
use std::collections::BTreeSet;
fn u16_at(b: &[u8], p: usize) -> Result<usize, PackageError> {
    let x = b.get(p..p + 2).ok_or(PackageError::Archive)?;
    Ok(u16::from_le_bytes([x[0], x[1]]) as usize)
}
fn u32_at(b: &[u8], p: usize) -> Result<usize, PackageError> {
    let x = b.get(p..p + 4).ok_or(PackageError::Archive)?;
    Ok(u32::from_le_bytes([x[0], x[1], x[2], x[3]]) as usize)
}
/// Bound central-directory allocation before invoking the ZIP reader; reject aliasing before its map can collapse duplicates.
pub(crate) fn check(b: &[u8]) -> Result<usize, PackageError> {
    if b.len() > MAX_ARCHIVE_BYTES {
        return Err(PackageError::Limit);
    }
    let end = b.len().checked_sub(22).ok_or(PackageError::Archive)?;
    if b.get(end..end + 4) != Some(b"PK\x05\x06")
        || u16_at(b, end + 20)? != 0
        || u16_at(b, end + 4)? != 0
        || u16_at(b, end + 6)? != 0
    {
        return Err(PackageError::Archive);
    }
    let count = u16_at(b, end + 10)?;
    if count == 0 || count > MAX_MEMBERS {
        return Err(PackageError::Limit);
    }
    if u16_at(b, end + 8)? != count {
        return Err(PackageError::Archive);
    }
    let size = u32_at(b, end + 12)?;
    let start = u32_at(b, end + 16)?;
    if start.checked_add(size) != Some(end) || b.get(..4) != Some(b"PK\x03\x04") {
        return Err(PackageError::Archive);
    }
    let mut at = start;
    let mut names = BTreeSet::new();
    let mut total = 0usize;
    for _ in 0..count {
        if b.get(at..at + 4) != Some(b"PK\x01\x02") || at + 46 > end {
            return Err(PackageError::Archive);
        }
        let flags = u16_at(b, at + 8)?;
        let method = u16_at(b, at + 10)?;
        if flags & !0x080e != 0 || ![0, 8].contains(&method) || u16_at(b, at + 34)? != 0 {
            return Err(PackageError::Archive);
        }
        let length = u32_at(b, at + 24)?;
        total = total.checked_add(length).ok_or(PackageError::Limit)?;
        if length > MAX_MEMBER_BYTES || total > MAX_EXPANDED_BYTES {
            return Err(PackageError::Limit);
        }
        let n = u16_at(b, at + 28)?;
        let extra = u16_at(b, at + 30)?;
        let comment = u16_at(b, at + 32)?;
        // V1 has no extensions, comments, ZIP64 or multi-disk records.
        if extra != 0 || comment != 0 || n == 0 || n > 128 {
            return Err(PackageError::Archive);
        }
        let next = at
            .checked_add(46 + n)
            .filter(|p| *p <= end)
            .ok_or(PackageError::Archive)?;
        let local = u32_at(b, at + 42)?;
        if local.checked_add(30).is_none_or(|p| p > start)
            || b.get(local..local + 4) != Some(b"PK\x03\x04")
            || u16_at(b, local + 26)? != n
            || u16_at(b, local + 28)? != 0
        {
            return Err(PackageError::Archive);
        }
        let name = std::str::from_utf8(&b[at + 46..next]).map_err(|_| PackageError::Path)?;
        let path = PackagePath::parse(name)?;
        if !names.insert(path) {
            return Err(PackageError::Inventory);
        }
        at = next;
    }
    if at != end {
        return Err(PackageError::Archive);
    }
    Ok(count)
}
