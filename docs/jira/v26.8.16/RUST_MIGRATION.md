# Rust Migration

CASTLE moved from a TypeScript/Node runtime to a Rust crate + CLI this milestone, to align
with the chatman/ggen-marketplace ecosystem's own convention for consumer projects. This
document records what changed, what was verified, and what carries forward as open work —
see [VISION.md](../../../VISION.md) for the longer-range 2030 framing this milestone's gaps
feed into.

Last updated: 2026-08-16 (v26.8.16).

## Scope

Everything under `src/`, `tests/`, and `src/bin/castle/` was rewritten from TypeScript to
Rust: `castle.ts` → `src/castle.rs`, `board.ts` → `src/board.rs`, `fortune5.ts` →
`src/fortune5.rs`, `blake3.ts` → `src/blake3.rs` (now backed by the real `blake3` crate
rather than a hand-rolled implementation), plus the two marketplace-generated files
(`generated.ts`/`fortune5.generated.ts` → `generated.rs`/`fortune5_generated.rs`). The
upstream `ggen-marketplace/packs/castle-pack` ontology pack was retargeted in lockstep —
its `ggen.toml` generation rules and `.tmpl` templates now emit Rust structs/consts instead
of TypeScript `as const` objects.

## What changed

### Language and dependencies

Dependency choices were pinned to match `~/ggen`'s own workspace, not chosen from scratch:
`blake3`, `ed25519-dalek`, `serde`/`serde_json`, `async-trait`, `rand`, `tracing`. `Cargo.toml`
carries a warn-only `[lints]` block (`clippy::all`/`pedantic`/`nursery`, `unsafe_code = "warn"`)
matching `~/ggen`'s own "inventory before enforce" posture, and `rust-version = "1.82"`.

`ConstructAdmission`'s forgery-proofing moved from a TypeScript `Symbol`-branded object to
Rust module privacy — a private field from a `mod sealed` marker type, so the compiler (not a
runtime check) guarantees only `admit_construct_for_do` can manufacture one.

### `castle` CLI

A `[[bin]]` target (`src/bin/castle/`) exposes the library over the ecosystem's real,
published `clap-noun-verb` / `clap-noun-verb-macros` / `linkme` crates (version-pinned to
`~/ggen`'s own `=26.7.4`), following the route/handler seam from
`~/ggen/examples/clap-noun-verb-cli`. Nouns: `fortune5` (`requirements`, `qualify`), `replay`
(`admit`), `impact` (`coverage`), `inventory` (`components`, `goals`). CONSTRUCT manufacture
and DO execution are deliberately not yet CLI-reachable — see VISION.md's gap #3.

### Test coverage added beyond the TypeScript predecessor

The TypeScript runtime shipped with zero tests for `board.ts` and no CLI-level coverage.
This milestone added:

- `tests/board.rs` — 14 tests, real Ed25519 keys + real `blake3`, no mocking.
- `tests/cli_fortune5.rs` + `tests/cli_replay_impact_inventory.rs` — 8 subprocess integration
  tests against the actual compiled binary via `CARGO_BIN_EXE_castle`.
- Two proptest suites: `canonical_json` determinism/order-independence (both independent
  copies, `castle.rs` and `board.rs`), and DfCM subset-minimality (`tests/prop_dfcm.rs`).

### 80/20 error-correction pass

A targeted sweep of production `src/` for panic-risk error handling (`.unwrap()`/`.expect()`/
`panic!`) found exactly two `INPUT_DERIVED` findings, both already guarded but fragile by
adjacency rather than by construction:

- `board.rs`'s `verify_receipt_node` — a receipt's `signature` field (attacker-controlled)
  was length-checked three lines before an `unwrap()`'d `TryInto<[u8; 64]>`. Folded the check
  into the fallible conversion itself, so a malformed length is a typed `REFUSED:*` result by
  construction, not by a guard staying in sync.
- `fortune5.rs`'s `minimum_impact_coverage` — a `partial_cmp(...).unwrap()` sort comparator
  over user-suppliable impact values, replaced with `f64::total_cmp`.

Both fixes carry a regression test proving the specific input that used to be panic-adjacent
now returns a clean typed error.

Everything classified `INTERNAL_INVARIANT` (safe by construction — e.g. slicing a value the
same function just proved non-empty) or not a real panic risk (`HashMap::get().unwrap_or_default()`
on an expected-absent key, followed by correct empty-case handling) was deliberately left
alone — see the full inventory in this milestone's session record, not duplicated here to
avoid the doc drifting from the code it describes.

### CI

`.github/workflows/ci.yml` ran `npm test` on Node 22 against a repository with no
`package.json` — a leftover from before this migration. Replaced with `cargo build
--workspace --all-targets` + `cargo test --workspace`, matching the ggen/ggen-marketplace
ecosystem's own PR-gating conventions: SHA-pinned actions (`actions/checkout`,
`dtolnay/rust-toolchain`, `Swatinem/rust-cache`), and `cargo clippy` run as an explicitly
non-blocking reporting step — consistent with this crate's own warn-only `[lints]` and with
`~/ggen`'s own PR-gating lane, where clippy debt is advisory, not blocking.

## Verification

47/47 tests passing (`cargo test --workspace`): 4 lib proptests, 14 `tests/board.rs`, 10
`tests/castle.rs`, 3 `tests/cli_fortune5.rs`, 5 `tests/cli_replay_impact_inventory.rs`, 9
`tests/fortune5.rs`, 2 `tests/prop_dfcm.rs`. `cargo build` clean, no errors.

| Check | Result |
|---|---|
| `cargo build --workspace --all-targets` | clean |
| `cargo test --workspace` | 47/47 passing |
| `cargo clippy` | 0 errors, warnings advisory-only (not gated) |
| `cargo fmt --check` | fails on pre-existing line-length formatting; not enforced by CI or by this crate's own conventions, tracked as a known gap, not fixed this milestone |

## Known gaps carried forward

These are stated precisely, not as a vague "future work" list, in
[VISION.md](../../../VISION.md)'s "2030 horizon" section: no real `GymActAdapter` ships
against any actual infrastructure; the planner ensemble has exactly one planner
(`WitnessPlanner`); `castle-pack`'s `ggen.toml` has no rule wiring that would let
`ggen sync run` regenerate `src/generated.rs`/`src/fortune5_generated.rs` inside this repo
(the checked-in files are hand-matched to the ontology, not pipeline output); and `cargo fmt`
is not currently enforced.

## See Also

- [VISION.md](../../../VISION.md) — present-status-vs-2030-target framing this milestone's
  gaps feed into.
- [README.md](../../../README.md) — canonical flow, CLI usage, current executable slice.
- [CLAUDE.md](../../../CLAUDE.md) — architecture notes for working in this codebase, including
  the CLI's route/handler seam and the two independent receipt systems.
- `ggen-marketplace/packs/castle-pack/README.md` — the ontology pack this crate's generated
  files are a projection of.
