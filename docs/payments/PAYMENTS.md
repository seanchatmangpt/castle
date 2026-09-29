# PAYMENTS

CASTLE SA2A payments kernel and its `castle payments` CLI. Source:
`src/payments/*.rs`, `src/bin/castle/verbs/payments_{routes,handlers}.rs`.

## Lifecycle

```text
prepare -> admit_payment -> execute_payment -> (reconcile | recover) -> explain
```

1. `prepare` builds a `PreparedEffect` (capability `payments.transfer.v1`). It is inert: the
   CLI prints `"grants_authority": false`. Its SA2A digest is the economic idempotency key.
2. `admit_payment` runs in order: strict parse, certificate verification (threshold
   signatures, epochs, audience, validity window), static spend policy, reversal checks,
   durable nonce claim, atomic budget reservation plus fsynced claim (`Reserved`).
3. `execute_payment` manufactures and admits a CONSTRUCT, then actuates only through
   `castle::execute_powl_with_gym_act` wrapped in durable BRCE journaling (reserve, post).
   Claim moves to `Executed`, `Refused` (definite absence, budget released) or
   `UnknownOutcome` (ambiguous; blind retry forbidden).
4. `reconcile` resolves `Reserved`/`UnknownOutcome` claims from the ledger's own record.
5. `recover` pairs BRCE prepare and outcome records after a crash.
6. `explain` answers "why did money move" from the persisted claim and ledger entry only.

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

SA2A certificate refusals surface as `REFUSED:<SecurityRefusal>` (for example
`REFUSED:NonceReplay`).

CLI-only refusals (`payments_handlers.rs`):

| Code | Meaning |
|---|---|
| `REFUSED:REFERENCE_LEDGER_NOT_CONFIRMED` | `--reference-ledger` omitted |
| `REFUSED:PAYMENT_LEDGER_NOT_FOUND` | `reconcile`/`explain` ledger dir has no `opening.json` |
| `REFUSED:PAYMENT_CLAIM_NOT_FOUND` | `explain` digest has no claim |
| `REFUSED:PAYMENT_INPUT_UNREADABLE` / `_INVALID` | Input file unreadable or not valid JSON |

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

Input shapes: `effect.json` is a `PreparedEffect`; `cert.json` an `ActuationCertificate`;
`registry.json` an array of `KeyRecord`; `policy.json` a `SpendPolicy`; `opening.json` an
array of `{account, currency, amount_minor}`. `--state-dir` holds `claims/`, `nonces/`,
`brce/`. Committed samples: `fixtures/payments/cli/{policy,opening}.json`. Schemas:
`schemas/payments/*.schema.json`.

`explain` output fields: `principal`, `payer`, `payee`, `amount_minor`, `amount_decimal`,
`currency`, `obligation_id`, `purpose`, `authority_audience`, `verified_custodian_ids`,
`construct_digest`, `ledger_seq`, `claim_state`.

## UNSUPPORTED / UNKNOWN in v1

- UNSUPPORTED: external payment rails (card, ACH, SWIFT, RTP).
- UNSUPPORTED: FX; one currency per transfer.
- UNSUPPORTED: delivery-versus-payment and payment-versus-payment atomic settlement.
- UNSUPPORTED: xaas Postgres ledger. The CLI drives `FileJournalLedger` only.
- UNKNOWN: cross-process budget atomicity. Claim uniqueness rests on fsynced `create_new`;
  budget sums and transitions are serialized by an in-process lock only.
- UNKNOWN: ISO 20022 XSD validation. The library projection is not schema-validated and
  the CLI exposes no ISO verb.
- `--receipt-seed-hex` is a command-line argument and is visible to process listings; a
  key-file or KMS input is not implemented.

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
  the table but not yet projected (finality and counterparty gaps remain open).

## See Also

- `README.md` (DfCM CONSTRUCT origin law)
- `CLAUDE.md` (receipt systems, typed refusals, CLI seam)
- `schemas/sa2a/` (certificate, key record, prepared effect schemas)
- `tests/cli_payments.rs`, `tests/payments_*.rs`
