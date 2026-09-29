//! Derived obligation identity: the obligation id is a pure function of
//! (payer, payee, purpose, invoice_ref), so it cannot be hand-typed.

use crate::sa2a_security::encoding::{push_field, sha256_tagged};
use crate::sa2a_security::PreparedEffect;

use super::effect::PaymentEffect;
use super::money::Currency;
use super::refusal::PayResult;

pub const OBLIGATION_ID_NOT_DERIVED: &str = "REFUSED:PAYMENT_OBLIGATION_ID_NOT_DERIVED";

const OBLIGATION_DOMAIN: &[u8] = b"CASTLE-OBLIGATION-V1";

#[must_use]
pub fn derive_obligation_id(payer: &str, payee: &str, purpose: &str, invoice_ref: &str) -> String {
    let mut body = Vec::new();
    push_field(&mut body, payer.as_bytes());
    push_field(&mut body, payee.as_bytes());
    push_field(&mut body, purpose.as_bytes());
    push_field(&mut body, invoice_ref.as_bytes());
    format!("obl:{}", sha256_tagged(OBLIGATION_DOMAIN, &body))
}

/// Prepare a transfer whose obligation id is derived from the invoice.
#[allow(clippy::too_many_arguments)]
pub fn prepare_for_invoice(
    principal: &str,
    payer: &str,
    payee: &str,
    amount_minor: &str,
    currency: Currency,
    purpose: &str,
    invoice_ref: &str,
    reverses: Option<&str>,
) -> PayResult<PreparedEffect> {
    let obligation = derive_obligation_id(payer, payee, purpose, invoice_ref);
    PaymentEffect::prepare_with_invoice(
        principal,
        payer,
        payee,
        amount_minor,
        currency,
        &obligation,
        purpose,
        Some(invoice_ref),
        reverses,
    )
}
