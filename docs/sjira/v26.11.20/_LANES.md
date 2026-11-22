# Lane Map — ECOSYSTEM-SPEC-26.11.20 Transition t_remediate-3-0

Date: 2026-11-20 · Base/HEAD: `3ffa1bbd66f6d1d871da046a94a32daf10ebf6b1` (all wave files
still untracked; coordinator owns every git transition). Remediation subject: the findings
ledger in `falsifier-checklist.md` (findings 1, 2, 5, 6 actionable). Pure Rust, zero
Python, zero mocks, zero runtime egress, `Cargo.toml`/`Cargo.lock` frozen.

| lane | role | owned files (disjoint) |
|---|---|---|
| L0 coordinator (this agent) | wiring + compile repairs + integration + git | `src/lib.rs`, `src/bin/castle/verbs/mod.rs`, narrow compile repairs in any lane file (disclosed in receipt) |
| L1 probes-pin | finding 2 / F-AV-1 | `src/probes/mod.rs`, `configs/probes-manifest.json` |
| L2 publish-anonymize | finding 5 / F-ANON-1..4 | `src/publish/` (incl. new `anonymize.rs`), new `src/bin/castle/verbs/castle_261120_routes.rs` (mod line owned by L0) |
| L3 probe-pinned-literal | finding 6 (probe half) | `src/probe/mod.rs` (test only) |
| L4 receiptocel-pinned-literal | finding 6 (receipt_ocel half) | `src/receipt_ocel.rs` (test only) |

Read-only plan lanes write nothing; write lanes land only on the files in their row.
Shared seams resolved here, not in chat: verbs `mod.rs` line for `castle_261120_routes`
belongs to L0 and is applied with the wiring edit; L2 owns the routes file contents.
