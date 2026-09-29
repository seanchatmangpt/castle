# RELEASE v26.9.29

Payments release line `castle::payments::VERSION` = `26.9.29`. Crate version is unchanged;
the release tags the closure layer over the v1 kernel described in `PAYMENTS.md`.

## What shipped

Kernel modules under `src/payments/`:

- `pee`: `PreparedEconomicEffect::seal(admission, &EffectBindings)`, `verify_identity`,
  `check_fresh`. Sealed identity binds amount, parties, authority, policy, law state,
  counterparty evidence, rail profile and validity window.
- `rail`, `rail_sim`: `RailActuator {submit, status}`; durable hostile `SimRail` with modes
  Honest, DropAckAfterAccept, RejectAfterPolls, SettleAfterPolls, ReturnAfterSettle, Down,
  DuplicateReports.
- `execute_rail`: `submit_via_rail` (hold, then rail submit, inside durable BRCE; no ledger
  entry) and `finalize_via_rail` (Pending, Applied, StillUnknown, ProvenAbsent).
- `settlement`: `apply_finality` over `FinalityEvidence`; conflicting evidence refused.
- `ledger`: `hold`, `release_hold`, `settle_hold`, `settle_return`, `available`, `hold_of`
  on `LedgerPort`, implemented by `FileJournalLedger`.
- `claim_store`: states `Submitted`, `Final`, `Returned` added.
- `compliance`, `counterparty`: `SanctionsList`, `run_controls`, `Lei`,
  `CounterpartyRegistry`; `admit_payment_screened`; policy flag `require_screening`.
- `obligation`: derived obligation ids; policy flag `require_derived_obligation`.
- `iso20022`: pain.001 and pacs.008 projection, checked against official XSDs.
- `fibo`: FIBO alignment generated from RDF by ggen.
- `replay`, `event_receipt`: admission journal and stateless replay; sealed event receipt
  with `explain`.
- `experience`: known-class rules; learning only from settled receipts.
- `dirlock`, `nonce`: cross-process directory lock and durable nonce fence.

CLI (`castle payments`): `rail-submit`, `finalize`, `replay`, `receipt` added to `prepare`,
`execute`, `reconcile`, `recover`, `explain`. `reconcile` now refuses while a rail hold is
live. Schemas: `claim-v1` gains `submitted`, `final`, `returned`; `spend-policy-v1` gains
`require_derived_obligation` and `require_screening`.

## Verification ladder

Run each rung with an isolated target dir:

```bash
export CARGO_TARGET_DIR=/Users/sac/castle/target-lane-cli-docs
cargo build --bin castle
cargo test --lib
cargo test --test payments_money --test payments_pee --test payments_rail_sim
cargo test --test payments_settlement --test payments_holds --test payments_compliance
cargo test --test payments_replay --test payments_event_receipt --test payments_screening
cargo test --test payments_rail_e2e
cargo test --test payments_report_falsifiers
cargo test --test payments_report_falsifiers -- --ignored
cargo test --test cli_payments --test cli_payments_rail
cargo test
```

Observed on 2026-09-29 with concurrent lanes editing other files: `cargo test` full suite
reported 0 failed in every test binary. Selected counts (passed): `cli_payments` 5,
`cli_payments_rail` 11, `payments_rail_e2e` 9, `payments_screening` 6, `payments_replay` 8,
`payments_event_receipt` 5, `payments_report_falsifiers` 10 with 3 ignored. The ignored
tests are findings; they were not run in that pass.

## Falsifiers F1 to F10

Each falsifier asserts the negation of one operator-report failure claim against real state
(real files, crypto, BRCE, `SimRail`, `FileJournalLedger`). Tests live in
`tests/payments_report_falsifiers.rs`; `tests/payments_qualification.rs` was not present
when this table was written.

- F1, authentication is not authorization:
  `rf1_authenticated_does_not_imply_authorized_payment`.
- F2, a changed effect cannot reuse an authorization:
  `rf2_changed_effect_cannot_reuse_authorization`.
- F3, a timeout is never a retry: `rf3_timeout_is_never_a_retry`.
- F4, the obligation-to-effect join is durable and total:
  `rf4_obligation_effect_join_is_durable_and_total`.
- F5, the ledger is never settled before observed finality:
  `rf5_ledger_never_settled_before_observed_finality`.
- F6, replay never actuates: `rf6_replay_never_actuates`.
- F7, a rail adapter cannot increase authority: `rf7_rail_adapter_cannot_increase_authority`.
- F8, ISO conformance is not the example ontology:
  `rf8_iso_conformance_is_not_the_example_ontology`.
- F9, the receipt reconstructs the six questions: `rf9_receipt_reconstructs_the_six_questions`.
- F10, a known class completes the full rail path without intelligence:
  `rf10_known_class_completes_full_rail_path_without_intelligence`.

Related CLI courts in `tests/cli_payments_rail.rs`:

- Dropped acknowledgement is `UnknownOutcome`, exits non-zero, is not resubmitted, and
  `finalize` yields one ledger entry:
  `drop_ack_is_unknown_nonzero_then_finalize_applies_exactly_one_entry`.
- Missing `--reference-rail` refuses before any state exists:
  `missing_reference_rail_refuses_before_any_state_is_written`.
- `reconcile` cannot prove absence while a rail hold is live:
  `reconcile_must_not_prove_absence_while_rail_hold_is_live`.

## Known gaps

Ignored findings in `tests/payments_report_falsifiers.rs` (fail under `--ignored`):

- `rf1_finding_single_custodian_self_declared_threshold_is_admitted`: the certificate
  verifier honors `certificate.threshold` with no policy minimum, so one custodian can
  self-declare threshold 1. Patch: add a minimum threshold sourced from `PrincipalPolicy`.
- `rf5_finding_fabricated_finality_evidence_settles_ledger`: `apply_finality` trusts
  caller-built `FinalityEvidence`. Patch: private fields, `pub(crate)` constructor from a
  polled `RailStatus`, and a correlation-id check.
- `rf9_finding_explain_omits_the_sealed_effect_identity`: `explain()` drops
  `pee_effect_id`. Patch: add it to the `explain()` JSON.

Other gaps:

- Library `reconcile()` (`src/payments/reconcile.rs`) resolves from the ledger alone and marks
  a rail-path `UnknownOutcome` claim `Refused` (proven absent) while a hold is live and the
  rail may have accepted. The CLI guards this; the library does not. Patch: refuse with
  `PAYMENT_NOT_RECONCILABLE` when `ledger.hold_of(digest)` is `Some`.
- `FinalizeResult::Applied` carries no evidence digest. The CLI derives its own finality
  digest (`CASTLE-CLI-FINALITY-OBSERVATION-V1`) from the rail history, which is not the
  digest passed to `apply_finality`. Patch: return the applied evidence digest.
- Library has no journaled-and-screened admission; CLI refuses `--journal-dir` with screening.
- `payments_iso20022_official` defaults its XSD directory to a session scratchpad path; set
  `CASTLE_ISO20022_XSD_DIR` elsewhere.
- CLI `finalize` records `observed_at_ms` from the wall clock; it is an observation time,
  not rail-supplied.
- UNSUPPORTED and UNKNOWN items (real rails, key custody, regulated standing, xaas Postgres
  ledger, ash_a2a digest contract, cross-process budget atomicity) are listed in
  `PAYMENTS.md`.

## Definition-of-done counters

Re-derive these before claiming done; values below are what was observed, or `UNKNOWN`.

- `cargo test` failures across all binaries: 0 (observed 2026-09-29).
- Ignored finding tests: 3 in `payments_report_falsifiers` (observed); each must be fixed or
  carry a named owner.
- Payments-related tests passed (`payments_*` plus `cli_payments*`): 187 (sum of per-binary
  counts observed in the same run).
- `tests/payments_qualification.rs`: not present, UNKNOWN.
- Mock grep over payments tests, expected zero:

```bash
grep -rnE "mockall|Mock::|unittest.mock|monkeypatch|jest.mock" tests/payments_*.rs \
  tests/cli_payments*.rs tests/common
```

- Float grep over payment sources, expected zero:

```bash
grep -rnE "f64|f32" src/payments src/bin/castle/verbs/payments_handlers.rs
```

## See Also

- `docs/payments/PAYMENTS.md`
- `tests/cli_payments_rail.rs`, `tests/payments_report_falsifiers.rs`
- `fixtures/payments/iso20022/xsd/PINS.json`
- `CLAUDE.md`
