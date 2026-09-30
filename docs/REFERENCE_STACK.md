# Reference stack (v26.9.29-r8)

`src/reference_stack.rs` fixes the only lawful ordering of the ecosystem owners and
refuses any projection that claims another owner's capability. It owns ordering, not
semantics, standing, receipts, or replay.

    GraphLaw semantic delta -> GraphLaw Knowledge Hook -> powerless SA2A intent
      -> Affidavit standing -> Reactor realization -> CASTLE BRCE (CONSTRUCT ceiling)
      -> independent postcondition -> CASTLE receipt/OCEL -> Beam4PM feedback

Real (not stubbed): CONSTRUCT manufacture, `admit_construct_for_do`, `execute_powl_with_gym_act`,
Ed25519-signed receipts and OCEL v2. The chain provenance is bound into the construct's `o_star`.

## Open seam: GraphLaw

`graphlaw` 26.9.28 is published and **does** ship a native Knowledge Hook surface
(`graphlaw::hooks`: `HookPack`, `Hook`, `Firing`, `materialize`). The local
`KnowledgeHookProjection` is therefore a seam, not a replacement: it must be fed from a
GraphLaw `Firing`, never re-implement hook triggering. `graphlaw` requires rustc 1.96 and
pulls PurRDF/Eyeron; this crate is `rust-version = "1.85"`, so the dependency is deferred
until the toolchain floor moves (or it is consumed via its WASI/ABI boundary).
