use orchestrata_solver::promise_check::{
    DecisionBundle, PromiseRequest, check_promises, replay_decision,
};
use std::io::{self, Read};

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 1 || !matches!(args[0].as_str(), "check" | "replay") {
        return Err("usage: orchestrata-promise-check check|replay < input.json".into());
    }
    let mut bytes = Vec::new();
    io::stdin()
        .lock()
        .take(1_048_577)
        .read_to_end(&mut bytes)
        .map_err(|_| "input read failed")?;
    if bytes.len() > 1_048_576 {
        return Err("input exceeds 1 MiB".into());
    }
    let bundle = if args[0] == "check" {
        let request: PromiseRequest =
            serde_json::from_slice(&bytes).map_err(|_| "invalid request JSON")?;
        check_promises(&request).map_err(|e| format!("decision rejected: {e:?}"))?
    } else {
        let bundle: DecisionBundle =
            serde_json::from_slice(&bytes).map_err(|_| "invalid bundle JSON")?;
        replay_decision(&bundle).map_err(|e| format!("replay rejected: {e:?}"))?;
        bundle
    };
    serde_json::to_writer_pretty(io::stdout().lock(), &bundle)
        .map_err(|_| "output write failed")?;
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
