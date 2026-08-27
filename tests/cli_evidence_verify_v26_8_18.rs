use std::fs;
use std::process::Command;

fn castle_bin() -> &'static str { env!("CARGO_BIN_EXE_castle") }

fn run(args: &[&str]) -> (bool, serde_json::Value, String) {
    let output = Command::new(castle_bin()).args(args).output().expect("spawn castle");
    let stdout = String::from_utf8(output.stdout).expect("stdout UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr UTF-8");
    let parsed = if stdout.trim().is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_str(&stdout)
            .unwrap_or_else(|error| panic!("stdout not JSON ({error}): {stdout}\nstderr:{stderr}"))
    };
    (output.status.success(), parsed, stderr)
}

#[test]
fn exact_post_do_evidence_is_content_address_verified_without_a_second_do() {
    let suffix = format!("{}-evidence-verify", std::process::id());
    let dir = std::env::temp_dir();
    let request_path = dir.join(format!("castle-evidence-request-{suffix}.json"));
    let key_path = dir.join(format!("castle-evidence-key-{suffix}.hex"));
    let evidence_dir = dir.join(format!("castle-evidence-store-{suffix}"));

    let request = serde_json::json!({
        "cell_id": "cell:evidence-verify",
        "evidence_dir": evidence_dir.to_string_lossy(),
        "subject": "system:evidence-verify",
        "authority": "bounded-do",
        "o_star": {"subject":"system:evidence-verify","admitted":true},
        "config_graph": {"zeroUnreceiptedActuation":true},
        "ontology": {"version":"26.8.18"},
        "process": {
            "id":"powl:evidence-verify",
            "goal_id":"goal:evidence-verify",
            "activities":[{"id":"activity:echo","transition_id":"echo","predecessors":[]}]
        },
        "envelope": {
            "system_id":"system:evidence-verify",
            "allowed_transition_ids":["echo"],
            "max_steps":1,
            "expires_at_epoch_ms":10000
        },
        "allowed_authorities":["bounded-do"],
        "adapter_policy": {
            "adapter_id":"local-proof",
            "provider":"local",
            "workload_identity":"workload:evidence-verify",
            "commands":{"echo":{
                "transition_id":"echo",
                "program":"/bin/echo",
                "args":["castle-evidence-verify"],
                "allowed_exit_codes":[0],
                "max_output_bytes":4096,
                "timeout_ms":2000
            }}
        }
    });

    fs::write(&request_path, serde_json::to_vec_pretty(&request).unwrap()).unwrap();
    fs::write(&key_path, "09".repeat(32)).unwrap();

    let request_path_s = request_path.to_string_lossy().into_owned();
    let key_path_s = key_path.to_string_lossy().into_owned();

    let (construct_ok, construct, construct_err) = run(&[
        "construct",
        "manufacture",
        "--request-path",
        &request_path_s,
        "--signing-key-path",
        &key_path_s,
        "--key-id",
        "evidence-verify-key",
        "--format",
        "json",
    ]);
    assert!(construct_ok, "construct failed: {construct_err}");
    let construct_digest = construct["construct_digest"].as_str().unwrap().to_string();

    let (do_ok, executed, do_err) = run(&[
        "do",
        "execute",
        "--request-path",
        &request_path_s,
        "--signing-key-path",
        &key_path_s,
        "--key-id",
        "evidence-verify-key",
        "--expected-construct-digest",
        &construct_digest,
        "--now-epoch-ms",
        "2",
        "--format",
        "json",
    ]);
    assert!(do_ok, "DO failed: {do_err}");

    let evidence_path = executed["evidence_commit"]["path"].as_str().unwrap().to_string();
    let (verify_ok, verified, verify_err) = run(&[
        "evidence",
        "verify",
        "--evidence-path",
        &evidence_path,
        "--format",
        "json",
    ]);
    assert!(verify_ok, "evidence verification failed: {verify_err}");
    assert_eq!(verified["standing"], "ALIVE");
    assert_eq!(verified["record"]["construct_digest"], construct_digest);
    assert_eq!(verified["record"]["subject"], "system:evidence-verify");
    assert_eq!(verified["record"]["event_count"], 1);
    assert_eq!(verified["record"]["brce_prepare_receipt_digests"].as_array().unwrap().len(), 1);
    assert_eq!(verified["record"]["brce_outcome_receipt_digests"].as_array().unwrap().len(), 1);

    // Renaming the same bytes breaks the content-address contract.
    let wrong_path = evidence_dir.join("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff.json");
    fs::copy(&evidence_path, &wrong_path).unwrap();
    let wrong_path_s = wrong_path.to_string_lossy().into_owned();
    let (wrong_ok, _, wrong_err) = run(&[
        "evidence",
        "verify",
        "--evidence-path",
        &wrong_path_s,
        "--format",
        "json",
    ]);
    assert!(!wrong_ok, "renamed evidence unexpectedly verified: {wrong_err}");

    let _ = fs::remove_file(request_path);
    let _ = fs::remove_file(key_path);
    let _ = fs::remove_dir_all(evidence_dir);
}
