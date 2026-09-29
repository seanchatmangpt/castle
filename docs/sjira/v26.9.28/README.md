# CASTLE semantic Jira — v26.9.28

This directory is the authority-free governing graph for the CASTLE board product.

## Crown equation

```text
CASTLE_STOP_26.9.28 =
  ProductCrown
  ∧ XaaSRuntimeCore
  ∧ SA2AExactSubjectTransport
  ∧ ProcessAndPlanning
  ∧ SemanticProjectionAndGraphLaw
  ∧ AffidavitStanding
  ∧ DeterministicWasmAndResourceBounds
  ∧ AdversarialFailureDiscovery
  ∧ EvidenceAndProcessIntelligence
  ∧ SemanticManufacture
  ∧ BoundedSurfaces
  ∧ BoardStrategicCommand
  ∧ SingleBRCEBoundary
  ∧ UniqueCapabilityOwnership
  ∧ RequiredWorkTerminalOrSuccessor
```

The graph itself cannot make that equation true. `goal.ttl` contains no
authored `sj:WorkOrder` and no authored `sj:Receipt`. Standing comes only
from executed courts and exact-subject receipts.

## Files

- `goal.ttl` — stable root and fourteen crown gates.
- `repos.ttl` — exact observed repository subjects, SHAs, dispositions and
  irreducible capability ownership.
- `courts/stop.rq` — receipt-driven stop witness.
- `courts/ownership.rq` — zero-row court for duplicate/missing ownership.
- `courts/runtime_crown.rq` — zero-row court proving XaaS owns runtime
  existence and CASTLE owns consequential admissibility.
- `courts/exclusions.rq` — zero-row court forbidding authored WorkOrders and
  Receipts in the governing goal graph.
- `tests/sjira_castle.rs` — repository-local anti-vacuity and conservation
  tests.

## Authority

```text
sJira              = work / goals / obligations
CASTLE             = consequential admissibility
XaaS               = runtime existence
GraphLaw           = semantic-law derivation
Affidavit          = cryptographic standing
BRCE               = sole protected CONSTRUCT -> DO crossing
```

Every checkpoint in this directory has an authority ceiling of `CONSTRUCT`.
A ticket, plan, model result, GraphLaw result, XaaS runtime capability,
generated artifact, strategy candidate or board SELECT is never sufficient
authority for protected DO.

## Generation law

The stable goal graph is not a manually maintained Jira backlog.

WorkOrders belong to the semantic-jira compiler/projection path. The intended
fleet path is:

```text
operator/source semantics
  -> ggen_igniter semantic-jira admission
  -> candidate propositions
  -> generated WorkOrders
  -> XaaS routes work
  -> courts execute
  -> receipts
  -> stop query
```

Generated artifacts never replace the governing semantic source.

## Repository conservation set

`repos.ttl` pins the 45 repositories considered by this CASTLE composition.
Twenty-two are retained as unique capability owners. Seven remain
`SUPPORTING_CANDIDATE` and cannot claim sovereign capability ownership until
they are explicitly dispositioned as KEEP, WRAP, GENERATE, ABSORB, REPLACE or
RETIRE.

The critical ownership equations are:

```text
RUNTIME_EXISTENCE            -> seanchatmangpt/xaas
CONSEQUENTIAL_ADMISSIBILITY  -> seanchatmangpt/castle
SEMANTIC_LAW_DERIVATION      -> seanchatmangpt/graphlaw
CRYPTOGRAPHIC_STANDING       -> seanchatmangpt/affidavit
```

## Admission / court order

1. Parse/admit `goal.ttl` and `repos.ttl`.
2. Run the exclusion court over `goal.ttl`; expect zero rows.
3. Run ownership and runtime-crown courts over the union of the two graphs;
   expect zero rows.
4. Generate/route WorkOrders outside the goal graph.
5. Execute required courts.
6. Attach receipts to exact checkpoint subjects.
7. Evaluate `courts/stop.rq` over goal + generated orders + receipts.
8. Only an observed `ASK=true` has stop standing.

The Rust tests do not substitute for RDF/SPARQL admission. They are an
anti-vacuity fence that prevents obvious architectural drift before the
semantic court runs.

## Successor

Work that does not falsify a v26.9.28 crown gate routes to
`GC-CASTLE-26.10.1`. It does not reopen the v26.9.28 crown.
