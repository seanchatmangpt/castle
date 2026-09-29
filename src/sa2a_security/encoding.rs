use sha2::{Digest, Sha256};

pub const PREPARED_EFFECT_DOMAIN: &[u8] = b"SA2A-PREPARED-EFFECT-V1";
pub const CERTIFICATE_DOMAIN: &[u8] = b"SA2A-C2-ACTUATION-CERTIFICATE-V1";

pub fn push_field(out: &mut Vec<u8>, value: &[u8]) {
    let len = u64::try_from(value.len()).expect("usize fits u64 on supported targets");
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(value);
}

#[must_use]
pub fn sha256_tagged(domain: &[u8], body: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(body);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut hex, "{byte:02x}").expect("writing to String is infallible");
    }
    format!("sha256:{hex}")
}

#[must_use]
pub fn valid_sha256_tag(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
