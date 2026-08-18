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

One thing stands between today's kernel and a system that does what `README.md`'s canonical flow
describes end-to-end (gaps #1 and #2 as of 2026-08-15 are closed — see the dated note below):

1. **No ontology-driven CLI verbs for CONSTRUCT/DO.** The `castle` binary can query and evaluate but
   not manufacture or actuate — by design, for now: CONSTRUCT manufacture and DO execution both
   require a `Blake3Provider`/`ReceiptSigner`/`GymActAdapter` triple that has no default, safe CLI
   shape yet. Wiring one prematurely would be worse than not having it.

### 2026-08-17/18: gaps #1 (`GymActAdapter`) and #2 (second planner) closed, with scope notes

**`GymActAdapter` (was gap #1).** `KindClusterReadOnlyGymAct` in `src/castle.rs` (~line 790) is the
crate's first non-test-double `GymActAdapter` implementor, run against a real, already-running
`kind-platform-eng-colima` kind cluster (verified live via `kubectl get nodes` / `docker ps` before
writing code, not assumed). It shells out to the real `kubectl` binary through a fixed,
construction-time allowlist of three read-only `kubectl get` queries (nodes / kube-system pods /
namespaces); any `transition_id` outside that map is refused before `kubectl` is ever invoked. The
real JSON response's `items[].metadata.name` becomes real `OcelObject`s, so the OCEL log names
actual observed cluster resources. It still routes through the existing `execute_powl_with_gym_act`
admission/envelope/digest chain, plus its own allowlist on top — `CONSTRUCT != DO` unrelaxed. Test
`kind_cluster_gymact_observes_real_nodes_and_yields_a_receipted_ocel_log` asserts on the real
`ReceiptedOcelLog` state (object ids prefixed `k8s:kind-platform-eng-colima:`), degrading to a named
`eprintln!("SKIPPED: ...")` — never a mock — if the cluster isn't reachable; a companion test
confirms an unlisted `transition_id` is refused without invoking `kubectl` at all. Real run:
`cargo test --test castle` → 14 passed, 0 failed, live-cluster test not skipped; full suite green;
`grep -rn "unittest.mock|Mock(|MagicMock|patch(|monkeypatch" src/ tests/` → zero matches. Honest
scope: this is one read-only adapter against one already-provisioned local cluster, not the
2030-horizon `ContainerGymActAdapter` (network-isolated `docker`/`podman` spawn, image allowlist,
`permit.expires_at_epoch_ms` enforcement) described below — that item is still open.

**Second planner (was gap #2).** `AutofdeLabPlanner` in `src/castle.rs` implements the existing
`Planner` trait by serializing the `PlanningProblem` to JSON, spawning a real Python subprocess
(`autofde-lab/scripts/castle_bridge/plan_astar.py`, a genuine A* forward search over the problem's
own `TransitionRule` set, Python stdlib `heapq`) and parsing its real stdout back into
`PlanCandidate`s — no mocking, degrades to `Vec::new()` on any subprocess/parse failure, same as
`WitnessPlanner`'s inapplicable-case behavior. Test
`ensemble_selects_between_witness_and_autofde_lab_planners` runs both planners through the real
`run_planner_ensemble` on a two-rule problem and shows the ascending-score sort placing
`autofde-lab-astar` (score 2, from a real search path) ahead of `witness-partial-order` (score 3) —
the winning planner and process differ structurally, not just in labeling, closing the exact
falsifiable condition the 2030 horizon item below describes. Real run: the new test passes; full
`cargo test` afterward stays green (`lib.rs` 14, `tests/castle.rs` 12, `cli_fortune5.rs` 3,
`cli_replay_impact_inventory.rs` 5, `fortune5.rs` 9, `prop_dfcm.rs` 2); `grep` for mock usage in
`src/`/`tests/` → zero matches. Honest scope: `plan_astar.py` does **not** route through
autofde-lab's registered `autofde_lab.hub.solver.astar.Astar` scikit-decide `Solver` — that deeper
integration (wiring the problem into `GoalDomain`'s state/action/`Memory`/`Value` protocol) was
verified buildable (`uv run python -c "import autofde_lab"` → ok) but not attempted in this pass.

Both changes are staged in `src/castle.rs`/`tests/castle.rs` in the castle repo, not committed; the
bridge script is new and untracked in the separate autofde-lab repo.

### 2026-08-18: production `GymActAdapter` against the live `gymact` service

`ProcessGymActAdapter` in `src/castle.rs` (~line 926, immediately after `KindClusterReadOnlyGymAct`)
is the crate's second non-test-double `GymActAdapter` implementor, and the first backed by a real
external *service's* CLI rather than a hand-rolled `kubectl` call: it shells out to the real
`gymact` Typer CLI (`~/gymact/.venv/bin/gymact verify <request.json>`, subprocess+JSON, the same
bridge pattern as `AutofdeLabPlanner`) against the real `kubernetes-reconciliation` provider on the
already-running `kind-platform-eng-colima` cluster, rather than gymact's HTTP/FastAPI surface
(avoids network/auth setup for a same-host CLI already installed and already authorized). A fixed,
construction-time `allowed_verifications` map (`transition_id` -> gymact `provider` name + expected
postcondition JSON) plays the same role as `KindClusterReadOnlyGymAct`'s allowlist: any
`transition_id` outside it is refused before `gymact` is ever invoked, so nothing from
`PowlActivity`/`WorldState` is interpolated into the gymact request. `gymact verify`'s CLI path
materializes a real subject per call (a real `kubernetes-reconciliation` Pod, confirmed live via
`kubectl get pods` before/after) but ships no `teardown` subcommand
(`~/gymact/src/gymact/cli.py` has none); the adapter best-effort tears its own materialized pod down
via a fixed `kubectl delete pod <observed-name> --now` afterward so repeated runs don't accumulate
live pods — verified: `kubectl --context kind-platform-eng-colima get pods -n default` showed the
pod present mid-run and `No resources found` after. It still routes through the existing
`execute_powl_with_gym_act` admission/envelope/digest chain — `CONSTRUCT != DO` unrelaxed; no new
actuation path bypasses `admit_construct_for_do`.

Two new tests in `tests/castle.rs` cover it:
`process_gym_act_adapter_runs_a_real_powl_sequence_against_the_live_kubernetes_reconciliation_provider_and_yields_a_receipted_ocel_log`
runs a real one-activity POWL process through `execute_powl_with_gym_act` against the live cluster
and asserts on the real `ReceiptedOcelLog` state (a `gymact:kubernetes-reconciliation:<episode-id>`
object, the real `gymact_observed.running == true` postcondition attached to the OCEL event, and a
non-empty real receipt digest) — never on "was gymact called"; and
`process_gym_act_adapter_refuses_transitions_outside_its_fixed_allowlist` confirms an unlisted
`transition_id` is refused without invoking `gymact` at all. Both degrade to a named
`eprintln!("SKIPPED: ...")` (never a mock) if the real cluster or the real `gymact` CLI isn't
reachable, matching `kind_cluster_gymact_observes_real_nodes_and_yields_a_receipted_ocel_log`'s
contract. Real run: `cargo test --test castle` -> 16 passed, 0 failed, both live-service tests not
skipped, full suite green; `grep -rn "unittest.mock|Mock(|MagicMock|patch(|monkeypatch|mockall" src/
tests/` -> zero matches.

Honest scope: this adapter drives gymact's `verify` command only (a materialize -> independently
observe/check cycle), not `execute`'s heavier BRCEBroker/`ExecutionGrant`/`admission_digest` DO
path — wiring a `ProcessGymActAdapter` variant against `gymact execute` for a true multi-capability
DO sequence (e.g. `scale_restart` then re-verify) remains open. Selecting this adapter over
`KindClusterReadOnlyGymAct` at the `execute_powl_with_gym_act` call site is not yet config/env-driven
in this repo; callers construct whichever `GymActAdapter` they want directly, same as both existing
adapters today.

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
- **A second planner exists and the ensemble's scoring/selection has been shown to matter.** ✅
  Satisfied as of 2026-08-17/18 by `AutofdeLabPlanner` — see the dated note above. The specific
  design sketched here (`TransitionRule`'s unused `cost`/`planner_hint` fields driving a
  `CostMinimizingPlanner` over `derive_vulnerabilities`'s `producers` map) was **not** what got
  built; `AutofdeLabPlanner` instead delegates to a real external A* subprocess. That
  `cost`/`planner_hint`-driven design is still open if a third, in-process planner is wanted later.
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
