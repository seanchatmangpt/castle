use castle::sa2a_security::epoch::{admit_epochs, SecurityEpochs};
use castle::sa2a_security::SecurityRefusal;

#[test]
fn stale_generation_refuses() {
    let expected = SecurityEpochs { policy: 1, revocation: 2, generation: 4 };
    let stale = SecurityEpochs { policy: 1, revocation: 2, generation: 3 };
    assert_eq!(admit_epochs(expected, stale), Err(SecurityRefusal::GenerationMismatch));
}
