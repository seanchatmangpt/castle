# SA2A cryptographic profile

The verification boundary is algorithm-agile and admits three explicit algorithms: Ed25519, ML-DSA-65, and SLH-DSA-SHAKE-128f.

Implementation reuses the repository's existing RustCrypto-family dependencies instead of implementing signature mathematics. Each registry key fixes one algorithm and one custodian identity. A certificate signature cannot change algorithms without failing the registry binding.

C3 quorum counts verified custodian identities, not signer labels.

## Portable effect identity

PreparedEffect identity is the exact AshA2A portable object `{version, principal, capability, subject, payload}`, canonicalized with RFC 8785/JCS and hashed with SHA-256. Rust reuses `serde_json_canonicalizer`; it does not define a second byte encoding.
