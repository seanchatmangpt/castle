//! BLAKE3-256 digesting, backed by the `blake3` crate (the ecosystem-standard choice,
//! matching `~/ggen`'s own dependency pin) rather than a hand-rolled implementation.

#[must_use]
pub fn blake3_hex_utf8(input: &str) -> String {
    blake3::hash(input.as_bytes()).to_hex().to_string()
}

pub const BLAKE3_EMPTY_HEX: &str = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

pub fn assert_blake3_self_test() -> Result<(), String> {
    let observed = blake3_hex_utf8("");
    if observed != BLAKE3_EMPTY_HEX {
        return Err(format!("REFUSED:BLAKE3_SELF_TEST_FAILED:{observed}"));
    }
    Ok(())
}
