# CASTLE Capability Fabric (SRFC v26.9.30)

Canonical architecture for CASTLE / XaaS / `ash_*` composition. CASTLE expresses product intent; the fabric
resolves `capability://` URIs to independently qualified realizations; no plane integrates with another
directly. Implementation: `src/fabric/`. Closure test ("CASTLE ALIVE"): `tests/castle_alive.rs`.

## Model

```text
OperationEnvelope --> Orchestrator --> Registry --resolve(uri)--> Capability realization
```

| Artifact | Code | Schema |
|---|---|---|
| OperationEnvelope | `fabric::OperationEnvelope` | `schemas/castle-fabric/operation-envelope-v1.schema.json` |
| CapabilityContract | `fabric::CapabilityContract` | `schemas/castle-fabric/capability-contract-v1.schema.json` |
| Capability bus | `fabric::Capability` (describe, qualify, observe, select, construct, execute, receipt, replay, health) | n/a |
| Registry + QRI | `fabric::Registry` | n/a |
| Failure taxonomy, refusals | `fabric::FailureClass`, `configs/CAPABILITY_FABRIC.json` | n/a |

## Planes (URIs)

| URI | Plane | Test realization |
|---|---|---|
| `capability://projection/map` | external representation to graph (`ash_r2rml`) | `RowProjection` |
| `capability://process/conformance` | process/event state (`ash_ex4pm`) | `OcelProcess` |
| `capability://law/admit` | admissibility, constructibility (`ash_graphlaw`) | `JournalLaw`, `DirectLaw` |
| `capability://evidence/attest` | receipts, standing (`ash_affidavit`) | `SignedEvidence` |
| `capability://agent/actuate` | qualified actor and the DO boundary (`ash_a2a`) | `LedgerActuator` |

## Sequence

Requested, Qualified, Constructed, Executed, Observed, Receipted, Reconciled. Each stage is reported
separately; `Standing::Alive` requires a verified receipt and a conforming process history.

## Enforced rules

- Invalid implies nonconstructible: a law refusal leaves no admission artifact; DO is never reached.
- A capability is driven only up to its declared `EffectClass`.
- DO never falls over to another realization (no double actuation); other classes do on
  `REALIZATION_FAILED`, `TRANSPORT_FAILED`, `UNSUPPORTED`. Semantic and authority refusals are terminal.
- QRI: a further realization of a URI must have equal `semantic_id`, effect class, invariants and
  evidence requirements, else `REFUSED:QRI_CONTRACT_MISMATCH`.
- Execution is not standing: no receipt means `Standing::Unknown`.
- Replay reconstructs the decision and re-verifies the receipt; it never re-actuates.
- The fabric source names no concrete realization (asserted by `castle_side_fabric_names_no_concrete_realization`).

## Scope note

`ash_*` and XaaS live outside this crate. Here each plane is a hand-written real implementation over the
real payments machinery; any other realization (including an Ash/Elixir bridge) joins by satisfying the
same contract.

## See Also

`docs/payments/`, `configs/CONSTRUCT.json`, `src/castle.rs` (`admit_construct_for_do`)
