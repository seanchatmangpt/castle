use std::fs;
use std::process::Command;

fn castle_bin() -> &'static str { env!("CARGO_BIN_EXE_castle") }

fn run(args: &[&str]) -> (bool, serde_json::Value, String) {
    let output = Command::new(castle_bin()).args(args).output().expect("spawn castle");
    let stdout = String::from_utf8(output.stdout).expect("stdout UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr UTF-8");
    let parsed = if stdout.trim().is_empty() { serde_json::Value::Null } else {
        serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("stdout not JSON ({e}): {stdout}\nstderr:{stderr}"))
    };
    (output.status.success(), parsed, stderr)
}

fn fixture_files() -> (String, String, String) {
    let suffix = format!("{}-{}", std::process::id(), std::thread::current().name().unwrap_or("test").replace(':', "-"));
    let dir = std::env::temp_dir();
    let request_path = dir.join(format!("castle-v26-request-{suffix}.json"));
    let key_path = dir.join(format!("castle-v26-key-{suffix}.hex"));
    let evidence_dir = dir.join(format!("castle-v26-evidence-{suffix}"));
    let request = serde_json::json!({
        "cell_id": "cell:cli-proof",
        "evidence_dir": evidence_dir.to_string_lossy(),
        "subject": "system:cli-proof",
        "authority": "bounded-do",
        "o_star": {"subject":"system:cli-proof","admitted":true},
        "config_graph": {"zeroUnreceiptedActuation":true},
        "ontology": {"version":"26.8.18"},
        "process": {
            "id":"powl:cli-proof","goal_id":"goal:cli-proof",
            "activities":[{"id":"activity:echo","transition_id":"echo","predecessors":[]}]
        },
        "envelope": {
            "system_id":"system:cli-proof","allowed_transition_ids":["echo"],"max_steps":1,"expires_at_epoch_ms":10000
        },
        "allowed_authorities":["bounded-do"],
        "adapter_policy": {
            "adapter_id":"local-proof","provider":"local","workload_identity":"workload:cli-proof",
            "commands":{"echo":{"transition_id":"echo","program":"/bin/echo","args":["castle-v26.8.18"],"allowed_exit_codes":[0],"max_output_bytes":4096,"timeout_ms":2000}}
        }
    });
    fs::write(&request_path, serde_json::to_vec_pretty(&request).unwrap()).unwrap();
    fs::write(&key_path, "09".repeat(32)).unwrap();
    (
        request_path.to_string_lossy().into_owned(),
        key_path.to_string_lossy().into_owned(),
        evidence_dir.to_string_lossy().into_owned(),
    )
}

#[test]
fn compiled_cli_requires_construct_checkpoint_before_real_do() {
    let (request_path, key_path, evidence_dir) = fixture_files();
    let (construct_ok, construct, construct_err) = run(&[
        "construct", "manufacture", "--request-path", &request_path, "--signing-key-path", &key_path,
        "--key-id", "cli-runtime-key", "--format", "json",
    ]);
    assert!(construct_ok, "construct failed: {construct_err}");
    assert_eq!(construct["standing"], "ALIVE");
    let digest = construct["construct_digest"].as_str().unwrap().to_string();
    assert_eq!(digest.len(), 64);

    let (do_ok, executed, do_err) = run(&[
        "do", "execute", "--request-path", &request_path, "--signing-key-path", &key_path,
        "--key-id", "cli-runtime-key", "--expected-construct-digest", &digest,
        "--now-epoch-ms", "2", "--format", "json",
    ]);
    assert!(do_ok, "DO failed: {do_err}");
    assert_eq!(executed["standing"], "ALIVE");
    assert_eq!(executed["event_count"], 1);
    assert_eq!(executed["brce_prepare_receipt_digests"].as_array().unwrap().len(), 1);
    assert_eq!(executed["brce_outcome_receipt_digests"].as_array().unwrap().len(), 1);
    assert_eq!(executed["evidence_commit"]["standing"], "ALIVE");
    let evidence_path = executed["evidence_commit"]["path"].as_str().unwrap();
    assert!(std::path::Path::new(evidence_path).is_file());

    let (wrong_ok, _, wrong_err) = run(&[
        "do", "execute", "--request-path", &request_path, "--signing-key-path", &key_path,
        "--key-id", "cli-runtime-key", "--expected-construct-digest", &"f".repeat(64),
        "--now-epoch-ms", "2", "--format", "json",
    ]);
    assert!(!wrong_ok, "mismatched checkpoint unexpectedly actuated; stderr={wrong_err}");

    let _ = fs::remove_file(request_path);
    let _ = fs::remove_file(key_path);
    let _ = fs::remove_dir_all(evidence_dir);
}

#[test]
fn crypto_cli_reports_dual_identity_and_ed25519_alive() {
    let (ok, value, stderr) = run(&["crypto", "capabilities", "--format", "json"]);
    assert!(ok, "{stderr}");
    assert_eq!(value["identity"]["blake3_256"].as_str().unwrap().len(), 64);
    assert_eq!(value["identity"]["sha256"].as_str().unwrap().len(), 64);
    assert_eq!(value["qualification"]["standing"], "ALIVE");
}
