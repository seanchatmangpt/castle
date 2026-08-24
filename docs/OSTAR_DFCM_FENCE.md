# OSTAR DfCM intake fence

CASTLE does not consume an unselected DfCM option graph as an admission.

The upstream OSTAR DfCM layer may preserve and prune reversible disposition candidates, but its graph remains `NO_AUTHORITY`, `UNSELECTED`, and non-actuating. CASTLE's existing `admit_empire_reconstitution_for_construct()` accepts only the exact `ggen.legacy.authority-vacuum.admission.v1` envelope. Therefore a `ggen.legacy.dfcm-option-graph.v1` document is refused at the schema boundary before capability closure, O* projection, signed CONSTRUCT, or DO.

This is deliberate composition rather than a second parser:

```text
DfCM option graph
  -> reversible evidence / topology only
  -> explicit authority contract outside CASTLE
  -> ggen-legacy admitted candidate envelope
  -> CASTLE exact admission parser
  -> inert O* projection
  -> signed CONSTRUCT rail
  -> independently admitted DO
```

No DfCM graph, pruning constraint, model output, generated projection, or option identifier carries ambient selection or actuation authority.
