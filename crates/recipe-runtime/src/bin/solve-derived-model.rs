use std::{env, error::Error, fs::File, io::Read, path::Path, process::ExitCode};
fn read(path: &Path, limit: u64) -> std::io::Result<String> {
    let mut text = String::new();
    File::open(path)?
        .take(limit + 1)
        .read_to_string(&mut text)?;
    Ok(text)
}
fn run() -> Result<(), Box<dyn Error>> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    let [xml, inputs] = args.as_slice() else {
        return Err("Usage: solve-derived-model <model.xml> <records.json>".into());
    };
    let result = orchestrata_recipe_runtime::mathematical::run_derived_model(
        &read(Path::new(xml), 32768)?,
        &read(Path::new(inputs), 524288)?,
    )?;
    println!("{}", serde_json::to_string(&result)?);
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
