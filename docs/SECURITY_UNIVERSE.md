# Federated Security Universe

CASTLE treats cybersecurity frameworks, ontologies, evidence schemas, cloud security products, and open-source tools as a federated authority graph—not as a copied mega-framework and not as a claim that adjacency implies equivalence.

## Canonical manufacture

The reusable source lives in `ggen-marketplace/packs/castle-pack`:

`security-universe.ttl + security-tools.ttl → SPARQL → ggen templates → generated Rust → runtime admission → receipt-bound standing`

CASTLE pins both the marketplace source revision and the ggen manufacturer revision in `ggen.ecosystem.lock.toml`. The consumer `ggen.toml` names every source, query, template, and output path. Generated Rust is a projection; edit the RDF/query/template source instead.

The generated projections are:

- `src/security_sources_generated.rs` — external authority identity, version policy, kind, machine surface, and canonical source URI.
- `src/security_tools_generated.rs` — integration topology and the explicit absence of direct CASTLE actuation authority.
- `src/security_core_generated.rs` — the 22-identity internal Fortune-5 security core.

## Authority law

Registry membership means **known topology only**. It does not mean certification, conformance, accreditation, endorsement, legal sufficiency, or semantic equivalence. External authorities retain their native normative meaning; CASTLE stores identity and machine-integration metadata, not proprietary normative text.

Mappings use an explicit relation (`Related`, `Implements`, `Assesses`, `Mitigates`, `Detects`, `Evidences`, and so on). `Equivalent` is uniquely privileged: it is refused without a 64-hex receipt naming an independent equivalence proof. This preserves the rule that adjacency is not refutation and similarity is not equivalence.

Security tools are observation, assessment, detection, intent-construction, or external-enforcer surfaces. The generated graph requires `directActuationAuthority = false` for every tool. Native enforcement products may operate in their own authorized domains, but they do not acquire CASTLE DO authority by appearing in the registry.

## Machine standing

`qualify_fortune5_security_universe` accepts receipt-bound evidence per required source. For each source it separates:

- imported objects,
- mapped objects,
- verified objects,
- source identity digest,
- receipt digest.

Counts must satisfy `verified <= mapped <= imported`. Imported evidence without a receipt is refused. Missing required sources produce `UNKNOWN`; present but incompletely verified sources produce `PARTIAL_ALIVE`; all 22 required identities with complete receipt-bound verification produce `ALIVE`. These standings describe the admitted machine evidence only.

`qualify_federated_fortune5` composes that security standing with CASTLE's existing generated Fortune-5 runtime qualification. Neither side can crown the other: base `UNKNOWN` remains `UNKNOWN`; any refusal remains refused; security `PARTIAL_ALIVE` prevents an overall `ALIVE` even when the base profile is alive.

## Extension rail

The generated tool catalog is not closed-world. A future product can enter through `ExternalToolEvidence`, which binds:

- tool and authority identity,
- native version/object identity,
- evidence surface,
- adapter identity digest,
- payload digest,
- receipt digest.

Admission of that evidence still sets `direct_actuation_authority = false`. This preserves combinatorial extensibility without manufacturing a side door around BRCE.

## Falsifiers

The security-universe claim is falsified if any of the following becomes true:

- two generated sources or tools share an identifier;
- a required Fortune-5 core identity is absent;
- the core cardinality drifts from 22 without a new admitted semantic version;
- a generated tool claims direct CASTLE actuation authority;
- an equivalence mapping is admitted without a proof receipt;
- imported evidence is accepted without a receipt;
- generated outputs cannot be reproduced from the pinned graph/query/template/manufacturer identities.
