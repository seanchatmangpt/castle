# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

CASTLE is a self-adversarial security-testing framework for owned/authorized infrastructure, implemented
as a Rust crate. It derives minimal vulnerability conditions backward from prohibited adversarial goals
(DfCM inverse construction), compiles them into POWL causal partial orders, and gates all actuation behind
a cryptographically receipted `CONSTRUCT` admission chain. There is no unreceipted DO path — see
`README.md`'s "DfCM CONSTRUCT origin law" section before touching `src/castle.rs`.

Core invariant, everywhere in this codebase: `CONSTRUCT != DO`. Planning/construction functions produce
inert candidates; only `execute_powl_with_gym_act` can actuate, and it requires a genuine
`ConstructAdmission` manufactured exclusively by `admit_construct_for_do`. Unlike the TypeScript
predecessor of this crate (which used a `Symbol`-branded object to make forgery impossible), this is
enforced by Rust module privacy: `ConstructAdmission` carries a private field from a `mod sealed` type, so
it is structurally impossible to construct one outside `castle::castle::admit_construct_for_do` — the
compiler, not a runtime check, is the guarantee.

## Commands

```bash
cargo build              # compile the lib + the `castle` CLI binary
cargo build --bin castle # compile just the CLI
cargo test                # run all tests (lib proptests, tests/castle.rs, tests/board.rs,
                           # tests/fortune5.rs, tests/prop_dfcm.rs, tests/cli_*.rs)
cargo test <name>         # run a single test by (substring of) name, e.g. cargo test dfcm_derives
cargo clippy              # warn-only pedantic/nursery lints (see [lints] in Cargo.toml) — never fails the build
```

No lint config beyond `[lints]` in `Cargo.toml` (warn-only `clippy::all`/`pedantic`/`nursery`,
`unsafe_code = "warn"` — deliberately inventory-before-enforce, matching `~/ggen`'s own stance).
Dependencies: `blake3`, `ed25519-dalek`, `serde`/`serde_json`, `async-trait`, `rand`, `tracing`,
`clap-noun-verb`/`clap-noun-verb-macros`/`linkme` — chosen to match `~/ggen`'s own dependency pins (the
chatman/ggen ecosystem's Rust engine), not a from-scratch or Node-ported choice.

## Architecture

### Generated vs. runtime code — do not hand-edit generated files

`src/generated.rs` and `src/fortune5_generated.rs` are projections of an external ontology
(`ggen-marketplace/packs/castle-pack/ontology.ttl`, referenced via `ggen.toml`) and are **not** owned by
this repo. `docs/GENERATED_ARCHITECTURE.md` and `docs/FORTUNE5_REQUIREMENTS.md` are the human-readable
projections of the same source. If a change is needed there, it belongs upstream in
`ggen-marketplace/packs/castle-pack/templates/src/generated.rs.tmpl` /
`.../fortune5_generated.rs.tmpl` — the runtime "must not silently redefine" those facts (see
`PROVENANCE.md`). Everything else under `src/` is hand-written runtime logic.

### The two independent receipt systems

The codebase has two separate, non-interoperable receipt/digest schemes — don't mix their types:

1. **`src/castle.rs` — CONSTRUCT chain** (`Receipt`, BLAKE3-only via the `Blake3Provider` trait, pluggable
   `ReceiptSigner`/`ReceiptVerifier` traits). Used for the DfCM → CONSTRUCT → GymAct → OCEL pipeline.
   Signing is caller-supplied (no built-in crypto in this module), so tests typically inject a real
   `blake3`-crate digester and a real `ed25519-dalek` keypair — see `tests/castle.rs`'s `RealBlake3` /
   `Ed25519Signer` / `Ed25519Verifier`, deliberately real collaborators rather than fakes.
2. **`src/board.rs` — ReceiptV2 chain** (`ReceiptCoreV2`/`ReceiptV2`, BLAKE3 + real Ed25519 baked in via
   `ed25519-dalek`, `TrustStore` with key epochs/revocation). Used for Fortune-5 evidence admission and
   board-package qualification (`qualify_verified_fortune5`, `qualify_fortune5_board`, `admit_evidence`,
   `verify_receipt_dag`).

Both independently reimplement canonical JSON serialization (`canonical_json` in each file) for
digest-stability — sorted object keys, no non-finite numbers (`board.rs`'s copy explicitly rejects them;
`castle.rs`'s does not need to since `serde_json::Number` cannot represent NaN/Infinity by construction).
Any new digestable artifact type must go through the local `canonical_json`/`digest_canonical` in whichever
file it belongs to — both operate on `serde_json::Value`, so any new artifact struct needs a `.to_json()`
method (see `PowlProcess::to_json`, `ConstructArtifact::to_json`, `OcelLog::to_json` for the pattern) rather
than deriving `Serialize` and relying on serde's own key ordering.

### Pipeline order (`src/castle.rs`)

```
derive_vulnerabilities()       DfCM: goal -> minimal predicate/witness sets (subset-minimal, depth-bounded DFS)
  -> compile_witness_to_powl() witness transitions -> PowlProcess (data-dependency partial order)
  -> run_planner_ensemble()    fan a PlanningProblem across &[Box<dyn Planner>] (WitnessPlanner is the only built-in)
  -> manufacture_construct_capability()  bind O*/config/ontology/process into a receipted, signed ConstructCapability
  -> admit_construct_for_do()  fail-closed verification against a ConstructTrustPolicy -> opaque ConstructAdmission
  -> execute_powl_with_gym_act()  the ONLY function that may call GymActAdapter::execute; re-verifies
                                   digests/bounds/expiry immediately before every transition, emits OCEL v2
```

`DependencyGraph::impacted_closure` / `construct_compromise` model counterfactual dependency compromise
(`epistemic_class: "COUNTERFACTUAL"`) separately from the goal-derivation path — these feed
`apply_zero_day_observation`, not the CONSTRUCT chain.

`Planner` and `GymActAdapter` are `#[async_trait]` traits (native `async fn` in traits can't be used as
`dyn` objects, and `run_planner_ensemble` needs heterogeneous `&[Box<dyn Planner>]`) — this is the one place
`async-trait` earns its keep as a dependency; prefer plain generics over trait objects elsewhere.

### Fortune-5 readiness gate (`src/fortune5.rs` + `src/board.rs`)

`qualify_fortune5` evaluates `FORTUNE5_REQUIREMENTS` (generated, ontology-sourced) against
`MetricObservation`s: missing evidence -> `Standing::Unknown`, invalid/failing/stale/future-dated evidence
-> `Standing::Refused`, only a fully-passing receipted observation -> `Standing::Alive`. `board.rs` wraps
this with real receipt verification (`admit_evidence` walks a full receipt DAG via `verify_receipt_dag`,
checking Ed25519 signature, trust epoch, key revocation, and orphaned parents) before qualification runs —
`qualify_fortune5` itself trusts its `MetricObservation` inputs and must never be called directly with
unverified evidence in production code paths.

`admit_replay` is a separate fail-closed gate: a replay class loses standing the moment structural
signature, ontology version, provider-semantics version, or invariant-set digest drift from the current
subject — no grandfathering.

`fortune5.rs` includes a hand-rolled `parse_epoch_ms` (RFC3339 → epoch milliseconds) since the crate has no
date/time dependency — it only needs to parse the `"YYYY-MM-DDTHH:MM:SS.sssZ"` shape used throughout this
codebase and its tests. If evidence timestamps ever need a wider RFC3339 surface (timezone offsets other
than `Z`, etc.), that function is the one to extend rather than reaching for a new crate dependency.

### Epistemic class discipline

Every artifact/event carries an explicit epistemic class (`EpistemicClass` in `castle.rs`:
`Constructed | Counterfactual | Replayed | Observed | Inferred`; `EvidenceEpistemicClass` in `fortune5.rs`
for the evidence-observation subset). This is load-bearing, not decorative — `admit_construct_for_do` and
`admit_evidence` both check it against expected values and refuse on mismatch. When adding new receipted
artifacts, pick the correct class rather than defaulting to `Observed`.

### Typed refusals

All fail-closed paths return `Err("REFUSED:<REASON>".to_string())` (or, for Fortune-5 controls,
`"UNKNOWN:..."` / `"ALIVE:..."` reason strings on the success path) rather than a typed error enum — see
the exhaustive list in `configs/CONSTRUCT.json`'s `typedRefusals` and the `refuse()` closure in
`admit_construct_for_do`. This mirrors the TypeScript predecessor's convention deliberately: callers
pattern-match / `.contains("REFUSED:...")` on reason strings, and tests assert on the exact string (see
`tests/castle.rs`, `tests/fortune5.rs`). New refusal paths should follow this convention rather than
introducing a `thiserror`-style enum.

### The `castle` CLI binary

`src/bin/castle/` is a `[[bin]]` target (same package, separate compilation unit from the `castle`
library — reach library items via `use castle::...`, not `crate::...`, inside it) built on the ecosystem's
real, published `clap-noun-verb` / `clap-noun-verb-macros` / `linkme` crates, version-pinned to match
`~/ggen`'s own `=26.7.4`. It follows the same generated-route / hand-written-handler seam as
`~/ggen/examples/clap-noun-verb-cli`:

- `verbs/routes.rs` — `#[verb("verb", "noun")]`-annotated thin wrappers plus `linkme::distributed_slice`
  noun registration. No business logic here, ever — every route is a one-line delegation to `handlers::*`.
- `verbs/handlers.rs` — all real logic, calling straight into the `castle` library and converting its
  `Result<T, String>` `"REFUSED:..."` errors into `clap_noun_verb::NounVerbError::ExecutionError`.

Current nouns: `fortune5` (`requirements`, `qualify`), `replay` (`admit`), `impact` (`coverage`),
`inventory` (`components`, `goals`). `clap-noun-verb` auto-generates long-form kebab-case flags from Rust
parameter names (`evidence_path: String` → `--evidence-path`, required; `Option<T>` → optional) — there is
no positional-argument form; always check the real `--help` output before assuming a flag name/shape rather
than guessing from the Rust signature. `tests/cli_fortune5.rs` and `tests/cli_replay_impact_inventory.rs`
exercise the actual compiled binary as a subprocess via `env!("CARGO_BIN_EXE_castle")` — no CLI mocking.
