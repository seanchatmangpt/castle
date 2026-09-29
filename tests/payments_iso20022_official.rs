//! Official-XSD conformance: real `xmllint --schema` subprocess against version-pinned XSDs.
//! Fails LOUDLY (never skips) when the XSDs are absent or do not match `PINS.json`.

mod common;

use castle::payments::iso20022::*;
use castle::payments::*;
use common::payments::*;
use std::path::PathBuf;
use std::process::Command;

const TS: &str = "2026-09-29T12:34:56Z";
const DEFAULT_DIR: &str =
    "/private/tmp/claude-501/-Users-sac-castle/62dd6f54-50cc-4146-9377-c4ca3cfee3bd/scratchpad/iso/xsd";

fn xsd_dir() -> PathBuf {
    PathBuf::from(std::env::var("CASTLE_ISO20022_XSD_DIR").unwrap_or_else(|_| DEFAULT_DIR.to_string()))
}

fn pinned_sha(name: &str) -> String {
    let p = format!("{}/fixtures/payments/iso20022/xsd/PINS.json", env!("CARGO_MANIFEST_DIR"));
    let txt = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {p}: {e}"));
    let at = txt.find(&format!("\"{name}\"")).unwrap_or_else(|| panic!("{name} not pinned"));
    let rest = &txt[at..];
    let k = rest.find("\"sha256\": \"").expect("sha256 pin") + 11;
    rest[k..k + 64].to_string()
}

/// Verified XSD path: exists, sha256 equals the pin, targetNamespace is exact.
fn xsd(name: &str, ns: &str) -> PathBuf {
    let path = xsd_dir().join(name);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        panic!("OFFICIAL XSD MISSING: {} ({e}); set CASTLE_ISO20022_XSD_DIR, see fixtures/payments/iso20022/xsd/PINS.json", path.display())
    });
    let got = blake3_free_sha256(&path);
    assert_eq!(got, pinned_sha(name), "XSD {name} does not match pinned sha256");
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains(&format!("targetNamespace=\"{ns}\"")), "wrong targetNamespace in {name}");
    path
}

/// sha256 via the system `shasum` (real subprocess; the crate has no sha256 dependency).
fn blake3_free_sha256(path: &std::path::Path) -> String {
    let out = Command::new("shasum").args(["-a", "256"]).arg(path).output().expect("shasum must exist");
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap().split_whitespace().next().unwrap().to_string()
}

fn validate(xsd: &std::path::Path, xml: &str, tag: &str) -> (bool, String) {
    let dir = std::env::temp_dir().join(format!("castle-iso-official-{}-{tag}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("doc.xml");
    std::fs::write(&f, xml).unwrap();
    let out = Command::new("xmllint")
        .args(["--noout", "--schema"])
        .arg(xsd)
        .arg(&f)
        .output()
        .expect("xmllint must be installed");
    let _ = std::fs::remove_dir_all(&dir);
    (out.status.success(), String::from_utf8_lossy(&out.stderr).to_string())
}

fn docs() -> Vec<(String, String, String)> {
    let mut fx = Fixture::new("official");
    let p = fx.policy.principals.get_mut(PRINCIPAL).unwrap();
    for c in [Currency::JPY, Currency::KWD] {
        p.per_effect_cap.insert(c, 10_000_000);
        p.epoch_cap.insert(c, 100_000_000);
    }
    let mut out = vec![];
    for (i, (amt, ccy, tag)) in
        [("123456", Currency::USD, "usd"), ("5000", Currency::JPY, "jpy"), ("1234567", Currency::KWD, "kwd")]
            .into_iter()
            .enumerate()
    {
        let eff = PaymentEffect::prepare(PRINCIPAL, PAYER, PAYEE, amt, ccy, &format!("OBL-{tag}-1"), "invoice-payment", None)
            .unwrap();
        let a = fx.admit(eff, &format!("n{i}")).unwrap();
        let pain = pain001_customer_credit_transfer(&a, TS, "Acme Treasury", "Supplier Ltd", "CASTUS33", "SUPPGB2LXXX").unwrap();
        let pacs = pacs008_fi_credit_transfer(&a, TS, "CASTUS33", "SUPPGB2LXXX", "CASTUS33", "SUPPGB2LXXX").unwrap();
        out.push((tag.to_string(), pain, pacs));
    }
    out
}

#[test]
fn generated_messages_validate_against_official_xsd() {
    let pain_xsd = xsd("pain.001.001.09.xsd", "urn:iso:std:iso:20022:tech:xsd:pain.001.001.09");
    let pacs_xsd = xsd("pacs.008.001.08.xsd", "urn:iso:std:iso:20022:tech:xsd:pacs.008.001.08");
    for (tag, pain, pacs) in docs() {
        let (ok, err) = validate(&pain_xsd, &pain, &format!("pain-{tag}"));
        assert!(ok, "pain.001 {tag} invalid: {err}");
        let (ok, err) = validate(&pacs_xsd, &pacs, &format!("pacs-{tag}"));
        assert!(ok, "pacs.008 {tag} invalid: {err}");
    }
}

#[test]
fn committed_goldens_validate_against_official_xsd() {
    let pain_xsd = xsd("pain.001.001.09.xsd", "urn:iso:std:iso:20022:tech:xsd:pain.001.001.09");
    let pacs_xsd = xsd("pacs.008.001.08.xsd", "urn:iso:std:iso:20022:tech:xsd:pacs.008.001.08");
    for tag in ["usd", "jpy", "kwd"] {
        for (prefix, x) in [("pain001", &pain_xsd), ("pacs008", &pacs_xsd)] {
            let p = format!("{}/fixtures/payments/iso20022/{prefix}_{tag}.xml", env!("CARGO_MANIFEST_DIR"));
            let xml = std::fs::read_to_string(&p).unwrap();
            let (ok, err) = validate(x, &xml, &format!("g-{prefix}-{tag}"));
            assert!(ok, "golden {p} invalid: {err}");
        }
    }
}

#[test]
fn mutated_documents_fail_xmllint() {
    let pain_xsd = xsd("pain.001.001.09.xsd", "urn:iso:std:iso:20022:tech:xsd:pain.001.001.09");
    let pacs_xsd = xsd("pacs.008.001.08.xsd", "urn:iso:std:iso:20022:tech:xsd:pacs.008.001.08");
    let (_, pain, pacs) = docs().into_iter().next().unwrap();
    assert!(validate(&pain_xsd, &pain, "base-pain").0);
    assert!(validate(&pacs_xsd, &pacs, "base-pacs").0);

    // Amount with 6 fraction digits violates fractionDigits=5.
    let bad_amt = pain.replace("1234.56</InstdAmt>", "1234.567891</InstdAmt>");
    assert_ne!(bad_amt, pain);
    assert!(!validate(&pain_xsd, &bad_amt, "amt").0, "over-precise amount must be rejected");

    // Removed mandatory DbtrAgt.
    let no_agt: String = pain.lines().filter(|l| !l.contains("<DbtrAgt>")).collect::<Vec<_>>().join("\n");
    assert!(!validate(&pain_xsd, &no_agt, "noagt").0, "missing DbtrAgt must be rejected");
    let no_cagt: String = pacs.lines().filter(|l| !l.contains("<CdtrAgt>")).collect::<Vec<_>>().join("\n");
    assert!(!validate(&pacs_xsd, &no_cagt, "nocagt").0, "missing CdtrAgt must be rejected");

    // Wrong namespace.
    let wrong_ns = pain.replace("pain.001.001.09", "pain.001.001.03");
    assert!(!validate(&pain_xsd, &wrong_ns, "ns").0);
}

#[test]
fn no_placeholder_namespace_anywhere() {
    fn walk(dir: &std::path::Path, hits: &mut Vec<String>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, hits);
            } else if let Ok(t) = std::fs::read_to_string(&p) {
                // Split literal so this test file never matches itself.
                if t.contains(&["example", ".org"].concat()) {
                    hits.push(p.display().to_string());
                }
            }
        }
    }
    let root = env!("CARGO_MANIFEST_DIR");
    let mut hits = vec![];
    walk(&PathBuf::from(format!("{root}/src/payments")), &mut hits);
    walk(&PathBuf::from(format!("{root}/fixtures/payments")), &mut hits);
    assert!(hits.is_empty(), "placeholder namespace found in: {hits:?}");
}
