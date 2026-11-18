use orchestrata_recipe_package::{limits::MAX_ARCHIVE_BYTES, load_package};
use std::{env, fs::File, io::Read, path::Path, process::ExitCode};
fn bytes(path: &Path, limit: usize) -> std::io::Result<Vec<u8>> {
    let mut b = Vec::new();
    File::open(path)?
        .take((limit + 1) as u64)
        .read_to_end(&mut b)?;
    Ok(b)
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if !(2..=3).contains(&args.len()) {
        return Err("Usage: solve-package <package.orcpkg> <records.json> [entrypoint]".into());
    }
    let p = load_package(&bytes(Path::new(&args[0]), MAX_ARCHIVE_BYTES)?)?;
    let records = String::from_utf8(bytes(Path::new(&args[1]), 524288)?)?;
    let selected = args
        .get(2)
        .map(|s| s.to_str().ok_or("Entrypoint must be UTF-8."))
        .transpose()?;
    let result = p.compile(selected, &records)?.solve()?;
    println!(
        "{}",
        serde_json::json!({"content_identity":result.content_identity.to_hex(),"execution_identity":result.execution_identity.to_hex(),"solution":result.solution})
    );
    Ok(())
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
