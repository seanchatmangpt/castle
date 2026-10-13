# castle-ggen-rdf

ggen RDF projection boundary for castle: consumes `ggen-engine`
(git rev `162e466d8f07d0a75a468b4441b4bc8b1aad369b`) and projects castle
strata events.

## Build requirement: nightly toolchain (required)

This crate does NOT build on stable. Transitive dependencies enable
`#![feature(...)]` gates that stable rejects with E0554:

- `wasm4pm-compat` (`/Users/sac/wasm4pm/wasm4pm/src/lib.rs:10-14`):
  `generic_const_exprs`, `adt_const_params`, `const_trait_impl`,
  `min_specialization`, `portable_simd`
- `bcinr-powl` (`/Users/sac/bcinr/crates/bcinr-powl/src/lib.rs:14-15`):
  `adt_const_params`, `generic_const_exprs`
- `bcinr-pddl` (`src/lib.rs:22`): `once_cell_try`

Verified witness (stable 1.97.0):

```text
error[E0554]: `#![feature]` may not be used on the stable release channel
   --> ~/.cargo/registry/src/.../wasm4pm-compat-26.8.7/src/lib.rs:148:1
error: could not compile `bcinr-powl` (lib) due to 1 previous error
error: could not compile `wasm4pm-compat` (lib) due to 1 previous error
```

## Invocation

```bash
cargo +nightly test    # from crates/castle-ggen-rdf (standalone crate, own Cargo.lock)
```

Nightly pin in use: `nightly-2026-06-22`
(`rustc 1.99.0-nightly (daf2e5e18 2026-07-13)` via `rustc +nightly --version`).

## Unblock path (to stable)

Remove the `#![feature]` gates upstream in `wasm4pm` and `bcinr-powl` /
`bcinr-pddl` (de-feature or gate behind feature flags), then re-pin this crate
to stable. Until then, stable builds are blocked, not deprecated — the crate's
own `rust-version = "1.82"` refers to its own source only, not the closure.
