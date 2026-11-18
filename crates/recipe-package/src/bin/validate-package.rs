use orchestrata_recipe_package::{limits::MAX_ARCHIVE_BYTES, load_package};
use std::{env, fs::File, io::Read, process::ExitCode};
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() != 1 {
        return Err("Usage: validate-package <package.orcpkg>".into());
    }
    let mut bytes = Vec::new();
    File::open(&args[0])?
        .take((MAX_ARCHIVE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    let package = load_package(&bytes)?;
    println!("{}", package.content_identity().to_hex());
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
