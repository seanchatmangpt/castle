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
configs/CONSTRUCT public construction graph
      |
      v
BLAKE3 parent receipts: O* + config + ontology + process
      |
      v
signed CONSTRUCT receipt + authority + subject + bounds + replay identity
      |
      v
fail-closed CONSTRUCT admission
      |
      v
GymAct transition permits
      |
      v
OCEL v2 observed evidence
      |
      v
child DO receipt
      |
      v
precompiled replay / invariant synthesis
```

## Source authority

The generated semantic inventory in `src/generated.ts` and the Fortune-5 readiness profile in `src/fortune5.generated.ts` are projected from `ggen-marketplace/packs/castle-pack/ontology.ttl`. The consumer `ggen.toml` references that pack using the marketplace local-path contract.

`CONSTRUCT` and planner results are candidates only. They have no consequential actuation authority. GymAct is restricted to an explicit test envelope over owned or authorized systems. Consequential defensive `DO` remains behind the BRCE receipt-bound boundary.

## DfCM CONSTRUCT origin law

`configs/CONSTRUCT.json` is intentionally public and reversible. Secrecy is not the security boundary. CASTLE obtains standing from cryptographic origin plus exact semantic binding.

`manufactureConstructCapability()` binds the admitted `O*`, configuration graph, ontology, exact POWL process, subject, authority, transition set, step bound, expiry, and deterministic replay identity. Each source receives a BLAKE3-256 receipt; the CONSTRUCT receipt names those four artifact digests as its parent chain. Receipt signatures cover the artifact identity, epistemic class, subject, parent digests, and origin key.

`admitConstructForDo()` then fails closed unless every source receipt and the CONSTRUCT receipt verify under an admitted trust root, the authority is admitted, subject identity matches, the process digest is exact, bounds are unchanged, and the capability is fresh. Successful admission returns an opaque frozen runtime capability manufactured inside the CASTLE module.

`executePowlWithGymAct()` is the exclusive exported DO path. It rejects missing or fabricated admission with `REFUSED:UNRECEIPTED_CONSTRUCT`, recomputes the process digest and bounds immediately before execution, passes a transition-specific permit to GymAct, stamps every OCEL event with the construct/process/config/O*/ontology/replay digests and authority, and returns a child receipt whose parent is the admitted CONSTRUCT digest.

The resulting property is not secrecy of the open-source implementation. A fork can execute modified code, but it cannot manufacture CASTLE standing under the deployment trust roots without an admitted CONSTRUCT origin:

```text
Understand CASTLE != authority to actuate CASTLE
SELECT != DO
CONSTRUCT != DO
DO => verified signed CONSTRUCT => receipted observed consequence
```

There is no raw-input, human, planner, model, replay, or unsigned-config fallback edge into the official DO path.

## Fortune-5 readiness admission

CASTLE treats enterprise readiness as a deterministic evidence gate rather than a checklist assertion. The marketplace ontology currently projects 40 controls across:

- authority separation and zero unreceipted actuation;
- cross-tenant isolation and privileged identity policy;
- software supply-chain, SBOM, provenance, and content addressing;
- replay version/invariant compatibility;
- OCEL and cryptographic receipt coverage;
- disaster recovery, failure injection, region failover, rollback, and failure-domain coverage;
- availability, p99 control-plane latency, quota headroom, and qualified scale;
- encryption, backup encryption, residency, and deny-by-default policy;
- deterministic replay, typed refusal, evidence retention, clock integrity, and immutable change evidence;
- air-gapped CONSTRUCT with zero network and secret dependencies;
- prohibited-goal and losing-region regression coverage;
- explicit owned-or-authorized GymAct execution scope.

`qualifyFortune5()` consumes receipted metric observations. Missing evidence is `UNKNOWN`; invalid or threshold-failing evidence is `REFUSED`; only complete passing evidence is `ALIVE`. The profile is an internal executable readiness gate, not a claim of external certification.

Replay is also fail-closed: `admitReplay()` requires an exact structural signature plus matching ontology version, provider-semantics version, invariant-set digest, and satisfied invariants. A previously successful replay class does not retain standing across semantic drift.

The 80/20 adversarial strategy is represented by `minimumImpactCoverage()`, which selects the smallest deterministic prefix of observed consequence classes needed to reach a requested impact-coverage threshold. Pareto skew is measured from evidence rather than assumed.

## Current executable slice

The TypeScript runtime implements:

- backward goal-to-vulnerability derivation with minimal-condition reduction;
- dependency impact closure and counterfactual compromise CONSTRUCT;
- planner interface and deterministic ensemble fanout/selection;
- causal partial-order compilation from transition witnesses;
- precompiled structural vulnerability matching;
- public reversible `configs/CONSTRUCT` policy with cryptographically fenced authority;
- BLAKE3 parent receipts for O*, config, ontology, and process;
- signed subject/authority/bounds/replay-bound CONSTRUCT capability manufacture;
- fail-closed trusted-origin admission with no unreceipted DO fallback;
- transition-specific GymAct permits and exact process/bounds revalidation at actuation;
- OCEL v2-shaped object/event evidence carrying construct provenance;
- receipted DO output chained to the admitted CONSTRUCT digest;
- zero-day capability observations as dependency-graph mutations;
- 40-control Fortune-5 readiness qualification from receipted observations;
- replay invalidation on structural, ontology, provider-semantics, or invariant drift;
- empirical Pareto consequence-coverage selection.

Run:

```bash
npm test
```

The runtime has no external npm dependencies; Node 22's type-stripping support executes the TypeScript tests directly.
