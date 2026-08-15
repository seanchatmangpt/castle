# CASTLE

CASTLE is a continuously self-adversarial system for owned or explicitly authorized infrastructure. It derives vulnerability conditions backward from prohibited goals, CONSTRUCTs dependency compromise worlds, fans planning problems across a planner ensemble, compiles witnesses into POWL-style partial orders, executes admitted tests through a bounded GymAct interface, records OCEL v2 evidence, and binds artifacts to BLAKE3-256 receipts with origin signatures.

## Canonical flow

```text
ADVERSARIAL GOAL
      |
      v
DfCM inverse construction
      |
      v
minimal vulnerability conditions
      |
      +--> dependency CONSTRUCT / hypothetical capability graph
      |
      v
AutoFDE-Lab planner ensemble interface
      |
      v
POWL causal workflow
      |
      v
GymAct test envelope
      |
      v
OCEL v2 observed evidence
      |
      v
BLAKE3 receipt + origin signature
      |
      v
precompiled replay / invariant synthesis
```

## Source authority

The generated semantic inventory in `src/generated.ts` is projected from `ggen-marketplace/packs/castle-pack/ontology.ttl`. The consumer `ggen.toml` references that pack using the marketplace local-path contract.

`CONSTRUCT` and planner results are candidates only. They have no consequential actuation authority. GymAct is restricted to an explicit test envelope over owned or authorized systems. Consequential defensive `DO` remains behind the BRCE receipt-bound boundary.

## Current executable slice

The TypeScript runtime currently implements:

- backward goal-to-vulnerability derivation with minimal-condition reduction;
- dependency impact closure and counterfactual compromise CONSTRUCT;
- planner interface and deterministic ensemble fanout/selection;
- causal partial-order compilation from transition witnesses;
- precompiled structural vulnerability matching;
- bounded POWL execution through a GymAct adapter;
- OCEL v2-shaped object/event evidence;
- BLAKE3-256 receipt-provider validation plus origin signing contract;
- zero-day capability observations as dependency-graph mutations.

Run:

```bash
npm test
```

The runtime has no external npm dependencies; Node 22's type-stripping support executes the TypeScript tests directly.
