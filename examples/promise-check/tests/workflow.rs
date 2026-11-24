use std::{
    io::Write,
    process::{Command, Stdio},
};
fn invoke(mode: &str, input: &[u8]) -> std::process::Output {
    let mut process = Command::new(env!("CARGO_BIN_EXE_orchestrata-promise-check"))
        .arg(mode)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = process.stdin.take().unwrap();
    // Large invalid input can be rejected before the writer finishes.
    let _ = stdin.write_all(input);
    drop(stdin);
    process.wait_with_output().unwrap()
}
#[test]
fn real_cli_exports_and_replays_identical_decision() {
    let input = include_bytes!("../../../backend_tests/fixtures/promise-check/maintenance.json");
    let result = invoke("check", input);
    assert!(result.status.success(), "{:?}", result.stderr);
    let replay = invoke("replay", &result.stdout);
    assert!(replay.status.success());
    assert_eq!(result.stdout, replay.stdout);
    let mut changed: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    changed["comparisons"][0]["plan"]["assigned"] = 200.into();
    let rejected = invoke("replay", &serde_json::to_vec(&changed).unwrap());
    assert!(!rejected.status.success());
    assert!(rejected.stdout.is_empty());
}
#[test]
fn invalid_unknown_and_oversized_input_produce_no_plan() {
    let original = include_bytes!("../../../backend_tests/fixtures/promise-check/maintenance.json");
    let mut value: serde_json::Value = serde_json::from_slice(original).unwrap();
    value["snapshot"]["jobs"][0]["routing"] = true.into();
    for bytes in [
        serde_json::to_vec(&value).unwrap(),
        b"{bad".to_vec(),
        vec![b' '; 1_048_577],
    ] {
        let output = invoke("check", &bytes);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
    }
}
