# PAYMENTS

CASTLE SA2A payments kernel and its `castle payments` CLI. Source:
`src/payments/*.rs`, `src/bin/castle/verbs/payments_{routes,handlers}.rs`. Release line
`castle::payments::VERSION` = `26.9.29`.

## Lifecycle

Internal-ledger path (v1, unchanged):

```text
prepare -> admit_payment -> execute_payment -> (reconcile | recover) -> explain
```

Rail path (v26.9.29):

```text
prepare -> admit (screened | journaled) -> rail-submit -> finalize x N -> receipt -> replay
```

1. `prepare` builds a `PreparedEffect` (capability `payments.transfer.v1`). It is inert: the
   CLI prints `"grants_authority": false`. Its SA2A digest is the economic idempotency key.
2. Admission runs in order: strict parse, [screening when requested], certificate
   verification (threshold signatures, epochs, audience, validity window), static spend
   policy, derived-obligation check when the policy requires it, reversal checks, durable
   nonce claim, atomic budget reservation plus fsynced claim (`Reserved`).
   - `admit_payment_screened` runs sanctions controls and resolves payer and payee in the
     counterparty registry BEFORE the nonce is burned or budget reserved.
   - `admit_payment_journaled` additionally persists the stateless admission inputs.
   - A policy with `require_screening: true` refuses the unscreened path
     (`REFUSED:PAYMENT_SCREENING_REQUIRED`).
3. `execute_payment` (internal ledger) manufactures and admits a CONSTRUCT, then actuates
   only through `castle::execute_powl_with_gym_act` wrapped in durable BRCE journaling
   (reserve, post). Claim moves to `Executed`, `Refused` or `UnknownOutcome`.
4. `submit_via_rail` (rail path) seals a `PreparedEconomicEffect`, builds the ISO 20022
   pain.001 payload, manufactures and admits a CONSTRUCT, and runs `payments.hold` then
   `payments.rail_submit` through the same DO path under durable BRCE. It places a hold and
   never writes a ledger entry. Claim moves to `Submitted`, `Refused` or `UnknownOutcome`.
5. `finalize_via_rail` polls the authoritative rail and applies sealed finality
   (`apply_finality`): settlement converts the hold into the ledger entry (`Final`), a
   rejection releases the hold (`Refused`), a return posts the inverse entry (`Returned`).
   The ledger shows settled only after finality was observed.
6. `reconcile` resolves `Reserved`/`UnknownOutcome` claims of the internal-ledger path from
   the ledger's own record. On the rail path, `finalize` is the resolver.
7. `recover` pairs BRCE prepare and outcome records after a crash.
8. `explain` answers "why did money move" from the persisted claim and ledger entry only.
9. `receipt` seals one `EventReceipt` from persisted claim, ledger, rail, submission and
   admission-journal state; `replay` re-derives the stateless admission decision.

## Claim state machine

- `Reserved`: admitted, budget reserved, DO not yet observed. Enters from admission. Leaves to
  `Submitted`, `Executed`, `Refused` or `UnknownOutcome`.
- `Submitted`: rail acknowledged, funds held, finality not observed. Enters from `Reserved`
  or `UnknownOutcome`. Leaves to `Final`, `Refused` or `UnknownOutcome`.
- `Final`: rail settlement observed, ledger entry written from it. Enters from `Submitted` or
  `UnknownOutcome`. Leaves to `Returned`.
- `Returned`: settled funds returned, inverse ledger entry posted. Enters from `Final`.
  Terminal; still counts as settled, so re-paying needs a new obligation.
- `Executed`: internal-ledger settlement observed. Enters from `Reserved` or `UnknownOutcome`.
  Terminal.
- `Refused`: definite non-submission, rejection or proven absence; budget released. Enters
  from `Reserved`, `Submitted` or `UnknownOutcome`. The same effect may be re-admitted.
- `UnknownOutcome`: ambiguous; hold retained; blind retry forbidden. Enters from `Reserved`
  or `Submitted`. Leaves to `Submitted`, `Final` or `Refused` via `finalize` (rail path), or
  to `Executed` or `Refused` via `reconcile` (internal-ledger path).

Serialized form is snake_case (`unknown_outcome`). Evidence that conflicts with the current
state (for example a return before settlement, or a rejection after settlement) is refused
with `REFUSED:PAYMENT_EVIDENCE_CONFLICT` and changes nothing.

## Invariants

- Amounts are canonical base-10 minor-unit strings (no sign, no leading zeros, no
  separators, non-zero, fits `u64`). No floating point in the kernel or CLI handlers.
- One claim file per effect digest, created with fsynced `create_new`.
- One live claim per (principal, obligation id), except reversals.
- Per-principal epoch cap is the sum of live non-reversal claims.
- Ledger holds at most one entry per effect digest; `post` is idempotent.
- Ledger conservation: balances sum to openings, per currency.
- `execute` refuses the file ledger unless `--reference-ledger` is passed, before any
  state is written.
- `reconcile` and `explain` never create a ledger; a missing `opening.json` refuses with
  `PAYMENT_LEDGER_NOT_FOUND`, so an empty fabricated ledger cannot "prove absence".
- Non-`Settled` execution exits non-zero with the reason and `standing=` in the error.
- `rail-submit` and `finalize` and `receipt` refuse without `--reference-rail`: `SimRail` is a
  test rail, not a bank. The refusal happens before any state is written.
- `rail-submit` exits non-zero for `Refused` and `UnknownOutcome`; `finalize` exits non-zero
  when the claim it leaves is `Refused` or `UnknownOutcome`. The reason is on stderr.
- `finalize` and `receipt` never create a ledger or a rail; a missing root refuses.
- `reconcile` refuses while a rail hold is live (`PAYMENT_RECONCILE_RAIL_HOLD_LIVE`): the
  ledger has no entry before finality, so "absent" would be false.
- An ambiguous rail outcome is never resubmitted; only `finalize` against the rail resolves it.
- No layer increases authority while translating an object: rail adapter, PEE seal and
  ISO projection carry the admitted amount, parties and digest, or refuse.

## Typed refusals

Kernel refusals (`src/payments/refusal.rs`, `reconcile.rs`):

| Code | Meaning |
|---|---|
| `REFUSED:PAYMENT_CAPABILITY_NOT_ADMITTED` | Effect capability is not `payments.transfer.v1` |
| `REFUSED:PAYMENT_PAYLOAD_INVALID` | Subject or payload shape or empty field invalid |
| `REFUSED:PAYMENT_AMOUNT_NOT_INTEGER` | Amount is not a canonical integer string |
| `REFUSED:PAYMENT_AMOUNT_ZERO` | Amount is zero |
| `REFUSED:PAYMENT_AMOUNT_EXCEEDS_CAP` | Amount above per-effect cap |
| `REFUSED:PAYMENT_CURRENCY_UNSUPPORTED` | Currency outside the closed set or no cap |
| `REFUSED:PAYMENT_ACCOUNT_NOT_ALLOWED` | Payer or payee not in principal policy |
| `REFUSED:PAYMENT_SAME_ACCOUNT` | Payer equals payee |
| `REFUSED:PAYMENT_BUDGET_EXCEEDED` | Epoch cap would be exceeded |
| `REFUSED:NO_SPEND_POLICY_FOR_PRINCIPAL` | No policy for the principal |
| `REFUSED:PAYMENT_OUTCOME_UNKNOWN` | Claim is `UnknownOutcome`; run `reconcile` |
| `REFUSED:PAYMENT_ALREADY_SETTLED` | Claim already `Executed` |
| `REFUSED:PAYMENT_OBLIGATION_ALREADY_CLAIMED` | Another live claim holds the obligation |
| `REFUSED:PAYMENT_IN_FLIGHT` | Claim is `Reserved` |
| `REFUSED:PAYMENT_INSUFFICIENT_FUNDS` | Ledger balance below amount |
| `REFUSED:PAYMENT_LEDGER_UNAVAILABLE` | Ledger unreachable or unreadable |
| `REFUSED:PAYMENT_REVERSAL_ORIGINAL_NOT_EXECUTED` | Reversal target missing or not `Executed` |
| `REFUSED:PAYMENT_REVERSAL_EXCEEDS_ORIGINAL` | Cumulative reversals above original |
| `REFUSED:PAYMENT_REVERSAL_MISMATCH` | Reversal parties, currency or principal differ |
| `REFUSED:RECONCILIATION_DRIFT` | Ledger entry disagrees with claim |
| `REFUSED:PAYMENT_NOT_RECONCILABLE` | Claim not `Reserved`/`UnknownOutcome` |
| `BLOCKED:PAYMENT_CLAIM_STORE_FAILED` | Claim store I/O failed |

Refusals from the v26.9.29 modules (grep `pub const` in `src/payments/*.rs` to regenerate):

| Code | Module | Meaning |
|---|---|---|
| `REFUSED:PAYMENT_CLAIM_NOT_RESERVED` | execute | Claim missing, not `Reserved`, or stale |
| `REFUSED:PAYMENT_RAIL_HOLD_UNAVAILABLE` | execute_rail | Ledger could not place the hold |
| `REFUSED:PAYMENT_RAIL_TIME_INVALID` | execute_rail | Clock value not representable |
| `REFUSED:PAYMENT_RAIL_REJECTED` | execute_rail | Rail acknowledged with a rejection |
| `REFUSED:PAYMENT_NOT_SUBMITTED` | settlement | Finality for a claim that was not submitted |
| `REFUSED:PAYMENT_NOT_HELD` | settlement | Settlement evidence with no hold and no entry |
| `REFUSED:PAYMENT_NOT_FINAL` | settlement | Return evidence before settlement |
| `REFUSED:PAYMENT_EVIDENCE_CONFLICT` | settlement | Evidence contradicts the claim state |
| `REFUSED:PAYMENT_SCREENING_REQUIRED` | admission | Screening required, path unscreened |
| `REFUSED:PAYMENT_SANCTIONS_HIT` | compliance | Payer or payee on the sanctions list |
| `REFUSED:SANCTIONS_LIST_INVALID` | compliance | List malformed, empty or duplicated |
| `REFUSED:NO_COMPLIANCE_CONTROLS` | compliance | Screening invoked with zero controls |
| `REFUSED:COMPLIANCE_CONTROL_DUPLICATE` | compliance | Same control id twice |
| `REFUSED:COUNTERPARTY_UNVERIFIED` | counterparty | Account unregistered or LEI missing |
| `REFUSED:COUNTERPARTY_LEI_INVALID` | counterparty | LEI fails format or check digits |
| `REFUSED:COUNTERPARTY_ACCOUNT_AMBIGUOUS` | counterparty | One account under two parties |
| `REFUSED:COUNTERPARTY_REGISTRY_INVALID` | counterparty | Registry malformed |
| `REFUSED:EFFECT_IDENTITY_MISMATCH` | pee | Sealed identity or digests do not recompute |
| `REFUSED:EFFECT_EXPIRED` / `REFUSED:EFFECT_NOT_YET_VALID` | pee | Outside the validity window |
| `REFUSED:EFFECT_BINDING_INCOMPLETE` | pee | An `EffectBindings` field is empty |
| `REFUSED:PAYMENT_OBLIGATION_ID_NOT_DERIVED` | obligation | Obligation id not derived |
| `REFUSED:PROJECTION_FIELD_INVALID` | iso20022 | Field cannot be projected into the message |
| `REFUSED:REPLAY_RECORD_MISSING` | replay | No admission record for the digest |
| `REFUSED:REPLAY_RECORD_CORRUPT` | replay | Record unreadable |
| `REFUSED:REPLAY_EFFECT_DIGEST_MISMATCH` | replay | Record digest differs from the request |
| `REFUSED:EVENT_RECEIPT_INCOMPLETE` | event_receipt | Effect, obligation or decision digest empty |
| `REFUSED:EVENT_RECEIPT_TAMPERED` | event_receipt | Receipt digest does not recompute |
| `REFUSED:EXPERIENCE_REQUIRES_SETTLED_RECEIPT` | experience | Learning from unsettled event |

Rail-side reason strings (not refusals of this kernel): `RAIL_NEVER_SAW_EFFECT`,
`RAIL_NEVER_SUBMITTED`, `DUPLICATE_CORRELATION_DIFFERENT_PAYLOAD`,
`REJECTED_BY_RAIL_AFTER_DELAY`, `RETURNED_BY_BENEFICIARY_BANK`.

SA2A certificate refusals surface as `REFUSED:<SecurityRefusal>` (for example
`REFUSED:NonceReplay`).

CLI-only refusals (`payments_handlers.rs`):

| Code | Meaning |
|---|---|
| `REFUSED:REFERENCE_LEDGER_NOT_CONFIRMED` | `--reference-ledger` omitted |
| `REFUSED:PAYMENT_LEDGER_NOT_FOUND` | `reconcile`/`explain` ledger dir has no `opening.json` |
| `REFUSED:PAYMENT_CLAIM_NOT_FOUND` | `explain` digest has no claim |
| `REFUSED:PAYMENT_INPUT_UNREADABLE` / `_INVALID` | Input file unreadable or not valid JSON |
| `REFUSED:REFERENCE_RAIL_NOT_CONFIRMED` | `--reference-rail` omitted |
| `REFUSED:PAYMENT_RAIL_NOT_FOUND` | `finalize`/`receipt` rail dir has no `mode.json` |
| `REFUSED:PAYMENT_RAIL_MODE_INVALID` | `--rail-mode` not one of the listed modes |
| `REFUSED:PAYMENT_SCREENING_INPUTS_INCOMPLETE` | Only one screening path given |
| `REFUSED:PAYMENT_JOURNAL_WITH_SCREENING_UNSUPPORTED` | `--journal-dir` plus screening |
| `REFUSED:PAYMENT_RECONCILE_RAIL_HOLD_LIVE` | `reconcile` while a rail hold is live |
| `REFUSED:RECEIPT_RAIL_HISTORY_MISSING_SETTLEMENT` | Claim `Final`, rail never settled |
| `REFUSED:RECEIPT_SUBMISSION_RECORD_MISMATCH` | Submission record differs from digest |

## CLI usage

All inputs are file-path driven JSON. Flags are required unless noted. Add
`--format json` for compact output. Verified by `tests/cli_payments.rs`.

```bash
castle payments prepare --principal principal:procurement-agent \
  --payer acct:treasury --payee acct:supplier-9821 --amount-minor 470000 \
  --currency USD --obligation-id invoice-9821 --purpose invoice-payment \
  [--reverses <effect-digest>]
```

```bash
castle payments execute --effect-path effect.json --certificate-path cert.json \
  --registry-path registry.json --policy-path policy.json --state-dir state \
  --ledger-dir ledger --opening-path opening.json --reference-ledger \
  --audience actuator:payments --policy-epoch 7 --revocation-epoch 0 --generation 1 \
  --now-ms 1000 --receipt-key-id receipt-key --receipt-seed-hex <64-hex> \
  --allowed-authority actuator:payments
```

```bash
castle payments reconcile --effect-digest sha256:<hex> --state-dir state \
  --ledger-dir ledger --reference-ledger
castle payments recover --state-dir state
castle payments explain --effect-digest sha256:<hex> --state-dir state \
  --ledger-dir ledger --reference-ledger
```

Rail verbs (verified by `tests/cli_payments_rail.rs`):

```bash
castle payments rail-submit --effect-path effect.json --certificate-path cert.json \
  --registry-path registry.json --policy-path policy.json --state-dir state \
  --ledger-dir ledger --opening-path opening.json --reference-ledger --reference-rail \
  --rail-dir rail --rail-mode settle-after-polls:2 \
  --audience actuator:payments --policy-epoch 7 --revocation-epoch 0 --generation 1 \
  --now-ms 1000 --receipt-key-id receipt-key --receipt-seed-hex <64-hex> \
  --allowed-authority actuator:payments \
  --debtor-agent-bic DEUTDEFF --creditor-agent-bic BOFAUS3N \
  --created-at-iso 2026-09-29T12:34:56Z \
  [--journal-dir journal] \
  [--sanctions-path sanctions.json --counterparties-path counterparties.json] \
  [--debtor-name <text>] [--creditor-name <text>] [--effect-expires-at-ms <ms>]
```

`--rail-mode` is one of `honest`, `drop-ack`, `settle-after-polls:N`, `reject-after-polls:N`,
`return-after-settle`, `down`, `duplicate-reports`. Screening inputs must be given together;
`--journal-dir` cannot be combined with them yet. Without `--effect-expires-at-ms` the
sealed effect is valid for one hour after `--now-ms`.

```bash
castle payments finalize --effect-digest sha256:<hex> --state-dir state \
  --ledger-dir ledger --rail-dir rail --reference-ledger --reference-rail [--rail-mode down]
castle payments replay --journal-dir journal --effect-digest sha256:<hex>
castle payments receipt --effect-digest sha256:<hex> --state-dir state --ledger-dir ledger \
  --rail-dir rail --journal-dir journal --reference-ledger --reference-rail
```

`finalize` polls once and prints `result`: `pending`, `applied` (with `applied.outcome` of
`settled`, `already_final`, `released` or `returned`), `still_unknown` or `proven_absent`,
plus the resulting `claim_state`. Call it until the claim is `Final`.

`receipt` prints `receipt` (the sealed `EventReceipt`), `explain` (audit answers derived
from the receipt alone) and `verified: true`. It names the effect and obligation ids, the
sealed effect id, the CONSTRUCT and BRCE digests, the rail correlation id and payload
digest, the finality evidence digest and observation time, the ledger entry digest and the
claim state. Fields the receipt does not carry are `{value: null, reason}`.

Persisted CLI state: `state/cli-rail/<hex>.submission.json` (written by `rail-submit`) and
`state/cli-rail/<hex>.finality.json` (written by the first `finalize` that sees settlement;
`observed_at_ms` is the wall clock at that observation).

Example failure (`REFUSED:REPLAY_RECORD_MISSING`; stderr keeps the legacy line, exit 2):

```text
ERROR: Command execution failed: REFUSED:REPLAY_RECORD_MISSING
```

The same error is additionally printed as JSON on stdout for callers that shell
out (XaaS effectors). Exit codes are typed: `REFUSED:`/`BLOCKED:` failures exit
2, `standing=` failures exit 3, anything else exits 1.

```text
{"ok":false,"class":"refused","code":"REFUSED:REPLAY_RECORD_MISSING","detail":"Command execution failed: REFUSED:REPLAY_RECORD_MISSING"}
```

Input shapes: `effect.json` is a `PreparedEffect`; `cert.json` an `ActuationCertificate`;
`registry.json` an array of `KeyRecord`; `policy.json` a `SpendPolicy`; `opening.json` an
array of `{account, currency, amount_minor}`. `--state-dir` holds `claims/`, `nonces/`,
`brce/` (and `brce/rail/` for rail submissions). Committed samples:
`fixtures/payments/cli/{policy,opening,sanctions-clear,sanctions-hit,counterparties}.json`. Schemas:
`schemas/payments/*.schema.json`.

`explain` output fields: `principal`, `payer`, `payee`, `amount_minor`, `amount_decimal`,
`currency`, `obligation_id`, `purpose`, `authority_audience`, `verified_custodian_ids`,
`construct_digest`, `ledger_seq`, `claim_state`.

## Hostile-rail behavior

`SimRail` (`src/payments/rail_sim.rs`) is a durable rail simulator under a root directory.
It is independent of the kernel and keeps its own record per correlation id.

- `honest`: `Accepted` on poll 1, `Settled` on poll 2. Kernel: `Submitted`, then `Final`.
- `drop-ack`: accept persisted, caller gets a timeout, settles on poll 2. Kernel:
  `UnknownOutcome` with the hold retained; `finalize` resolves to `Final`; one ledger entry.
- `settle-after-polls:N`: `Accepted` for N polls, then `Settled`. Kernel: `pending` N times,
  then `Final`.
- `reject-after-polls:N`: `Accepted` for N polls, then `Rejected`. Kernel: hold released,
  claim `Refused`, `finalize` exits non-zero.
- `return-after-settle`: `Settled`, then `Returned`. Kernel: `Final`, then `Returned` with
  the inverse ledger entry.
- `down`: every call errors. Kernel: `rail-submit` gives `UnknownOutcome`; `finalize` gives
  `still_unknown` with state unchanged.
- `duplicate-reports`: the terminal report repeats. Kernel: idempotent, one ledger entry.

- A duplicate correlation id with a different payload is rejected by the rail
  (`DUPLICATE_CORRELATION_DIFFERENT_PAYLOAD`); the same payload is acknowledged again.
- A rail acknowledgement for a different correlation id is treated as `UnknownOutcome`.
- A `Returned` report implies prior settlement: `finalize` applies settlement first if unseen.
- A reachable rail that has no record of the correlation proves absence only for a claim in
  `UnknownOutcome` (`proven_absent`: hold released, claim `Refused`). For any other state
  the result is `still_unknown` and nothing changes.

## Replay semantics

`admit_payment_journaled` writes an `AdmissionRecord` after a successful admission: prepared
effect, certificate, key registry records, epochs, audience, clock, spend policy. A refused
admission writes nothing. `replay_admission` (CLI: `payments replay`) reads only that record.
It never touches the claim store, ledger, rail, nonce fence or network and never actuates.

Re-derived: strict parse, effect digest, certificate verification against the recorded
registry and epochs, static spend policy, derived-obligation rule. Result `Reproduced`
carries a `decision_digest` over the checks performed.

Excluded by design: durable nonce, budget and epoch cap, claim uniqueness, reversal-original
lookup. These depend on what else was admitted before or since. The verdict therefore reads
"the decision was lawful given its recorded inputs", not "it would be admitted again now".

A missing or corrupt record is an error (`REPLAY_RECORD_MISSING`, `REPLAY_RECORD_CORRUPT`).
A readable record that fails a stateless check is `Diverged`, which exits non-zero.

## ISO 20022

`castle::payments::iso20022` projects an admitted effect into `pain.001.001.09` (customer
credit transfer) and `pacs.008` (FI credit transfer). Projection needs debtor and creditor
agent BICs and refuses fields it cannot represent (`PROJECTION_FIELD_INVALID`). The payload
digest is bound into the sealed effect and the rail instruction.

`tests/payments_iso20022_official.rs` validates output with `xmllint --schema` against
version-pinned XSDs. The XSDs are third-party mirrors and are not committed. Pins:
`fixtures/payments/iso20022/xsd/PINS.json`; location: `CASTLE_ISO20022_XSD_DIR`. The test
fails when the files are absent or their sha256 differs from the pin; it never skips.

## ISO 20022 conformance testing

`tests/payments_iso20022_official.rs` is the official-XSD conformance suite: five tests that
run a real `xmllint --schema` subprocess over the projection output. It validates (1) generated
messages for USD, JPY and KWD, (2) the committed goldens under `fixtures/payments/iso20022/`,
(3) that mutated documents (over-precise amounts, removed `DbtrAgt`/`CdtrAgt`, wrong namespace)
are rejected, (4) that no placeholder `example.org` namespace appears anywhere in `src/payments`
or `fixtures/payments`, and (5) that kernel-derived long obligation ids project into bounded,
round-trippable, XSD-valid messages.

The suite never skips: a missing or mismatched XSD is a loud failure, not a degraded pass.
Because `iso20022.org` returns HTTP 403 to non-browser clients, the XSD bytes are not
committed; instead `fixtures/payments/iso20022/xsd/PINS.json` pins each file by sha256 with a
fetchable URL (byte-identical mirrors in `php-sepa-xml` and `moov-io/fedwire20022`), so any
machine can reconstruct byte-identical inputs and a conformance pass means the same thing
everywhere — reproducibility, not convenience.

To run:

```sh
mkdir -p /tmp/iso-xsd && cd /tmp/iso-xsd
curl -fsSL -o pain.001.001.09.xsd \
  https://raw.githubusercontent.com/php-sepa-xml/php-sepa-xml/HEAD/doc/ISO20022/pain/001/001/pain.001.001.09.xsd
curl -fsSL -o pacs.008.001.08.xsd \
  https://raw.githubusercontent.com/moov-io/fedwire20022/HEAD/xsd/iso/pacs.008.001.08.xsd
shasum -a 256 *.xsd   # must match PINS.json
cd /path/to/castle
CASTLE_ISO20022_XSD_DIR=/tmp/iso-xsd cargo test --test payments_iso20022_official
```

Witnessed pass: the v26.10.8 fleet campaign ran the full 5-test suite green with fetched,
pin-verified XSDs on this machine.

## UNSUPPORTED / UNKNOWN

- UNSUPPORTED: Fedwire, SWIFT, ACH, FedNow and card rails. Only `SimRail` exists; no adapter
  talks to a bank.
- UNSUPPORTED: HSM or KMS key custody. Signing keys are software keys in tests;
  `--receipt-seed-hex` is a process argument.
- NONE: regulated-entity standing. No licence, safeguarding account or regulator relationship.
- UNSUPPORTED: xaas Postgres ledger. The CLI drives `FileJournalLedger` only, behind
  `--reference-ledger`.
- UNSUPPORTED: ash_a2a digest contract. Digests are SA2A `PreparedEffect` digests.
- UNKNOWN: cross-process budget atomicity. Claim uniqueness rests on fsynced `create_new`;
  budget sums are serialized by an in-process lock and `DirLock` on one filesystem.
- UNKNOWN: ISO 20022 XSD conformance is third-party mirrors, not committed; it depends on
  `CASTLE_ISO20022_XSD_DIR`.
- UNSUPPORTED: FX and multi-currency transfers (one currency per transfer).
- UNSUPPORTED: delivery-versus-payment and payment-versus-payment atomic settlement.
- UNSUPPORTED: screening combined with `--journal-dir` in the CLI. The library has no
  journaled-and-screened admission; the CLI refuses the combination.
- UNKNOWN: sanctions list provenance. The list is a caller-supplied JSON file; no feed and no
  freshness proof.
- UNKNOWN: finality evidence provenance. `apply_finality` accepts caller-built evidence; see
  the `rf5_finding_*` entry in `RELEASE_v26.9.29.md`.

## FIBO alignment

Payments vocabulary aligns to the Financial Industry Business Ontology (FIBO) as a
projection, never as authority.

- Source: `ontology/payments-fibo/payments-fibo.ttl` (RDF). Generated table:
  `ontology/payments-fibo/generated/fibo_generated.rs`, produced by
  `cd ontology/payments-fibo && ggen sync run`. Do not hand-edit generated output.
- Hand-written API: `castle::payments::fibo` (`currency_iri`, `monetary_amount_json`,
  `claim_type_iri`, `claim_jsonld`, `mapping_for`, `unverified_terms`).
- Mapped terms: `MonetaryAmount`, `Currency`, `hasAmount`, `hasCurrency`, ISO 4217
  individuals (USD, EUR, GBP, JPY, KWD), `Payment`, `PaymentObligation`, `Payer`,
  `Payee`, `hasPaymentAmount`, `Settlement`, `LegalEntityIdentifier`.
- Typing rule: only a settled (`Executed`) claim projects as FIBO `Payment`; reserved,
  refused and unknown-outcome claims stay `castle:PaymentEffect`.
- Amounts: `hasAmount` is a decimal string using the currency exponent; minor units
  are carried in `castle:minorUnits`. No floats.
- Pinning: each row records the corpus file sha256 and that module's own
  `owl:versionIRI`. The corpus modules carry different release versionIRIs, so no
  single FIBO release tag is claimed. Corpus:
  `ggen-marketplace/ontologies/public/fibo` (override `CASTLE_FIBO_CORPUS`).
- Court: `tests/payments_fibo.rs` fails if a verified IRI is not declared in the
  corpus, if the corpus file drifts from its pinned sha256, or if the generated table
  is stale against the RDF source.
- Not in FIBO, kept in the `castle:` namespace: reversal, finality, sanctions, sealed
  admission, `CONSTRUCT != DO`. `Settlement` and `LegalEntityIdentifier` are mapped in
  the table. Finality (`Submitted`, `Final`, `Returned`) and LEI counterparty evidence now
  exist in the kernel; the FIBO projection still types only `Executed` claims as `Payment`,
  and does not yet project `Final` claims, `Settlement` or `LegalEntityIdentifier`.

## See Also

- `README.md` (DfCM CONSTRUCT origin law)
- `CLAUDE.md` (receipt systems, typed refusals, CLI seam)
- `schemas/sa2a/` (certificate, key record, prepared effect schemas)
- `docs/payments/RELEASE_v26.9.29.md` (release contents, verification ladder, falsifiers)
- `tests/cli_payments.rs`, `tests/cli_payments_rail.rs`, `tests/payments_*.rs`
