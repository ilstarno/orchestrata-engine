use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::{Cursor, Read, Write},
};
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter, write::SimpleFileOptions};
pub const PACKAGE: &[u8] = include_bytes!("../fixtures/maintenance.orcpkg");
pub const INPUT: &str = include_str!("../fixtures/inputs/maintenance.json");
pub fn files() -> BTreeMap<String, Vec<u8>> {
    let mut z = ZipArchive::new(Cursor::new(PACKAGE)).unwrap();
    let mut map = BTreeMap::new();
    for i in 0..z.len() {
        let mut f = z.by_index(i).unwrap();
        let name = std::str::from_utf8(f.name_raw()).unwrap().to_owned();
        let mut b = Vec::new();
        f.read_to_end(&mut b).unwrap();
        map.insert(name, b);
    }
    map
}
pub fn pack(files: &BTreeMap<String, Vec<u8>>, reverse: bool, stored: bool) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let mut names = files.keys().collect::<Vec<_>>();
    if reverse {
        names.reverse();
    }
    for name in names {
        let options = SimpleFileOptions::default()
            .compression_method(if stored {
                CompressionMethod::Stored
            } else {
                CompressionMethod::Deflated
            })
            .unix_permissions(0o644)
            .last_modified_time(
                DateTime::from_date_and_time(if reverse { 2025 } else { 2026 }, 1, 1, 0, 0, 0)
                    .unwrap(),
            );
        writer.start_file(name, options).unwrap();
        writer.write_all(&files[name]).unwrap();
    }
    writer.finish().unwrap().into_inner()
}
pub fn manifest(files: &mut BTreeMap<String, Vec<u8>>, change: impl FnOnce(String) -> String) {
    let old = String::from_utf8(files["recipe.xml"].clone()).unwrap();
    files.insert("recipe.xml".into(), change(old).into_bytes());
}
pub fn asset(files: &mut BTreeMap<String, Vec<u8>>, path: &str, bytes: Vec<u8>) {
    let old = files[path].clone();
    let old_hash = Sha256::digest(&old)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let hash = Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    manifest(files, |s| {
        s.replace(
            &format!("bytes=\"{}\" sha256=\"{old_hash}\"", old.len()),
            &format!("bytes=\"{}\" sha256=\"{hash}\"", bytes.len()),
        )
    });
    files.insert(path.into(), bytes);
}
pub fn central_positions(b: &[u8]) -> Vec<usize> {
    let end = b.len() - 22;
    let count = u16::from_le_bytes(b[end + 10..end + 12].try_into().unwrap());
    let mut p = u32::from_le_bytes(b[end + 16..end + 20].try_into().unwrap()) as usize;
    let mut out = Vec::new();
    for _ in 0..count {
        out.push(p);
        let n = u16::from_le_bytes(b[p + 28..p + 30].try_into().unwrap()) as usize;
        let x = u16::from_le_bytes(b[p + 30..p + 32].try_into().unwrap()) as usize;
        let c = u16::from_le_bytes(b[p + 32..p + 34].try_into().unwrap()) as usize;
        p += 46 + n + x + c;
    }
    out
}
