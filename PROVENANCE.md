# Provenance

CASTLE separates semantic authority from consumer consequences.

- Marketplace authority: `seanchatmangpt/ggen-marketplace/packs/castle-pack`
- Consumer: `seanchatmangpt/castle`
- Generated consequences: `src/generated.rs`, `src/fortune5_generated.rs`, `docs/GENERATED_ARCHITECTURE.md`, `docs/FORTUNE5_REQUIREMENTS.md`
- Runtime consequences: `src/castle.rs`, `src/board.rs`, `src/fortune5.rs`, tests, and adapters

The marketplace ontology owns the generated component inventory, authority boundaries, and default prohibited-goal priorities. The runtime must not silently redefine those facts.

A BLAKE3 digest establishes content identity, not producer identity by itself. CASTLE receipts therefore require both a BLAKE3-256 digest provider and an origin signer. Epistemic class (`CONSTRUCTED`, `COUNTERFACTUAL`, `REPLAYED`, `OBSERVED`, `INFERRED`) remains explicit so constructed OCEL histories cannot be confused with observed history.
