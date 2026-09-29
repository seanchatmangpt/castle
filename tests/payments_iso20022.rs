mod common;

use castle::payments::iso20022::*;
use castle::payments::*;
use common::payments::*;
use std::collections::BTreeSet;

const PAYEE2: &str = "acct:supplier-2";
const TS: &str = "2026-09-29T12:34:56Z";
const DAGT: &str = "CASTUS33";
const CAGT: &str = "SUPPGB2LXXX";

fn multi_fx(tag: &str) -> Fixture {
    let mut fx = Fixture::new(tag);
    let p = fx.policy.principals.get_mut(PRINCIPAL).unwrap();
    for c in [Currency::JPY, Currency::KWD, Currency::EUR] {
        p.per_effect_cap.insert(c, 10_000_000);
        p.epoch_cap.insert(c, 100_000_000);
    }
    p.allowed_payees.insert(PAYEE2.to_string());
    fx
}

fn admit_ccy(fx: &Fixture, amount: &str, ccy: Currency, obl: &str, payee: &str, nonce: &str) -> PaymentAdmission {
    let eff = PaymentEffect::prepare(PRINCIPAL, PAYER, payee, amount, ccy, obl, "invoice-payment", None).unwrap();
    fx.admit(eff, nonce).unwrap()
}

fn golden(name: &str) -> String {
    let p = format!("{}/fixtures/payments/iso20022/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("missing golden {p}: {e}"))
}

fn check_golden(name: &str, actual: &str) {
    if std::env::var("CASTLE_WRITE_GOLDEN").is_ok() {
        let p = format!("{}/fixtures/payments/iso20022/{name}", env!("CARGO_MANIFEST_DIR"));
        std::fs::write(p, actual).unwrap();
        return;
    }
    assert_eq!(golden(name), actual, "golden mismatch for {name}");
}

fn pain(a: &PaymentAdmission) -> String {
    pain001_customer_credit_transfer(a, TS, "Acme Treasury", "Supplier Ltd", DAGT, CAGT).unwrap()
}

#[test]
fn golden_usd_jpy_kwd() {
    let fx = multi_fx("golden");
    let usd = admit_ccy(&fx, "123456", Currency::USD, "OBL-USD-1", PAYEE, "n1");
    let jpy = admit_ccy(&fx, "5000", Currency::JPY, "OBL-JPY-1", PAYEE, "n2");
    let kwd = admit_ccy(&fx, "1234567", Currency::KWD, "OBL-KWD-1", PAYEE, "n3");
    for (tag, a) in [("usd", &usd), ("jpy", &jpy), ("kwd", &kwd)] {
        check_golden(&format!("pain001_{tag}.xml"), &pain(a));
        check_golden(
            &format!("pacs008_{tag}.xml"),
            &pacs008_fi_credit_transfer(a, TS, "CASTUS33", "SUPPGB2LXXX", DAGT, CAGT).unwrap(),
        );
    }
    assert!(pain(&usd).contains("<InstdAmt Ccy=\"USD\">1234.56</InstdAmt>"));
    assert!(pain(&jpy).contains("<InstdAmt Ccy=\"JPY\">5000</InstdAmt>"));
    assert!(pain(&kwd).contains("<InstdAmt Ccy=\"KWD\">1234.567</InstdAmt>"));
    assert!(pain(&kwd).contains("<CtrlSum>1234.567</CtrlSum>"));
}

#[test]
fn deterministic_bytes() {
    let fx = multi_fx("det");
    let a = admit_ccy(&fx, "100", Currency::USD, "OBL-1", PAYEE, "n1");
    assert_eq!(pain(&a), pain(&a));
}

#[test]
fn digest_bound_to_effect_fields() {
    let fx = multi_fx("bind");
    let base = admit_ccy(&fx, "100", Currency::USD, "OBL-1", PAYEE, "n1");
    let d = |a: &PaymentAdmission| project_effect_digest_from_pain001(&pain(a)).unwrap();
    assert_eq!(d(&base), base.effect().digest());
    let amount = admit_ccy(&multi_fx("v-amount"), "101", Currency::USD, "OBL-1", PAYEE, "n2");
    let payee = admit_ccy(&multi_fx("v-payee"), "100", Currency::USD, "OBL-1", PAYEE2, "n3");
    let ccy = admit_ccy(&multi_fx("v-ccy"), "100", Currency::EUR, "OBL-1", PAYEE, "n4");
    let digests = [d(&base), d(&amount), d(&payee), d(&ccy)];
    let set: BTreeSet<_> = digests.iter().collect();
    assert_eq!(set.len(), 4);
    // MsgId never collides across distinct admissions.
    let msg = |a: &PaymentAdmission| {
        let x = pain(a);
        let s = x.find("<MsgId>").unwrap();
        x[s..x[s..].find("</MsgId>").unwrap() + s].to_string()
    };
    let ids: BTreeSet<_> = [&base, &amount, &payee, &ccy].iter().map(|a| msg(a)).collect();
    assert_eq!(ids.len(), 4);
}

#[test]
fn xml_injection_is_escaped() {
    let fx = multi_fx("inj");
    let a = admit_ccy(&fx, "100", Currency::USD, "OBL-1", PAYEE, "n1");
    let evil = "</Nm><Injected a=\"1\">&'x";
    let xml = pain001_customer_credit_transfer(&a, TS, evil, evil, DAGT, CAGT).unwrap();
    assert!(!xml.contains("<Injected"));
    assert!(xml.contains("&lt;/Nm&gt;&lt;Injected a=&quot;1&quot;&gt;&amp;&apos;x"));
    assert_eq!(xml.matches("<Dbtr>").count(), 1);
    assert_eq!(xml.matches("<Cdtr>").count(), 1);
    assert_eq!(project_effect_digest_from_pain001(&xml).unwrap(), a.effect().digest());
}

#[test]
fn invalid_fields_refused() {
    let fx = multi_fx("bad");
    let a = admit_ccy(&fx, "100", Currency::USD, "OBL-1", PAYEE, "n1");
    let refused = "REFUSED:PROJECTION_FIELD_INVALID";
    for name in ["", "bad\nname", "tab\tname", "nul\0"] {
        assert_eq!(pain001_customer_credit_transfer(&a, TS, name, "ok", DAGT, CAGT).unwrap_err(), refused);
        assert_eq!(pain001_customer_credit_transfer(&a, TS, "ok", name, DAGT, CAGT).unwrap_err(), refused);
    }
    for ts in [
        "", "2026-09-29 12:34:56Z", "2026-09-29T12:34:56", "2026-13-29T12:34:56Z", "2026-09-32T12:34:56Z",
        "2026-09-29T24:34:56Z", "2026-09-29T12:60:56Z", "2026-09-29T12:34:60Z", "2026-09-29T12:34:56+00:00",
        "2026-09-29T12:34:5aZ", "2026-00-29T12:34:56Z",
    ] {
        assert_eq!(pain001_customer_credit_transfer(&a, ts, "a", "b", DAGT, CAGT).unwrap_err(), refused, "{ts}");
        assert_eq!(pacs008_fi_credit_transfer(&a, ts, "CASTUS33", "SUPPGB2L", DAGT, CAGT).unwrap_err(), refused, "{ts}");
    }
    for bic in ["", "CASTUS3", "castus33", "CASTUS33X", "CAST1S33", "CASTUS33XX", "CASTUS3!", "CASTUS33 XX"] {
        assert_eq!(pacs008_fi_credit_transfer(&a, TS, bic, "SUPPGB2L", DAGT, CAGT).unwrap_err(), refused, "{bic}");
        assert_eq!(pacs008_fi_credit_transfer(&a, TS, "CASTUS33", "SUPPGB2L", bic, CAGT).unwrap_err(), refused, "{bic}");
    }
    assert!(pacs008_fi_credit_transfer(&a, TS, "CASTUS33", "SUPPGB2LXXX", DAGT, CAGT).is_ok());
    assert!(pacs008_fi_credit_transfer(&a, TS, "CASTUS33", "SUPPGB2L", DAGT, CAGT).is_ok());
}

#[test]
fn extract_returns_none_without_instrid() {
    assert_eq!(project_effect_digest_from_pain001("<Document/>"), None);
}

#[test]
fn pain001_agent_bics_and_length_bounds_refused() {
    let fx = multi_fx("pain-bic");
    let a = admit_ccy(&fx, "100", Currency::USD, "OBL-1", PAYEE, "n1");
    let refused = "REFUSED:PROJECTION_FIELD_INVALID";
    for bic in ["", "castus33", "CASTUS3", "CASTUS33X"] {
        assert_eq!(pain001_customer_credit_transfer(&a, TS, "a", "b", bic, CAGT).unwrap_err(), refused, "{bic}");
        assert_eq!(pain001_customer_credit_transfer(&a, TS, "a", "b", DAGT, bic).unwrap_err(), refused, "{bic}");
    }
    let long = "x".repeat(141);
    assert_eq!(pain001_customer_credit_transfer(&a, TS, &long, "b", DAGT, CAGT).unwrap_err(), refused);
    assert!(pain001_customer_credit_transfer(&a, TS, &"x".repeat(140), "b", DAGT, CAGT).is_ok());
}

#[test]
fn profile_pins_official_namespaces() {
    assert!(ISO20022_PROFILE.contains("urn:iso:std:iso:20022:tech:xsd:pain.001.001.09"));
    assert!(ISO20022_PROFILE.contains("urn:iso:std:iso:20022:tech:xsd:pacs.008.001.08"));
    assert!(message_profile_version().contains("pain.001.001.09"));
}
