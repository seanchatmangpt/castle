# castle migration plan v26.11.16

Standing: UNVERIFIED

## Identity
- Path: /Users/sac/castle
- Remote: https://github.com/seanchatmangpt/castle
- Branch: main@5b2ae09 (origin/main: ahead 19)
- Last commit: 2026-10-13; commits: 442; size: 5236 MB (.git objects 5.1 GB in 47 packs)
- Languages/build: Rust (Cargo.toml, crates/), package.json (dd-ui), ggen.toml, ontology
- Nested repos: none

## Observed state
- Dirty 3: `Cargo.lock`, `tests/castle.rs`, `tests/chicago_tdd_tools_smoke.rs`
- Untracked 1: `crates/castle-ggen-rdf/Cargo.lock`
- Ahead 19 on main (unpushed). Local-only branches: chore/commit-pending-changes,
  exp/cost-minimizing-planner, feat/ggen-pack-composition (3 unpushed)
- Stash 0. No target/ or node_modules present. Working tree is ~3 MB; .git is 5.1 GB.
- `.ggen/` present (check for keys before copying).

## What it is
Board-governed consequence and strategic-command system / self-adversarial security-testing
framework in Rust: DfCM inverse construction, POWL partial orders, receipted CONSTRUCT gate.

## Classification
ACTIVE (census agrees).

## Migration risk
HIGH. 19 unpushed commits on main, 3 unpushed local branches, 3 modified test files.
5.1 GB .git (47 packs) suggests large blobs were committed; a re-clone may be far smaller.

## Recommended action
COMMIT-PUSH
1. `cd /Users/sac/castle && git diff --stat` ; DECISION: commit or discard test edits.
2. `git add -A && git commit -m "wip: pending castle test/lock changes"`
3. `git push origin main`
4. `git push -u origin chore/commit-pending-changes exp/cost-minimizing-planner feat/ggen-pack-composition`
5. DECISION: is the 5.1 GB history wanted? If pack bloat is local-only (`git count-objects -vH`
   shows 47 packs), run `git gc --aggressive` is NOT allowed here; instead compare size of a
   fresh clone on the new machine.
6. Do not copy; CLONE-FRESH afterwards. Inspect `.ggen/` for keys and move via secure channel.

## First commands on the new machine
`cargo check --workspace`

## See Also
- ../v26.9.19/
- /Users/sac/docs/jira/v26.11.16/README.md
