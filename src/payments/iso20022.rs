//! ISO 20022 rail projection of an admitted payment.
//!
//! Pure, deterministic re-expression of already-admitted fields (`PaymentAdmission`).
//! A projection carries no authority: it cannot be turned back into an admission and
//! never alters amount, parties, or currency. No clock, no maps, no floats.

use crate::sa2a_security::encoding::sha256_tagged;

use super::admission::PaymentAdmission;
use super::refusal::{refuse, PayResult};

pub const PROJECTION_FIELD_INVALID: &str = "REFUSED:PROJECTION_FIELD_INVALID";

/// Official message profile this projection targets. `status` is `CONFORMANCE_CHECKED_AGAINST_XSD`
/// only when `tests/payments_iso20022_official.rs` validated real output with `xmllint` against
/// the version-pinned XSDs listed in `fixtures/payments/iso20022/xsd/PINS.json`.
pub const ISO20022_PROFILE: &str = concat!(
    "{\"messages\":[",
    "{\"id\":\"pain.001.001.09\",\"version\":\"09\",",
    "\"xsd_target_namespace\":\"urn:iso:std:iso:20022:tech:xsd:pain.001.001.09\"},",
    "{\"id\":\"pacs.008.001.08\",\"version\":\"08\",",
    "\"xsd_target_namespace\":\"urn:iso:std:iso:20022:tech:xsd:pacs.008.001.08\"}],",
    "\"status\":\"CONFORMANCE_CHECKED_AGAINST_XSD\"}"
);

/// Profile version tag: changes whenever the emitted element set changes.
#[must_use]
pub const fn message_profile_version() -> &'static str {
    "castle-iso20022-profile/2:pain.001.001.09+pacs.008.001.08:xsd-validated"
}

/// Marker prefix of the `Ustrd` remittance line that carries the full effect digest
/// (`InstrId` is Max35Text in the XSD and can only hold a digest prefix).
const DIGEST_MARKER: &str = "castle:effect-digest=";

/// Marker prefix of the `Ustrd` line carrying the FULL obligation id when it does not fit
/// `EndToEndId` (Max35Text) and a bounded surrogate is projected instead.
const OBLIGATION_MARKER: &str = "castle:obligation-id=";

/// `EndToEndId` value plus the optional obligation-id `Ustrd` line. Ids of <=35 chars are
/// projected unchanged; longer ids project as `OBL-` + 28 hex of the obligation-id digest and
/// the full id rides in a `Ustrd` marker (Max140Text). If the marker cannot fit: refused typed.
fn end_to_end(obligation_id: &str) -> PayResult<(String, Option<String>)> {
    check_text(obligation_id)?;
    if obligation_id.chars().count() <= 35 {
        return Ok((obligation_id.to_string(), None));
    }
    let marker = format!("{OBLIGATION_MARKER}{obligation_id}");
    if marker.chars().count() > 140 {
        return invalid();
    }
    let d = sha256_tagged(b"CASTLE-ISO-E2E-V1", obligation_id.as_bytes());
    Ok((format!("OBL-{}", id_prefix(&d, 28)), Some(marker)))
}

fn check_max(s: &str, max: usize) -> PayResult<()> {
    check_text(s)?;
    if s.chars().count() > max {
        return invalid();
    }
    Ok(())
}

fn invalid<T>() -> PayResult<T> {
    refuse(PROJECTION_FIELD_INVALID)
}

/// Escape the five XML predefined entities.
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Non-empty, no control characters (including tab/newline).
fn check_text(s: &str) -> PayResult<()> {
    if s.is_empty() || s.chars().any(char::is_control) {
        return invalid();
    }
    Ok(())
}

/// Strict `YYYY-MM-DDTHH:MM:SSZ`.
fn check_timestamp(ts: &str) -> PayResult<()> {
    let b = ts.as_bytes();
    if b.len() != 20 {
        return invalid();
    }
    let digit = |i: usize| b[i].is_ascii_digit();
    let shape = [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18].iter().all(|&i| digit(i))
        && b[4] == b'-'
        && b[7] == b'-'
        && b[10] == b'T'
        && b[13] == b':'
        && b[16] == b':'
        && b[19] == b'Z';
    if !shape {
        return invalid();
    }
    let n = |r: std::ops::Range<usize>| ts[r].parse::<u32>().unwrap_or(u32::MAX);
    let (mo, d, h, mi, s) = (n(5..7), n(8..10), n(11..13), n(14..16), n(17..19));
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) || h > 23 || mi > 59 || s > 59 {
        return invalid();
    }
    Ok(())
}

/// `^[A-Z]{6}[A-Z0-9]{2}([A-Z0-9]{3})?$`
fn check_bic(bic: &str) -> PayResult<()> {
    let b = bic.as_bytes();
    if b.len() != 8 && b.len() != 11 {
        return invalid();
    }
    let ok = b.iter().enumerate().all(|(i, c)| {
        if i < 6 {
            c.is_ascii_uppercase()
        } else {
            c.is_ascii_uppercase() || c.is_ascii_digit()
        }
    });
    if !ok {
        return invalid();
    }
    Ok(())
}

/// Digest prefix used for bounded (Max35Text) identifiers.
fn id_prefix(digest: &str, n: usize) -> &str {
    let bare = digest.split_once(':').map_or(digest, |(_, h)| h);
    let end = bare.char_indices().nth(n).map_or(bare.len(), |(i, _)| i);
    &bare[..end]
}

/// pain.001.001.09 (CustomerCreditTransferInitiation), XSD-mandatory elements included.
/// Agent BICs are caller-supplied; nothing is invented.
pub fn pain001_customer_credit_transfer(
    admission: &PaymentAdmission,
    created_at_iso: &str,
    debtor_name: &str,
    creditor_name: &str,
    debtor_agent_bic: &str,
    creditor_agent_bic: &str,
) -> PayResult<String> {
    check_timestamp(created_at_iso)?;
    check_max(debtor_name, 140)?;
    check_max(creditor_name, 140)?;
    check_bic(debtor_agent_bic)?;
    check_bic(creditor_agent_bic)?;
    let e = admission.effect();
    check_max(e.payer(), 34)?;
    check_max(e.payee(), 34)?;
    let (e2e, obl_marker) = end_to_end(e.obligation_id())?;
    check_max(e.purpose(), 140)?;

    let digest = e.digest();
    let money = e.money();
    let amt = money.to_decimal_string();
    let ccy = money.currency.code();
    let date = &created_at_iso[..10];
    let msg_id = format!("CASTLE-{}", id_prefix(digest, 28));
    let pmt_inf_id = format!("PMT-{}", id_prefix(digest, 28));
    let instr_id = digest.chars().take(35).collect::<String>();

    let mut x = String::new();
    x.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    x.push_str("<Document xmlns=\"urn:iso:std:iso:20022:tech:xsd:pain.001.001.09\">\n");
    x.push_str("  <CstmrCdtTrfInitn>\n");
    x.push_str("    <GrpHdr>\n");
    x.push_str(&format!("      <MsgId>{}</MsgId>\n", esc(&msg_id)));
    x.push_str(&format!("      <CreDtTm>{created_at_iso}</CreDtTm>\n"));
    x.push_str("      <NbOfTxs>1</NbOfTxs>\n");
    x.push_str(&format!("      <CtrlSum>{amt}</CtrlSum>\n"));
    x.push_str(&format!("      <InitgPty><Nm>{}</Nm></InitgPty>\n", esc(debtor_name)));
    x.push_str("    </GrpHdr>\n");
    x.push_str("    <PmtInf>\n");
    x.push_str(&format!("      <PmtInfId>{}</PmtInfId>\n", esc(&pmt_inf_id)));
    x.push_str("      <PmtMtd>TRF</PmtMtd>\n");
    x.push_str(&format!("      <ReqdExctnDt><Dt>{date}</Dt></ReqdExctnDt>\n"));
    x.push_str(&format!("      <Dbtr><Nm>{}</Nm></Dbtr>\n", esc(debtor_name)));
    x.push_str(&format!(
        "      <DbtrAcct><Id><Othr><Id>{}</Id></Othr></Id></DbtrAcct>\n",
        esc(e.payer())
    ));
    x.push_str(&format!(
        "      <DbtrAgt><FinInstnId><BICFI>{debtor_agent_bic}</BICFI></FinInstnId></DbtrAgt>\n"
    ));
    x.push_str("      <CdtTrfTxInf>\n");
    x.push_str("        <PmtId>\n");
    x.push_str(&format!("          <InstrId>{}</InstrId>\n", esc(&instr_id)));
    x.push_str(&format!("          <EndToEndId>{}</EndToEndId>\n", esc(&e2e)));
    x.push_str("        </PmtId>\n");
    x.push_str(&format!("        <Amt><InstdAmt Ccy=\"{ccy}\">{amt}</InstdAmt></Amt>\n"));
    x.push_str(&format!(
        "        <CdtrAgt><FinInstnId><BICFI>{creditor_agent_bic}</BICFI></FinInstnId></CdtrAgt>\n"
    ));
    x.push_str(&format!("        <Cdtr><Nm>{}</Nm></Cdtr>\n", esc(creditor_name)));
    x.push_str(&format!(
        "        <CdtrAcct><Id><Othr><Id>{}</Id></Othr></Id></CdtrAcct>\n",
        esc(e.payee())
    ));
    let obl_line = obl_marker.map(|m| format!("<Ustrd>{}</Ustrd>", esc(&m))).unwrap_or_default();
    x.push_str(&format!(
        "        <RmtInf><Ustrd>{}</Ustrd><Ustrd>{DIGEST_MARKER}{}</Ustrd>{obl_line}</RmtInf>\n",
        esc(e.purpose()),
        esc(digest)
    ));
    x.push_str("      </CdtTrfTxInf>\n");
    x.push_str("    </PmtInf>\n");
    x.push_str("  </CstmrCdtTrfInitn>\n");
    x.push_str("</Document>\n");
    Ok(x)
}

/// pacs.008.001.08 (FIToFICustomerCreditTransfer), XSD-mandatory elements included.
pub fn pacs008_fi_credit_transfer(
    admission: &PaymentAdmission,
    created_at_iso: &str,
    instructing_agent_bic: &str,
    instructed_agent_bic: &str,
    debtor_agent_bic: &str,
    creditor_agent_bic: &str,
) -> PayResult<String> {
    check_timestamp(created_at_iso)?;
    check_bic(instructing_agent_bic)?;
    check_bic(instructed_agent_bic)?;
    check_bic(debtor_agent_bic)?;
    check_bic(creditor_agent_bic)?;
    let e = admission.effect();
    check_max(e.payer(), 34)?;
    check_max(e.payee(), 34)?;
    let (e2e, obl_marker) = end_to_end(e.obligation_id())?;

    let digest = e.digest();
    let money = e.money();
    let amt = money.to_decimal_string();
    let ccy = money.currency.code();
    let msg_id = format!("CASTLE-{}", id_prefix(digest, 28));
    let tx_id = id_prefix(digest, 35);
    let instr_id = digest.chars().take(35).collect::<String>();

    let mut x = String::new();
    x.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    x.push_str("<Document xmlns=\"urn:iso:std:iso:20022:tech:xsd:pacs.008.001.08\">\n");
    x.push_str("  <FIToFICstmrCdtTrf>\n");
    x.push_str("    <GrpHdr>\n");
    x.push_str(&format!("      <MsgId>{}</MsgId>\n", esc(&msg_id)));
    x.push_str(&format!("      <CreDtTm>{created_at_iso}</CreDtTm>\n"));
    x.push_str("      <NbOfTxs>1</NbOfTxs>\n");
    x.push_str("      <SttlmInf><SttlmMtd>INDA</SttlmMtd></SttlmInf>\n");
    x.push_str("    </GrpHdr>\n");
    x.push_str("    <CdtTrfTxInf>\n");
    x.push_str("      <PmtId>\n");
    x.push_str(&format!("        <InstrId>{}</InstrId>\n", esc(&instr_id)));
    x.push_str(&format!("        <EndToEndId>{}</EndToEndId>\n", esc(&e2e)));
    x.push_str(&format!("        <TxId>{}</TxId>\n", esc(tx_id)));
    x.push_str("      </PmtId>\n");
    x.push_str(&format!("      <IntrBkSttlmAmt Ccy=\"{ccy}\">{amt}</IntrBkSttlmAmt>\n"));
    x.push_str("      <ChrgBr>SLEV</ChrgBr>\n");
    x.push_str(&format!(
        "      <InstgAgt><FinInstnId><BICFI>{instructing_agent_bic}</BICFI></FinInstnId></InstgAgt>\n"
    ));
    x.push_str(&format!(
        "      <InstdAgt><FinInstnId><BICFI>{instructed_agent_bic}</BICFI></FinInstnId></InstdAgt>\n"
    ));
    x.push_str(&format!("      <Dbtr><Nm>{}</Nm></Dbtr>\n", esc(e.payer())));
    x.push_str(&format!(
        "      <DbtrAcct><Id><Othr><Id>{}</Id></Othr></Id></DbtrAcct>\n",
        esc(e.payer())
    ));
    x.push_str(&format!(
        "      <DbtrAgt><FinInstnId><BICFI>{debtor_agent_bic}</BICFI></FinInstnId></DbtrAgt>\n"
    ));
    x.push_str(&format!(
        "      <CdtrAgt><FinInstnId><BICFI>{creditor_agent_bic}</BICFI></FinInstnId></CdtrAgt>\n"
    ));
    x.push_str(&format!("      <Cdtr><Nm>{}</Nm></Cdtr>\n", esc(e.payee())));
    x.push_str(&format!(
        "      <CdtrAcct><Id><Othr><Id>{}</Id></Othr></Id></CdtrAcct>\n",
        esc(e.payee())
    ));
    let obl_line = obl_marker.map(|m| format!("<Ustrd>{}</Ustrd>", esc(&m))).unwrap_or_default();
    x.push_str(&format!(
        "      <RmtInf><Ustrd>{DIGEST_MARKER}{}</Ustrd>{obl_line}</RmtInf>\n",
        esc(digest)
    ));
    x.push_str("    </CdtTrfTxInf>\n");
    x.push_str("  </FIToFICstmrCdtTrf>\n");
    x.push_str("</Document>\n");
    Ok(x)
}

/// Extract the full effect digest from a projection (carried in a `Ustrd` remittance line,
/// since `InstrId` is length-bounded by the XSD).
#[must_use]
pub fn project_effect_digest_from_pain001(xml: &str) -> Option<String> {
    let open = format!("<Ustrd>{DIGEST_MARKER}");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find("</Ustrd>")? + start;
    Some(xml[start..end].to_string())
}

fn unesc(s: &str) -> String {
    s.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&amp;", "&")
}

/// Recover the full obligation id from a projection: the `Ustrd` marker when a surrogate
/// `EndToEndId` was projected, otherwise the `EndToEndId` value itself.
#[must_use]
pub fn project_obligation_id_from_pain001(xml: &str) -> Option<String> {
    let open = format!("<Ustrd>{OBLIGATION_MARKER}");
    if let Some(i) = xml.find(&open) {
        let start = i + open.len();
        let end = xml[start..].find("</Ustrd>")? + start;
        return Some(unesc(&xml[start..end]));
    }
    let start = xml.find("<EndToEndId>")? + "<EndToEndId>".len();
    let end = xml[start..].find("</EndToEndId>")? + start;
    Some(unesc(&xml[start..end]))
}
