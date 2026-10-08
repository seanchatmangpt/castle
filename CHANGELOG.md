# Changelog

All notable changes to CASTLE are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
with CalVer-style release tags (`YY.MM.DD+dfcm.N`).

## [26.10.8+dfcm.1] - 2026-10-08

### Changed

- Release bump `26.8.18+dfcm.1` -> `26.10.8+dfcm.1` (`cb779b5`): `Cargo.toml`/`Cargo.lock`,
  `RELEASE_VERSION` const in `src/v26_8_18/mod.rs`, CLI test assertions in
  `tests/cli_v26_8_18.rs`, and the deployment manifest renamed via `git mv` to
  `configs/fortune5-v26.10.8+dfcm.1.json`. Module name `src/v26_8_18` kept as a
  historical surface name (no rename in this bump). Qualification
  (`qualify_fortune5()`) is coupled to the new manifest's receipted metric
  observations. Full `cargo test` suite green
  (`CASTLE_ISO20022_XSD_DIR=/tmp/castle-xsd`, pinned-SHA XSDs).

### Added

- Capability intake and typed CLI failure JSON documented (`d66a616`):
  `README.md` describes `castle::capability_intake` (commit `5c73d66`, merged as
  PR #25 `7c08090`) with the CONSTRUCT authority ceiling and no DO authority on
  donors; `docs/payments/PAYMENTS.md` documents the machine-readable failure
  JSON on stdout and typed exit codes (2 = REFUSED/BLOCKED, 3 = non-settled
  standing) shipped in `a548be4`.

## [Unreleased chronology - 2026-09-30]

### Changed

- Fabric harness retired (`8553cb1`): CASTLE keeps only the envelope. The closure
  test moved to XaaS against real planes. Added envelope tests (digest,
  schema-required fields, effect ordering, no-runtime source check) in
  `tests/operation_envelope.rs`; removed the in-repo Rust fabric harness
  (`tests/common/fabric/*`, `tests/castle_alive.rs`,
  `tests/common/fabric_planes.rs`; -1183 lines). `configs/CAPABILITY_FABRIC.json`
  and `docs/rfc/castle-fabric/README.md` updated to the XaaS-hosted design.

### Added

- Machine-readable failure JSON on stdout with typed exit codes (`a548be4`):
  exit 2 for `REFUSED:`/`BLOCKED:`, 3 for non-settled standing, 1 otherwise;
  stderr line unchanged. Lets XaaS effectors classify castle failures without
  parsing stderr. (`src/bin/castle/main.rs`, `tests/cli_payments.rs`.)
