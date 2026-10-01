# CASTLE Capability Fabric (SRFC v26.9.30)

CASTLE expresses product intent; XaaS composes `ash_*` capability planes; no plane integrates with another
directly. CASTLE owns only the envelope (`src/operation_envelope.rs`,
`schemas/castle-fabric/operation-envelope-v1.schema.json`). Runtime composition is `Xaas.Fabric`
(`~/xaas/lib/xaas/fabric*`), a thin adapter layer over existing modules; the closure test ("CASTLE ALIVE") is
`~/xaas/test/xaas/fabric/castle_alive_test.exs`.

## Planes (URIs) and realizations

| URI | Plane | Realization in XaaS |
|---|---|---|
| `capability://projection/map` | external representation to graph | `Xaas.Semantics.VKG` (AshR2RML) |
| `capability://law/admit` | admissibility, constructibility | `AshGraphLaw.law/3` (SHACL cap) |
| `capability://evidence/attest` | receipts | `AshAffidavit` `assemble`/`verify` |
| `capability://process/conformance` | process history | `AshAffidavit` `assemble`/`conform` (PARTIAL: not `ash_ex4pm`) |
| `capability://agent/actuate` | the single DO boundary | `Xaas.Castle.run/2` (CASTLE BRCE via the castle binary) |

## Enforced rules

- Invalid implies nonconstructible: a law refusal leaves no construct; DO is never reached.
- A realization is driven only up to its declared ceiling (`Xaas.Fabric.Plane.permit/2`).
- Transient `realization_failed` closes one edge and the next realization serves; DO is attempted once and
  never failed over.
- One failure taxonomy over graphlaw, affidavit, r2rml and castle refusals (`Xaas.Fabric.Failure`).
- Standing is derived, never asserted: `:evidenced` needs a verified receipt and a conforming history.
- Castle failures are machine-readable: `castle ...` prints `{"ok":false,"class":...}` on stdout and exits
  2 (`REFUSED:`/`BLOCKED:`), 3 (non-settled standing) or 1.

## Not yet wired (recorded gaps)

- Resolution/interchangeability through `Ultracode.CapabilityResolver` and `SubstitutionCourt`: realizations
  are an ordered list per plane today.
- `ash_ex4pm`/`Ex4pm.*` for process conformance (pin conflict `ex4pm == 26.9.9`).
- The payments rails (`castle payments ...`) as an actuator; the first slice DOs via the generic POWL bridge.
- VKG source runner in the closure test is `Xaas.Test.VKGObservationEngine` (the repo's injected-runner
  implementation), not a real R2RML SQL engine.

## See Also

`docs/sjira/v26.9.28/repos.ttl`, `configs/CAPABILITY_FABRIC.json`, `src/castle.rs` (`admit_construct_for_do`)
