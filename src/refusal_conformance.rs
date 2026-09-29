//! Refusal-conformance witness for arbitrary admission implementations.
//!
//! This module decides no security policy and has no actuation surface. It verifies
//! that a claimed refusal is typed exactly and that the refused path changed no
//! observed world state.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefusalObservation<'a> {
    pub expected: &'a str,
    pub actual: Result<(), &'a str>,
    pub observed_world_change: bool,
}

pub fn verify_refusal(observation: RefusalObservation<'_>) -> Result<(), &'static str> {
    if !observation.expected.starts_with("REFUSED:") {
        return Err("REFUSED:EXPECTED_CODE_NOT_REFUSAL");
    }
    if observation.observed_world_change {
        return Err("REFUSED:REFUSAL_ALLOWED_WORLD_CHANGE");
    }
    match observation.actual {
        Err(code) if code == observation.expected => Ok(()),
        Err(_) => Err("REFUSED:WRONG_REFUSAL"),
        Ok(()) => Err("REFUSED:EXPECTED_REFUSAL_BUT_ADMITTED"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_foreign_refusal_conforms() {
        assert_eq!(
            verify_refusal(RefusalObservation {
                expected: "REFUSED:AUTHORITY_REQUIRED",
                actual: Err("REFUSED:AUTHORITY_REQUIRED"),
                observed_world_change: false,
            }),
            Ok(())
        );
    }

    #[test]
    fn refusal_with_world_change_is_never_conformant() {
        assert_eq!(
            verify_refusal(RefusalObservation {
                expected: "REFUSED:AUTHORITY_REQUIRED",
                actual: Err("REFUSED:AUTHORITY_REQUIRED"),
                observed_world_change: true,
            }),
            Err("REFUSED:REFUSAL_ALLOWED_WORLD_CHANGE")
        );
    }

    #[test]
    fn accidental_admission_fails_refusal_conformance() {
        assert_eq!(
            verify_refusal(RefusalObservation {
                expected: "REFUSED:RECEIPT_CAPABILITY_REQUIRED",
                actual: Ok(()),
                observed_world_change: false,
            }),
            Err("REFUSED:EXPECTED_REFUSAL_BUT_ADMITTED")
        );
    }

    #[test]
    fn wrong_refusal_is_not_equivalent() {
        assert_eq!(
            verify_refusal(RefusalObservation {
                expected: "REFUSED:AUTHORITY_REQUIRED",
                actual: Err("REFUSED:SCOPE_REQUIRED"),
                observed_world_change: false,
            }),
            Err("REFUSED:WRONG_REFUSAL")
        );
    }
}
