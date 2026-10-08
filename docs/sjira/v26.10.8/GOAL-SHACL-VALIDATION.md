# GOAL-SHACL-VALIDATION

pyshacl validation of `docs/sjira/v26.10.8/goal.ttl` against
`ggen_igniter/priv/ggen/semantic-jira-pack/shapes/{work-order,proposition}.shacl.ttl`.

- Validator: pyshacl 0.31.0 (Python API, `advanced=True`; the `pyshacl` CLI
  masks work-order-shape results when a second `-s` file is supplied — CLI
  alone reported a false `Conforms: True`).
- File: `docs/sjira/v26.10.8/goal.ttl` (69 triples: 5 `sj:GoalCheckpoint`,
  1 `sj:AcceptanceCriterion`, 2 `sj:Falsifier`; zero authored WorkOrders or
  Receipts — per the header law).

## Conforms table (post-fix)

| Surface | Result |
|---|---|
| As-is, CLI default (non-advanced) | Conforms: True (masks SPARQL constraints) |
| Both shapes, advanced Python API, pre-fix | Conforms: False (12 violations) |
| Both shapes, advanced Python API, post-fix | Conforms: False (11 violations) |

## Real defect fixed (1)

| Focus | Defect | Fix |
|---|---|---|
| `c10:GC-CASTLE-26.10.8` `sj:baseSha` | `"fe78477"` fails shape pattern `^[0-9a-f]{40}$`; predecessor `v26.9.28` carries the full 40-hex form | `fe784778a400adecc4bcfa95716ea2efba576c31` (verified: `git rev-parse fe78477`) |

## Shape-expectation mismatch — not fixed (11, by design)

Castle's GoalCheckpoint successor-bucket grammar (inherited verbatim from
`docs/sjira/v26.9.28/goal.ttl`, which fails identically) is not the Friday-style
grammar the shapes assume:

| Count | Violation | Classification |
|---|---|---|
| 5 | non-root GoalCheckpoint requires `sj:courtCommand` | mismatch — castle GCs carry `authorityCeiling "CONSTRUCT"` and no court command by design (no DO granted) |
| 5 | non-root GoalCheckpoint requires `sj:boundaryClass` | mismatch — same grammar difference; predecessor graph fails identically |
| 1 | `sj:checkpointOf` value `c28:GC-CASTLE-26.9.28` "does not have class sj:GoalCheckpoint" | graph-partition artifact — the typed node lives in `docs/sjira/v26.9.28/goal.ttl` (verified: typed `sj:GoalCheckpoint` there); single-file validation cannot see it |

## Note on shape targeting

WorkOrder/Proposition shapes never fire on this file (no nodes of those types);
the binding constraints come from `sj:GoalCheckpointShape`
(`targetClass sj:GoalCheckpoint`) and the `targetSubjectsOf sj:boundaryClass` /
`sj:checkpointOf` shapes. A zero-target run is vacuously conformant — the
forced-typing probe (typing GCs as WorkOrder/Proposition) was used to confirm
no WorkOrder-tuple or Proposition-surface terms are half-present.

See Also: `docs/sjira/v26.9.28/goal.ttl` (predecessor grammar),
`~/ggen_igniter/priv/ggen/semantic-jira-pack/shapes/work-order.shacl.ttl`
(`sj:GoalCheckpointShape`, lines 301-346)
