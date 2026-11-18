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
        return Err("Usage: allocate-recipe <recipe.xml> <inputs.json>".into());
    };
    let result = orchestrata_recipe_runtime::allocate_named(
        &read(Path::new(xml), 32 * 1024)?,
        &read(Path::new(inputs), 512 * 1024)?,
    )?;
    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
