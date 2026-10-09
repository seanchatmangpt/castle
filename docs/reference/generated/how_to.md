# How to: Using castle

## Prerequisites


- @chatman/castle::test (script)

- CastlePaaS.Admission::class_iri (str_key)

- CastlePaaS.Admission::table (str_key)

- CastlePaaS.Admission::tenant_scoped? (str_key)

- CastlePaaS.AdmissionProvider.Refuse::BLOCKED_ADMISSION_PROVIDER_NOT_CONFIGURED (str_key)

- CastlePaaS.AdmissionProvider.Refuse::admit (function)

- CastlePaaS.AdmissionWitness::REFUSED_ADMISSION_WITNESS_MISMATCH (str_key)

- CastlePaaS.AdmissionWitness::REFUSED_INCOMPLETE_ADMISSION_WITNESS (str_key)

- CastlePaaS.AdmissionWitness::REFUSED_INVALID_ADMISSION_WITNESS (str_key)

- CastlePaaS.AdmissionWitness::external_id (function)

- CastlePaaS.AdmissionWitness::external_id (str_key)

- CastlePaaS.AdmissionWitness::external_id (str_key)

- CastlePaaS.AdmissionWitness::id (str_key)

- CastlePaaS.AdmissionWitness::id (str_key)

- CastlePaaS.AdmissionWitness::verify (function)

- CastlePaaS.Application::name (str_key)

- CastlePaaS.Application::start (function)

- CastlePaaS.Application::strategy (str_key)

- CastlePaaS.Canonical::case (str_key)

- CastlePaaS.Canonical::sha256 (function)

- CastlePaaS.Capability::class_iri (str_key)

- CastlePaaS.Capability::table (str_key)

- CastlePaaS.Capability::tenant_scoped? (str_key)

- CastlePaaS.Domain::extensions (str_key)

- CastlePaaS.Evidence::class_iri (str_key)

- CastlePaaS.Evidence::table (str_key)

- CastlePaaS.Evidence::tenant_scoped? (str_key)

- CastlePaaS.ExecutionIntent::class_iri (str_key)

- CastlePaaS.ExecutionIntent::table (str_key)

- CastlePaaS.ExecutionIntent::tenant_scoped? (str_key)

- CastlePaaS.Generated.Resource::CastlePaaS.Generated.Resource (ash_resource)

- CastlePaaS.Generated.Resource::allow_nil? (str_key)

- CastlePaaS.Generated.Resource::allow_nil? (str_key)

- CastlePaaS.Generated.Resource::class_iri (str_key)

- CastlePaaS.Generated.Resource::constraints (str_key)

- CastlePaaS.Generated.Resource::data_layer (str_key)

- CastlePaaS.Generated.Resource::default (str_key)

- CastlePaaS.Generated.Resource::default (str_key)

- CastlePaaS.Generated.Resource::domain (str_key)

- CastlePaaS.Generated.Resource::extensions (str_key)


## Steps


1. Use `test` from `@chatman/castle`.

2. Use `admission_provider` from `CastlePaaS`.

3. Use `kernel` from `CastlePaaS`.

4. Use `receipt_verifier` from `CastlePaaS`.

5. Use `semantic_bundle` from `CastlePaaS`.

6. Use `class_iri` from `CastlePaaS.Admission`.

7. Use `table` from `CastlePaaS.Admission`.

8. Use `tenant_scoped?` from `CastlePaaS.Admission`.

9. Use `admit` from `CastlePaaS.AdmissionProvider.Refuse`.

10. Use `BLOCKED_ADMISSION_PROVIDER_NOT_CONFIGURED` from `CastlePaaS.AdmissionProvider.Refuse`.

11. Use `external_id` from `CastlePaaS.AdmissionWitness`.

12. Use `verify` from `CastlePaaS.AdmissionWitness`.


## Verified snippet

<!-- The snippet slot carries code copied from the extracted code surface -->
<!-- (doc:Claim rows whose doc:attribute is "snippet"), never agent prose. -->

```rust
// CastlePaaS :: admission_provider
admission_provider/0
```

<!-- AGENT-COMMENTARY-BEGIN -->
<!-- The ONLY region an agent may write into. Bounds: <= 12 lines,    -->
<!-- <= 100 chars/line, no new code facts (any new symbol mentioned   -->
<!-- must exist in queries/ast_extract.rq output; the doc_quality     -->
<!-- court fails Phi_halluc > 0.001 otherwise). No tables, no         -->
<!-- signatures, no parameters, no error lists — AGENT-FORBIDDEN      -->
<!-- everywhere.                                                      -->
<!-- AGENT-COMMENTARY-END -->
