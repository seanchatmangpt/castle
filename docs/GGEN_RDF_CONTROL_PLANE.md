# ggen RDF control plane

CASTLE delegates RDF semantics to ggen. It does not carry a second Turtle parser, SPARQL engine, reasoner, or triple store.

## Boundary

```text
RDF / O*
   |
   v
 ggen v26.8.15
 Oxigraph + SPARQL + mu1 CONSTRUCT
   |
   +--> SELECT / ASK / property paths
   +--> ontology normalization / inference
   +--> semantic validation
   |
   v
CASTLE typed bindings
   |
   +--> DependencyGraph
   +--> DfCM goal inversion
   +--> counterfactual CONSTRUCT
   +--> POWL
   +--> GymAct
   +--> OCEL / receipts
```

The boundary is intentionally one-way with respect to semantic authority: ggen evaluates the RDF world; CASTLE consumes query results and turns them into bounded adversarial-testing structures. Planner or runtime code does not silently invent RDF facts.

## Public dependency vocabulary

The initial dependency profile uses existing public vocabularies:

- `prov:Entity` (`http://www.w3.org/ns/prov#Entity`) identifies graph entities.
- `dcterms:type` (`http://purl.org/dc/terms/type`) provides an optional entity kind.
- `dcterms:requires` (`http://purl.org/dc/terms/requires`) expresses dependency direction: the subject requires the object.

For `A dcterms:requires B`, CASTLE treats B as upstream of A. The impacted closure of B is therefore B plus every entity reachable through inverse/transitive `dcterms:requires` paths.

## Runtime interface

`src/ggen-rdf.ts` provides the bounded adapter:

- `assertPinnedVersion()` — refuses ggen version drift.
- `validate()` — delegates graph/ontology validation to `ggen graph validate`.
- `query()` / `queryRaw()` — delegates SPARQL to `ggen graph query`.
- `constructOntology()` — delegates ontology normalization/inference to `ggen sync --stage mu1`.
- `loadDependencyGraph()` — projects PROV/DCTERMS RDF into CASTLE graph types.
- `impactedClosure()` — computes dependency consequence closure inside ggen using a SPARQL property path.
- `constructCompromise()` — creates a `COUNTERFACTUAL` CASTLE compromise using the ggen-derived closure.

The process runner uses direct `spawn()` with `shell: false`, time and output bounds, and typed refusal on non-zero process status. Dynamic resource IRIs are validated before they are admitted into SPARQL.

## Exact subject

`configs/ggen.lock.json` pins the semantic engine to:

- version: `26.8.15`
- git commit: `162e466d8f07d0a75a468b4441b4bc8b1aad369b`
- Linux x86_64 release archive SHA-256: `f4c4ea396f5cec12cfa2dd46c13ac620291f9b83138f17ded6fa59c510dcfc42`

The ggen `v26.8.15` tag resolves to that exact commit. CASTLE CI downloads the release archive, verifies the archive digest, proves the binary version, and runs a real RDF/SPARQL qualification against `rdf/examples/dependency-system.ttl`.

## Standing law

```text
RDF standing
  => exact ggen subject
  => checksum-verified ggen binary
  => successful ggen query/CONSTRUCT
  => structurally validated bindings
  => typed CASTLE consequence
```

A fake runner is used only for adapter-unit tests. The separate integration court must execute the real ggen binary before the ggen RDF boundary can claim ALIVE standing.
