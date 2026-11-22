# Wiring Plan — CASTLE-SPEC-26.11.20 (lane zai-18-court-reviewer)

Date: 2026-11-20
Standing of this document: plan; every "current state" claim below was read from the working
tree this session (see Evidence section). Sibling-authored files are **UNKNOWN** (not yet on
disk at review time) — the wiring edits are specified so they are invariant to those files'
internals; integration-time verification steps cover the residual shape risk.

## Purpose

Specify the exact minimal edits that wire six new library modules and one new binary routes
module into the crate and CLI:

| artifact | assumed path | wired by |
|---|---|---|
| `probe` | `src/probe.rs` or `src/probe/mod.rs` (+ sibling files under `src/probe/`) | `src/lib.rs` |
| `audit` | `src/audit.rs` or `src/audit/mod.rs` | `src/lib.rs` |
| `publish` | `src/publish.rs` or `src/publish/mod.rs` | `src/lib.rs` |
| `deletion` | `src/deletion.rs` or `src/deletion/mod.rs` | `src/lib.rs` |
| `gatekeeper` | `src/gatekeeper.rs` or `src/gatekeeper/mod.rs` | `src/lib.rs` |
| `receipt_ocel` | `src/receipt_ocel.rs` or `src/receipt_ocel/mod.rs` | `src/lib.rs` |
| `castle_261120_routes` | `src/bin/castle/verbs/castle_261120_routes.rs` | `src/bin/castle/verbs/mod.rs` |

Both accepted shapes for a library module (`single-file.rs` vs `dir/mod.rs`) wire identically —
the `pub mod <name>;` line does not distinguish them. Submodule files *inside* a directory
module (`src/probe/foo.rs` declared by `src/probe/mod.rs`) never get `lib.rs` lines.

## Why the wiring edit is load-bearing (repo-proven, not hypothetical)

`src/sa2a/mod.rs`, `src/security_core_generated.rs`, `src/security_sources_generated.rs`, and
`src/security_tools_generated.rs` exist on disk but are declared **nowhere** — a
whole-tree grep finds zero references (`mod sa2a;`, the three generated names: no hits outside
the files themselves). They compile to nothing; their contents are invisible to the crate.

Consequences the integration court must enforce:

1. A sibling module whose `pub mod` line never lands is not a defect the compiler reports.
   It is a silent zero: no symbols, no `#[cfg(test)]` tests executed.
2. A green `cargo test` therefore proves nothing about the six modules. The only non-vacuous
   proof of wiring is a **test-count delta** and a **CLI noun/verb delta** (see Verification).
3. A `#[verb]` route in a file not reachable from `mod verbs;` is likewise never compiled —
   the CLI silently lacks the command. The `linkme` distributed slice only collects from
   linked code.

## Current state (evidence, this session)

- `src/lib.rs` (38 lines): 25 `pub mod` lines, then `pub use` re-exports. No `mod.rs`
  indirection; every wired module is a flat declaration. Includes `pub use castle::*;`.
- `src/bin/castle/main.rs` (45 lines): thin entry — `mod verbs;` +
  `clap_noun_verb::run()`. Verb discovery is fully automatic via `linkme::distributed_slice`;
  main.rs contains **no per-feature wiring ever**.
- `src/bin/castle/verbs/mod.rs` (14 lines): declares `dfcm_handlers`, `evidence_handlers`,
  `evidence_routes`, `handlers`, `routes`, `payments_handlers`, `payments_routes`.
- Routes-file convention (`routes.rs`, `evidence_routes.rs`, `payments_routes.rs`): thin
  `#[verb("verb", "noun")]` wrappers returning `clap_noun_verb::Result<serde_json::Value>`,
  each delegating to a handler module; routes never call `castle::*` directly (stated in
  `handlers.rs:1-4`); each file ends with one `#[linkme::distributed_slice(
  ::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]` static per new noun.
- Registered nouns today (16): `fortune5`, `replay`, `impact`, `inventory`, `deployment`,
  `release`, `protocol`, `construct`, `do`, `crypto`, `chaos`, `live-check`, `replication`,
  `dfcm`, `evidence`, `payments`.
- `Cargo.toml`: deps already include `serde` (derive), `serde_json`,
  `serde_json_canonicalizer`, `sha2`, `blake3`, `ed25519-dalek`, `ml-dsa`, `slh-dsa`,
  `rand`, `tracing`, `clap-noun-verb =26.7.4`, `clap-noun-verb-macros`, `linkme`, `tokio`,
  `async-trait`. Lints: `unsafe_code = warn`; clippy `all`/`pedantic`/`nursery = warn`.
- No name collisions: grep of `lib.rs` and `verbs/mod.rs` for `probe|audit|publish|deletion|
  gatekeeper|receipt_ocel|castle_261120` — zero hits.

## Edit 1 — `src/lib.rs` (required, +6 lines)

Insert one contiguous block immediately after `pub mod xaas_feed;` (current line 25, the last
`pub mod`), before the blank line that precedes `pub use castle::*;`:

```rust
pub mod audit;
pub mod deletion;
pub mod gatekeeper;
pub mod probe;
pub mod publish;
pub mod receipt_ocel;
```

- Alphabetical **within the block**; placed at the tail of the existing mod block (not merged
  into the alphabetical run above) so the 20-lane wave produces one adjacent-line merge at a
  single anchor instead of six scattered conflicts.
- Do **not** add `pub use` re-exports for the new modules as part of wiring. The existing
  bottom `pub use` block re-exports only high-traffic items; handler code should use full
  paths (`castle::probe::…`). Re-exports, if wanted later, are a follow-up edit by the owning
  lane, not a wiring prerequisite.
- Accept either on-disk shape (`src/probe.rs` or `src/probe/mod.rs`); the line is identical.
  If a lane delivers both a file and a directory (e.g. `src/probe.rs` + `src/probe/…` with the
  file using `#[path]`-free submodule declarations), that still needs exactly this one line.

## Edit 2 — `src/bin/castle/verbs/mod.rs` (required, +1 line, +1 conditional)

Insert after `pub mod payments_routes;` (current line 14, last declaration):

```rust
pub mod castle_261120_routes;
```

Conditional sibling (only if a lane authored a separate handler module — the repo convention
is a `<name>_routes` / `<name>_handlers` pair, see `evidence_routes.rs` → `evidence_handlers`):

```rust
pub mod castle_261120_handlers;
```

Decision rule for the coordinator: if the routes file is self-contained (thin wrappers calling
other already-declared handler modules or trivially delegating), one line suffices. If the
routes file references `super::castle_261120_handlers::…` and no lane owns that file, the
routes lane must absorb the handlers (or the coordinator must assign them) — declare the
module or the binary will not compile. **The declared path must match the delivered file's
actual location;** the assumed path is
`src/bin/castle/verbs/castle_261120_routes.rs`. If a sibling landed the file elsewhere inside
`src/bin/castle/verbs/`, adjust the single declaration rather than moving the file. Do not
resort to `#[path]`.

## Edit 3 — `src/bin/castle/main.rs`: **no edit**

`clap_noun_verb::run()` auto-discovers every `#[verb]` and `__NOUN_REGISTRY` static reachable
from `mod verbs;`. Wiring the routes module into `verbs/mod.rs` (Edit 2) is sufficient. Zero
lines of main.rs change — this is the pack convention the binary documents in its own header.

## Edit 4 — `Cargo.toml`: **no edit**

All deps the milestone constraint set allows (std + `serde` + `serde_json` + `blake3`, plus
everything already ambient) are declared. Path-based intra-crate modules never touch
`Cargo.toml`. If any sibling module introduced a new external dependency, that is a violation
of the wave constraint and an integration refusal (`REFUSED_SCOPE: Cargo.toml is not a lane
target`); the coordinator routes it through a separate admitted change.

## Contract the routes file must satisfy (checked at integration)

1. Header imports exactly: `use clap_noun_verb::Result;` and
   `use clap_noun_verb_macros::verb;`.
2. Every command: `#[verb("<verb>", "<noun>")]`, returns
   `Result<serde_json::Value>`, body is a one-line delegation
   (`super::<handlers>::…_handler(…)` or direct thin call). No business logic in routes.
3. Verb parameter types must be clap-representable scalars used elsewhere in this binary:
   `String`, `bool`, `i64`, `Option<String>`, `Option<i64>` (see `routes.rs` for the full
   observed set).
4. One noun-registration static per **new** noun introduced by the file:

   ```rust
   #[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
   static REGISTER_<IDENT>_NOUN: fn() = register_<ident>_noun;

   fn register_<ident>_noun() {
       ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
           "<noun>",
           "<one-line noun description>",
       );
   }
   ```

5. `(noun, verb)` pairs must not collide with the 16 registered nouns above. Reusing a verb
   word under a **new** noun is legal (`verify` already exists under both `evidence` and
   `dfcm`). If the milestone reuses an existing noun (e.g. adds verbs under `evidence`), the
   noun-registration static must be **omitted** — registering an already-registered noun is
   the one integration check with no existing precedent in this repo; prefer new nouns.
6. Routes delegate only to handler modules declared in `verbs/mod.rs` or to code already
   reachable — never to unwired library modules (which would fail compile only at integration,
   after Edit 1 lands — acceptable, since Edit 1 is part of this same wiring).

## Verification ladder (post-wiring, cheapest high-information first)

Run from the repo root; every step must be observed, not inferred:

1. `cargo check --all-targets` — exit 0. Catches: missing mod declarations (unused-file
   silence is the inverse risk, not this), path mismatches, missing handlers.
2. `cargo test` — **record the summary line before and after** (e.g.
   `test result: ok. N passed`). The delta must be ≥ the number of `#[test]`s the six modules
   contain. A zero delta with new test code on disk = wiring failed even if exit 0 (the
   anti-vacuity tripwire for this whole plan).
3. `cargo build` then `./target/debug/castle --help` — the new noun(s) appear. Then one
   `--help` per new noun to list its verbs (registry discovery proof).
4. `cargo clippy --all-targets` — lints are `warn`-level in `Cargo.toml`; treat new warnings
   in the six modules as the owning lane's debt, not a wiring blocker. `unsafe_code = warn`:
   sibling modules using `unsafe` must surface it in this pass.
5. `./target/debug/castle <new-noun> <one-read-only-verb>` with a harmless fixture — one
   observed execution per noun, proving the routes→handlers→lib chain end to end.

Falsifier for this plan (do once, revert immediately, never commit): comment out Edit 1's
block → `cargo check` still exits 0 while the modules' tests vanish from step 2's count. That
asymmetry is the documented reason steps 2–3, not step 1, are the proof of wiring.

## Integration order (coordinator)

1. Confirm each sibling file's actual path/shape (this plan's assumptions are flagged UNKNOWN).
2. Apply Edit 1 + Edit 2 as single contiguous insertions at the named anchors.
3. Run the verification ladder in order; stop at first failure, name the owning lane.
4. Commit per lane (atomic); this document and the wiring edits may share the integration
   commit since the plan is the receipt for the edits.

## History

- ts 2026-11-20 | standing UNKNOWN→plan complete | lane zai-18-court-reviewer | read-only
  review; gates: tree grep + file reads (exits 0) | remaining: coordinator applies Edits 1–2,
  runs verification ladder.
