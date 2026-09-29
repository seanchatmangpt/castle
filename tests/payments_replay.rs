mod common;
use common::payments::*;

use std::collections::BTreeMap;
use std::path::Path;

use castle::payments::replay::*;
use castle::payments::*;

fn hash_dir(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(base, &p, out);
            } else {
                out.insert(p.strip_prefix(base).unwrap().display().to_string(), std::fs::read(&p).unwrap());
            }
        }
    }
    walk(dir, dir, &mut out);
    out
}

fn admit_j(fx: &Fixture, j: &AdmissionJournal, amount: &str, obl: &str, nonce: &str) -> (String, PaymentAdmission) {
    let e = fx.effect(amount, obl);
    let c = fx.cert(&e, nonce, &["mac", "phone"]);
    let a = admit_payment_journaled(e, &c, &fx.admission_ctx(), j).expect("admits");
    (a.effect().digest().to_string(), a)
}

fn record_path(fx: &Fixture, digest: &str) -> std::path::PathBuf {
    fx.dir.join("journal").join(format!("{}.admission.json", digest.trim_start_matches("sha256:")))
}

fn edit(fx: &Fixture, digest: &str, f: impl FnOnce(&mut serde_json::Value)) {
    let p = record_path(fx, digest);
    let mut v: serde_json::Value = serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();
    f(&mut v);
    std::fs::write(&p, serde_json::to_vec(&v).unwrap()).unwrap();
}

fn diverged(v: ReplayVerdict) -> String {
    match v {
        ReplayVerdict::Diverged { reason } => reason,
        other => panic!("expected Diverged, got {other:?}"),
    }
}

#[test]
fn journaled_admission_replays_reproduced_with_stable_decision_digest() {
    let fx = Fixture::new("replay-ok");
    let j = AdmissionJournal::open(fx.dir.join("journal")).unwrap();
    let (d, _a) = admit_j(&fx, &j, "1000", "inv-1", "n-1");
    let v1 = replay_admission(&j, &d).unwrap();
    let v2 = replay_admission(&j, &d).unwrap();
    assert_eq!(v1, v2);
    match v1 {
        ReplayVerdict::Reproduced { effect_digest, decision_digest } => {
            assert_eq!(effect_digest, d);
            assert!(decision_digest.starts_with("sha256:"));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn tampered_amount_diverges_on_digest() {
    let fx = Fixture::new("replay-amt");
    let j = AdmissionJournal::open(fx.dir.join("journal")).unwrap();
    let (d, _a) = admit_j(&fx, &j, "1000", "inv-1", "n-1");
    edit(&fx, &d, |v| v["prepared"]["payload"]["amount"] = "999999".into());
    assert_eq!(diverged(replay_admission(&j, &d).unwrap()), REPLAY_DIGEST_MISMATCH);
}

#[test]
fn swapped_policy_cap_diverges_with_cap_refusal() {
    let fx = Fixture::new("replay-cap");
    let j = AdmissionJournal::open(fx.dir.join("journal")).unwrap();
    let (d, _a) = admit_j(&fx, &j, "1000", "inv-1", "n-1");
    edit(&fx, &d, |v| {
        v["policy"]["principals"][PRINCIPAL]["per_effect_cap"]["USD"] = 10.into();
    });
    assert_eq!(diverged(replay_admission(&j, &d).unwrap()), "REFUSED:PAYMENT_AMOUNT_EXCEEDS_CAP");
}

#[test]
fn tampered_certificate_signature_diverges() {
    let fx = Fixture::new("replay-sig");
    let j = AdmissionJournal::open(fx.dir.join("journal")).unwrap();
    let (d, _a) = admit_j(&fx, &j, "1000", "inv-1", "n-1");
    edit(&fx, &d, |v| {
        let sig = &mut v["certificate"]["signatures"][0]["signature"];
        let mut bytes: Vec<u8> = serde_json::from_value(sig.clone()).unwrap();
        bytes[0] ^= 0xff;
        *sig = serde_json::to_value(bytes).unwrap();
    });
    let reason = diverged(replay_admission(&j, &d).unwrap());
    assert!(reason.starts_with("REFUSED:"), "{reason}");
    assert_eq!(reason, "REFUSED:InvalidSignature");
}

#[test]
fn replay_leaves_every_store_byte_identical() {
    let fx = Fixture::new("replay-ro");
    let j = AdmissionJournal::open(fx.dir.join("journal")).unwrap();
    let (d, _a) = admit_j(&fx, &j, "1000", "inv-1", "n-1");
    let snap = |fx: &Fixture| {
        (
            hash_dir(&fx.dir.join("claims")),
            hash_dir(&fx.dir.join("ledger")),
            hash_dir(&fx.dir.join("nonces")),
            hash_dir(&fx.dir.join("journal")),
        )
    };
    let before = snap(&fx);
    assert!(!before.0.is_empty() && !before.2.is_empty());
    for _ in 0..3 {
        assert!(matches!(replay_admission(&j, &d).unwrap(), ReplayVerdict::Reproduced { .. }));
    }
    assert_eq!(before, snap(&fx));
}

#[test]
fn replay_needs_only_the_journal_path() {
    let journal_dir;
    let digest;
    {
        let fx = Fixture::new("replay-indep");
        journal_dir = fx.dir.join("journal");
        let j = AdmissionJournal::open(&journal_dir).unwrap();
        let (d, _a) = admit_j(&fx, &j, "1000", "inv-1", "n-1");
        digest = d;
        // move journal out of the fixture dir so it is the only survivor
        let keep = common::payments::unique_dir("replay-keep");
        for e in std::fs::read_dir(&journal_dir).unwrap().flatten() {
            std::fs::copy(e.path(), keep.join(e.file_name())).unwrap();
        }
        journal_dir_swap(&keep, &mut |p| {
            let _ = std::fs::remove_dir_all(&fx.dir);
            let j2 = AdmissionJournal::open(p).unwrap();
            assert!(matches!(replay_admission(&j2, &digest).unwrap(), ReplayVerdict::Reproduced { .. }));
        });
    }
}

fn journal_dir_swap(p: &Path, f: &mut dyn FnMut(&Path)) {
    f(p);
}

#[test]
fn refused_admission_writes_no_record() {
    let fx = Fixture::new("replay-refused");
    let j = AdmissionJournal::open(fx.dir.join("journal")).unwrap();
    let e = fx.effect("999999999", "inv-big");
    let d = e.digest().unwrap();
    let c = fx.cert(&e, "n-1", &["mac", "phone"]);
    let r = admit_payment_journaled(e, &c, &fx.admission_ctx(), &j);
    assert_eq!(r.unwrap_err(), "REFUSED:PAYMENT_AMOUNT_EXCEEDS_CAP");
    assert!(!record_path(&fx, &d).exists());
    assert!(hash_dir(&fx.dir.join("journal")).is_empty());
    assert_eq!(replay_admission(&j, &d).unwrap_err(), REPLAY_RECORD_MISSING);
}

#[tokio::test]
async fn replay_after_claim_final_is_still_reproduced() {
    let fx = Fixture::new("replay-final");
    let j = AdmissionJournal::open(fx.dir.join("journal")).unwrap();
    let (d, a) = admit_j(&fx, &j, "1000", "inv-1", "n-1");
    let before = replay_admission(&j, &d).unwrap();
    let exec = execute_payment(a, &fx.exec_ctx()).await.unwrap();
    assert_eq!(exec.standing, PaymentStanding::Settled);
    fx.claims.transition(&d, &[ClaimState::Executed], ClaimState::Final, None, "FINAL").unwrap();
    assert_eq!(fx.claims.get(&d).unwrap().unwrap().state, ClaimState::Final);
    assert_eq!(replay_admission(&j, &d).unwrap(), before);
}
