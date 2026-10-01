//! Refusal-conformance witness for arbitrary admission implementations.
//!
//! This module does not decide security policy and cannot actuate. It verifies
//! that a claimed refusal happened exactly and that a refused path produced no
//! observed world change.

pub fn verify_refusal(expected: &str, actual: Result<(), &str>, observed_world_change: bool) -> Result<(), &'static str> {
    if !expected.starts_with("REFUSED:") { return Err("REFUSED:EXPECTED_CODE_NOT_REFUSAL"); }
    if observed_world_change { return Err("REFUSED:REFUSAL_ALLOWED_WORLD_CHANGE"); }
    match actual {
        Err(code) if code == expected => Ok(()),
        Err(_) => Err("REFUSED:WRONG_REFUSAL"),
        Ok(()) => Err("REFUSED:EXPECTED_REFUSAL_BUT_ADMITTED"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn exact_foreign_refusal_conforms() { assert_eq!(verify_refusal("REFUSED:AUTHORITY_REQUIRED",Err("REFUSED:AUTHORITY_REQUIRED"),false),Ok(())); }
    #[test] fn refusal_with_world_change_is_never_conformant() { assert_eq!(verify_refusal("REFUSED:AUTHORITY_REQUIRED",Err("REFUSED:AUTHORITY_REQUIRED"),true),Err("REFUSED:REFUSAL_ALLOWED_WORLD_CHANGE")); }
    #[test] fn accidental_admission_fails_refusal_conformance() { assert_eq!(verify_refusal("REFUSED:RECEIPT_CAPABILITY_REQUIRED",Ok(()),false),Err("REFUSED:EXPECTED_REFUSAL_BUT_ADMITTED")); }
}
