# ggen RDF control plane

CASTLE delegates RDF semantics to ggen. It does not carry a second Turtle parser, SPARQL engine, reasoner, or triple store.

## Boundary

```text
RDF / O*
   |
   v
castle-ggen-rdf
   |
   v
ggen-engine v26.8.15 @ 162e466d...
GraphLawStore + deterministic Oxigraph mirror
   |
   +--> Turtle parse / canonicalization
   +--> SELECT / ASK / CONSTRUCT / property paths
   +--> BLAKE3 graph-state hash
   +--> GraphLaw materialization / validation capability
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

The boundary is intentionally one-way with respect to semantic authority: ggen evaluates the RDF world; CASTLE consumes engine-neutral query results and turns them into bounded adversarial-testing structures. Planner or runtime code does not silently invent RDF facts.

`castle-ggen-rdf` is only an adapter. Its `Cargo.toml` pins the `ggen-engine` package to one exact git commit, and its source calls the public `GraphEngine`/`GraphLawStore` API. It contains no RDF parser, SPARQL implementation, or graph algorithm of its own.

## Why the bridge exists

Source inspection of ggen v26.8.15 established that the released `graph` CLI noun currently exposes `validate` only. The RDF query capability exists one layer lower in `ggen-engine`: `GraphEngine::query` supports SELECT, ASK, CONSTRUCT/DESCRIBE, canonical graph state, and GraphLaw-backed reasoning. CASTLE therefore binds to that public semantic engine directly rather than pretending a nonexistent `ggen graph query` command exists.

## Public dependency vocabulary

The initial dependency profile uses existing public vocabularies:

- `prov:Entity` (`http://www.w3.org/ns/prov#Entity`) identifies graph entities.
- `dcterms:type` (`http://purl.org/dc/terms/type`) provides an optional entity kind.
- `dcterms:requires` (`http://purl.org/dc/terms/requires`) expresses dependency direction: the subject requires the object.

For `A dcterms:requires B`, CASTLE treats B as upstream of A. The impacted closure of B is therefore B plus every entity reachable through transitive `dcterms:requires` paths from dependents to B.

## Runtime interface

`src/ggen-rdf.ts` provides the bounded TypeScript adapter:

- `assertPinnedVersion()` — refuses ggen package/version/commit drift.
- `validate()` — asks the bridge to parse/canonicalize the Turtle graph with ggen and return graph state hash/count.
- `query()` / `queryRaw()` — execute SPARQL through `ggen-engine`.
- `construct()` — executes SPARQL CONSTRUCT through the same ggen engine.
- `loadDependencyGraph()` — projects PROV/DCTERMS query results into CASTLE graph types.
- `impactedClosure()` — computes dependency consequence closure inside ggen using a SPARQL property path.
- `constructCompromise()` — creates a `COUNTERFACTUAL` CASTLE compromise using the ggen-derived closure.

The process runner uses direct `spawn()` with `shell: false`, time and output bounds, and typed refusal on non-zero process status. Dynamic resource IRIs are validated before they are admitted into SPARQL.

## Exact subject

`configs/ggen.lock.json` and `crates/castle-ggen-rdf/Cargo.toml` pin:

- package: `ggen-engine`
- version asserted by the bridge: `26.8.15`
- git commit: `162e466d8f07d0a75a468b4441b4bc8b1aad369b`

CASTLE CI builds the bridge from that exact git revision, asks the bridge to report the semantic subject it was compiled for, and then runs real RDF/SPARQL/CONSTRUCT qualification against `rdf/examples/dependency-system.ttl`.

## Standing law

```text
RDF standing
  => exact ggen-engine source revision
  => successfully built semantic bridge
  => exact engine/version/commit assertion
  => successful real Turtle parse + state hash
  => successful SELECT + property-path + CONSTRUCT
  => structurally validated bindings
  => typed CASTLE consequence
```

A scripted runner is used only for adapter-unit tests. The separate integration court must execute the real bridge backed by the exact ggen-engine revision before the RDF boundary can claim ALIVE standing.
