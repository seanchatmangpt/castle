# CASTLE — Vision 2030

This document separates two things that are easy to blur in a vision statement: what CASTLE
**is today** (grounded in `src/`, verified by `cargo test`) and what it is **aiming at by 2030**
(a target, not a claim). Every 2030 item below states what would have to be true — a concrete,
falsifiable condition — not an adjective.

## Where CASTLE stands today (ALIVE, verified 2026-08-15)

A single Rust crate + CLI, 47/47 tests passing, implementing one complete but narrow slice of the
canonical flow in `README.md`:

- DfCM backward goal-to-vulnerability derivation, subset-minimal, over a hand-supplied
  `TransitionRule` set (no external fact ingestion yet).
- Dependency CONSTRUCT (counterfactual compromise + impact closure) over a hand-built
  `DependencyGraph` (no live dependency-graph source yet).
- A one-planner ensemble (`WitnessPlanner`) — the `Planner` trait supports more, none are written.
- POWL partial-order compilation from witness transitions.
- A fail-closed, cryptographically receipted CONSTRUCT admission chain (BLAKE3 + pluggable
  signer/verifier), with `ConstructAdmission` structurally unforgeable outside
  `admit_construct_for_do` (Rust module privacy, not a runtime check).
- `execute_powl_with_gym_act` as the sole DO path, against a caller-supplied `GymActAdapter` — no
  adapter for any real system ships in this repo; every test uses an in-process recording double.
- Fortune-5 readiness qualification (40 controls) and board-package admission, with a real Ed25519
  receipt chain (`board.rs`) — evaluated against evidence you supply; nothing self-observes yet.
- A `castle` CLI (`fortune5`, `replay`, `impact`, `inventory` nouns) — read/evaluate operations only;
  there is no `castle construct` or `castle gymact` verb yet, so CONSTRUCT manufacture and DO
  execution are library-only, not yet CLI-reachable.
- CI (`.github/workflows/ci.yml`) runs `npm test` on Node 22, not `cargo test` — there is no
  `package.json` in this repo and no Rust test step in CI. The 47/47 figure above is a real,
  reproducible local result; it currently has no CI backstop.

What this means concretely: CASTLE today is a **verified reasoning and receipt-admission kernel**,
not a system that observes or tests anything on its own. Every "OBSERVED" event in the test suite
was produced by a test double. That boundary is the honest starting line for the rest of this
document.

## The gap between kernel and system

Three things stand between today's kernel and a system that does what `README.md`'s canonical flow
describes end-to-end:

1. **No real `GymActAdapter`.** The bounded test-envelope executor has no implementation against
   any actual infrastructure (a cloud API, a container runtime, a CI system). Building one, scoped
   and receipt-bound the same way the kernel already is, is the highest-leverage next step —
   everything downstream (real OCEL evidence, real Fortune-5 metrics, real replay classes) depends
   on it existing.
2. **No planner beyond `WitnessPlanner`.** AutoFDE-Lab's "ensemble" is currently an ensemble of one.
   A second, structurally different planner (even a simple heuristic-search one) is the first real
   test of whether `run_planner_ensemble`'s scoring/selection logic generalizes.
3. **No ontology-driven CLI verbs for CONSTRUCT/DO.** The `castle` binary can query and evaluate but
   not manufacture or actuate — by design, for now: CONSTRUCT manufacture and DO execution both
   require a `Blake3Provider`/`ReceiptSigner`/`GymActAdapter` triple that has no default, safe CLI
   shape yet. Wiring one prematurely would be worse than not having it.

## 2030 horizon (target, not status)

Framed as conditions that would have to hold, each traceable to a file that would need to exist and
pass tests — not aspirational language:

- **At least one real GymAct adapter, receipt-bound and test-envelope-scoped, ships in-repo** —
  concretely, a `ContainerGymActAdapter` implementing the existing `GymActAdapter` trait
  (`src/castle.rs:645-648`) against a local, network-isolated container runtime (`docker`/`podman`
  via `tokio::process::Command`, `--network=none`, an explicit image allowlist) — no credentials or
  network egress required to exercise it. It must return `GymActStatus::Refused`, never a
  fabricated `Observed`, on nonzero exit or spawn failure; it must not construct its own
  `ConstructAdmission` (module-private `sealed::AdmissionBrand` already makes that impossible); it
  must honor `permit.expires_at_epoch_ms`. Tested against a real local daemon (a named skip, not a
  mock, when one isn't running), asserting on the real returned `GymActResult`'s exit-code
  attribute and a BLAKE3 digest of captured stdout — not on whether the subprocess was called.
- **`admit_replay`'s fail-closed discipline has been exercised against real semantic drift** — an
  actual ontology version bump or provider-semantics change that a real replay class survived or was
  correctly refused for, not just the synthetic drift in `tests/fortune5.rs`.
- **The Fortune-5 profile has been evaluated against at least one real receipted evidence pipeline**
  end to end — `castle fortune5 qualify` consuming evidence a real system emitted, not a hand-authored
  fixture — with the qualification result independently checkable against that system's own state.
- **A second planner exists and the ensemble's scoring/selection has been shown to matter.**
  `TransitionRule` already carries unused `cost`/`planner_hint` fields that `WitnessPlanner::plan`
  never reads — a `CostMinimizingPlanner` would search `problem.rules` for alternative producers of
  the same effect predicate (`derive_vulnerabilities`'s `producers: HashMap<&str, Vec<&TransitionRule>>`
  already exposes them) and score by summed `cost` instead of activity/predicate count. The test:
  two `TransitionRule` producers for one effect — one cheap-but-longer, one expensive-but-shorter —
  run through `run_planner_ensemble`, asserting the returned `Vec<PlanCandidate>`'s `score`,
  `planner_id`, and `process.activities` show the winning planner differs from `WitnessPlanner` and
  the winning process differs structurally, not just in labeling.
- **castle-pack's ontology has driven at least one real regeneration of
  `src/generated.rs`/`src/fortune5_generated.rs` via `ggen sync run`, gated in CI.** `ggen` is
  installed and runnable today (`ggen 26.8.8`), but `ggen.toml` currently has no rule wiring
  `templates/src/generated.rs.tmpl` to any output — only its `.ts` sibling is wired, and the
  `fortune5_generated.rs` rule that does exist has no output path that lands inside this repo
  (`output_dir` is relative to the pack directory, not `castle/`). Closing the loop means: adding a
  `[[generation.rules]]` entry for `src/generated.rs`; a CI step running `ggen sync run --dry-run`
  and hashing its output against the committed file; and a second job asserting
  `git diff --exit-code src/generated.rs src/fortune5_generated.rs` is empty after a real (writing)
  run — proving the checked-in files are pipeline output, not hand-matched to the "Do not
  hand-edit" header they already carry.
- **The `castle` CLI gains `construct` and `gymact` verbs** once a real adapter exists to make them
  meaningful, with the same subprocess-integration-test discipline `tests/cli_*.rs` already holds the
  rest of the CLI to.

## What does not change

The invariant that makes any of the above safe to build toward is already load-bearing and is not
up for revision: `CONSTRUCT != DO`. Every future adapter, planner, or CLI verb extends the kernel;
none of them get a side door around `admit_construct_for_do`. A 2030 CASTLE with real adapters and a
real planner ensemble is only worth building if it inherits — not relaxes — the fail-closed admission
chain that exists today.
