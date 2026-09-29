use castle::sa2a_security::nonce::NonceFence;
use castle::sa2a_security::SecurityRefusal;

#[test]
fn replay_same_key_nonce_refuses() {
    let mut fence = NonceFence::default();
    fence.claim("k1", "n1").unwrap();
    assert_eq!(fence.claim("k1", "n1"), Err(SecurityRefusal::NonceReplay));
}
