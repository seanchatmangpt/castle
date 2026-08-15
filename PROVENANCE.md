# Provenance

CASTLE separates semantic authority from consumer consequences.

- Marketplace authority: `seanchatmangpt/ggen-marketplace/packs/castle-pack`
- RDF/semantic engine source: `seanchatmangpt/ggen` package `ggen-engine` v26.8.15 at commit `162e466d8f07d0a75a468b4441b4bc8b1aad369b`
- Exact consumer bridge: `crates/castle-ggen-rdf`
- Consumer: `seanchatmangpt/castle`
- Generated consequences: `src/generated.ts`, `docs/GENERATED_ARCHITECTURE.md`
- Runtime consequences: `src/castle.ts`, `src/ggen-rdf.ts`, tests, and adapters

The marketplace ontology owns the generated component inventory, authority boundaries, and default prohibited-goal priorities. The runtime must not silently redefine those facts.

CASTLE does not implement an independent RDF parser, SPARQL evaluator, reasoner, or triple store. The Rust bridge depends directly on the exact-revision `ggen-engine` package and calls its public `GraphEngine` / `GraphLawStore` surface. Turtle parsing, canonicalization, BLAKE3 graph state hashing, SELECT/ASK/CONSTRUCT execution, and dependency property-path evaluation therefore execute inside ggen. CASTLE receives only the engine-neutral JSON projection and turns those bindings into typed DfCM/runtime structures.

`configs/ggen.lock.json` records the admitted ggen subject. CI builds the bridge from the exact git revision and then executes real RDF/SPARQL/CONSTRUCT qualification against the CASTLE dependency fixture before this boundary can claim standing.

A BLAKE3 digest establishes content identity, not producer identity by itself. CASTLE receipts therefore require both a BLAKE3-256 digest provider and an origin signer. Epistemic class (`CONSTRUCTED`, `COUNTERFACTUAL`, `REPLAYED`, `OBSERVED`, `INFERRED`) remains explicit so constructed OCEL histories cannot be confused with observed history.
