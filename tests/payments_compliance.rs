//! Real-collaborator tests for reference compliance controls and counterparty identity.
use castle::payments::compliance::*;
use castle::payments::counterparty::*;
use castle::payments::fibo::IRI_LEGAL_ENTITY_IDENTIFIER;
use castle::payments::{Currency, PaymentEffect};

fn effect(payer: &str, payee: &str) -> PaymentEffect {
    let p = PaymentEffect::prepare("principal:a", payer, payee, "100", Currency::USD, "ob-1", "invoice", None).unwrap();
    PaymentEffect::from_prepared(p).unwrap()
}

fn list(id: &str, entries: &[&str]) -> SanctionsList {
    let j = serde_json::json!({"list_id": id, "version": "1", "entries": entries});
    SanctionsList::from_json(j.to_string().as_bytes()).unwrap()
}

#[test]
fn sanctions_hit_on_payer_payee_and_alias_normalized() {
    let l = list("ofac-ref", &["ACCT:BAD", "  bad   actor ltd "]);
    assert_eq!(l.screen(&effect("acct:bad", "acct:ok")).unwrap_err(), SANCTIONS_HIT);
    assert_eq!(l.screen(&effect("acct:ok", "  ACCT:Bad ")).unwrap_err(), SANCTIONS_HIT);
    let aliased = l.clone().with_aliases(&[("acct:front", "Bad\t Actor  LTD")]);
    assert_eq!(aliased.screen(&effect("acct:ok", "acct:front")).unwrap_err(), SANCTIONS_HIT);
    // same effect is clear without the alias registration
    assert!(l.screen(&effect("acct:ok", "acct:front")).is_ok());
}

#[test]
fn clear_passes_and_digest_tracks_entries() {
    let l = list("ofac-ref", &["ACCT:BAD"]);
    let ev = l.screen(&effect("acct:a", "acct:b")).unwrap();
    assert_eq!(ev.outcome, "CLEAR");
    assert_eq!(ev.control_id, "ofac-ref");
    let l2 = list("ofac-ref", &["ACCT:BAD2"]);
    assert_ne!(l.digest, l2.digest);
    assert_eq!(l.digest, list("ofac-ref", &["acct:bad"]).digest);
    assert_ne!(ev.evidence_digest, l2.screen(&effect("acct:a", "acct:b")).unwrap().evidence_digest);
}

#[test]
fn list_json_validation() {
    for bad in [
        r#"{"list_id":"x","version":"1","entries":["A","a "]}"#,
        r#"{"list_id":"x","version":"1","entries":["A","  "]}"#,
        r#"{"list_id":"x","version":"1","entries":[]}"#,
        r#"{"list_id":"x","version":"1","entries":["A"],"extra":1}"#,
        "not json",
    ] {
        assert_eq!(SanctionsList::from_json(bad.as_bytes()).unwrap_err(), SANCTIONS_LIST_INVALID, "{bad}");
    }
}

#[test]
fn empty_control_set_refused_and_bundle_deterministic() {
    let e = effect("acct:a", "acct:b");
    assert_eq!(run_controls(&[], &e).unwrap_err(), NO_COMPLIANCE_CONTROLS);
    let a = list("a-list", &["X"]);
    let b = list("b-list", &["Y"]);
    let b1 = run_controls(&[&a, &b], &e).unwrap();
    let b2 = run_controls(&[&b, &a], &e).unwrap();
    assert_eq!(b1, b2);
    assert_eq!(b1.evidence.len(), 2);
    assert_eq!(b1.evidence[0].control_id, "a-list");
    let c = list("b-list", &["Z"]);
    assert_ne!(b1.bundle_digest, run_controls(&[&a, &c], &e).unwrap().bundle_digest);
    // first failure refuses even when other controls are clear
    let hit = list("c-list", &["ACCT:A"]);
    assert_eq!(run_controls(&[&a, &hit, &b], &e).unwrap_err(), SANCTIONS_HIT);
    let dup = list("a-list", &["Q"]);
    assert_eq!(run_controls(&[&a, &dup], &e).unwrap_err(), COMPLIANCE_CONTROL_DUPLICATE);
}

const REAL_LEI: &str = "5493001KJTIIGC8Y1R12";

#[test]
fn lei_valid_and_invalid_table() {
    let built = Lei::with_check_digits("549300ABCDEFGHIJ12").unwrap();
    assert!(Lei::parse(built.as_str()).is_ok());
    assert!(Lei::parse(REAL_LEI).is_ok(), "published LEI must verify");
    assert_eq!(Lei::parse(&REAL_LEI.to_lowercase()).unwrap_err(), LEI_INVALID);
    assert_eq!(Lei::parse(&REAL_LEI[..19]).unwrap_err(), LEI_INVALID);
    assert_eq!(Lei::parse(&format!("{REAL_LEI}0")).unwrap_err(), LEI_INVALID);
    assert_eq!(Lei::parse("").unwrap_err(), LEI_INVALID);
    assert_eq!(Lei::parse("5493001KJTIIGC8Y1R-2").unwrap_err(), LEI_INVALID);
    assert_eq!(Lei::with_check_digits("short").unwrap_err(), LEI_INVALID);
    assert_eq!(Lei::with_check_digits("549300abcdefghij12").unwrap_err(), LEI_INVALID);
}

#[test]
fn lei_single_character_corruption_caught() {
    let good = Lei::with_check_digits("724500A1B2C3D4E5F6").unwrap();
    let chars: Vec<char> = good.as_str().chars().collect();
    let alphabet: Vec<char> = ('0'..='9').chain('A'..='Z').collect();
    let (mut total, mut caught, mut digit_swaps, mut digit_caught) = (0, 0, 0, 0);
    for pos in 0..20 {
        for &c in &alphabet {
            if c == chars[pos] {
                continue;
            }
            let mut m = chars.clone();
            m[pos] = c;
            let refused = Lei::parse(&m.iter().collect::<String>()).is_err();
            total += 1;
            caught += usize::from(refused);
            if pos >= 18 {
                assert!(refused, "check digit corruption at {pos} -> {c} must be caught");
            }
            if chars[pos].is_ascii_digit() && c.is_ascii_digit() {
                digit_swaps += 1;
                digit_caught += usize::from(refused);
            }
        }
    }
    assert_eq!(digit_swaps, digit_caught, "digit-for-digit substitutions are always caught");
    assert!(caught * 100 >= total * 95, "caught {caught}/{total}");
}

fn registry_json(require: bool) -> String {
    let lei = Lei::with_check_digits("549300ABCDEFGHIJ12").unwrap();
    serde_json::json!({
        "require_lei": require,
        "parties": [
            {"party_id": "p1", "legal_name": "Treasury Co", "lei": lei.as_str(), "accounts": ["acct:treasury"]},
            {"party_id": "p2", "legal_name": "Supplier Ltd", "accounts": ["acct:supplier"]}
        ]
    })
    .to_string()
}

#[test]
fn registry_resolution_ambiguity_and_require_lei() {
    let lax = CounterpartyRegistry::from_json(registry_json(false).as_bytes()).unwrap();
    let p = lax.resolve("acct:treasury").unwrap();
    assert!(p.lei.is_some());
    assert_eq!(lax.resolve("acct:supplier").unwrap().party_id, "p2");
    assert_eq!(lax.resolve("acct:nobody").unwrap_err(), COUNTERPARTY_UNVERIFIED);

    let strict = CounterpartyRegistry::from_json(registry_json(true).as_bytes()).unwrap();
    assert!(strict.resolve("acct:treasury").is_ok());
    assert_eq!(strict.resolve("acct:supplier").unwrap_err(), COUNTERPARTY_UNVERIFIED);
    assert_eq!(lax.clone().with_require_lei(true).resolve("acct:supplier").unwrap_err(), COUNTERPARTY_UNVERIFIED);

    let ambiguous = r#"{"parties":[
        {"party_id":"a","legal_name":"A","accounts":["acct:x"]},
        {"party_id":"b","legal_name":"B","accounts":["acct:x"]}]}"#;
    assert_eq!(CounterpartyRegistry::from_json(ambiguous.as_bytes()).unwrap_err(), ACCOUNT_AMBIGUOUS);
    let unknown = r#"{"parties":[],"surprise":1}"#;
    assert_eq!(CounterpartyRegistry::from_json(unknown.as_bytes()).unwrap_err(), REGISTRY_INVALID);
    let bad_lei = r#"{"parties":[{"party_id":"a","legal_name":"A","lei":"5493001KJTIIGC8Y1R13","accounts":["x"]}]}"#;
    assert_eq!(CounterpartyRegistry::from_json(bad_lei.as_bytes()).unwrap_err(), LEI_INVALID);

    let a = lax.resolve("acct:treasury").unwrap();
    let b = lax.resolve("acct:supplier").unwrap();
    assert_eq!(lax.evidence_digest(a, b), lax.evidence_digest(a, b));
    assert_ne!(lax.evidence_digest(a, b), lax.evidence_digest(b, a));
}

#[test]
fn fibo_party_json_carries_lei_class_iri() {
    let reg = CounterpartyRegistry::from_json(registry_json(false).as_bytes()).unwrap();
    let with = fibo_party_json(reg.resolve("acct:treasury").unwrap());
    let text = with.to_string();
    assert!(text.contains(IRI_LEGAL_ENTITY_IDENTIFIER));
    assert!(text.contains("549300ABCDEFGHIJ12"));
    let without = fibo_party_json(reg.resolve("acct:supplier").unwrap());
    assert!(!without.to_string().contains(IRI_LEGAL_ENTITY_IDENTIFIER));
}
