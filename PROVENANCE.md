# Provenance

CASTLE separates semantic authority from consumer consequences.

- Marketplace authority: `seanchatmangpt/ggen-marketplace/packs/castle-pack`
- RDF/semantic engine: `seanchatmangpt/ggen` v26.8.15 at commit `162e466d8f07d0a75a468b4441b4bc8b1aad369b`
- ggen Linux x86_64 release artifact SHA-256: `f4c4ea396f5cec12cfa2dd46c13ac620291f9b83138f17ded6fa59c510dcfc42`
- Consumer: `seanchatmangpt/castle`
- Generated consequences: `src/generated.ts`, `docs/GENERATED_ARCHITECTURE.md`
- Runtime consequences: `src/castle.ts`, `src/ggen-rdf.ts`, tests, and adapters

The marketplace ontology owns the generated component inventory, authority boundaries, and default prohibited-goal priorities. The runtime must not silently redefine those facts.

CASTLE does not implement an independent RDF parser or triple store. RDF validation, SPARQL execution, dependency-graph traversal, and mu1 ontology CONSTRUCT/inference are delegated to the pinned ggen semantic engine. CASTLE consumes the resulting semantic bindings as typed DfCM/runtime structures. `configs/ggen.lock.json` records the admitted ggen subject; CI downloads the exact release artifact and verifies its SHA-256 before granting RDF integration standing.

A BLAKE3 digest establishes content identity, not producer identity by itself. CASTLE receipts therefore require both a BLAKE3-256 digest provider and an origin signer. Epistemic class (`CONSTRUCTED`, `COUNTERFACTUAL`, `REPLAYED`, `OBSERVED`, `INFERRED`) remains explicit so constructed OCEL histories cannot be confused with observed history.
