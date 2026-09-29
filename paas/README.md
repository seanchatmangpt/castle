# CASTLE PaaS

CASTLE PaaS is the multi-tenant application/control plane around the existing CASTLE Rust kernel. It uses Ash Framework for the domain model, AshPostgres for persistence, AshR2RML for semantic correspondence and OBDA, Reactor for dependency-resolving sagas, AshJsonApi for client-facing APIs, and ggen for deterministic manufacture.

## Constitutional split

```text
clients / portals / SDKs / automation
            |
       AshJsonApi
            |
     Ash Domain/Actions
            |
   +--------+---------+
   |                  |
AshPostgres       Reactor sagas
   |                  |
AshR2RML IR       SELECT/CONSTRUCT
   |                  |
R2RML/SHACL/OBDA      |
   |            CASTLE Kernel Port
   |                  |
public graph      exact admission
                  BRCE PREPARE
                       |
                 admitted adapter DO
                       |
                  BRCE OUTCOME
                       |
               receipt + OCEL/replay
```

`Ash != authority broker`. `Reactor != actuator`. `AshR2RML != data layer`. `ggen != runtime authority`. The Rust kernel continues to own exact-subject admission and the consequential BRCE boundary.

## Source of truth

The reusable source is `ggen-marketplace/packs/castle-paas-pack` on the companion purpose branch. Its RDF graph describes the PaaS resource/workflow topology and aligns every runtime resource to public ontology classes. The pack produces deterministic resource/reactor catalogs and ontology references.

The PaaS then uses the compiled Ash resources as the maintained application graph. `AshR2RML.Ggen.compile_ash_ttl_bundle/1` derives ontology, operational SHACL and R2RML from that actual compiled graph. `AshR2RML.Ggen.compile_api_bundle/2` may derive GraphQL/JSON:API resource blocks from the same admitted mapping IR. ggen remains the owner of file writes, replay and manufacture receipts.

This yields a closed correspondence:

```text
marketplace RDF topology
  -> ggen projection
  -> compiled Ash resources/actions
  -> AshR2RML.Mapping IR
  -> R2RML + SHACL + public RDF
  -> API/schema projections
  -> Reactor execution topology
  -> CASTLE admission/BRCE
  -> receipt + replay
```

## Public ontology profile

CASTLE PaaS does not invent a parallel enterprise ontology. Runtime semantics use public vocabularies:

- **PROV-O** — subject/evidence/receipt entities, admission/replay activities, plans, generation/usage provenance;
- **ODRL 2.2** — requests, policies, constraints, permission/prohibition semantics around execution intent;
- **DCAT 3** — platform data services, evidence/catalog datasets and distributions;
- **DCTERMS** — identifiers, titles, creation dates, conformance links;
- **SKOS** — capability/control concepts and taxonomic mappings;
- **SOSA/SSN** — observations, observed properties, features of interest and result time;
- **QUDT** — measured quantities, units, thresholds and SLO evidence;
- **SHACL** — operational profile/admission shapes;
- **RDF/RDFS/OWL** — graph substrate and type semantics;
- **Schema.org** — organization/service/software identities;
- **UCO/CASE** — optional security-evidence alignment where their native identity is appropriate.

Application-specific IRIs identify instances and generation metadata; they do not redefine public classes.

## Ash resource plane

The first platform slice contains:

1. `Organization` — tenant/owner identity (`schema:Organization`).
2. `PlatformService` — service catalog/API surface (`dcat:DataService`).
3. `Subject` — exact system/workload/repository subject (`prov:Entity`).
4. `Observation` — observed facts with digest/time/standing (`sosa:Observation`).
5. `Admission` — O -> O* decision and witness (`prov:Activity`).
6. `Plan` — inert constructed plan (`prov:Plan`).
7. `ExecutionIntent` — non-actuating request (`odrl:Request`).
8. `Evidence` — digest-bound evidence (`prov:Entity`).
9. `Receipt` — PREPARE/OUTCOME/REPLAY receipt entity (`prov:Entity`).
10. `Replay` — replay verification activity (`prov:Activity`).
11. `Capability` — public taxonomy concept (`skos:Concept`).

All tenant-owned records carry an Ash attribute multitenancy key. Secrets and credentials are not semantic attributes and are never mapped by AshR2RML.

## Action model

Ash actions are the public application contract:

- observe/register actions may create observations and subjects;
- admission actions create a request and store a kernel result, but cannot synthesize an admitted witness;
- construct actions create Plans/ExecutionIntents only;
- receipt creation is private/internal and accepts only a kernel-produced receipt payload;
- replay actions verify existing receipt identity and correspondence;
- client-facing API routes expose safe resource actions, not kernel or provider command strings.

Non-Elixir clients should use the derived JSON:API/OpenAPI surface. They do not need to know Ash exists, while every data transition still passes through an Ash action.

## Reactor plane

Seven named reactors form the platform workflow vocabulary:

### RegisterSubject

`validate input -> create/read Organization -> upsert Subject -> emit provenance observation`

No DO authority.

### AdmitSubject

`load Subject -> collect Observation -> AshR2RML semantic compile -> SHACL/profile validate -> evaluate policy -> kernel admission -> persist Admission/Evidence`

A Reactor result is not O*. Only a valid kernel admission witness persisted with its exact subject/policy/evidence digests may represent O*.

### ConstructIntent

`load admitted subject -> enumerate reversible candidates -> select admitted plan policy -> persist Plan -> persist inert ExecutionIntent`

No provider call is reachable.

### ExecuteIntent

`load exact Admission + Intent -> kernel BRCE PREPARE -> verify prepare receipt -> invoke allowlisted provider adapter -> capture observed outcome -> kernel BRCE OUTCOME -> persist Evidence + Receipt -> append OCEL event`

This is the **only** PaaS Reactor allowed to cross the DO boundary. The companion ggen pack refuses any topology with more than one DO-bound Reactor or a DO-bound Reactor whose authority is not `BRCE_ONLY`.

### QualifyEvidence

`ingest -> preserve native identity -> semantic alignment -> qualify coverage/standing -> persist evidence`

Evidence never self-promotes into admission or authority.

### ReplayReceipt

`load receipt DAG -> verify digest/signature -> re-run deterministic verifier -> compare exact subject/config/toolchain identity -> persist Replay standing`

### PublishProjection

`compile Ash resource graph -> AshR2RML IR -> ggen path/content graph -> deterministic two-pass manufacture -> verify diff/receipt -> publish generated consequences`

Generation remains CONSTRUCT. It does not deploy or actuate the platform.

## Reactor compensation law

Reactor rollback may:

- roll back local database mutations when still inside a transaction;
- mark a local intent/admission/run as refused/failed;
- manufacture a compensating **intent** for later admission;
- emit a receipt/evidence event describing an incomplete transition.

Reactor rollback may **not** call a provider to undo an external side effect unless that compensating operation independently traverses `Admission -> BRCE PREPARE -> DO -> BRCE OUTCOME`. This prevents saga compensation from becoming a hidden second actuator.

## Kernel port

The Elixir service talks to CASTLE through a narrow `CastlePaaS.Kernel` behaviour. The initial adapter is a supervised external port/CLI boundary to the exact CASTLE binary. The payload is typed JSON; executable command text is never accepted from a client or model.

Kernel operations are intentionally small:

- `observe_version/0`
- `admit/1`
- `prepare/1`
- `outcome/1`
- `verify_receipt/1`
- `replay/1`

Every response carries the kernel identity and evidence/receipt digests needed for persistence. A transport-success response without a valid semantic result is not success.

## Semantic projection fence

The RDF/OBDA surface is a dissemination surface. The following are structurally excluded from AshR2RML mappings:

- credentials, tokens, private keys and provider secrets;
- decrypted secret values;
- private command payloads;
- soft-deleted/archived records unless a dedicated evidence projection intentionally includes their non-secret metadata;
- any field whose Ash field policy is the only thing preventing disclosure.

Safe semantic projection resources should contain public IDs, classifications, provenance, standing, hashes, timestamps, measurements and non-secret metadata. This explicitly avoids relying on a query backend to reconstruct runtime authorization after data leaves the Ash action path.

## Deployment plane

The PaaS is a normal OTP application with Postgres. Deployment can be container/Kubernetes/BEAM release based, but deployment tooling has no CASTLE DO authority by association. Recommended production topology:

```text
2+ CastlePaaS BEAM nodes
  -> PostgreSQL primary + replica/PITR
  -> local or sidecar CASTLE kernel binary pinned by digest
  -> admitted provider adapters
  -> OTLP collector
  -> immutable receipt/evidence archive
```

Read replicas may serve catalog/evidence reads. Admission/intent/receipt writes remain on the authoritative write path.

## Version pins for this slice

- CASTLE source base: `5059c45a8ef007cd1e095213c729c8bfe6db9e79`
- AshR2RML exact source: `067954ad406fd637fd47646bdb10c4580809c79d`
- Ash: `3.32.x`
- AshPostgres: `2.12.x`
- AshJsonApi: `1.7.x`
- Reactor: `1.0.x`

Pin exact lockfile identities in the manufactured consumer. Version ranges describe compatibility, not an execution receipt.

## Acceptance ladder

1. ggen marketplace structural/gate validation.
2. ggen two-pass deterministic projection equality.
3. `mix format --check-formatted`.
4. `mix compile --warnings-as-errors`.
5. pure resource/action and refusal tests.
6. real Postgres migration/action tests.
7. AshR2RML compile + R2RML/SHACL round-trip tests.
8. semantic projection exclusion tests for secret/private fields.
9. Reactor dependency + rollback tests.
10. kernel-port protocol tests against the exact CASTLE binary.
11. end-to-end `AdmitSubject` with a real persisted O* witness.
12. end-to-end `ExecuteIntent` proving PREPARE -> observed DO -> OUTCOME and persisted receipt.
13. deterministic `ReplayReceipt` against the exact receipt DAG.
14. JSON:API/OpenAPI contract tests.
15. release/container smoke test against Postgres and exact kernel digest.

Only stages actually executed against the exact admitted head can contribute `ALIVE` standing.

## Falsifiers

Revoke/refuse PaaS standing if any of the following becomes true:

- an Ash action, Reactor step, API route, model output or ggen projection can call provider DO without the kernel BRCE prepare witness;
- Reactor compensation performs ambient external actuation;
- a transport `200`/exit `0` is treated as semantic admission without verifying the response identity;
- receipt creation is exposed as an unrestricted client action;
- semantic projection includes credentials/decrypted secret plaintext;
- database identity is silently reused as RDF identity without an explicit subject mapping;
- a private CASTLE class replaces a suitable public ontology identity merely for convenience;
- generated files are edited as source authority;
- a replay crowns a receipt whose source/config/toolchain identity differs from the admitted subject;
- a skipped external integration test is reported as executed success.
