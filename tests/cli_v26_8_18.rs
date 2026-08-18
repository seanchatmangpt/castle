use std::process::Command;

fn castle_bin() -> &'static str {
    env!("CARGO_BIN_EXE_castle")
}

fn run(args: &[&str]) -> (bool, serde_json::Value, String) {
    let output = Command::new(castle_bin()).args(args).output().expect("failed to spawn castle binary");
    let stdout = String::from_utf8(output.stdout).expect("stdout UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr UTF-8");
    let parsed = if stdout.trim().is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("stdout was not JSON ({e}): {stdout}\nstderr: {stderr}"))
    };
    (output.status.success(), parsed, stderr)
}

#[test]
fn release_info_reports_v26_8_18_and_zero_unreceipted_do() {
    let (ok, value, stderr) = run(&["release", "info", "--format", "json"]);
    assert!(ok, "{stderr}");
    assert_eq!(value["release"], "26.8.18");
    let invariants = value["invariants"].as_array().unwrap();
    assert!(invariants.iter().any(|v| v == "CONSTRUCT != DO"));
}

#[test]
fn checked_in_global_manifest_qualifies_alive() {
    let path = format!("{}/configs/fortune5-v26.8.18.json", env!("CARGO_MANIFEST_DIR"));
    let (ok, value, stderr) = run(&[
        "deployment", "qualify", "--manifest-path", &path, "--now-epoch-ms", "1787080000000", "--format", "json",
    ]);
    assert!(ok, "{stderr}");
    assert_eq!(value["standing"], "ALIVE");
    assert_eq!(value["cells"], 5);
    assert!(value["findings"].as_array().unwrap().is_empty());
}

#[test]
fn provider_adapter_catalog_is_exposed() {
    let (ok, value, stderr) = run(&["deployment", "adapters", "--format", "json"]);
    assert!(ok, "{stderr}");
    assert_eq!(value["release"], "26.8.18");
    assert_eq!(value["adapters"].as_array().unwrap().len(), 5);
}

#[test]
fn protocol_catalogs_are_exposed_without_ambient_do() {
    let (mcp_ok, mcp, mcp_err) = run(&["protocol", "mcp", "--format", "json"]);
    assert!(mcp_ok, "{mcp_err}");
    let tools = mcp["tools"].as_array().unwrap();
    assert_eq!(tools.iter().filter(|tool| tool["consequential"] == true).count(), 1);

    let (a2a_ok, a2a, a2a_err) = run(&["protocol", "a2a", "--format", "json"]);
    assert!(a2a_ok, "{a2a_err}");
    assert_eq!(a2a["default_authority"], "CONSTRUCT_ONLY");
}
