//! Subprocess tests for the `castle payments` CLI noun.
//!
//! Chicago-style: the real compiled binary, real Ed25519 custodian signatures,
//! real fsynced claim/nonce/ledger/BRCE state in temp dirs. No mocks.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use castle::sa2a_security::{
    ActuationCertificate, CertificateSignature, KeyRecord, KeyState, PreparedEffect, SignatureAlgorithm,
};
use ed25519_dalek::{Signer, SigningKey};
use serde_json::Value;

const AUDIENCE: &str = "actuator:payments";
const PRINCIPAL: &str = "principal:procurement-agent";
const PAYER: &str = "acct:treasury";
const PAYEE: &str = "acct:supplier-9821";
const RECEIPT_SEED_HEX: &str = "6363636363636363636363636363636363636363636363636363636363636363";
const NOW_MS: i64 = 1_000;

fn castle_bin() -> &'static str {
    env!("CARGO_BIN_EXE_castle")
}

fn fixture(name: &str) -> String {
    format!("{}/fixtures/payments/cli/{name}", env!("CARGO_MANIFEST_DIR"))
}

fn schema(name: &str) -> Value {
    let path = format!("{}/schemas/payments/{name}.schema.json", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_slice(&std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"))).unwrap()
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_dir(tag: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("castle-cli-pay-{}-{n}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run(args: &[&str]) -> Output {
    Command::new(castle_bin()).args(args).arg("--format").arg("json").output().expect("spawn castle")
}

fn combined(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn json_of(o: &Output) -> Value {
    assert!(o.status.success(), "non-zero exit: {}", combined(o));
    serde_json::from_slice(&o.stdout).unwrap_or_else(|e| panic!("stdout not JSON ({e}): {}", combined(o)))
}

/// Real custodians: three Ed25519 keys, threshold 2.
fn custodians() -> Vec<(&'static str, SigningKey)> {
    ["mac", "phone", "hsm"].iter().enumerate().map(|(i, n)| (*n, SigningKey::from_bytes(&[(i as u8) + 11; 32]))).collect()
}

fn write_registry(path: &Path) {
    let records: Vec<KeyRecord> = custodians()
        .iter()
        .map(|(n, k)| KeyRecord {
            key_id: format!("key-{n}"),
            custodian_id: format!("device:{n}"),
            algorithm: SignatureAlgorithm::Ed25519,
            public_key: k.verifying_key().to_bytes().to_vec(),
            state: KeyState::Active,
            not_before_ms: 0,
            expires_at_ms: 10_000_000,
            revocation_epoch: 0,
        })
        .collect();
    std::fs::write(path, serde_json::to_vec(&records).unwrap()).unwrap();
}

fn write_cert(path: &Path, effect: &PreparedEffect, nonce: &str) {
    let mut cert = ActuationCertificate {
        version: 1,
        effect_digest: effect.digest().unwrap(),
        principal: effect.principal.clone(),
        policy_epoch: 7,
        revocation_epoch: 0,
        generation: 1,
        nonce: nonce.to_string(),
        not_before_ms: 0,
        expires_at_ms: 100_000,
        audience: AUDIENCE.to_string(),
        threshold: 2,
        signatures: vec![],
    };
    let msg = cert.signing_message().unwrap();
    for (name, key) in custodians().iter().filter(|(n, _)| *n == "mac" || *n == "phone") {
        cert.signatures.push(CertificateSignature {
            key_id: format!("key-{name}"),
            algorithm: SignatureAlgorithm::Ed25519,
            signature: key.sign(&msg).to_bytes().to_vec(),
        });
    }
    std::fs::write(path, serde_json::to_vec(&cert).unwrap()).unwrap();
}

struct World {
    dir: PathBuf,
}

impl World {
    fn new(tag: &str) -> Self {
        let dir = unique_dir(tag);
        write_registry(&dir.join("registry.json"));
        Self { dir }
    }
    fn p(&self, name: &str) -> String {
        self.dir.join(name).to_string_lossy().into_owned()
    }
    fn prepare(&self, amount: &str, obligation: &str) -> (PreparedEffect, String) {
        let out = json_of(&run(&[
            "payments", "prepare", "--principal", PRINCIPAL, "--payer", PAYER, "--payee", PAYEE, "--amount-minor", amount,
            "--currency", "USD", "--obligation-id", obligation, "--purpose", "invoice-payment",
        ]));
        assert_eq!(out["grants_authority"], Value::Bool(false));
        let effect: PreparedEffect = serde_json::from_value(out["effect"].clone()).unwrap();
        let digest = out["effect_digest"].as_str().unwrap().to_string();
        assert_eq!(effect.digest().unwrap(), digest);
        std::fs::write(self.dir.join("effect.json"), serde_json::to_vec(&effect).unwrap()).unwrap();
        (effect, digest)
    }
    fn execute(&self, cert_file: &str, reference_ledger: bool) -> Output {
        let mut args = vec![
            "payments".to_string(), "execute".into(),
            "--effect-path".into(), self.p("effect.json"),
            "--certificate-path".into(), self.p(cert_file),
            "--registry-path".into(), self.p("registry.json"),
            "--policy-path".into(), fixture("policy.json"),
            "--state-dir".into(), self.p("state"),
            "--ledger-dir".into(), self.p("ledger"),
            "--opening-path".into(), fixture("opening.json"),
            "--audience".into(), AUDIENCE.into(),
            "--policy-epoch".into(), "7".into(),
            "--revocation-epoch".into(), "0".into(),
            "--generation".into(), "1".into(),
            "--now-ms".into(), NOW_MS.to_string(),
            "--receipt-key-id".into(), "receipt-key".into(),
            "--receipt-seed-hex".into(), RECEIPT_SEED_HEX.into(),
            "--allowed-authority".into(), AUDIENCE.into(),
        ];
        if reference_ledger {
            args.push("--reference-ledger".into());
        }
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        run(&refs)
    }
    fn explain(&self, digest: &str) -> Output {
        run(&["payments", "explain", "--effect-digest", digest, "--state-dir", &self.p("state"), "--ledger-dir", &self.p("ledger"), "--reference-ledger"])
    }
}

/// Minimal JSON Schema (draft 2020-12 subset) checker so schemas are exercised
/// against real serialized values without adding a dependency.
fn validate(schema_root: &Value, schema: &Value, value: &Value, at: &str, errs: &mut Vec<String>) {
    if let Some(r) = schema.get("$ref").and_then(Value::as_str) {
        let name = r.strip_prefix("#/$defs/").expect("local $defs ref");
        return validate(schema_root, &schema_root["$defs"][name], value, at, errs);
    }
    if let Some(t) = schema.get("type") {
        let allowed: Vec<&str> = match t {
            Value::String(s) => vec![s.as_str()],
            Value::Array(a) => a.iter().filter_map(Value::as_str).collect(),
            _ => vec![],
        };
        let ok = allowed.iter().any(|t| match *t {
            "string" => value.is_string(),
            "integer" => value.is_u64() || value.is_i64(),
            "object" => value.is_object(),
            "array" => value.is_array(),
            "null" => value.is_null(),
            "boolean" => value.is_boolean(),
            _ => false,
        });
        if !ok {
            errs.push(format!("{at}: type {allowed:?} != {value}"));
            return;
        }
    }
    if let Some(e) = schema.get("enum").and_then(Value::as_array) {
        if !e.contains(value) {
            errs.push(format!("{at}: {value} not in enum"));
        }
    }
    if let Some(c) = schema.get("const") {
        if c != value {
            errs.push(format!("{at}: {value} != const {c}"));
        }
    }
    if let (Some(min), Some(n)) = (schema.get("minLength").and_then(Value::as_u64), value.as_str()) {
        if (n.len() as u64) < min {
            errs.push(format!("{at}: shorter than {min}"));
        }
    }
    if let (Some(min), Some(n)) = (schema.get("minimum").and_then(Value::as_i64), value.as_i64()) {
        if n < min {
            errs.push(format!("{at}: below minimum {min}"));
        }
    }
    if let Some(obj) = value.as_object() {
        for req in schema.get("required").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str) {
            if !obj.contains_key(req) {
                errs.push(format!("{at}: missing required key {req}"));
            }
        }
        let props = schema.get("properties").and_then(Value::as_object);
        for (k, v) in obj {
            if let Some(names) = schema.get("propertyNames").and_then(|p| p.get("enum")).and_then(Value::as_array) {
                if !names.contains(&Value::String(k.clone())) {
                    errs.push(format!("{at}: key {k} not in propertyNames enum"));
                }
            }
            match (props.and_then(|p| p.get(k)), schema.get("additionalProperties")) {
                (Some(s), _) => validate(schema_root, s, v, &format!("{at}.{k}"), errs),
                (None, Some(Value::Bool(false))) => errs.push(format!("{at}: unexpected key {k}")),
                (None, Some(s)) if s.is_object() => validate(schema_root, s, v, &format!("{at}.{k}"), errs),
                (None, _) => {}
            }
        }
    }
    if let (Some(items), Some(arr)) = (schema.get("items"), value.as_array()) {
        for (i, v) in arr.iter().enumerate() {
            validate(schema_root, items, v, &format!("{at}[{i}]"), errs);
        }
    }
}

fn assert_conforms(schema_name: &str, value: &Value) {
    let s = schema(schema_name);
    let mut errs = Vec::new();
    validate(&s, &s, value, "$", &mut errs);
    assert!(errs.is_empty(), "{schema_name} violations: {errs:?}");
}

fn only_file(dir: &Path, suffix: &str) -> PathBuf {
    let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.ends_with(suffix)))
        .collect();
    assert_eq!(found.len(), 1, "expected exactly one *{suffix} in {}", dir.display());
    found.remove(0)
}

#[test]
fn prepare_refuses_non_integer_amount_and_unknown_currency() {
    for (amount, currency, code) in [
        ("1.50", "USD", "PAYMENT_AMOUNT_NOT_INTEGER"),
        ("007", "USD", "PAYMENT_AMOUNT_NOT_INTEGER"),
        ("0", "USD", "PAYMENT_AMOUNT_ZERO"),
        ("100", "XXX", "PAYMENT_CURRENCY_UNSUPPORTED"),
    ] {
        let out = run(&[
            "payments", "prepare", "--principal", PRINCIPAL, "--payer", PAYER, "--payee", PAYEE, "--amount-minor", amount,
            "--currency", currency, "--obligation-id", "o", "--purpose", "p",
        ]);
        assert!(!out.status.success(), "{amount} {currency} accepted");
        assert!(combined(&out).contains(code), "{amount} {currency}: {}", combined(&out));
    }
}

#[test]
fn full_lifecycle_settles_once_explains_recovers_and_reconciles() {
    let w = World::new("lifecycle");
    let (_effect, digest) = w.prepare("470000", "invoice-9821");
    let effect: PreparedEffect = serde_json::from_slice(&std::fs::read(w.dir.join("effect.json")).unwrap()).unwrap();
    write_cert(&w.dir.join("cert1.json"), &effect, "n-1");
    write_cert(&w.dir.join("cert2.json"), &effect, "n-2");

    // Without --reference-ledger the file ledger is refused and no state is created.
    let refused = w.execute("cert1.json", false);
    assert!(!refused.status.success());
    assert!(combined(&refused).contains("REFUSED:REFERENCE_LEDGER_NOT_CONFIRMED"), "{}", combined(&refused));
    assert!(!w.dir.join("state").exists(), "refusal must precede any state write");
    assert!(!w.dir.join("ledger").exists());

    // Settles.
    let settled = json_of(&w.execute("cert1.json", true));
    assert_eq!(settled["standing"], "Settled");
    assert_eq!(settled["effect_digest"], digest.as_str());
    assert!(settled["construct_digest"].is_string());
    assert!(settled["ocel_receipt_digest"].is_string());
    assert_eq!(settled["brce_prepare_receipt_digests"].as_array().unwrap().len(), 2);
    assert_eq!(settled["brce_outcome_receipt_digests"].as_array().unwrap().len(), 2);
    assert_eq!(settled["ledger_entry"]["seq"], 1);
    assert_eq!(settled["ledger_entry"]["amount_minor"], 470_000);
    assert_eq!(settled["ledger_entry"]["debit_account"], PAYER);
    assert_eq!(settled["ledger_entry"]["credit_account"], PAYEE);

    // Second execute of the same effect with a fresh certificate and nonce is refused.
    let again = w.execute("cert2.json", true);
    assert!(!again.status.success());
    assert!(combined(&again).contains("PAYMENT_ALREADY_SETTLED"), "{}", combined(&again));
    assert_eq!(std::fs::read_dir(w.dir.join("ledger")).unwrap().filter(|e| e.as_ref().unwrap().file_name().to_string_lossy().ends_with(".entry.json")).count(), 1);

    // Explain: every board field, from persisted state only.
    let ex = json_of(&w.explain(&digest));
    assert_eq!(ex["principal"], PRINCIPAL);
    assert_eq!(ex["payer"], PAYER);
    assert_eq!(ex["payee"], PAYEE);
    assert_eq!(ex["amount_minor"], "470000");
    assert_eq!(ex["amount_decimal"], "4700.00");
    assert_eq!(ex["currency"], "USD");
    assert_eq!(ex["obligation_id"], "invoice-9821");
    assert_eq!(ex["purpose"], "invoice-payment");
    assert_eq!(ex["authority_audience"], AUDIENCE);
    assert_eq!(ex["verified_custodian_ids"], serde_json::json!(["device:mac", "device:phone"]));
    assert_eq!(ex["construct_digest"], settled["construct_digest"]);
    assert_eq!(ex["ledger_seq"], 1);
    assert_eq!(ex["claim_state"], "executed");

    // Recover: 2 BRCE records, both complete.
    let rec = json_of(&run(&["payments", "recover", "--state-dir", &w.p("state")]));
    assert_eq!(rec["complete_count"], 2);
    assert_eq!(rec["prepared_without_outcome_count"], 0);

    // Reconcile refuses an already-Executed claim.
    let not_rec = run(&["payments", "reconcile", "--effect-digest", &digest, "--state-dir", &w.p("state"), "--ledger-dir", &w.p("ledger"), "--reference-ledger"]);
    assert!(!not_rec.status.success());
    assert!(combined(&not_rec).contains("PAYMENT_NOT_RECONCILABLE"), "{}", combined(&not_rec));

    // Simulate a crash that left the claim UnknownOutcome; reconcile resolves from the ledger.
    let claim_path = only_file(&w.dir.join("state/claims"), ".claim.json");
    let mut claim: Value = serde_json::from_slice(&std::fs::read(&claim_path).unwrap()).unwrap();
    claim["state"] = Value::String("unknown_outcome".into());
    std::fs::write(&claim_path, serde_json::to_vec(&claim).unwrap()).unwrap();
    let rc = json_of(&run(&["payments", "reconcile", "--effect-digest", &digest, "--state-dir", &w.p("state"), "--ledger-dir", &w.p("ledger"), "--reference-ledger"]));
    assert_eq!(rc["resolution"], "settled");
    assert_eq!(json_of(&w.explain(&digest))["claim_state"], "executed");
}

#[test]
fn reconcile_and_explain_never_fabricate_a_missing_ledger() {
    let w = World::new("noledger");
    let digest = format!("sha256:{}", "a".repeat(64));
    for verb in ["reconcile", "explain"] {
        let out = run(&["payments", verb, "--effect-digest", &digest, "--state-dir", &w.p("state"), "--ledger-dir", &w.p("ledger"), "--reference-ledger"]);
        assert!(!out.status.success());
        assert!(combined(&out).contains("PAYMENT_LEDGER_NOT_FOUND"), "{verb}: {}", combined(&out));
        assert!(!w.dir.join("ledger").exists(), "{verb} created a ledger");
    }
    let no_flag = run(&["payments", "explain", "--effect-digest", &digest, "--state-dir", &w.p("state"), "--ledger-dir", &w.p("ledger")]);
    assert!(combined(&no_flag).contains("REFERENCE_LEDGER_NOT_CONFIRMED"));
}

#[test]
fn schemas_match_real_serialized_values() {
    let w = World::new("schemas");
    let (effect, digest) = w.prepare("250", "inv-schema");
    assert_conforms("payment-effect-subject-v1", &effect.subject);
    assert_conforms("payment-effect-payload-v1", &effect.payload);
    let policy: Value = serde_json::from_slice(&std::fs::read(fixture("policy.json")).unwrap()).unwrap();
    assert_conforms("spend-policy-v1", &policy);

    write_cert(&w.dir.join("cert.json"), &effect, "n-s");
    let _ = json_of(&w.execute("cert.json", true));
    let claim: Value = serde_json::from_slice(&std::fs::read(only_file(&w.dir.join("state/claims"), ".claim.json")).unwrap()).unwrap();
    assert_eq!(claim["effect_digest"], digest.as_str());
    assert_conforms("claim-v1", &claim);
    let entry: Value = serde_json::from_slice(&std::fs::read(only_file(&w.dir.join("ledger"), ".entry.json")).unwrap()).unwrap();
    assert_conforms("ledger-entry-v1", &entry);

    // The checker rejects a real value with an extra key and one with a missing key.
    let mut extra = entry.clone();
    extra["surprise"] = Value::Bool(true);
    let s = schema("ledger-entry-v1");
    let mut errs = Vec::new();
    validate(&s, &s, &extra, "$", &mut errs);
    assert!(!errs.is_empty());
    let mut missing = claim;
    missing.as_object_mut().unwrap().remove("state");
    let s = schema("claim-v1");
    let mut errs = Vec::new();
    validate(&s, &s, &missing, "$", &mut errs);
    assert!(!errs.is_empty());
}

#[test]
fn cli_sources_contain_no_float_amount_handling() {
    for file in ["payments_handlers.rs", "payments_routes.rs"] {
        let path = format!("{}/src/bin/castle/verbs/{file}", env!("CARGO_MANIFEST_DIR"));
        let src = std::fs::read_to_string(&path).unwrap();
        for banned in ["f64", "f32", "parse::<f", "as f6", "as f3", "to_f64", "as_f64", "from_f64"] {
            assert!(!src.contains(banned), "{file} contains `{banned}`");
        }
    }
}
