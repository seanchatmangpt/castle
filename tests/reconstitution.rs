use std::fmt::Write as _;

use castle::admit_empire_reconstitution_for_construct;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(string) => serde_json::to_string(string).expect("serialize string"),
        Value::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Object(map) => {
            let mut keys = map.keys().collect::<Vec<_>>();
            keys.sort();
            let fields = keys
                .into_iter()
                .map(|key| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(key).expect("serialize key"),
                        canonical_json(&map[key])
                    )
                })
                .collect::<Vec<_>>();
            format!("{{{}}}", fields.join(","))
        }
    }
}

fn digest(value: &Value) -> String {
    let mut hasher = Sha256::new();
    hasher.update(canonical_json(value).as_bytes());
    hasher.update(b"\n");
    let mut output = String::with_capacity(64);
    for byte in hasher.finalize() {
        write!(&mut output, "{byte:02x}").expect("write digest");
    }
    output
}

fn capability(id: &str, disposition: &str) -> Value {
    json!({
        "id": id,
        "disposition": disposition,
        "evidence_ids": [format!("evidence:{id}")],
        "observable_surfaces": ["exit_code", "diagnostics"]
    })
}

fn valid_core() -> Value {
    json!({
        "schema": "ggen.legacy.authority-vacuum.admission.v1",
        "study_id": "OSTAR-EMPIRE-001",
        "authority_state": "ADMITTED_CANDIDATE",
        "claim_ceiling": "SCHEMA_VALIDATED",
        "semantic_scope": {
            "mode": "bounded-observable-surfaces",
            "universal_equivalence_claimed": false
        },
        "authority": {
            "id": "empire-reconstitution-authority",
            "digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        },
        "observation_receipt_digest": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "capabilities": [
            capability("ontostar-admission-manufacture", "PRESERVED"),
            capability("ostar-cli-load", "REFUSED"),
            capability("ostar-codemanufactory-manufacture", "SUBSUMED"),
            capability("ostar-dteam-process-intelligence", "REPLACED"),
            capability("ostar-ggen-projection-contract", "ARCHIVED"),
            capability("ostar-governance-pipeline", "PRESERVED")
        ],
        "disposition_coverage": ["ARCHIVED", "PRESERVED", "REFUSED", "REPLACED", "SUBSUMED"],
        "standing": "PARTIAL_ALIVE",
        "actuation_authority": false
    })
}

fn envelope(core: &Value) -> Value {
    let artifact_digest = digest(core);
    json!({
        "core": core,
        "receipt": {
            "schema": "ggen.legacy.authority-vacuum.receipt.v1",
            "algorithm": "SHA-256",
            "artifact_digest": artifact_digest,
            "epistemic_class": "CONSTRUCTED",
            "authority": false,
            "parent_digests": [
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            ]
        }
    })
}

#[test]
fn no_authority_observation_is_refused_before_construct() {
    let observation = json!({
        "core": {
            "schema": "ggen.legacy.authority-vacuum.observation.v1",
            "study_id": "OSTAR-EMPIRE-001",
            "authority_state": "NO_AUTHORITY"
        },
        "receipt": {
            "schema": "ggen.legacy.authority-vacuum.receipt.v1",
            "authority": false
        }
    });
    let refusal = admit_empire_reconstitution_for_construct(&observation.to_string())
        .expect_err("NO_AUTHORITY cannot cross the CASTLE construct gate");
    assert_eq!(refusal.code, "ADMISSION_SCHEMA_INVALID");
}

#[test]
fn bounded_complete_contract_is_construct_only() {
    let document = envelope(&valid_core());
    let admission = admit_empire_reconstitution_for_construct(&document.to_string())
        .expect("bounded fixture should admit for CONSTRUCT");
    assert_eq!(admission.study_id(), "OSTAR-EMPIRE-001");
    assert_eq!(admission.capabilities().len(), 6);
    assert!(!admission.may_actuate());
    let o_star = admission.to_o_star_value();
    assert_eq!(o_star["actuation_authority"], false);
    assert_eq!(o_star["claim_ceiling"], "SCHEMA_VALIDATED");
}

#[test]
fn rice_unbounded_claim_is_refused() {
    let mut core = valid_core();
    core["semantic_scope"]["universal_equivalence_claimed"] = json!(true);
    let refusal = admit_empire_reconstitution_for_construct(&envelope(&core).to_string())
        .expect_err("unbounded semantic claim must fail closed");
    assert_eq!(refusal.code, "RICE_SCOPE_UNBOUNDED");
}

#[test]
fn all_alive_scope_is_a_failure() {
    let mut core = valid_core();
    for capability in core["capabilities"]
        .as_array_mut()
        .expect("capability array")
    {
        if capability["disposition"] == "REFUSED" {
            capability["disposition"] = json!("PRESERVED");
        }
    }
    core["disposition_coverage"] = json!(["ARCHIVED", "PRESERVED", "REPLACED", "SUBSUMED"]);
    let refusal = admit_empire_reconstitution_for_construct(&envelope(&core).to_string())
        .expect_err("a required-refusal trial with no refusal must re-scope");
    assert_eq!(refusal.code, "SCOPING_FAILURE_NO_REFUSAL");
}

#[test]
fn receipt_tampering_is_refused() {
    let mut document = envelope(&valid_core());
    document["core"]["authority"]["id"] = json!("tampered-authority");
    let refusal = admit_empire_reconstitution_for_construct(&document.to_string())
        .expect_err("changed core must invalidate the receipt");
    assert_eq!(refusal.code, "ADMISSION_RECEIPT_INVALID");
}

#[test]
fn unknown_disposition_is_refused_by_name() {
    let mut core = valid_core();
    core["capabilities"][0]["disposition"] = json!("UNKNOWN");
    let refusal = admit_empire_reconstitution_for_construct(&envelope(&core).to_string())
        .expect_err("UNKNOWN is not a final disposition");
    assert_eq!(refusal.code, "ADMISSION_DISPOSITION_UNKNOWN");
}
