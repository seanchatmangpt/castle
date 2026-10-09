# castle reference

<!-- ============================================================= -->
<!-- AGENT-FORBIDDEN-BEGIN: reference body is RIGID                -->
<!-- Every row below is rendered from queries/ast_extract.rq.      -->
<!-- Agents MUST NOT add, edit, reorder, or remove any row or      -->
<!-- table cell. Prose outside the fenced slot below is refused    -->
<!-- by the doc_quality court.                                     -->
<!-- ============================================================= -->

## Modules


### @chatman/castle

| `test` | script | node --experimental-strip-types --test test/*.test.ts |  |  |  |  |


### CastlePaaS

| `admission_provider` | function | admission_provider/0 |  |  |  |  |

| `kernel` | function | kernel/0 |  |  |  |  |

| `receipt_verifier` | function | receipt_verifier/0 |  |  |  |  |

| `semantic_bundle` | function | semantic_bundle/0 |  |  |  |  |


### CastlePaaS.Admission

| `class_iri` | str_key | class_iri: "http: |  |  |  |  |

| `table` | str_key | table: "castle_admissions" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: true |  |  |  |  |


### CastlePaaS.AdmissionProvider.Refuse

| `admit` | function | admit/3 |  |  |  |  |

| `BLOCKED_ADMISSION_PROVIDER_NOT_CONFIGURED` | str_key | {:error, :BLOCKED_ADMISSION_PROVIDER_NOT_CONFIGURED} |  |  |  |  |


### CastlePaaS.AdmissionWitness

| `external_id` | function | external_id/1 |  |  |  |  |

| `verify` | function | verify/3 |  |  |  |  |

| `REFUSED_ADMISSION_WITNESS_MISMATCH` | str_key | {:error, :REFUSED_ADMISSION_WITNESS_MISMATCH} |  |  |  |  |

| `REFUSED_INCOMPLETE_ADMISSION_WITNESS` | str_key | {:error, :REFUSED_INCOMPLETE_ADMISSION_WITNESS} |  |  |  |  |

| `REFUSED_INVALID_ADMISSION_WITNESS` | str_key | {:error, :REFUSED_INVALID_ADMISSION_WITNESS} |  |  |  |  |

| `external_id` | str_key | external_id: value |  |  |  |  |

| `external_id` | str_key | "external_id" => _ |  |  |  |  |

| `id` | str_key | id: value |  |  |  |  |

| `id` | str_key | "id" => _ |  |  |  |  |


### CastlePaaS.Application

| `start` | function | start/2 |  |  |  |  |

| `name` | str_key | name: CastlePaaS.Supervisor |  |  |  |  |

| `strategy` | str_key | strategy: :one_for_one |  |  |  |  |


### CastlePaaS.Canonical

| `sha256` | function | sha256/1 |  |  |  |  |

| `case` | str_key | case: :lower |  |  |  |  |


### CastlePaaS.Capability

| `class_iri` | str_key | class_iri: "http: |  |  |  |  |

| `table` | str_key | table: "castle_capabilities" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: false |  |  |  |  |


### CastlePaaS.Domain

| `extensions` | str_key | extensions: [AshJsonApi.Domain] |  |  |  |  |


### CastlePaaS.Evidence

| `class_iri` | str_key | class_iri: "http: |  |  |  |  |

| `table` | str_key | table: "castle_evidence" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: true |  |  |  |  |


### CastlePaaS.ExecutionIntent

| `class_iri` | str_key | class_iri: "http: |  |  |  |  |

| `table` | str_key | table: "castle_execution_intents" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: true |  |  |  |  |


### CastlePaaS.Generated.Resource

| `CastlePaaS.Generated.Resource` | ash_resource |  |  |  |  |  |

| `allow_nil?` | str_key | allow_nil?: false |  |  |  |  |

| `allow_nil?` | str_key | allow_nil?: true |  |  |  |  |

| `class_iri` | str_key | Keyword.fetch!("class_iri") |  |  |  |  |

| `constraints` | str_key | constraints: [ one_of: [ :UNKNOWN, :PARTIAL_ALIVE, :ALIVE, :BLOCKED, :BUILD_BROKEN, :UNSUPPORTED, :REFUSED ] ] |  |  |  |  |

| `data_layer` | str_key | data_layer: AshPostgres.DataLayer |  |  |  |  |

| `default` | str_key | default: :UNKNOWN |  |  |  |  |

| `default` | str_key | default: %{} |  |  |  |  |

| `domain` | str_key | domain: CastlePaaS.Domain |  |  |  |  |

| `extensions` | str_key | extensions: [AshR2RML, AshJsonApi.Resource] |  |  |  |  |

| `one_of` | str_key | one_of: [ :UNKNOWN, :PARTIAL_ALIVE, :ALIVE, :BLOCKED, :BUILD_BROKEN, :UNSUPPORTED, :REFUSED ] |  |  |  |  |

| `public?` | str_key | public?: false |  |  |  |  |

| `public?` | str_key | public?: true |  |  |  |  |

| `table` | str_key | Keyword.fetch!("table") |  |  |  |  |

| `tenant_scoped?` | str_key | Keyword.fetch!("tenant_scoped?") |  |  |  |  |


### CastlePaaS.Kernel

| `result` | type | @type result :: {:ok, map()} | {:error, term()} |  |  |  |  |


### CastlePaaS.Kernel.CLI

| `execute` | function | execute/3 |  |  |  |  |

| `manufacture` | function | manufacture/1 |  |  |  |  |

| `release_info` | function | release_info/0 |  |  |  |  |

| `BLOCKED_CASTLE_BINARY` | str_key | {:error, {:BLOCKED_CASTLE_BINARY, reason}} |  |  |  |  |

| `BLOCKED_CASTLE_RUNTIME_CONFIGURATION` | str_key | {:error, :BLOCKED_CASTLE_RUNTIME_CONFIGURATION} |  |  |  |  |

| `BLOCKED_KERNEL_TRANSPORT` | str_key | {:error, {:BLOCKED_KERNEL_TRANSPORT, Exception.message(error)}} |  |  |  |  |

| `REFUSED_AMBIENT_COMMAND_POLICY` | str_key | {:error, :REFUSED_AMBIENT_COMMAND_POLICY} |  |  |  |  |

| `REFUSED_CASTLE_RUNTIME_CONFIGURATION` | str_key | {:error, :REFUSED_CASTLE_RUNTIME_CONFIGURATION} |  |  |  |  |

| `REFUSED_CASTLE_RUNTIME_IDENTITY` | str_key | {:error, :REFUSED_CASTLE_RUNTIME_IDENTITY} |  |  |  |  |

| `REFUSED_CONSTRUCT_NOT_ALIVE` | str_key | {:error, :REFUSED_CONSTRUCT_NOT_ALIVE} |  |  |  |  |

| `REFUSED_INVALID_ADAPTER_PROFILE` | str_key | {:error, :REFUSED_INVALID_ADAPTER_PROFILE} |  |  |  |  |

| `REFUSED_INVALID_BRCE_RECEIPT_DIGEST` | str_key | {:error, :REFUSED_INVALID_BRCE_RECEIPT_DIGEST} |  |  |  |  |

| `REFUSED_INVALID_DIGEST` | str_key | {:error, :REFUSED_INVALID_DIGEST} |  |  |  |  |

| `REFUSED_KERNEL_DO` | str_key | {:error, {:REFUSED_KERNEL_DO, other}} |  |  |  |  |

| `REFUSED_KERNEL_EXIT` | str_key | {:error, {:REFUSED_KERNEL_EXIT, status, output}} |  |  |  |  |

| `REFUSED_KERNEL_MANUFACTURE` | str_key | {:error, {:REFUSED_KERNEL_MANUFACTURE, other}} |  |  |  |  |

| `REFUSED_NON_JSON_KERNEL_RESPONSE` | str_key | {:error, {:REFUSED_NON_JSON_KERNEL_RESPONSE, output}} |  |  |  |  |

| `REFUSED_O_STAR_REQUIRED` | str_key | {:error, :REFUSED_O_STAR_REQUIRED} |  |  |  |  |

| `REFUSED_REQUIRED_FIELD` | str_key | {:error, {:REFUSED_REQUIRED_FIELD, key}} |  |  |  |  |

| `REFUSED_UNKNOWN_ADAPTER_PROFILE` | str_key | {:error, :REFUSED_UNKNOWN_ADAPTER_PROFILE} |  |  |  |  |

| `REFUSED_UNRECEIPTED_DO` | str_key | {:error, :REFUSED_UNRECEIPTED_DO} |  |  |  |  |

| `REFUSED_WRONG_KERNEL_IDENTITY` | str_key | {:error, :REFUSED_WRONG_KERNEL_IDENTITY} |  |  |  |  |

| `adapter_policy` | str_key | Map.get("adapter_policy") |  |  |  |  |

| `adapter_policy` | str_key | "adapter_policy" => _ |  |  |  |  |

| `adapter_profile_id` | str_key | Map.get("adapter_profile_id") |  |  |  |  |

| `admitted` | str_key | Map.get("admitted") |  |  |  |  |

| `allowed_authorities` | str_key | Map.get("allowed_authorities") |  |  |  |  |

| `allowed_authorities` | str_key | "allowed_authorities" => _ |  |  |  |  |

| `authority` | str_key | "authority" => _ |  |  |  |  |

| `bin` | str_key | bin: bin |  |  |  |  |

| `bin_sha256` | str_key | bin_sha256: actual |  |  |  |  |

| `brce_outcome_receipt_digests` | str_key | "brce_outcome_receipt_digests" => _ |  |  |  |  |

| `brce_prepare_receipt_digests` | str_key | "brce_prepare_receipt_digests" => _ |  |  |  |  |

| `case` | str_key | case: :lower |  |  |  |  |

| `cell_id` | str_key | "cell_id" => _ |  |  |  |  |

| `cell_id` | str_key | Map.get("cell_id") |  |  |  |  |

| `config_graph` | str_key | "config_graph" => _ |  |  |  |  |

| `config_graph` | str_key | Map.get("config_graph") |  |  |  |  |

| `construct_digest` | str_key | "construct_digest" => _ |  |  |  |  |

| `envelope` | str_key | "envelope" => _ |  |  |  |  |

| `evidence_commit` | str_key | "evidence_commit" => _ |  |  |  |  |

| `evidence_dir` | str_key | "evidence_dir" => _ |  |  |  |  |

| `key_id` | str_key | key_id: key_id |  |  |  |  |

| `name` | str_key | "name" => _ |  |  |  |  |

| `o_star` | str_key | "o_star" => _ |  |  |  |  |

| `ontology` | str_key | "ontology" => _ |  |  |  |  |

| `ontology` | str_key | Map.get("ontology") |  |  |  |  |

| `process` | str_key | "process" => _ |  |  |  |  |

| `release` | str_key | "release" => _ |  |  |  |  |

| `signing_key_path` | str_key | signing_key_path: signing_key_path |  |  |  |  |

| `standing` | str_key | "standing" => _ |  |  |  |  |

| `stderr_to_stdout` | str_key | stderr_to_stdout: true |  |  |  |  |

| `subject` | str_key | "subject" => _ |  |  |  |  |

| `version` | str_key | "version" => _ |  |  |  |  |

| `zeroUnreceiptedActuation` | str_key | "zeroUnreceiptedActuation" => _ |  |  |  |  |


### CastlePaaS.Observation

| `class_iri` | str_key | class_iri: "http: |  |  |  |  |

| `table` | str_key | table: "castle_observations" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: true |  |  |  |  |


### CastlePaaS.Organization

| `class_iri` | str_key | class_iri: "https: |  |  |  |  |

| `table` | str_key | table: "castle_organizations" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: false |  |  |  |  |


### CastlePaaS.Persistence

| `record` | function | record/3 |  |  |  |  |


### CastlePaaS.Plan

| `class_iri` | str_key | class_iri: "http: |  |  |  |  |

| `table` | str_key | table: "castle_plans" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: true |  |  |  |  |


### CastlePaaS.PlatformService

| `class_iri` | str_key | class_iri: "http: |  |  |  |  |

| `table` | str_key | table: "castle_platform_services" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: true |  |  |  |  |


### CastlePaaS.Reactors.AdmitSubject

| `admission` | str_key | admission: args.admission |  |  |  |  |

| `digest` | str_key | digest: args.witness["witness_digest"] |  |  |  |  |

| `external_id` | str_key | external_id: "admission:#{CastlePaaS.AdmissionWitness.external_id(args.subject)}" |  |  |  |  |

| `label` | str_key | label: "CASTLE O* admission" |  |  |  |  |

| `metadata` | str_key | metadata: args.witness |  |  |  |  |

| `provider` | str_key | provider: provider |  |  |  |  |

| `standing` | str_key | standing: :ALIVE |  |  |  |  |

| `witness` | str_key | witness: args.witness |  |  |  |  |


### CastlePaaS.Reactors.ConstructIntent

| `admission_result` | str_key | admission_result: %{witness: witness} |  |  |  |  |

| `admission_witness` | str_key | admission_witness: args.witness |  |  |  |  |

| `digest` | str_key | digest: digest |  |  |  |  |

| `external_id` | str_key | external_id: Map.get(process, :id) || Map.get(process, "id") || "plan:unidentified" |  |  |  |  |

| `external_id` | str_key | external_id: "intent:#{subject}:#{digest}" |  |  |  |  |

| `id` | str_key | Map.get("id") |  |  |  |  |

| `intent` | str_key | intent: intent |  |  |  |  |

| `intent` | str_key | intent: args.intent |  |  |  |  |

| `label` | str_key | label: "CASTLE inert plan" |  |  |  |  |

| `label` | str_key | label: "CASTLE inert execution intent" |  |  |  |  |

| `metadata` | str_key | metadata: %{process: process} |  |  |  |  |

| `metadata` | str_key | metadata: %{admission_witness: args.witness, intent: args.intent} |  |  |  |  |

| `now_epoch_ms` | str_key | now_epoch_ms: now |  |  |  |  |

| `plan` | str_key | plan: args.plan |  |  |  |  |

| `process` | str_key | Map.get("process") |  |  |  |  |

| `process` | str_key | process: process |  |  |  |  |

| `record` | str_key | record: record |  |  |  |  |

| `runtime_intent` | str_key | runtime_intent: Map.put(args.intent, :o_star, args.witness) |  |  |  |  |

| `standing` | str_key | standing: :PARTIAL_ALIVE |  |  |  |  |

| `subject` | str_key | Map.get("subject") |  |  |  |  |

| `witness` | str_key | witness: witness |  |  |  |  |


### CastlePaaS.Reactors.QualifyEvidence

| `digest` | str_key | Map.get("digest") |  |  |  |  |

| `digest` | str_key | digest: digest |  |  |  |  |

| `evidence` | str_key | evidence: evidence |  |  |  |  |

| `external_id` | str_key | external_id: Map.get(evidence, :external_id) || Map.get(evidence, "external_id") || "evidence:#{digest}" |  |  |  |  |

| `external_id` | str_key | Map.get("external_id") |  |  |  |  |

| `label` | str_key | label: Map.get(evidence, :label) || Map.get(evidence, "label") || "CASTLE evidence" |  |  |  |  |

| `label` | str_key | Map.get("label") |  |  |  |  |

| `metadata` | str_key | metadata: evidence |  |  |  |  |

| `standing` | str_key | Map.get("standing") |  |  |  |  |

| `standing` | str_key | standing: standing |  |  |  |  |

| `tenant` | str_key | tenant: tenant |  |  |  |  |


### CastlePaaS.Reactors.RegisterSubject

| `attrs` | str_key | attrs: attrs |  |  |  |  |

| `tenant` | str_key | tenant: tenant |  |  |  |  |


### CastlePaaS.Reactors.ReplayReceipt

| `digest` | str_key | digest: digest |  |  |  |  |

| `external_id` | str_key | external_id: "replay:#{digest}" |  |  |  |  |

| `label` | str_key | label: "CASTLE receipt replay" |  |  |  |  |

| `metadata` | str_key | metadata: args.verification |  |  |  |  |

| `receipt_digest` | str_key | Map.get("receipt_digest") |  |  |  |  |

| `standing` | str_key | standing: :ALIVE |  |  |  |  |


### CastlePaaS.Receipt

| `class_iri` | str_key | class_iri: "http: |  |  |  |  |

| `table` | str_key | table: "castle_receipts" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: true |  |  |  |  |


### CastlePaaS.ReceiptVerifier.Refuse

| `verify` | function | verify/1 |  |  |  |  |

| `BLOCKED_RECEIPT_VERIFIER_NOT_CONFIGURED` | str_key | {:error, :BLOCKED_RECEIPT_VERIFIER_NOT_CONFIGURED} |  |  |  |  |


### CastlePaaS.Replay

| `class_iri` | str_key | class_iri: "http: |  |  |  |  |

| `table` | str_key | table: "castle_replays" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: true |  |  |  |  |


### CastlePaaS.Repo

| `installed_extensions` | function | installed_extensions/0 |  |  |  |  |

| `min_pg_version` | function | min_pg_version/0 |  |  |  |  |

| `major` | str_key | major: 14 |  |  |  |  |

| `minor` | str_key | minor: 0 |  |  |  |  |

| `otp_app` | str_key | otp_app: :castle_paas |  |  |  |  |

| `patch` | str_key | patch: 0 |  |  |  |  |


### CastlePaaS.Standing

| `parse` | function | parse/1 |  |  |  |  |

| `ALIVE` | str_key | "ALIVE" => _ |  |  |  |  |

| `BLOCKED` | str_key | "BLOCKED" => _ |  |  |  |  |

| `BUILD_BROKEN` | str_key | "BUILD_BROKEN" => _ |  |  |  |  |

| `PARTIAL_ALIVE` | str_key | "PARTIAL_ALIVE" => _ |  |  |  |  |

| `REFUSED` | str_key | "REFUSED" => _ |  |  |  |  |

| `REFUSED_INVALID_STANDING` | str_key | {:error, :REFUSED_INVALID_STANDING} |  |  |  |  |

| `UNKNOWN` | str_key | "UNKNOWN" => _ |  |  |  |  |

| `UNSUPPORTED` | str_key | "UNSUPPORTED" => _ |  |  |  |  |


### CastlePaaS.Subject

| `class_iri` | str_key | class_iri: "http: |  |  |  |  |

| `table` | str_key | table: "castle_subjects" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: true |  |  |  |  |


### crates/castle-ggen-rdf/src/main.rs

| `162e466d8f07d0a75a468b4441b4bc8b1aad369b` | str_key | GGEN_COMMIT = "162e466d8f07d0a75a468b4441b4bc8b1aad369b" |  |  |  |  |

| `26.8.15` | str_key | GGEN_VERSION = "26.8.15" |  |  |  |  |


### ontology/payments-fibo/generated/fibo_generated.rs

| `FIBO_MAPPINGS` | const | FIBO_MAPPINGS: &[FiboMapping] |  |  |  |  |

| `IRI_CURRENCY` | const | IRI_CURRENCY: &str |  |  |  |  |

| `IRI_CURRENCY_EUR` | const | IRI_CURRENCY_EUR: &str |  |  |  |  |

| `IRI_CURRENCY_GBP` | const | IRI_CURRENCY_GBP: &str |  |  |  |  |

| `IRI_CURRENCY_JPY` | const | IRI_CURRENCY_JPY: &str |  |  |  |  |

| `IRI_CURRENCY_KWD` | const | IRI_CURRENCY_KWD: &str |  |  |  |  |

| `IRI_CURRENCY_USD` | const | IRI_CURRENCY_USD: &str |  |  |  |  |

| `IRI_HAS_AMOUNT` | const | IRI_HAS_AMOUNT: &str |  |  |  |  |

| `IRI_HAS_CURRENCY` | const | IRI_HAS_CURRENCY: &str |  |  |  |  |

| `IRI_HAS_PAYMENT_AMOUNT` | const | IRI_HAS_PAYMENT_AMOUNT: &str |  |  |  |  |

| `IRI_LEGAL_ENTITY_IDENTIFIER` | const | IRI_LEGAL_ENTITY_IDENTIFIER: &str |  |  |  |  |

| `IRI_MONETARY_AMOUNT` | const | IRI_MONETARY_AMOUNT: &str |  |  |  |  |

| `IRI_PAYEE` | const | IRI_PAYEE: &str |  |  |  |  |

| `IRI_PAYER` | const | IRI_PAYER: &str |  |  |  |  |

| `IRI_PAYMENT` | const | IRI_PAYMENT: &str |  |  |  |  |

| `IRI_PAYMENT_OBLIGATION` | const | IRI_PAYMENT_OBLIGATION: &str |  |  |  |  |

| `IRI_SETTLEMENT` | const | IRI_SETTLEMENT: &str |  |  |  |  |

| `currency` | str_key | FIBO_MAPPINGS = "currency" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/BE/LegalEntities/LEIEntities/LegalEntityIdentifier` | str_key | IRI_LEGAL_ENTITY_IDENTIFIER = "https://spec.edmcouncil.org/fibo/ontology/BE/LegalEntities/LEIEntities/LegalEntityIdentifier" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FBC/FinancialInstruments/Settlement/Settlement` | str_key | IRI_SETTLEMENT = "https://spec.edmcouncil.org/fibo/ontology/FBC/FinancialInstruments/Settlement/Settlement" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/CurrencyAmount/Currency` | str_key | IRI_CURRENCY = "https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/CurrencyAmount/Currency" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/CurrencyAmount/MonetaryAmount` | str_key | IRI_MONETARY_AMOUNT = "https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/CurrencyAmount/MonetaryAmount" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/CurrencyAmount/hasAmount` | str_key | IRI_HAS_AMOUNT = "https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/CurrencyAmount/hasAmount" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/CurrencyAmount/hasCurrency` | str_key | IRI_HAS_CURRENCY = "https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/CurrencyAmount/hasCurrency" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/ISO4217-CurrencyCodes/EUR` | str_key | IRI_CURRENCY_EUR = "https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/ISO4217-CurrencyCodes/EUR" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/ISO4217-CurrencyCodes/GBP` | str_key | IRI_CURRENCY_GBP = "https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/ISO4217-CurrencyCodes/GBP" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/ISO4217-CurrencyCodes/JPY` | str_key | IRI_CURRENCY_JPY = "https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/ISO4217-CurrencyCodes/JPY" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/ISO4217-CurrencyCodes/KWD` | str_key | IRI_CURRENCY_KWD = "https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/ISO4217-CurrencyCodes/KWD" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/ISO4217-CurrencyCodes/USD` | str_key | IRI_CURRENCY_USD = "https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/ISO4217-CurrencyCodes/USD" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/PaymentsAndSchedules/Payee` | str_key | IRI_PAYEE = "https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/PaymentsAndSchedules/Payee" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/PaymentsAndSchedules/Payer` | str_key | IRI_PAYER = "https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/PaymentsAndSchedules/Payer" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/PaymentsAndSchedules/Payment` | str_key | IRI_PAYMENT = "https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/PaymentsAndSchedules/Payment" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/PaymentsAndSchedules/PaymentObligation` | str_key | IRI_PAYMENT_OBLIGATION = "https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/PaymentsAndSchedules/PaymentObligation" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/PaymentsAndSchedules/hasPaymentAmount` | str_key | IRI_HAS_PAYMENT_AMOUNT = "https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/PaymentsAndSchedules/hasPaymentAmount" |  |  |  |  |

| `FiboMapping` | struct | FiboMapping { pub id: &'static str, pub castle_term: &'static str, pub fibo_iri: &'static str, pub fibo_kind: &'static str, pub match_kind: &'static str, pub status: &'static str, pub fibo_module: &'static str, pub version_iri: &'static str, pub corpus_file: &'static str, pub corpus_sha256: &'static str } |  |  |  |  |


### src/bin/castle/verbs/dfcm_handlers.rs

| `crypto_capabilities_handler` | function | crypto_capabilities_handler() -> Result<Value> |  |  |  |  |

| `deployment_bind_policy_handler` | function | deployment_bind_policy_handler( binding_path: String, policy_path: String, ) -> Result<Value> |  |  |  |  |

| `dfcm_qualify_handler` | function | dfcm_qualify_handler( manifest_path: String, evidence_path: String, now_epoch_ms: i64, max_evidence_age_ms: i64, ) -> Result<Value> |  |  |  |  |

| `dfcm_verify_handler` | function | dfcm_verify_handler() -> Result<Value> |  |  |  |  |

| `live_check_plan_handler` | function | live_check_plan_handler(manifest_path: String) -> Result<Value> |  |  |  |  |

| `live_check_qualify_handler` | function | live_check_qualify_handler( manifest_path: String, evidence_path: String, now_epoch_ms: i64, max_evidence_age_ms: i64, ) -> Result<Value> |  |  |  |  |

| `live_check_run_handler` | function | live_check_run_handler(spec_path: String, observed_at_epoch_ms: i64) -> Result<Value> |  |  |  |  |

| `protocol_dispatch_handler` | function | protocol_dispatch_handler(intent_path: String) -> Result<Value> |  |  |  |  |

| `replication_admit_handler` | function | replication_admit_handler( state_path: String, checkpoint_path: String, receiver_id: String, ) -> Result<Value> |  |  |  |  |


### src/bin/castle/verbs/evidence_handlers.rs

| `evidence_verify_handler` | function | evidence_verify_handler(evidence_path: String) -> Result<Value> |  |  |  |  |


### src/bin/castle/verbs/handlers.rs

| `chaos_qualify_handler` | function | chaos_qualify_handler(evidence_path: String) -> Result<Value> |  |  |  |  |

| `construct_manufacture_handler` | function | construct_manufacture_handler(request_path: String, signing_key_path: String, key_id: String) -> Result<Value> |  |  |  |  |

| `crypto_capabilities_handler` | function | crypto_capabilities_handler() -> Result<Value> |  |  |  |  |

| `deployment_adapters_handler` | function | deployment_adapters_handler() -> Result<Value> |  |  |  |  |

| `deployment_qualify_handler` | function | deployment_qualify_handler(manifest_path: String, now_epoch_ms: i64) -> Result<Value> |  |  |  |  |

| `do_execute_handler` | function | do_execute_handler(request_path: String, signing_key_path: String, key_id: String, expected_construct_digest: String, now_epoch_ms: i64) -> Result<Value> |  |  |  |  |

| `fortune5_qualify_handler` | function | fortune5_qualify_handler(subject: String, evidence_path: String, now_epoch_ms: Option<i64>, max_evidence_age_ms: Option<i64>) -> Result<Value> |  |  |  |  |

| `fortune5_requirements_handler` | function | fortune5_requirements_handler() -> Result<Value> |  |  |  |  |

| `impact_coverage_handler` | function | impact_coverage_handler(classes_path: String, target_coverage_bps: Option<i64>) -> Result<Value> |  |  |  |  |

| `inventory_components_handler` | function | inventory_components_handler() -> Result<Value> |  |  |  |  |

| `inventory_goals_handler` | function | inventory_goals_handler() -> Result<Value> |  |  |  |  |

| `protocol_a2a_handler` | function | protocol_a2a_handler() -> Result<Value> |  |  |  |  |

| `protocol_mcp_handler` | function | protocol_mcp_handler() -> Result<Value> |  |  |  |  |

| `release_info_handler` | function | release_info_handler() -> Result<Value> |  |  |  |  |

| `replay_admit_handler` | function | replay_admit_handler(replay_class_id: String, structural_signature: String, ontology_version: String, provider_semantics_version: String, invariant_set_digest: String, process_digest: String, invariants_hold: bool) -> Result<Value> |  |  |  |  |


### src/bin/castle/verbs/payments_handlers.rs

| `execute_handler` | function | execute_handler(a: ExecuteArgs) -> Result<Value> |  |  |  |  |

| `explain_handler` | function | explain_handler(effect_digest: String, state_dir: String, ledger_dir: String, reference_ledger: bool) -> Result<Value> |  |  |  |  |

| `finalize_handler` | function | finalize_handler( effect_digest: String, state_dir: String, ledger_dir: String, rail_dir: String, rail_mode: Option<String>, reference_ledger: bool, reference_rail: bool, ) -> Result<Value> |  |  |  |  |

| `prepare_handler` | function | prepare_handler( principal: String, payer: String, payee: String, amount_minor: String, currency: String, obligation_id: String, purpose: String, reverses: Option<String>, ) -> Result<Value> |  |  |  |  |

| `rail_submit_handler` | function | rail_submit_handler(a: RailSubmitArgs) -> Result<Value> |  |  |  |  |

| `receipt_handler` | function | receipt_handler( effect_digest: String, state_dir: String, ledger_dir: String, rail_dir: String, journal_dir: String, reference_ledger: bool, reference_rail: bool, ) -> Result<Value> |  |  |  |  |

| `reconcile_handler` | function | reconcile_handler(effect_digest: String, state_dir: String, ledger_dir: String, reference_ledger: bool) -> Result<Value> |  |  |  |  |

| `recover_handler` | function | recover_handler(state_dir: String) -> Result<Value> |  |  |  |  |

| `replay_handler` | function | replay_handler(journal_dir: String, effect_digest: String) -> Result<Value> |  |  |  |  |

| `CASTLE-CLI-FINALITY-OBSERVATION-V1` | str_key | FINALITY_DOMAIN = "CASTLE-CLI-FINALITY-OBSERVATION-V1" |  |  |  |  |

| `REFUSED:PAYMENT_CLAIM_NOT_FOUND` | str_key | CLAIM_NOT_FOUND = "REFUSED:PAYMENT_CLAIM_NOT_FOUND" |  |  |  |  |

| `REFUSED:PAYMENT_JOURNAL_WITH_SCREENING_UNSUPPORTED` | str_key | JOURNAL_WITH_SCREENING_UNSUPPORTED = "REFUSED:PAYMENT_JOURNAL_WITH_SCREENING_UNSUPPORTED" |  |  |  |  |

| `REFUSED:PAYMENT_LEDGER_NOT_FOUND` | str_key | LEDGER_NOT_FOUND = "REFUSED:PAYMENT_LEDGER_NOT_FOUND" |  |  |  |  |

| `REFUSED:PAYMENT_LEDGER_UNAVAILABLE` | str_key | LEDGER_UNAVAILABLE = "REFUSED:PAYMENT_LEDGER_UNAVAILABLE" |  |  |  |  |

| `REFUSED:PAYMENT_RAIL_MODE_INVALID` | str_key | RAIL_MODE_INVALID = "REFUSED:PAYMENT_RAIL_MODE_INVALID" |  |  |  |  |

| `REFUSED:PAYMENT_RAIL_NOT_FOUND` | str_key | RAIL_NOT_FOUND = "REFUSED:PAYMENT_RAIL_NOT_FOUND" |  |  |  |  |

| `REFUSED:PAYMENT_RECONCILE_RAIL_HOLD_LIVE` | str_key | RECONCILE_RAIL_HOLD_LIVE = "REFUSED:PAYMENT_RECONCILE_RAIL_HOLD_LIVE" |  |  |  |  |

| `REFUSED:PAYMENT_SCREENING_INPUTS_INCOMPLETE` | str_key | SCREENING_INPUTS_INCOMPLETE = "REFUSED:PAYMENT_SCREENING_INPUTS_INCOMPLETE" |  |  |  |  |

| `REFUSED:RECEIPT_RAIL_HISTORY_MISSING_SETTLEMENT` | str_key | RAIL_HISTORY_MISSING_SETTLEMENT = "REFUSED:RECEIPT_RAIL_HISTORY_MISSING_SETTLEMENT" |  |  |  |  |

| `REFUSED:REFERENCE_LEDGER_NOT_CONFIRMED` | str_key | REFERENCE_LEDGER_NOT_CONFIRMED = "REFUSED:REFERENCE_LEDGER_NOT_CONFIRMED" |  |  |  |  |

| `REFUSED:REFERENCE_RAIL_NOT_CONFIRMED` | str_key | REFERENCE_RAIL_NOT_CONFIRMED = "REFUSED:REFERENCE_RAIL_NOT_CONFIRMED" |  |  |  |  |

| `ExecuteArgs` | struct | ExecuteArgs { pub effect_path: String, pub certificate_path: String, pub registry_path: String, pub policy_path: String, pub state_dir: String, pub ledger_dir: String, pub opening_path: String, pub reference_ledger: bool, pub audience: String, pub policy_epoch: i64, pub revocation_epoch: i64, pub generation: i64, pub now_ms: i64, pub receipt_key_id: String, pub receipt_seed_hex: String, pub allowed_authority: String } |  |  |  |  |

| `RailSubmitArgs` | struct | RailSubmitArgs { pub base: ExecuteArgs, pub reference_rail: bool, pub rail_dir: String, pub rail_mode: String, pub sanctions_path: Option<String>, pub counterparties_path: Option<String>, pub journal_dir: Option<String>, pub debtor_agent_bic: String, pub creditor_agent_bic: String, pub created_at_iso: String, pub debtor_name: Option<String>, pub creditor_name: Option<String>, pub effect_expires_at_ms: Option<String> } |  |  |  |  |


### src/blake3.rs

| `BLAKE3_EMPTY_HEX` | const | BLAKE3_EMPTY_HEX: &str |  |  |  |  |

| `assert_blake3_self_test` | function | assert_blake3_self_test() -> Result<(), String> |  |  |  |  |

| `blake3_hex_utf8` | function | blake3_hex_utf8(input: &str) -> String |  |  |  |  |

| `af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262` | str_key | BLAKE3_EMPTY_HEX = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262" |  |  |  |  |


### src/board.rs

| `SIGNATURE_ALGORITHM` | const | SIGNATURE_ALGORITHM: &str |  |  |  |  |

| `FailureMode` | enum | FailureMode { FailClosed, SafeDegrade, LocalCapability, Defer } |  |  |  |  |

| `MaterialityDimension` | enum | MaterialityDimension { Financial, Operational, Customer, Legal, Regulatory, Reputational, Systemic } |  |  |  |  |

| `admit_evidence` | function | admit_evidence(bundle: &EvidenceBundle, trust: &TrustStore, store: &HashMap<String, ReceiptV2>) -> EvidenceAdmission |  |  |  |  |

| `admit_failure_semantics` | function | admit_failure_semantics(input: &FailureSemanticsInput) -> FailureSemanticsDecision |  |  |  |  |

| `assess_materiality` | function | assess_materiality(event: &MaterialityEvent, policy: &MaterialityPolicy) -> Result<MaterialityAssessment, String> |  |  |  |  |

| `board_requirements` | function | board_requirements() -> &'static [Fortune5Requirement] |  |  |  |  |

| `build_board_package` | function | build_board_package(admission: &BoardAdmission, generated_at: &str, material_refused_subjects: u32, risk_appetite_breaches: u32) -> Result<BoardPackage, String> |  |  |  |  |

| `classify_icfr_subject` | function | classify_icfr_subject(input: &IcfrSubject) -> IcfrClassification |  |  |  |  |

| `decode` | function | decode(input: &str) -> Result<Vec<u8>, String> |  |  |  |  |

| `detect_segregation_of_duty_violations` | function | detect_segregation_of_duty_violations(assignments: &[RoleAssignment], incompatible_role_pairs: &[(String, String)]) -> Vec<SodViolation> |  |  |  |  |

| `encode` | function | encode(input: &[u8]) -> String |  |  |  |  |

| `issue_evidence_receipt` | function | issue_evidence_receipt(input: &EvidenceInput, context: &ReceiptIssueContext, signing_key: &SigningKey) -> Result<EvidenceBundle, String> |  |  |  |  |

| `qualify_fortune5_board` | function | qualify_fortune5_board(input: BoardAdmissionInput) -> BoardAdmission |  |  |  |  |

| `qualify_verified_fortune5` | function | qualify_verified_fortune5( bundles: &[EvidenceBundle], context: &QualificationContext, trust: &TrustStore, receipt_store: &HashMap<String, ReceiptV2>, requirements: &[Fortune5Requirement], ) -> VerifiedQualification |  |  |  |  |

| `verify_receipt_dag` | function | verify_receipt_dag(root: &ReceiptV2, trust: &TrustStore, store: &HashMap<String, ReceiptV2>) -> ReceiptVerification |  |  |  |  |

| `ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/` | str_key | ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/" |  |  |  |  |

| `Ed25519` | str_key | SIGNATURE_ALGORITHM = "Ed25519" |  |  |  |  |

| `alpha` | str_key | KEYS = "alpha" |  |  |  |  |

| `BoardAdmission` | struct | BoardAdmission { pub standing: Standing, pub enterprise: VerifiedQualification, pub castle: VerifiedQualification, pub reasons: Vec<String> } |  |  |  |  |

| `BoardAdmissionInput` | struct | BoardAdmissionInput { pub enterprise_context: QualificationContext, pub enterprise_evidence: Vec<EvidenceBundle>, pub castle_context: QualificationContext, pub castle_evidence: Vec<EvidenceBundle>, pub trust: &'a TrustStore, pub receipt_store: HashMap<String, ReceiptV2>, pub castle_assurance_domain: String, pub independent_assurance_domain: String } |  |  |  |  |

| `BoardPackage` | struct | BoardPackage { pub profile: &'static str, pub enterprise_subject: String, pub castle_subject: String, pub generated_at: String, pub enterprise_standing: Standing, pub castle_standing: Standing, pub material_refused_subjects: u32, pub risk_appetite_breaches: u32, pub control_count: usize, pub evidence_digest: String } |  |  |  |  |

| `EvidenceAdmission` | struct | EvidenceAdmission { pub standing: Standing, pub reasons: Vec<String>, pub observation: Option<MetricObservation> } |  |  |  |  |

| `EvidenceBundle` | struct | EvidenceBundle { pub observation: MetricObservation, pub receipt: ReceiptV2 } |  |  |  |  |

| `EvidenceInput` | struct | EvidenceInput { pub metric: String, pub value: crate::fortune5::MetricValue, pub subject: String, pub observed_at: String, pub epistemic_class: crate::fortune5::EvidenceEpistemicClass } |  |  |  |  |

| `FailureSemanticsDecision` | struct | FailureSemanticsDecision { pub standing: Standing, pub may_actuate: bool, pub reason: String } |  |  |  |  |

| `FailureSemanticsInput` | struct | FailureSemanticsInput { pub mode: FailureMode, pub castle_available: bool, pub local_capability_verified: bool, pub receipt_channel_available: bool } |  |  |  |  |

| `IcfrClassification` | struct | IcfrClassification { pub subject: String, pub in_scope: bool, pub reasons: Vec<String> } |  |  |  |  |

| `IcfrSubject` | struct | IcfrSubject { pub subject: String, pub processes: Vec<String>, pub material_accounts: Vec<String>, pub affects_financial_reporting: bool } |  |  |  |  |

| `MaterialityAssessment` | struct | MaterialityAssessment { pub material: bool, pub score_bps: i64, pub triggering_dimensions: Vec<MaterialityDimension>, pub escalate_by_epoch_ms: Option<i64>, pub policy_digest: String, pub authority_digest: String } |  |  |  |  |

| `MaterialityEvent` | struct | MaterialityEvent { pub id: String, pub subject: String, pub occurred_at_epoch_ms: i64, pub impact_bps: BTreeMap<MaterialityDimension, i64> } |  |  |  |  |

| `MaterialityPolicy` | struct | MaterialityPolicy { pub policy_digest: String, pub authority_digest: String, pub per_dimension_threshold_bps: BTreeMap<MaterialityDimension, i64>, pub aggregate_threshold_bps: i64, pub escalation_within_ms: i64 } |  |  |  |  |

| `ReceiptCoreV2` | struct | ReceiptCoreV2 { pub version: &'static str, pub algorithm: &'static str, pub signature_algorithm: &'static str, pub payload_digest: String, pub subject: String, pub metric: String, pub policy_digest: String, pub authority_digest: String, pub parent_digests: Vec<String>, pub key_id: String, pub trust_epoch: i64, pub issued_at: String, pub assurance_domain: String } |  |  |  |  |

| `ReceiptIssueContext` | struct | ReceiptIssueContext { pub policy_digest: String, pub authority_digest: String, pub parent_digests: Vec<String>, pub key_id: String, pub trust_epoch: i64, pub assurance_domain: String } |  |  |  |  |

| `ReceiptV2` | struct | ReceiptV2 { pub core: ReceiptCoreV2, pub receipt_digest: String, pub signature: String } |  |  |  |  |

| `ReceiptVerification` | struct | ReceiptVerification { pub standing: Standing, pub reasons: Vec<String>, pub verified_digests: Vec<String> } |  |  |  |  |

| `RoleAssignment` | struct | RoleAssignment { pub principal: String, pub role: String } |  |  |  |  |

| `SodViolation` | struct | SodViolation { pub principal: String, pub roles: (String, String) } |  |  |  |  |

| `TrustKey` | struct | TrustKey { pub key_id: String, pub verifying_key: VerifyingKey, pub valid_from_epoch: i64, pub revoked_at_epoch: Option<i64>, pub assurance_domain: String } |  |  |  |  |

| `TrustStore` | struct | TrustStore { pub current_epoch: i64, pub keys: HashMap<String, TrustKey> } |  |  |  |  |

| `VerifiedQualification` | struct | VerifiedQualification { pub standing: Standing, pub qualification: Fortune5Qualification, pub evidence_refusals: Vec<String> } |  |  |  |  |


### src/capability_intake.rs

| `AUTHORITY_CEILING` | const | AUTHORITY_CEILING: &str |  |  |  |  |

| `DONORS` | const | DONORS: &[CapabilityDonor] |  |  |  |  |

| `OWNER_CAPABILITY` | const | OWNER_CAPABILITY: &str |  |  |  |  |

| `PROJECTION_SOURCE` | const | PROJECTION_SOURCE: &str |  |  |  |  |

| `RUNTIME_CORE` | const | RUNTIME_CORE: &str |  |  |  |  |

| `do_authority` | function | do_authority(_repository: &str) -> bool |  |  |  |  |

| `donor` | function | donor(repository: &str) -> Option<&'static CapabilityDonor> |  |  |  |  |

| `CONSEQUENTIAL_ADMISSIBILITY` | str_key | OWNER_CAPABILITY = "CONSEQUENTIAL_ADMISSIBILITY" |  |  |  |  |

| `CONSTRUCT` | str_key | AUTHORITY_CEILING = "CONSTRUCT" |  |  |  |  |

| `seanchatmangpt/ash_surface` | str_key | DONORS = "seanchatmangpt/ash_surface" |  |  |  |  |

| `seanchatmangpt/ggen-ecosystem@50fdfa20c84205a80c6eb94e916cffbedc4b816e` | str_key | PROJECTION_SOURCE = "seanchatmangpt/ggen-ecosystem@50fdfa20c84205a80c6eb94e916cffbedc4b816e" |  |  |  |  |

| `seanchatmangpt/xaas` | str_key | RUNTIME_CORE = "seanchatmangpt/xaas" |  |  |  |  |

| `CapabilityDonor` | struct | CapabilityDonor { pub repository: &'static str, pub sha: &'static str, pub capability: &'static str, pub disposition: &'static str } |  |  |  |  |


### src/castle.rs

| `EpistemicClass` | enum | EpistemicClass { Constructed, Counterfactual, Replayed, Observed, Inferred } |  |  |  |  |

| `GymActStatus` | enum | GymActStatus { Observed, Refused } |  |  |  |  |

| `admit_construct_for_do` | function | admit_construct_for_do( capability: &ConstructCapability, process: &PowlProcess, envelope: &TestEnvelope, blake3: &dyn Blake3Provider, verifier: &dyn ReceiptVerifier, policy: &ConstructTrustPolicy, now: impl Fn() -> i64, ) -> Result<ConstructAdmission, String> |  |  |  |  |

| `apply_zero_day_observation` | function | apply_zero_day_observation(graph: &DependencyGraph, observation: ZeroDayObservation) -> Result<ZeroDayImpact, String> |  |  |  |  |

| `as_str` | function | as_str(&self) -> &'static str |  |  |  |  |

| `compile_adversarial_classes` | function | compile_adversarial_classes(goals: &[AdversarialGoal], rules: &[TransitionRule], planners: &[Box<dyn Planner>]) -> Vec<CompiledAdversarialClass> |  |  |  |  |

| `compile_witness_to_powl` | function | compile_witness_to_powl(id: &str, vulnerability: &VulnerabilityCondition, rules: &[TransitionRule]) -> PowlProcess |  |  |  |  |

| `construct_compromise` | function | construct_compromise(&self, dependency_id: &str, capability: &str) -> Result<ConstructedCompromise, String> |  |  |  |  |

| `create_receipt` | function | create_receipt( artifact: &Value, epistemic_class: EpistemicClass, subject: &str, parent_digests: &[String], blake3: &dyn Blake3Provider, signer: &dyn ReceiptSigner, ) -> Result<Receipt, String> |  |  |  |  |

| `derive_vulnerabilities` | function | derive_vulnerabilities(goal: &AdversarialGoal, rules: &[TransitionRule], max_depth: u32) -> Vec<VulnerabilityCondition> |  |  |  |  |

| `enabled_activities` | function | enabled_activities(process: &PowlProcess, completed: &BTreeSet<String>) -> Vec<PowlActivity> |  |  |  |  |

| `execute_powl_with_gym_act` | function | execute_powl_with_gym_act( process: &PowlProcess, state: &WorldState, envelope: &TestEnvelope, gymact: &dyn GymActAdapter, authorization: DoAuthorizationContext<'_>, ) -> Result<ReceiptedOcelLog, String> |  |  |  |  |

| `impacted_closure` | function | impacted_closure(&self, changed_ids: I) -> Vec<String> |  |  |  |  |

| `local_docker_default` | function | local_docker_default(docker_bin: impl Into<String>) -> Self |  |  |  |  |

| `manufacture_construct_capability` | function | manufacture_construct_capability(request: ConstructRequest, blake3: &dyn Blake3Provider, signer: &dyn ReceiptSigner) -> Result<ConstructCapability, String> |  |  |  |  |

| `match_compiled_classes` | function | match_compiled_classes(classes: &'a [CompiledAdversarialClass], facts: &BTreeSet<Predicate>) -> Vec<&'a CompiledAdversarialClass> |  |  |  |  |

| `new` | function | new(nodes: Vec<DependencyNode>, edges: Vec<DependencyEdge>) -> Result<Self, String> |  |  |  |  |

| `platform_eng_colima_default` | function | platform_eng_colima_default() -> Self |  |  |  |  |

| `run_planner_ensemble` | function | run_planner_ensemble(problem: &PlanningProblem<'_>, planners: &[Box<dyn Planner>]) -> Vec<PlanCandidate> |  |  |  |  |

| `verify_receipt` | function | verify_receipt(artifact: &Value, receipt: &Receipt, blake3: &dyn Blake3Provider, verifier: &dyn ReceiptVerifier, trusted_origin_key_ids: &BTreeSet<String>) -> bool |  |  |  |  |

| `alpha` | str_key | KEYS = "alpha" |  |  |  |  |

| `ActuationPermit` | struct | ActuationPermit { pub construct_digest: String, pub process_digest: String, pub subject: String, pub authority: String, pub transition_id: String, pub expires_at_epoch_ms: i64 } |  |  |  |  |

| `AdmissionBrand` | struct |  |  |  |  |  |

| `AdversarialGoal` | struct | AdversarialGoal { pub id: String, pub predicate: Predicate, pub consequence: i64 } |  |  |  |  |

| `AutofdeLabPlanner` | struct | AutofdeLabPlanner { pub id: String, pub script_path: std::path::PathBuf, pub python_bin: String } |  |  |  |  |

| `CompiledAdversarialClass` | struct | CompiledAdversarialClass { pub key: String, pub goal: AdversarialGoal, pub vulnerability: VulnerabilityCondition, pub process: PowlProcess } |  |  |  |  |

| `ConstructAdmission` | struct | ConstructAdmission { pub standing: &'static str, pub construct_digest: String, pub process_digest: String, pub o_star_digest: String, pub config_graph_digest: String, pub ontology_digest: String, pub replay_identity_digest: String, pub subject: String, pub authority: String, pub allowed_transition_ids: Vec<String>, pub max_steps: u32, pub expires_at_epoch_ms: i64, _brand: sealed::AdmissionBrand } |  |  |  |  |

| `ConstructArtifact` | struct | ConstructArtifact { pub kind: &'static str, pub algorithm: &'static str, pub subject: String, pub authority: String, pub o_star_digest: String, pub config_graph_digest: String, pub ontology_digest: String, pub process_digest: String, pub replay_identity_digest: String, pub allowed_transition_ids: Vec<String>, pub max_steps: u32, pub expires_at_epoch_ms: i64 } |  |  |  |  |

| `ConstructCapability` | struct | ConstructCapability { pub sources: ConstructSources, pub artifact: ConstructArtifact, pub source_receipts: ConstructSourceReceipts, pub receipt: Receipt } |  |  |  |  |

| `ConstructRequest` | struct | ConstructRequest { pub subject: String, pub authority: String, pub o_star: Value, pub config_graph: Value, pub ontology: Value, pub process: PowlProcess, pub envelope: TestEnvelope } |  |  |  |  |

| `ConstructSourceReceipts` | struct | ConstructSourceReceipts { pub o_star: Receipt, pub config_graph: Receipt, pub ontology: Receipt, pub process: Receipt } |  |  |  |  |

| `ConstructSources` | struct | ConstructSources { pub o_star: Value, pub config_graph: Value, pub ontology: Value } |  |  |  |  |

| `ConstructTrustPolicy` | struct | ConstructTrustPolicy { pub trusted_origin_key_ids: BTreeSet<String>, pub allowed_authorities: BTreeSet<String> } |  |  |  |  |

| `ConstructedCompromise` | struct | ConstructedCompromise { pub dependency_id: String, pub capability: String, pub facts: Vec<Predicate>, pub impacted: Vec<String>, pub epistemic_class: &'static str } |  |  |  |  |

| `ContainerGymActAdapter` | struct | ContainerGymActAdapter { pub docker_bin: String, pub now_ms: fn() -> i64, pub allowed_images: BTreeMap<String, String> } |  |  |  |  |

| `CostMinimizingPlanner` | struct | CostMinimizingPlanner { pub id: String } |  |  |  |  |

| `DependencyEdge` | struct | DependencyEdge { pub from: String, pub to: String, pub relation: String } |  |  |  |  |

| `DependencyGraph` | struct | DependencyGraph { pub nodes: BTreeMap<String, DependencyNode>, pub edges: Vec<DependencyEdge>, dependents: BTreeMap<String, BTreeSet<String>> } |  |  |  |  |

| `DependencyNode` | struct | DependencyNode { pub id: String, pub kind: String } |  |  |  |  |

| `DoAuthorizationContext` | struct | DoAuthorizationContext { pub admission: &'a ConstructAdmission, pub blake3: &'a dyn Blake3Provider, pub receipt_signer: &'a dyn ReceiptSigner, pub now: Box<dyn Fn() -> i64 + 'a> } |  |  |  |  |

| `GymActResult` | struct | GymActResult { pub transition_id: String, pub status: GymActStatus, pub objects: Vec<OcelObject>, pub attributes: BTreeMap<String, Value> } |  |  |  |  |

| `KindClusterReadOnlyGymAct` | struct | KindClusterReadOnlyGymAct { pub kube_context: String, pub allowed_read_only_queries: BTreeMap<String, Vec<String>> } |  |  |  |  |

| `OcelEvent` | struct | OcelEvent { pub id: String, pub kind: String, pub time: String, pub attributes: BTreeMap<String, Value>, pub object_ids: Vec<String> } |  |  |  |  |

| `OcelLog` | struct | OcelLog { pub version: &'static str, pub objects: Vec<OcelObject>, pub events: Vec<OcelEvent> } |  |  |  |  |

| `OcelObject` | struct | OcelObject { pub id: String, pub kind: String } |  |  |  |  |

| `PlanCandidate` | struct | PlanCandidate { pub planner_id: String, pub process: PowlProcess, pub score: i64 } |  |  |  |  |

| `PlanningProblem` | struct | PlanningProblem { pub goal: &'a AdversarialGoal, pub vulnerability: &'a VulnerabilityCondition, pub rules: &'a [TransitionRule] } |  |  |  |  |

| `PowlActivity` | struct | PowlActivity { pub id: String, pub transition_id: String, pub predecessors: Vec<String> } |  |  |  |  |

| `PowlProcess` | struct | PowlProcess { pub id: String, pub goal_id: String, pub activities: Vec<PowlActivity> } |  |  |  |  |

| `ProcessGymActAdapter` | struct | ProcessGymActAdapter { pub gymact_bin: String, pub kube_context: String, pub allowed_verifications: BTreeMap<String, (String, Value)> } |  |  |  |  |

| `Receipt` | struct | Receipt { pub algorithm: &'static str, pub artifact_digest: String, pub receipt_digest: String, pub epistemic_class: EpistemicClass, pub subject: String, pub parent_digests: Vec<String>, pub origin_key_id: String, pub origin_signature: String } |  |  |  |  |

| `ReceiptedOcelLog` | struct | ReceiptedOcelLog { pub log: OcelLog, pub construct_digest: String, pub receipt: Receipt } |  |  |  |  |

| `TestEnvelope` | struct | TestEnvelope { pub system_id: String, pub allowed_transition_ids: BTreeSet<String>, pub max_steps: u32, pub expires_at_epoch_ms: i64 } |  |  |  |  |

| `TransitionRule` | struct | TransitionRule { pub id: String, pub preconditions: Vec<Predicate>, pub effects: Vec<Predicate>, pub cost: Option<f64>, pub planner_hint: Option<String> } |  |  |  |  |

| `VulnerabilityCondition` | struct | VulnerabilityCondition { pub goal_id: String, pub predicates: Vec<Predicate>, pub witness_transitions: Vec<String> } |  |  |  |  |

| `WitnessPlanner` | struct | WitnessPlanner { pub id: String } |  |  |  |  |

| `WorldState` | struct | WorldState { pub system_id: String, pub facts: BTreeSet<Predicate> } |  |  |  |  |

| `ZeroDayImpact` | struct | ZeroDayImpact { pub observation: ZeroDayObservation, pub impacted_dependencies: Vec<String>, pub newly_admitted_fact: Predicate } |  |  |  |  |

| `ZeroDayObservation` | struct | ZeroDayObservation { pub dependency_id: String, pub capability: String } |  |  |  |  |

| `Blake3Provider` | trait |  |  |  |  |  |

| `GymActAdapter` | trait |  |  |  |  |  |

| `Planner` | trait |  |  |  |  |  |

| `ReceiptSigner` | trait |  |  |  |  |  |

| `ReceiptVerifier` | trait |  |  |  |  |  |


### src/dd_ui.rs

| `DdUiRefusal` | enum | DdUiRefusal { IrreversiblePresentationSelection, RenderAuthorityEscalation, DirectDoFromUi, UnadmittedConstruct, NonBrceDo, UnreplayablePresentation } |  |  |  |  |

| `admit` | function | admit(&self) -> Result<(), DdUiRefusal> |  |  |  |  |

| `PresentationAuthority` | struct | PresentationAuthority { pub irreversible_selections: u32, pub render_actuation_authority: bool, pub output_kind: &'a str, pub construct_admitted: bool, pub do_route: &'a str, pub grammar_digest: Option<&'a str>, pub world_digest: Option<&'a str>, pub frontier_digest: Option<&'a str>, pub screen_digest: Option<&'a str> } |  |  |  |  |


### src/fortune5.rs

| `EvidenceEpistemicClass` | enum | EvidenceEpistemicClass { Observed, Replayed, Inferred } |  |  |  |  |

| `MetricValue` | enum | MetricValue { Number(f64), Bool(bool), Str(String) } |  |  |  |  |

| `Standing` | enum | Standing { Alive, Refused, Unknown } |  |  |  |  |

| `admit_replay` | function | admit_replay(manifest: &ReplayManifest, subject: &ReplaySubject) -> ReplayAdmission |  |  |  |  |

| `as_str` | function | as_str(&self) -> &'static str |  |  |  |  |

| `minimum_impact_coverage` | function | minimum_impact_coverage(classes: &[AdversarialImpactClass], target_coverage_bps: i64) -> Result<ImpactCoverageSelection, String> |  |  |  |  |

| `qualify_fortune5` | function | qualify_fortune5( observations: &[MetricObservation], context: &QualificationContext, requirements: &[Fortune5Requirement], ) -> Fortune5Qualification |  |  |  |  |

| `qualify_fortune5_default` | function | qualify_fortune5_default(observations: &[MetricObservation], context: &QualificationContext) -> Fortune5Qualification |  |  |  |  |

| `AdversarialImpactClass` | struct | AdversarialImpactClass { pub key: String, pub impact: f64 } |  |  |  |  |

| `ControlEvaluation` | struct | ControlEvaluation { pub control_id: String, pub category: String, pub metric: String, pub standing: Standing, pub observed: Option<MetricValue>, pub expected: String, pub receipt_digest: Option<String>, pub reason: String } |  |  |  |  |

| `Fortune5Qualification` | struct | Fortune5Qualification { pub standing: Standing, pub subject: String, pub profile: &'static str, pub controls: Vec<ControlEvaluation>, pub alive: usize, pub refused: usize, pub unknown: usize, pub categories: BTreeMap<String, Standing> } |  |  |  |  |

| `ImpactCoverageSelection` | struct | ImpactCoverageSelection { pub selected: Vec<AdversarialImpactClass>, pub coverage_bps: i64, pub total_impact: f64, pub selected_impact: f64 } |  |  |  |  |

| `MetricObservation` | struct | MetricObservation { pub metric: String, pub value: MetricValue, pub receipt_digest: String, pub subject: String, pub observed_at: String, pub epistemic_class: EvidenceEpistemicClass } |  |  |  |  |

| `QualificationContext` | struct | QualificationContext { pub subject: String, pub now_epoch_ms: Option<i64>, pub max_evidence_age_ms: Option<i64> } |  |  |  |  |

| `ReplayAdmission` | struct | ReplayAdmission { pub standing: Standing, pub replay_class_id: String, pub reasons: Vec<String> } |  |  |  |  |

| `ReplayManifest` | struct | ReplayManifest { pub replay_class_id: String, pub structural_signature: String, pub ontology_version: String, pub provider_semantics_version: String, pub invariant_set_digest: String, pub process_digest: String } |  |  |  |  |

| `ReplaySubject` | struct | ReplaySubject { pub structural_signature: String, pub ontology_version: String, pub provider_semantics_version: String, pub invariant_set_digest: String, pub invariants_hold: bool } |  |  |  |  |


### src/fortune5_generated.rs

| `FORTUNE5_REQUIREMENTS` | const | FORTUNE5_REQUIREMENTS: &[Fortune5Requirement] |  |  |  |  |

| `F5-AUTH-001` | str_key | FORTUNE5_REQUIREMENTS = "F5-AUTH-001" |  |  |  |  |

| `Fortune5Requirement` | struct | Fortune5Requirement { pub order: u32, pub control_id: &'static str, pub category: &'static str, pub description: &'static str, pub metric: &'static str, pub comparator: &'static str, pub target: &'static str, pub authority: &'static str } |  |  |  |  |


### src/generated.rs

| `GENERATED_BINDINGS` | const | GENERATED_BINDINGS: &[GeneratedBinding] |  |  |  |  |

| `default_adversarial_goals` | function | default_adversarial_goals() -> Vec<DefaultAdversarialGoal> |  |  |  |  |

| `generated_components` | function | generated_components() -> impl Iterator<Item = &'static GeneratedBinding> |  |  |  |  |

| `component` | str_key | GENERATED_BINDINGS = "component" |  |  |  |  |

| `DefaultAdversarialGoal` | struct | DefaultAdversarialGoal { pub id: &'static str, pub predicate: &'static str, pub consequence: i64 } |  |  |  |  |

| `GeneratedBinding` | struct | GeneratedBinding { pub kind: &'static str, pub order: u32, pub identifier: &'static str, pub slug: &'static str, pub role: &'static str, pub authority: &'static str, pub predicate: &'static str, pub consequence: i64 } |  |  |  |  |


### src/gymact_container.rs

| `DOCKER_BIN` | env_key | std::env::var("DOCKER_BIN") |  |  |  |  |

| `execute_default_container_observation` | function | execute_default_container_observation( process: &PowlProcess, state: &WorldState, envelope: &TestEnvelope, authorization: DoAuthorizationContext<'_>, docker_bin: impl Into<String>, ) -> Result<ReceiptedOcelLog, String> |  |  |  |  |

| `alpine:3.20` | str_key | DEFAULT_IMAGE = "alpine:3.20" |  |  |  |  |

| `observe-alpine-container` | str_key | DEFAULT_TRANSITION = "observe-alpine-container" |  |  |  |  |


### src/lib.rs

| `castle::*` | use | castle::* |  |  |  |  |

| `dd_ui::{DdUiRefusal, PresentationAuthority}` | use | dd_ui::{DdUiRefusal, PresentationAuthority} |  |  |  |  |

| `generated::{ default_adversarial_goals, generated_components, DefaultAdversarialGoal, GeneratedBinding, GENERATED_BINDINGS, }` | use | generated::{ default_adversarial_goals, generated_components, DefaultAdversarialGoal, GeneratedBinding, GENERATED_BINDINGS, } |  |  |  |  |

| `gymact_container::execute_default_container_observation` | use | gymact_container::execute_default_container_observation |  |  |  |  |

| `planner_minimal::MinimalActionPlanner` | use | planner_minimal::MinimalActionPlanner |  |  |  |  |

| `reconstitution::{ admit_empire_reconstitution_for_construct, EmpireReconstitutionAdmission, FinalDisposition, ReconstitutedCapability, ReconstitutionRefusal, }` | use | reconstitution::{ admit_empire_reconstitution_for_construct, EmpireReconstitutionAdmission, FinalDisposition, ReconstitutedCapability, ReconstitutionRefusal, } |  |  |  |  |

| `refusal_conformance::{verify_refusal, RefusalObservation}` | use | refusal_conformance::{verify_refusal, RefusalObservation} |  |  |  |  |


### src/operation_envelope.rs

| `EffectClass` | enum | EffectClass { Observe, Select, Construct, Do } |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `digest` | function | digest(&self) -> String |  |  |  |  |

| `to_json` | function | to_json(&self) -> Value |  |  |  |  |

| `OperationEnvelope` | struct | OperationEnvelope { pub operation_id: String, pub subject: String, pub intent: String, pub capability: String, pub context: Value, pub ontology_refs: Vec<String>, pub input: Value, pub actor: String, pub authority: String, pub effect_class: EffectClass, pub standing_requirements: Vec<String>, pub prior_receipts: Vec<String>, pub correlation: String } |  |  |  |  |


### src/payments/adapter.rs

| `T_POST` | const | T_POST: &str |  |  |  |  |

| `T_RESERVE` | const | T_RESERVE: &str |  |  |  |  |

| `StepOutcome` | enum | StepOutcome { Reserved, Posted(LedgerEntry), Refused { code: &'static str, definite: bool } } |  |  |  |  |

| `outcomes` | function | outcomes(&self) -> Vec<StepOutcome> |  |  |  |  |

| `payments.post` | str_key | T_POST = "payments.post" |  |  |  |  |

| `payments.reserve` | str_key | T_RESERVE = "payments.reserve" |  |  |  |  |

| `PaymentGymActAdapter` | struct | PaymentGymActAdapter { ledger: &'a dyn LedgerPort, admission: &'a PaymentAdmission, subject: String, token: ActuationToken, outcomes: Mutex<Vec<StepOutcome>> } |  |  |  |  |


### src/payments/admission.rs

| `PAYMENT_SCREENING_REQUIRED` | const | PAYMENT_SCREENING_REQUIRED: &str |  |  |  |  |

| `admit_payment` | function | admit_payment( prepared: PreparedEffect, certificate: &ActuationCertificate, ctx: &AdmissionContext<'_>, ) -> PayResult<PaymentAdmission> |  |  |  |  |

| `admit_payment_screened` | function | admit_payment_screened( prepared: PreparedEffect, certificate: &ActuationCertificate, ctx: &AdmissionContext<'_>, screening: &Screening<'_>, ) -> PayResult<(PaymentAdmission, ScreeningEvidence)> |  |  |  |  |

| `effect` | function | effect(&self) -> &PaymentEffect |  |  |  |  |

| `generation` | function | generation(&self) -> u64 |  |  |  |  |

| `nonce` | function | nonce(&self) -> &str |  |  |  |  |

| `screening` | function | screening(&self) -> Option<&ScreeningEvidence> |  |  |  |  |

| `verification` | function | verification(&self) -> &VerificationReceipt |  |  |  |  |

| `REFUSED:PAYMENT_SCREENING_REQUIRED` | str_key | PAYMENT_SCREENING_REQUIRED = "REFUSED:PAYMENT_SCREENING_REQUIRED" |  |  |  |  |

| `AdmissionContext` | struct | AdmissionContext { pub registry: &'a KeyRegistry, pub epochs: SecurityEpochs, pub audience: &'a str, pub now_ms: u64, pub policy: &'a SpendPolicy, pub claims: &'a ClaimStore, pub nonces: &'a DurableNonceFence } |  |  |  |  |

| `PaymentAdmission` | struct | PaymentAdmission { effect: PaymentEffect, receipt: VerificationReceipt, generation: u64, nonce: String, screening: Option<ScreeningEvidence>, _seal: () } |  |  |  |  |

| `Screening` | struct | Screening { pub controls: &'a [&'a dyn ComplianceControl], pub counterparties: &'a CounterpartyRegistry } |  |  |  |  |

| `ScreeningEvidence` | struct | ScreeningEvidence { pub compliance_bundle_digest: String, pub counterparty_evidence_digest: String } |  |  |  |  |


### src/payments/claim_store.rs

| `ClaimState` | enum | ClaimState { Reserved, Executed, Refused, UnknownOutcome, Submitted, Final, Returned } |  |  |  |  |

| `get` | function | get(&self, digest: &str) -> PayResult<Option<Claim>> |  |  |  |  |

| `list` | function | list(&self) -> PayResult<Vec<Claim>> |  |  |  |  |

| `open` | function | open(root: impl Into<PathBuf>) -> PayResult<Self> |  |  |  |  |

| `reserve` | function | reserve(&self, claim: &Claim, epoch_cap: Option<u64>) -> PayResult<()> |  |  |  |  |

| `reversed_total` | function | reversed_total(&self, original: &str) -> PayResult<u64> |  |  |  |  |

| `transition` | function | transition( &self, digest: &str, from: &[ClaimState], to: ClaimState, construct_digest: Option<&str>, detail: &str, ) -> PayResult<Claim> |  |  |  |  |

| `Claim` | struct | Claim { pub effect_digest: String, pub principal: String, pub payer: String, pub payee: String, pub amount_minor: u64, pub currency: Currency, pub generation: u64, pub state: ClaimState, pub reverses: Option<String>, pub construct_digest: Option<String>, pub detail: String, pub obligation_id: String, pub purpose: String, pub audience: String, pub verified_custodian_ids: Vec<String> } |  |  |  |  |

| `ClaimStore` | struct | ClaimStore { root: PathBuf, lock: Mutex<()> } |  |  |  |  |


### src/payments/compliance.rs

| `COMPLIANCE_CONTROL_DUPLICATE` | const | COMPLIANCE_CONTROL_DUPLICATE: &str |  |  |  |  |

| `NO_COMPLIANCE_CONTROLS` | const | NO_COMPLIANCE_CONTROLS: &str |  |  |  |  |

| `SANCTIONS_HIT` | const | SANCTIONS_HIT: &str |  |  |  |  |

| `SANCTIONS_LIST_INVALID` | const | SANCTIONS_LIST_INVALID: &str |  |  |  |  |

| `from_json` | function | from_json(bytes: &[u8]) -> PayResult<Self> |  |  |  |  |

| `normalize` | function | normalize(text: &str) -> String |  |  |  |  |

| `run_controls` | function | run_controls(controls: &[&dyn ComplianceControl], effect: &PaymentEffect) -> PayResult<ComplianceBundle> |  |  |  |  |

| `with_aliases` | function | with_aliases(mut self, aliases: &[(&str, &str)]) -> Self |  |  |  |  |

| `CASTLE-COMPLIANCE-BUNDLE-V1` | str_key | BUNDLE_DOMAIN = "CASTLE-COMPLIANCE-BUNDLE-V1" |  |  |  |  |

| `CASTLE-CONTROL-EVIDENCE-V1` | str_key | EVIDENCE_DOMAIN = "CASTLE-CONTROL-EVIDENCE-V1" |  |  |  |  |

| `CASTLE-SANCTIONS-LIST-V1` | str_key | LIST_DOMAIN = "CASTLE-SANCTIONS-LIST-V1" |  |  |  |  |

| `REFUSED:COMPLIANCE_CONTROL_DUPLICATE` | str_key | COMPLIANCE_CONTROL_DUPLICATE = "REFUSED:COMPLIANCE_CONTROL_DUPLICATE" |  |  |  |  |

| `REFUSED:NO_COMPLIANCE_CONTROLS` | str_key | NO_COMPLIANCE_CONTROLS = "REFUSED:NO_COMPLIANCE_CONTROLS" |  |  |  |  |

| `REFUSED:PAYMENT_SANCTIONS_HIT` | str_key | SANCTIONS_HIT = "REFUSED:PAYMENT_SANCTIONS_HIT" |  |  |  |  |

| `REFUSED:SANCTIONS_LIST_INVALID` | str_key | SANCTIONS_LIST_INVALID = "REFUSED:SANCTIONS_LIST_INVALID" |  |  |  |  |

| `ComplianceBundle` | struct | ComplianceBundle { pub evidence: Vec<ControlEvidence>, pub bundle_digest: String } |  |  |  |  |

| `ControlEvidence` | struct | ControlEvidence { pub control_id: String, pub version: String, pub outcome: String, pub evidence_digest: String } |  |  |  |  |

| `SanctionsList` | struct | SanctionsList { pub list_id: String, pub version: String, pub entries: BTreeSet<String>, pub digest: String, aliases: BTreeMap<String, BTreeSet<String>> } |  |  |  |  |

| `ComplianceControl` | trait |  |  |  |  |  |


### src/payments/counterparty.rs

| `ACCOUNT_AMBIGUOUS` | const | ACCOUNT_AMBIGUOUS: &str |  |  |  |  |

| `COUNTERPARTY_UNVERIFIED` | const | COUNTERPARTY_UNVERIFIED: &str |  |  |  |  |

| `LEI_INVALID` | const | LEI_INVALID: &str |  |  |  |  |

| `REGISTRY_INVALID` | const | REGISTRY_INVALID: &str |  |  |  |  |

| `as_str` | function | as_str(&self) -> &str |  |  |  |  |

| `evidence_digest` | function | evidence_digest(&self, payer: &Counterparty, payee: &Counterparty) -> String |  |  |  |  |

| `fibo_party_json` | function | fibo_party_json(cp: &Counterparty) -> Value |  |  |  |  |

| `from_json` | function | from_json(bytes: &[u8]) -> PayResult<Self> |  |  |  |  |

| `parse` | function | parse(text: &str) -> PayResult<Self> |  |  |  |  |

| `resolve` | function | resolve(&self, account: &str) -> PayResult<&Counterparty> |  |  |  |  |

| `with_check_digits` | function | with_check_digits(prefix18: &str) -> PayResult<Self> |  |  |  |  |

| `with_require_lei` | function | with_require_lei(mut self, require: bool) -> Self |  |  |  |  |

| `CASTLE-COUNTERPARTY-EVIDENCE-V1` | str_key | EVIDENCE_DOMAIN = "CASTLE-COUNTERPARTY-EVIDENCE-V1" |  |  |  |  |

| `REFUSED:COUNTERPARTY_ACCOUNT_AMBIGUOUS` | str_key | ACCOUNT_AMBIGUOUS = "REFUSED:COUNTERPARTY_ACCOUNT_AMBIGUOUS" |  |  |  |  |

| `REFUSED:COUNTERPARTY_LEI_INVALID` | str_key | LEI_INVALID = "REFUSED:COUNTERPARTY_LEI_INVALID" |  |  |  |  |

| `REFUSED:COUNTERPARTY_REGISTRY_INVALID` | str_key | REGISTRY_INVALID = "REFUSED:COUNTERPARTY_REGISTRY_INVALID" |  |  |  |  |

| `REFUSED:COUNTERPARTY_UNVERIFIED` | str_key | COUNTERPARTY_UNVERIFIED = "REFUSED:COUNTERPARTY_UNVERIFIED" |  |  |  |  |

| `Counterparty` | struct | Counterparty { pub party_id: String, pub legal_name: String, pub lei: Option<Lei>, pub accounts: BTreeSet<String> } |  |  |  |  |

| `CounterpartyRegistry` | struct | CounterpartyRegistry { parties: Vec<Counterparty>, by_account: BTreeMap<String, usize>, require_lei: bool } |  |  |  |  |

| `Lei` | struct | Lei { String } |  |  |  |  |


### src/payments/dirlock.rs

| `acquire` | function | acquire(root: &Path) -> PayResult<Self> |  |  |  |  |

| `publish_new` | function | publish_new(root: &Path, final_path: &Path, bytes: &[u8]) -> PayResult<bool> |  |  |  |  |

| `publish_replace` | function | publish_replace(root: &Path, final_path: &Path, bytes: &[u8]) -> PayResult<()> |  |  |  |  |

| `sync_dir` | function | sync_dir(root: &Path) -> PayResult<()> |  |  |  |  |

| `DirLock` | struct | DirLock { path: PathBuf } |  |  |  |  |


### src/payments/effect.rs

| `PAYMENT_CAPABILITY` | const | PAYMENT_CAPABILITY: &str |  |  |  |  |

| `digest` | function | digest(&self) -> &str |  |  |  |  |

| `from_prepared` | function | from_prepared(prepared: PreparedEffect) -> PayResult<Self> |  |  |  |  |

| `invoice_ref` | function | invoice_ref(&self) -> Option<&str> |  |  |  |  |

| `money` | function | money(&self) -> Money |  |  |  |  |

| `obligation_id` | function | obligation_id(&self) -> &str |  |  |  |  |

| `payee` | function | payee(&self) -> &str |  |  |  |  |

| `payer` | function | payer(&self) -> &str |  |  |  |  |

| `prepare` | function | prepare( principal: &str, payer: &str, payee: &str, amount_minor: &str, currency: Currency, obligation_id: &str, purpose: &str, reverses: Option<&str>, ) -> PayResult<PreparedEffect> |  |  |  |  |

| `prepare_with_invoice` | function | prepare_with_invoice( principal: &str, payer: &str, payee: &str, amount_minor: &str, currency: Currency, obligation_id: &str, purpose: &str, invoice_ref: Option<&str>, reverses: Option<&str>, ) -> PayResult<PreparedEffect> |  |  |  |  |

| `prepared` | function | prepared(&self) -> &PreparedEffect |  |  |  |  |

| `principal` | function | principal(&self) -> &str |  |  |  |  |

| `purpose` | function | purpose(&self) -> &str |  |  |  |  |

| `reverses` | function | reverses(&self) -> Option<&str> |  |  |  |  |

| `account_transfer` | str_key | SUBJECT_KIND = "account_transfer" |  |  |  |  |

| `payments.transfer.v1` | str_key | PAYMENT_CAPABILITY = "payments.transfer.v1" |  |  |  |  |

| `PaymentEffect` | struct | PaymentEffect { prepared: PreparedEffect, digest: String, payer: String, payee: String, money: Money, obligation_id: String, purpose: String, reverses: Option<String>, invoice_ref: Option<String> } |  |  |  |  |


### src/payments/event_receipt.rs

| `EVENT_RECEIPT_INCOMPLETE` | const | EVENT_RECEIPT_INCOMPLETE: &str |  |  |  |  |

| `EVENT_RECEIPT_TAMPERED` | const | EVENT_RECEIPT_TAMPERED: &str |  |  |  |  |

| `explain` | function | explain(receipt: &EventReceipt) -> Value |  |  |  |  |

| `ledger_entry_digest` | function | ledger_entry_digest(entry: &LedgerEntry) -> String |  |  |  |  |

| `seal_event_receipt` | function | seal_event_receipt(inputs: &EventInputs) -> PayResult<EventReceipt> |  |  |  |  |

| `verify_event_receipt` | function | verify_event_receipt(receipt: &EventReceipt) -> PayResult<()> |  |  |  |  |

| `CASTLE-EVENT-RECEIPT-V1` | str_key | RECEIPT_DOMAIN = "CASTLE-EVENT-RECEIPT-V1" |  |  |  |  |

| `CASTLE-LEDGER-ENTRY-V1` | str_key | LEDGER_DOMAIN = "CASTLE-LEDGER-ENTRY-V1" |  |  |  |  |

| `REFUSED:EVENT_RECEIPT_INCOMPLETE` | str_key | EVENT_RECEIPT_INCOMPLETE = "REFUSED:EVENT_RECEIPT_INCOMPLETE" |  |  |  |  |

| `REFUSED:EVENT_RECEIPT_TAMPERED` | str_key | EVENT_RECEIPT_TAMPERED = "REFUSED:EVENT_RECEIPT_TAMPERED" |  |  |  |  |

| `EventInputs` | struct | EventInputs { pub effect_id: String, pub obligation_id: String, pub admission_decision_digest: String, pub pee_effect_id: Option<String>, pub construct_digest: Option<String>, pub brce_prepare_digests: Vec<String>, pub brce_outcome_digests: Vec<String>, pub rail_correlation_id: Option<String>, pub rail_payload_digest: Option<String>, pub finality_evidence_digest: Option<String>, pub ledger_entry_digest: Option<String>, pub claim_state: String, pub compliance_bundle_digest: Option<String>, pub counterparty_evidence_digest: Option<String>, pub settled_at_ms: Option<u64> } |  |  |  |  |

| `EventReceipt` | struct | EventReceipt { pub inputs: EventInputs, pub receipt_digest: String } |  |  |  |  |


### src/payments/execute.rs

| `PaymentStanding` | enum | PaymentStanding { Settled, Refused, UnknownOutcome } |  |  |  |  |

| `build_construct` | function | build_construct( admission: &PaymentAdmission, ctx: &ExecutionContext<'_>, ) -> PayResult<(ConstructAdmission, PowlProcess, TestEnvelope)> |  |  |  |  |

| `build_construct_with` | function | build_construct_with( admission: &PaymentAdmission, ctx: &ExecutionContext<'_>, process: PowlProcess, allowed_transitions: BTreeSet<String>, ) -> PayResult<(ConstructAdmission, PowlProcess, TestEnvelope)> |  |  |  |  |

| `execute_payment` | function | execute_payment(admission: PaymentAdmission, ctx: &ExecutionContext<'_>) -> PayResult<PaymentExecution> |  |  |  |  |

| `ExecutionContext` | struct | ExecutionContext { pub blake3: &'a dyn Blake3Provider, pub signer: &'a dyn ReceiptSigner, pub verifier: &'a dyn ReceiptVerifier, pub allowed_authorities: BTreeSet<String>, pub journal_root: PathBuf, pub now_epoch_ms: i64, pub ledger: &'a dyn LedgerPort, pub claims: &'a ClaimStore, pub policy: &'a SpendPolicy } |  |  |  |  |

| `PaymentExecution` | struct | PaymentExecution { pub standing: PaymentStanding, pub effect_digest: String, pub construct_digest: Option<String>, pub ocel_receipt_digest: Option<String>, pub brce_prepare_receipt_digests: Vec<String>, pub brce_outcome_receipt_digests: Vec<String>, pub ledger_entry: Option<LedgerEntry>, pub detail: String } |  |  |  |  |


### src/payments/execute_rail.rs

| `NOT_ABANDONABLE` | const | NOT_ABANDONABLE: &str |  |  |  |  |

| `RAIL_HOLD_UNAVAILABLE` | const | RAIL_HOLD_UNAVAILABLE: &str |  |  |  |  |

| `RAIL_NEVER_SAW_EFFECT` | const | RAIL_NEVER_SAW_EFFECT: &str |  |  |  |  |

| `RAIL_NEVER_SUBMITTED` | const | RAIL_NEVER_SUBMITTED: &str |  |  |  |  |

| `RAIL_TIME_INVALID` | const | RAIL_TIME_INVALID: &str |  |  |  |  |

| `T_HOLD` | const | T_HOLD: &str |  |  |  |  |

| `T_RAIL_SUBMIT` | const | T_RAIL_SUBMIT: &str |  |  |  |  |

| `FinalizeResult` | enum | FinalizeResult { Pending, Applied { outcome: FinalityOutcome, evidence_digest: String }, StillUnknown, ProvenAbsent } |  |  |  |  |

| `RailStandingAfterSubmit` | enum | RailStandingAfterSubmit { Submitted, Refused, UnknownOutcome } |  |  |  |  |

| `abandon_unsubmitted` | function | abandon_unsubmitted( claims: &ClaimStore, ledger: &dyn LedgerPort, rail: &dyn RailActuator, effect_digest: &str, ) -> PayResult<FinalizeResult> |  |  |  |  |

| `finalize_via_rail` | function | finalize_via_rail( effect_digest: &str, claims: &ClaimStore, ledger: &dyn LedgerPort, rail: &dyn RailActuator, ) -> PayResult<FinalizeResult> |  |  |  |  |

| `submit_via_rail` | function | submit_via_rail( admission: PaymentAdmission, params: &RailExecutionParams, ctx: &ExecutionContext<'_>, rail: &dyn RailActuator, ) -> PayResult<RailSubmission> |  |  |  |  |

| `CASTLE-RAIL-LOCAL-EVIDENCE-V1` | str_key | LOCAL_EVIDENCE_DOMAIN = "CASTLE-RAIL-LOCAL-EVIDENCE-V1" |  |  |  |  |

| `RAIL_NEVER_SAW_EFFECT` | str_key | RAIL_NEVER_SAW_EFFECT = "RAIL_NEVER_SAW_EFFECT" |  |  |  |  |

| `RAIL_NEVER_SUBMITTED` | str_key | RAIL_NEVER_SUBMITTED = "RAIL_NEVER_SUBMITTED" |  |  |  |  |

| `REFUSED:PAYMENT_NOT_ABANDONABLE` | str_key | NOT_ABANDONABLE = "REFUSED:PAYMENT_NOT_ABANDONABLE" |  |  |  |  |

| `REFUSED:PAYMENT_RAIL_HOLD_UNAVAILABLE` | str_key | RAIL_HOLD_UNAVAILABLE = "REFUSED:PAYMENT_RAIL_HOLD_UNAVAILABLE" |  |  |  |  |

| `REFUSED:PAYMENT_RAIL_TIME_INVALID` | str_key | RAIL_TIME_INVALID = "REFUSED:PAYMENT_RAIL_TIME_INVALID" |  |  |  |  |

| `payments.hold` | str_key | T_HOLD = "payments.hold" |  |  |  |  |

| `payments.rail_submit` | str_key | T_RAIL_SUBMIT = "payments.rail_submit" |  |  |  |  |

| `RailExecutionParams` | struct | RailExecutionParams { pub bindings: EffectBindings, pub created_at_iso: String, pub debtor_name: String, pub creditor_name: String, pub debtor_agent_bic: String, pub creditor_agent_bic: String, pub rail_profile: String } |  |  |  |  |

| `RailSubmission` | struct | RailSubmission { pub effect_id: String, pub correlation_id: String, pub standing: RailStandingAfterSubmit, pub construct_digest: Option<String>, pub brce_prepare_receipt_digests: Vec<String>, pub brce_outcome_receipt_digests: Vec<String>, pub pee: PreparedEconomicEffect, pub payload_digest: String, pub detail: String } |  |  |  |  |


### src/payments/experience.rs

| `Knowledge` | enum | Knowledge { Known(ClassRule), Unknown } |  |  |  |  |

| `classify` | function | classify(&self, effect: &PaymentEffect) -> PayResult<Knowledge> |  |  |  |  |

| `key` | function | key(&self) -> String |  |  |  |  |

| `of` | function | of(effect: &PaymentEffect) -> Self |  |  |  |  |

| `open` | function | open(root: impl Into<PathBuf>) -> PayResult<Self> |  |  |  |  |

| `record_settled` | function | record_settled(&self, effect: &PaymentEffect, exec: &PaymentExecution) -> PayResult<()> |  |  |  |  |

| `requires_intelligence` | function | requires_intelligence(k: &Knowledge) -> bool |  |  |  |  |

| `BLOCKED:EXPERIENCE_STORE_FAILED` | str_key | STORE_FAILED = "BLOCKED:EXPERIENCE_STORE_FAILED" |  |  |  |  |

| `CASTLE-PAYMENT-CLASS-V1` | str_key | CLASS_DOMAIN = "CASTLE-PAYMENT-CLASS-V1" |  |  |  |  |

| `REFUSED:EXPERIENCE_REQUIRES_SETTLED_RECEIPT` | str_key | REQUIRES_SETTLED = "REFUSED:EXPERIENCE_REQUIRES_SETTLED_RECEIPT" |  |  |  |  |

| `ClassRule` | struct | ClassRule { pub class_key: String, pub per_effect_cap_minor: u64, pub settled_count: u64, pub last_construct_digest: String, pub last_ocel_receipt_digest: String } |  |  |  |  |

| `ExperienceStore` | struct | ExperienceStore { root: PathBuf } |  |  |  |  |

| `PaymentClass` | struct | PaymentClass { principal: String, payer: String, payee: String, currency: Currency, purpose: String } |  |  |  |  |


### src/payments/fibo.rs

| `CASTLE_NS` | const | CASTLE_NS: &str |  |  |  |  |

| `CASTLE_PAYMENT_EFFECT` | const | CASTLE_PAYMENT_EFFECT: &str |  |  |  |  |

| `claim_jsonld` | function | claim_jsonld(claim: &Claim, entry: Option<&LedgerEntry>) -> Value |  |  |  |  |

| `claim_type_iri` | function | claim_type_iri(claim: &Claim) -> &'static str |  |  |  |  |

| `currency_iri` | function | currency_iri(currency: Currency) -> &'static str |  |  |  |  |

| `mapping_for` | function | mapping_for(id: &str) -> Option<&'static FiboMapping> |  |  |  |  |

| `monetary_amount_json` | function | monetary_amount_json(money: &Money) -> Value |  |  |  |  |

| `unverified_terms` | function | unverified_terms() -> Vec<&'static FiboMapping> |  |  |  |  |

| `https://chatmangpt.com/ontology/castle#` | str_key | CASTLE_NS = "https://chatmangpt.com/ontology/castle#" |  |  |  |  |

| `https://chatmangpt.com/ontology/castle#PaymentEffect` | str_key | CASTLE_PAYMENT_EFFECT = "https://chatmangpt.com/ontology/castle#PaymentEffect" |  |  |  |  |

| `https://chatmangpt.com/ontology/castle#constructDigest` | str_key | CASTLE_CONSTRUCT_DIGEST = "https://chatmangpt.com/ontology/castle#constructDigest" |  |  |  |  |

| `https://chatmangpt.com/ontology/castle#effectDigest` | str_key | CASTLE_EFFECT_DIGEST = "https://chatmangpt.com/ontology/castle#effectDigest" |  |  |  |  |

| `https://chatmangpt.com/ontology/castle#minorUnits` | str_key | CASTLE_MINOR_UNITS = "https://chatmangpt.com/ontology/castle#minorUnits" |  |  |  |  |

| `https://chatmangpt.com/ontology/castle#obligation` | str_key | CASTLE_OBLIGATION = "https://chatmangpt.com/ontology/castle#obligation" |  |  |  |  |

| `https://chatmangpt.com/ontology/castle#obligationId` | str_key | CASTLE_OBLIGATION_ID = "https://chatmangpt.com/ontology/castle#obligationId" |  |  |  |  |

| `https://chatmangpt.com/ontology/castle#payeeAccount` | str_key | CASTLE_PAYEE = "https://chatmangpt.com/ontology/castle#payeeAccount" |  |  |  |  |

| `https://chatmangpt.com/ontology/castle#payerAccount` | str_key | CASTLE_PAYER = "https://chatmangpt.com/ontology/castle#payerAccount" |  |  |  |  |


### src/payments/iso20022.rs

| `ISO20022_PROFILE` | const | ISO20022_PROFILE: &str |  |  |  |  |

| `PROJECTION_FIELD_INVALID` | const | PROJECTION_FIELD_INVALID: &str |  |  |  |  |

| `message_profile_version` | function | message_profile_version() -> &'static str |  |  |  |  |

| `pacs008_fi_credit_transfer` | function | pacs008_fi_credit_transfer( admission: &PaymentAdmission, created_at_iso: &str, instructing_agent_bic: &str, instructed_agent_bic: &str, debtor_agent_bic: &str, creditor_agent_bic: &str, ) -> PayResult<String> |  |  |  |  |

| `pain001_customer_credit_transfer` | function | pain001_customer_credit_transfer( admission: &PaymentAdmission, created_at_iso: &str, debtor_name: &str, creditor_name: &str, debtor_agent_bic: &str, creditor_agent_bic: &str, ) -> PayResult<String> |  |  |  |  |

| `project_effect_digest_from_pain001` | function | project_effect_digest_from_pain001(xml: &str) -> Option<String> |  |  |  |  |

| `project_obligation_id_from_pain001` | function | project_obligation_id_from_pain001(xml: &str) -> Option<String> |  |  |  |  |

| `REFUSED:PROJECTION_FIELD_INVALID` | str_key | PROJECTION_FIELD_INVALID = "REFUSED:PROJECTION_FIELD_INVALID" |  |  |  |  |

| `castle:effect-digest=` | str_key | DIGEST_MARKER = "castle:effect-digest=" |  |  |  |  |

| `castle:obligation-id=` | str_key | OBLIGATION_MARKER = "castle:obligation-id=" |  |  |  |  |

| `{` | str_key | ISO20022_PROFILE = "{" |  |  |  |  |


### src/payments/ledger.rs

| `LedgerError` | enum | LedgerError { InsufficientFunds, Unavailable(String) } |  |  |  |  |

| `conserves` | function | conserves(&self, currency: Currency) -> Result<bool, LedgerError> |  |  |  |  |

| `construct_digest` | function | construct_digest(&self) -> &str |  |  |  |  |

| `entries` | function | entries(&self) -> Result<Vec<LedgerEntry>, LedgerError> |  |  |  |  |

| `from_construct` | function | from_construct(admission: &ConstructAdmission) -> Self |  |  |  |  |

| `open` | function | open(root: impl Into<PathBuf>, opening: &[(&str, Currency, u64)]) -> Result<Self, LedgerError> |  |  |  |  |

| `returns` | function | returns(&self) -> Result<Vec<LedgerEntry>, LedgerError> |  |  |  |  |

| `ActuationToken` | struct | ActuationToken { construct_digest: String } |  |  |  |  |

| `FileJournalLedger` | struct | FileJournalLedger { root: PathBuf, lock: Mutex<()> } |  |  |  |  |

| `LedgerEntry` | struct | LedgerEntry { pub seq: u64, pub effect_digest: String, pub debit_account: String, pub credit_account: String, pub amount_minor: u64, pub currency: Currency, pub construct_digest: String } |  |  |  |  |

| `LedgerHold` | struct | LedgerHold { pub effect_digest: String, pub account: String, pub currency: Currency, pub amount_minor: u64, pub construct_digest: String } |  |  |  |  |

| `LedgerPort` | trait |  |  |  |  |  |


### src/payments/mod.rs

| `VERSION` | const | VERSION: &str |  |  |  |  |

| `26.9.29` | str_key | VERSION = "26.9.29" |  |  |  |  |

| `admission::{admit_payment, AdmissionContext, PaymentAdmission}` | use | admission::{admit_payment, AdmissionContext, PaymentAdmission} |  |  |  |  |

| `admission::{admit_payment_screened, Screening, ScreeningEvidence}` | use | admission::{admit_payment_screened, Screening, ScreeningEvidence} |  |  |  |  |

| `claim_store::{Claim, ClaimState, ClaimStore}` | use | claim_store::{Claim, ClaimState, ClaimStore} |  |  |  |  |

| `effect::{PaymentEffect, PAYMENT_CAPABILITY}` | use | effect::{PaymentEffect, PAYMENT_CAPABILITY} |  |  |  |  |

| `event_receipt::{explain, seal_event_receipt, verify_event_receipt, EventInputs, EventReceipt}` | use | event_receipt::{explain, seal_event_receipt, verify_event_receipt, EventInputs, EventReceipt} |  |  |  |  |

| `execute::{build_construct, execute_payment, ExecutionContext, PaymentExecution, PaymentStanding}` | use | execute::{build_construct, execute_payment, ExecutionContext, PaymentExecution, PaymentStanding} |  |  |  |  |

| `execute_rail::{abandon_unsubmitted, finalize_via_rail, submit_via_rail, FinalizeResult, RailExecutionParams, RailStandingAfterSubmit, RailSubmission}` | use | execute_rail::{abandon_unsubmitted, finalize_via_rail, submit_via_rail, FinalizeResult, RailExecutionParams, RailStandingAfterSubmit, RailSubmission} |  |  |  |  |

| `experience::{ClassRule, ExperienceStore, Knowledge, PaymentClass}` | use | experience::{ClassRule, ExperienceStore, Knowledge, PaymentClass} |  |  |  |  |

| `ledger::{ActuationToken, FileJournalLedger, LedgerEntry, LedgerError, LedgerHold, LedgerPort}` | use | ledger::{ActuationToken, FileJournalLedger, LedgerEntry, LedgerError, LedgerHold, LedgerPort} |  |  |  |  |

| `money::{Currency, Money}` | use | money::{Currency, Money} |  |  |  |  |

| `nonce::DurableNonceFence` | use | nonce::DurableNonceFence |  |  |  |  |

| `pee::{EffectBindings, PreparedEconomicEffect}` | use | pee::{EffectBindings, PreparedEconomicEffect} |  |  |  |  |

| `policy::{PrincipalPolicy, SpendPolicy}` | use | policy::{PrincipalPolicy, SpendPolicy} |  |  |  |  |

| `rail::{RailAck, RailActuator, RailError, RailInstruction, RailStatus}` | use | rail::{RailAck, RailActuator, RailError, RailInstruction, RailStatus} |  |  |  |  |

| `rail_sim::{SimMode, SimRail}` | use | rail_sim::{SimMode, SimRail} |  |  |  |  |

| `reconcile::{reconcile, recover_journal, JournalRecovery, ReconcileResolution}` | use | reconcile::{reconcile, recover_journal, JournalRecovery, ReconcileResolution} |  |  |  |  |

| `replay::{admit_payment_journaled, admit_payment_screened_journaled, replay_admission, replay_admission_anchored, AdmissionJournal, AdmissionRecord, ReplayVerdict}` | use | replay::{admit_payment_journaled, admit_payment_screened_journaled, replay_admission, replay_admission_anchored, AdmissionJournal, AdmissionRecord, ReplayVerdict} |  |  |  |  |

| `settlement::{apply_rail_report, observe_rail, FinalityKind, FinalityOutcome, RailReport}` | use | settlement::{apply_rail_report, observe_rail, FinalityKind, FinalityOutcome, RailReport} |  |  |  |  |


### src/payments/money.rs

| `Currency` | enum | Currency { USD, EUR, GBP, JPY, KWD } |  |  |  |  |

| `code` | function | code(self) -> &'static str |  |  |  |  |

| `exponent` | function | exponent(self) -> u32 |  |  |  |  |

| `parse_minor` | function | parse_minor(text: &str, currency: Currency) -> PayResult<Self> |  |  |  |  |

| `to_decimal_string` | function | to_decimal_string(self) -> String |  |  |  |  |

| `Money` | struct | Money { pub minor: u64, pub currency: Currency } |  |  |  |  |


### src/payments/nonce.rs

| `claim` | function | claim(&self, principal: &str, nonce: &str) -> PayResult<()> |  |  |  |  |

| `open` | function | open(root: impl Into<PathBuf>) -> PayResult<Self> |  |  |  |  |

| `DurableNonceFence` | struct | DurableNonceFence { root: PathBuf } |  |  |  |  |


### src/payments/obligation.rs

| `OBLIGATION_ID_NOT_DERIVED` | const | OBLIGATION_ID_NOT_DERIVED: &str |  |  |  |  |

| `derive_obligation_id` | function | derive_obligation_id(payer: &str, payee: &str, purpose: &str, invoice_ref: &str) -> String |  |  |  |  |

| `prepare_for_invoice` | function | prepare_for_invoice( principal: &str, payer: &str, payee: &str, amount_minor: &str, currency: Currency, purpose: &str, invoice_ref: &str, reverses: Option<&str>, ) -> PayResult<PreparedEffect> |  |  |  |  |

| `CASTLE-OBLIGATION-V1` | str_key | OBLIGATION_DOMAIN = "CASTLE-OBLIGATION-V1" |  |  |  |  |

| `REFUSED:PAYMENT_OBLIGATION_ID_NOT_DERIVED` | str_key | OBLIGATION_ID_NOT_DERIVED = "REFUSED:PAYMENT_OBLIGATION_ID_NOT_DERIVED" |  |  |  |  |


### src/payments/pee.rs

| `PEE_BINDING_INCOMPLETE` | const | PEE_BINDING_INCOMPLETE: &str |  |  |  |  |

| `PEE_EXPIRED` | const | PEE_EXPIRED: &str |  |  |  |  |

| `PEE_IDENTITY_MISMATCH` | const | PEE_IDENTITY_MISMATCH: &str |  |  |  |  |

| `PEE_NOT_YET_VALID` | const | PEE_NOT_YET_VALID: &str |  |  |  |  |

| `check_fresh` | function | check_fresh(&self, now_ms: u64) -> PayResult<()> |  |  |  |  |

| `reseal_identity` | function | reseal_identity(&mut self) -> PayResult<()> |  |  |  |  |

| `seal` | function | seal(admission: &PaymentAdmission, b: &EffectBindings) -> PayResult<Self> |  |  |  |  |

| `verify_against` | function | verify_against(&self, admission: &PaymentAdmission) -> PayResult<()> |  |  |  |  |

| `verify_identity` | function | verify_identity(&self) -> PayResult<()> |  |  |  |  |

| `CASTLE-PEE-AUTHORITY-V1` | str_key | AUTHORITY_DOMAIN = "CASTLE-PEE-AUTHORITY-V1" |  |  |  |  |

| `CASTLE-PEE-V1` | str_key | PEE_DOMAIN = "CASTLE-PEE-V1" |  |  |  |  |

| `REFUSED:EFFECT_BINDING_INCOMPLETE` | str_key | PEE_BINDING_INCOMPLETE = "REFUSED:EFFECT_BINDING_INCOMPLETE" |  |  |  |  |

| `REFUSED:EFFECT_EXPIRED` | str_key | PEE_EXPIRED = "REFUSED:EFFECT_EXPIRED" |  |  |  |  |

| `REFUSED:EFFECT_IDENTITY_MISMATCH` | str_key | PEE_IDENTITY_MISMATCH = "REFUSED:EFFECT_IDENTITY_MISMATCH" |  |  |  |  |

| `REFUSED:EFFECT_NOT_YET_VALID` | str_key | PEE_NOT_YET_VALID = "REFUSED:EFFECT_NOT_YET_VALID" |  |  |  |  |

| `effect_id` | str_key | EXCLUDED = "effect_id" |  |  |  |  |

| `EffectBindings` | struct | EffectBindings { pub policy_profile_id: String, pub law_state_digest: String, pub counterparty_evidence_digest: String, pub funding_source_id: String, pub resource_reservation_id: String, pub rail_profile_id: String, pub message_profile_version: String, pub parent_receipt: String, pub created_at_ms: u64, pub expires_at_ms: u64 } |  |  |  |  |

| `PreparedEconomicEffect` | struct | PreparedEconomicEffect { pub version: u32, pub effect_id: String, pub obligation_id: String, pub principal_id: String, pub payer_account: String, pub beneficiary_account: String, pub amount_minor: String, pub currency: Currency, pub purpose: String, pub authority_grant_id: String, pub authority_digest: String, pub policy_profile_id: String, pub law_state_digest: String, pub counterparty_evidence_digest: String, pub funding_source_id: String, pub resource_reservation_id: String, pub rail_profile_id: String, pub message_profile_version: String, pub idempotency_key: String, pub created_at_ms: u64, pub expires_at_ms: u64, pub nonce: String, pub parent_receipt: String, pub canonical_payload_digest: String } |  |  |  |  |


### src/payments/policy.rs

| `INSUFFICIENT_QUORUM` | const | INSUFFICIENT_QUORUM: &str |  |  |  |  |

| `check_quorum` | function | check_quorum(&self, certificate_threshold: u16) -> PayResult<()> |  |  |  |  |

| `check_static` | function | check_static(&self, effect: &PaymentEffect) -> PayResult<&PrincipalPolicy> |  |  |  |  |

| `default_min_quorum` | function | default_min_quorum() -> u16 |  |  |  |  |

| `to_json` | function | to_json(&self) -> serde_json::Value |  |  |  |  |

| `REFUSED:InsufficientQuorum` | str_key | INSUFFICIENT_QUORUM = "REFUSED:InsufficientQuorum" |  |  |  |  |

| `PrincipalPolicy` | struct | PrincipalPolicy { pub allowed_payers: BTreeSet<String>, pub allowed_payees: BTreeSet<String>, pub per_effect_cap: BTreeMap<Currency, u64>, pub epoch_cap: BTreeMap<Currency, u64>, pub min_quorum: u16 } |  |  |  |  |

| `SpendPolicy` | struct | SpendPolicy { pub principals: BTreeMap<String, PrincipalPolicy>, pub require_derived_obligation: bool, pub require_screening: bool } |  |  |  |  |


### src/payments/rail.rs

| `PAYLOAD_DOMAIN` | const | PAYLOAD_DOMAIN: &[u8] |  |  |  |  |

| `RailAck` | enum | RailAck { Accepted { correlation_id: String }, Rejected { correlation_id: String, reason_code: String } } |  |  |  |  |

| `RailError` | enum | RailError { Timeout, Unavailable(String) } |  |  |  |  |

| `RailStatus` | enum | RailStatus { Unknown, Accepted, Settled { final_ref: String }, Rejected { reason: String }, Returned { reason: String, return_ref: String } } |  |  |  |  |

| `correlation_id_for` | function | correlation_id_for(effect_id: &str) -> String |  |  |  |  |

| `payload_digest` | function | payload_digest(&self) -> String |  |  |  |  |

| `CASTLE-RAIL-PAYLOAD-V1` | str_key | PAYLOAD_DOMAIN = "CASTLE-RAIL-PAYLOAD-V1" |  |  |  |  |

| `RailInstruction` | struct | RailInstruction { pub effect_id: String, pub correlation_id: String, pub message_profile: String, pub payload: String, pub amount_minor: u64, pub currency: Currency, pub payer: String, pub payee: String } |  |  |  |  |

| `RailActuator` | trait |  |  |  |  |  |


### src/payments/rail_sim.rs

| `SIM_REJECT_AFTER_DELAY` | const | SIM_REJECT_AFTER_DELAY: &str |  |  |  |  |

| `SIM_REJECT_DUPLICATE_DIFFERENT_PAYLOAD` | const | SIM_REJECT_DUPLICATE_DIFFERENT_PAYLOAD: &str |  |  |  |  |

| `SIM_RETURN_REASON` | const | SIM_RETURN_REASON: &str |  |  |  |  |

| `SimMode` | enum | SimMode { Honest, DropAckAfterAccept, RejectAfterPolls(u32), SettleAfterPolls(u32), ReturnAfterSettle, Down, DuplicateReports } |  |  |  |  |

| `accepted_correlations` | function | accepted_correlations(&self) -> Vec<String> |  |  |  |  |

| `open` | function | open(root: &Path, mode: SimMode) -> Result<Self, String> |  |  |  |  |

| `set_mode` | function | set_mode(&self, mode: SimMode) -> Result<(), String> |  |  |  |  |

| `settlement_count` | function | settlement_count(&self, correlation: &str) -> usize |  |  |  |  |

| `status_history` | function | status_history(&self, correlation: &str) -> Vec<RailStatus> |  |  |  |  |

| `submissions_seen` | function | submissions_seen(&self, correlation: &str) -> usize |  |  |  |  |

| `CASTLE-SIMRAIL-V1` | str_key | SIM_DOMAIN = "CASTLE-SIMRAIL-V1" |  |  |  |  |

| `DUPLICATE_CORRELATION_DIFFERENT_PAYLOAD` | str_key | SIM_REJECT_DUPLICATE_DIFFERENT_PAYLOAD = "DUPLICATE_CORRELATION_DIFFERENT_PAYLOAD" |  |  |  |  |

| `REJECTED_BY_RAIL_AFTER_DELAY` | str_key | SIM_REJECT_AFTER_DELAY = "REJECTED_BY_RAIL_AFTER_DELAY" |  |  |  |  |

| `RETURNED_BY_BENEFICIARY_BANK` | str_key | SIM_RETURN_REASON = "RETURNED_BY_BENEFICIARY_BANK" |  |  |  |  |

| `SimRail` | struct | SimRail { root: PathBuf } |  |  |  |  |


### src/payments/reconcile.rs

| `NOT_RECONCILABLE` | const | NOT_RECONCILABLE: &str |  |  |  |  |

| `ReconcileResolution` | enum | ReconcileResolution { Settled(LedgerEntry), ProvenAbsent, StillUnknown } |  |  |  |  |

| `reconcile` | function | reconcile(effect_digest: &str, claims: &ClaimStore, ledger: &dyn LedgerPort) -> PayResult<ReconcileResolution> |  |  |  |  |

| `recover_journal` | function | recover_journal(journal_root: &Path) -> PayResult<JournalRecovery> |  |  |  |  |

| `REFUSED:PAYMENT_NOT_RECONCILABLE` | str_key | NOT_RECONCILABLE = "REFUSED:PAYMENT_NOT_RECONCILABLE" |  |  |  |  |

| `JournalRecovery` | struct | JournalRecovery { pub prepared_without_outcome: Vec<String>, pub complete: Vec<String> } |  |  |  |  |


### src/payments/refusal.rs

| `ACCOUNT_NOT_ALLOWED` | const | ACCOUNT_NOT_ALLOWED: &str |  |  |  |  |

| `ALREADY_SETTLED` | const | ALREADY_SETTLED: &str |  |  |  |  |

| `AMOUNT_EXCEEDS_CAP` | const | AMOUNT_EXCEEDS_CAP: &str |  |  |  |  |

| `AMOUNT_NOT_INTEGER` | const | AMOUNT_NOT_INTEGER: &str |  |  |  |  |

| `AMOUNT_ZERO` | const | AMOUNT_ZERO: &str |  |  |  |  |

| `BUDGET_EXCEEDED` | const | BUDGET_EXCEEDED: &str |  |  |  |  |

| `CAPABILITY_NOT_ADMITTED` | const | CAPABILITY_NOT_ADMITTED: &str |  |  |  |  |

| `CLAIM_NOT_RESERVED` | const | CLAIM_NOT_RESERVED: &str |  |  |  |  |

| `CLAIM_STORE_FAILED` | const | CLAIM_STORE_FAILED: &str |  |  |  |  |

| `CURRENCY_UNSUPPORTED` | const | CURRENCY_UNSUPPORTED: &str |  |  |  |  |

| `INSUFFICIENT_FUNDS` | const | INSUFFICIENT_FUNDS: &str |  |  |  |  |

| `IN_FLIGHT` | const | IN_FLIGHT: &str |  |  |  |  |

| `LEDGER_UNAVAILABLE` | const | LEDGER_UNAVAILABLE: &str |  |  |  |  |

| `NO_SPEND_POLICY` | const | NO_SPEND_POLICY: &str |  |  |  |  |

| `OBLIGATION_ALREADY_CLAIMED` | const | OBLIGATION_ALREADY_CLAIMED: &str |  |  |  |  |

| `OUTCOME_UNKNOWN` | const | OUTCOME_UNKNOWN: &str |  |  |  |  |

| `PAYLOAD_INVALID` | const | PAYLOAD_INVALID: &str |  |  |  |  |

| `RECONCILIATION_DRIFT` | const | RECONCILIATION_DRIFT: &str |  |  |  |  |

| `REVERSAL_EXCEEDS_ORIGINAL` | const | REVERSAL_EXCEEDS_ORIGINAL: &str |  |  |  |  |

| `REVERSAL_MISMATCH` | const | REVERSAL_MISMATCH: &str |  |  |  |  |

| `REVERSAL_ORIGINAL_NOT_EXECUTED` | const | REVERSAL_ORIGINAL_NOT_EXECUTED: &str |  |  |  |  |

| `SAME_ACCOUNT` | const | SAME_ACCOUNT: &str |  |  |  |  |

| `refuse` | function | refuse(code: &str) -> PayResult<T> |  |  |  |  |

| `BLOCKED:PAYMENT_CLAIM_STORE_FAILED` | str_key | CLAIM_STORE_FAILED = "BLOCKED:PAYMENT_CLAIM_STORE_FAILED" |  |  |  |  |

| `REFUSED:NO_SPEND_POLICY_FOR_PRINCIPAL` | str_key | NO_SPEND_POLICY = "REFUSED:NO_SPEND_POLICY_FOR_PRINCIPAL" |  |  |  |  |

| `REFUSED:PAYMENT_ACCOUNT_NOT_ALLOWED` | str_key | ACCOUNT_NOT_ALLOWED = "REFUSED:PAYMENT_ACCOUNT_NOT_ALLOWED" |  |  |  |  |

| `REFUSED:PAYMENT_ALREADY_SETTLED` | str_key | ALREADY_SETTLED = "REFUSED:PAYMENT_ALREADY_SETTLED" |  |  |  |  |

| `REFUSED:PAYMENT_AMOUNT_EXCEEDS_CAP` | str_key | AMOUNT_EXCEEDS_CAP = "REFUSED:PAYMENT_AMOUNT_EXCEEDS_CAP" |  |  |  |  |

| `REFUSED:PAYMENT_AMOUNT_NOT_INTEGER` | str_key | AMOUNT_NOT_INTEGER = "REFUSED:PAYMENT_AMOUNT_NOT_INTEGER" |  |  |  |  |

| `REFUSED:PAYMENT_AMOUNT_ZERO` | str_key | AMOUNT_ZERO = "REFUSED:PAYMENT_AMOUNT_ZERO" |  |  |  |  |

| `REFUSED:PAYMENT_BUDGET_EXCEEDED` | str_key | BUDGET_EXCEEDED = "REFUSED:PAYMENT_BUDGET_EXCEEDED" |  |  |  |  |

| `REFUSED:PAYMENT_CAPABILITY_NOT_ADMITTED` | str_key | CAPABILITY_NOT_ADMITTED = "REFUSED:PAYMENT_CAPABILITY_NOT_ADMITTED" |  |  |  |  |

| `REFUSED:PAYMENT_CLAIM_NOT_RESERVED` | str_key | CLAIM_NOT_RESERVED = "REFUSED:PAYMENT_CLAIM_NOT_RESERVED" |  |  |  |  |

| `REFUSED:PAYMENT_CURRENCY_UNSUPPORTED` | str_key | CURRENCY_UNSUPPORTED = "REFUSED:PAYMENT_CURRENCY_UNSUPPORTED" |  |  |  |  |

| `REFUSED:PAYMENT_INSUFFICIENT_FUNDS` | str_key | INSUFFICIENT_FUNDS = "REFUSED:PAYMENT_INSUFFICIENT_FUNDS" |  |  |  |  |

| `REFUSED:PAYMENT_IN_FLIGHT` | str_key | IN_FLIGHT = "REFUSED:PAYMENT_IN_FLIGHT" |  |  |  |  |

| `REFUSED:PAYMENT_LEDGER_UNAVAILABLE` | str_key | LEDGER_UNAVAILABLE = "REFUSED:PAYMENT_LEDGER_UNAVAILABLE" |  |  |  |  |

| `REFUSED:PAYMENT_OBLIGATION_ALREADY_CLAIMED` | str_key | OBLIGATION_ALREADY_CLAIMED = "REFUSED:PAYMENT_OBLIGATION_ALREADY_CLAIMED" |  |  |  |  |

| `REFUSED:PAYMENT_OUTCOME_UNKNOWN` | str_key | OUTCOME_UNKNOWN = "REFUSED:PAYMENT_OUTCOME_UNKNOWN" |  |  |  |  |

| `REFUSED:PAYMENT_PAYLOAD_INVALID` | str_key | PAYLOAD_INVALID = "REFUSED:PAYMENT_PAYLOAD_INVALID" |  |  |  |  |

| `REFUSED:PAYMENT_REVERSAL_EXCEEDS_ORIGINAL` | str_key | REVERSAL_EXCEEDS_ORIGINAL = "REFUSED:PAYMENT_REVERSAL_EXCEEDS_ORIGINAL" |  |  |  |  |

| `REFUSED:PAYMENT_REVERSAL_MISMATCH` | str_key | REVERSAL_MISMATCH = "REFUSED:PAYMENT_REVERSAL_MISMATCH" |  |  |  |  |

| `REFUSED:PAYMENT_REVERSAL_ORIGINAL_NOT_EXECUTED` | str_key | REVERSAL_ORIGINAL_NOT_EXECUTED = "REFUSED:PAYMENT_REVERSAL_ORIGINAL_NOT_EXECUTED" |  |  |  |  |

| `REFUSED:PAYMENT_SAME_ACCOUNT` | str_key | SAME_ACCOUNT = "REFUSED:PAYMENT_SAME_ACCOUNT" |  |  |  |  |

| `REFUSED:RECONCILIATION_DRIFT` | str_key | RECONCILIATION_DRIFT = "REFUSED:RECONCILIATION_DRIFT" |  |  |  |  |


### src/payments/replay.rs

| `REPLAY_DIGEST_MISMATCH` | const | REPLAY_DIGEST_MISMATCH: &str |  |  |  |  |

| `REPLAY_RECORD_CORRUPT` | const | REPLAY_RECORD_CORRUPT: &str |  |  |  |  |

| `REPLAY_RECORD_MISSING` | const | REPLAY_RECORD_MISSING: &str |  |  |  |  |

| `REPLAY_REGISTRY_UNTRUSTED` | const | REPLAY_REGISTRY_UNTRUSTED: &str |  |  |  |  |

| `ReplayVerdict` | enum | ReplayVerdict { Reproduced { effect_digest: String, decision_digest: String }, Diverged { reason: String } } |  |  |  |  |

| `admit_payment_journaled` | function | admit_payment_journaled( prepared: PreparedEffect, certificate: &ActuationCertificate, ctx: &AdmissionContext<'_>, journal: &AdmissionJournal, ) -> PayResult<PaymentAdmission> |  |  |  |  |

| `admit_payment_screened_journaled` | function | admit_payment_screened_journaled( prepared: PreparedEffect, certificate: &ActuationCertificate, ctx: &AdmissionContext<'_>, screening: &Screening<'_>, journal: &AdmissionJournal, ) -> PayResult<(PaymentAdmission, ScreeningEvidence)> |  |  |  |  |

| `load` | function | load(&self, effect_digest: &str) -> PayResult<AdmissionRecord> |  |  |  |  |

| `open` | function | open(root: impl Into<PathBuf>) -> PayResult<Self> |  |  |  |  |

| `replay_admission` | function | replay_admission(journal: &AdmissionJournal, effect_digest: &str) -> PayResult<ReplayVerdict> |  |  |  |  |

| `replay_admission_anchored` | function | replay_admission_anchored( journal: &AdmissionJournal, effect_digest: &str, trusted_registry: &KeyRegistry, ) -> PayResult<ReplayVerdict> |  |  |  |  |

| `CASTLE-REPLAY-DECISION-V1` | str_key | DECISION_DOMAIN = "CASTLE-REPLAY-DECISION-V1" |  |  |  |  |

| `REFUSED:REPLAY_EFFECT_DIGEST_MISMATCH` | str_key | REPLAY_DIGEST_MISMATCH = "REFUSED:REPLAY_EFFECT_DIGEST_MISMATCH" |  |  |  |  |

| `REFUSED:REPLAY_RECORD_CORRUPT` | str_key | REPLAY_RECORD_CORRUPT = "REFUSED:REPLAY_RECORD_CORRUPT" |  |  |  |  |

| `REFUSED:REPLAY_RECORD_MISSING` | str_key | REPLAY_RECORD_MISSING = "REFUSED:REPLAY_RECORD_MISSING" |  |  |  |  |

| `REFUSED:REPLAY_REGISTRY_UNTRUSTED` | str_key | REPLAY_REGISTRY_UNTRUSTED = "REFUSED:REPLAY_REGISTRY_UNTRUSTED" |  |  |  |  |

| `AdmissionJournal` | struct | AdmissionJournal { root: PathBuf } |  |  |  |  |

| `AdmissionRecord` | struct | AdmissionRecord { pub prepared: PreparedEffect, pub certificate: ActuationCertificate, pub registry: Vec<KeyRecord>, pub policy_epoch: u64, pub revocation_epoch: u64, pub generation: u64, pub audience: String, pub now_ms: u64, pub policy: SpendPolicy } |  |  |  |  |


### src/payments/settlement.rs

| `EVIDENCE_CONFLICT` | const | EVIDENCE_CONFLICT: &str |  |  |  |  |

| `PAYMENT_NOT_FINAL` | const | PAYMENT_NOT_FINAL: &str |  |  |  |  |

| `PAYMENT_NOT_HELD` | const | PAYMENT_NOT_HELD: &str |  |  |  |  |

| `PAYMENT_NOT_SUBMITTED` | const | PAYMENT_NOT_SUBMITTED: &str |  |  |  |  |

| `REPORT_CORRELATION_MISMATCH` | const | REPORT_CORRELATION_MISMATCH: &str |  |  |  |  |

| `REPORT_NOT_TERMINAL` | const | REPORT_NOT_TERMINAL: &str |  |  |  |  |

| `FinalityKind` | enum | FinalityKind { Final, Rejected, Returned } |  |  |  |  |

| `FinalityOutcome` | enum | FinalityOutcome { Settled(LedgerEntry), AlreadyFinal, Released, Returned(LedgerEntry) } |  |  |  |  |

| `apply_finality` | function | apply_finality(claims: &ClaimStore, ledger: &dyn LedgerPort, ev: &FinalityEvidence) -> PayResult<FinalityOutcome> |  |  |  |  |

| `apply_rail_report` | function | apply_rail_report(claims: &ClaimStore, ledger: &dyn LedgerPort, report: &RailReport) -> PayResult<FinalityOutcome> |  |  |  |  |

| `correlation_id` | function | correlation_id(&self) -> &str |  |  |  |  |

| `effect_digest` | function | effect_digest(&self) -> &str |  |  |  |  |

| `evidence_digest` | function | evidence_digest(&self) -> &str |  |  |  |  |

| `kind` | function | kind(&self) -> FinalityKind |  |  |  |  |

| `observe_rail` | function | observe_rail(rail: &dyn RailActuator, effect_digest: &str) -> Result<RailReport, RailError> |  |  |  |  |

| `payee` | function | payee(&self) -> &str |  |  |  |  |

| `reason` | function | reason(&self) -> &str |  |  |  |  |

| `status` | function | status(&self) -> &RailStatus |  |  |  |  |

| `CASTLE-RAIL-STATUS-EVIDENCE-V1` | str_key | STATUS_EVIDENCE_DOMAIN = "CASTLE-RAIL-STATUS-EVIDENCE-V1" |  |  |  |  |

| `REFUSED:PAYMENT_EVIDENCE_CONFLICT` | str_key | EVIDENCE_CONFLICT = "REFUSED:PAYMENT_EVIDENCE_CONFLICT" |  |  |  |  |

| `REFUSED:PAYMENT_NOT_FINAL` | str_key | PAYMENT_NOT_FINAL = "REFUSED:PAYMENT_NOT_FINAL" |  |  |  |  |

| `REFUSED:PAYMENT_NOT_HELD` | str_key | PAYMENT_NOT_HELD = "REFUSED:PAYMENT_NOT_HELD" |  |  |  |  |

| `REFUSED:PAYMENT_NOT_SUBMITTED` | str_key | PAYMENT_NOT_SUBMITTED = "REFUSED:PAYMENT_NOT_SUBMITTED" |  |  |  |  |

| `REFUSED:PAYMENT_REPORT_CORRELATION_MISMATCH` | str_key | REPORT_CORRELATION_MISMATCH = "REFUSED:PAYMENT_REPORT_CORRELATION_MISMATCH" |  |  |  |  |

| `REFUSED:PAYMENT_REPORT_NOT_TERMINAL` | str_key | REPORT_NOT_TERMINAL = "REFUSED:PAYMENT_REPORT_NOT_TERMINAL" |  |  |  |  |

| `FinalityEvidence` | struct | FinalityEvidence { effect_digest: String, correlation_id: String, evidence_digest: String, kind: FinalityKind, reason: String } |  |  |  |  |

| `RailReport` | struct | RailReport { effect_digest: String, correlation_id: String, status: RailStatus, evidence_digest: String, _seal: () } |  |  |  |  |


### src/planner_minimal.rs

| `MinimalActionPlanner` | struct | MinimalActionPlanner { pub id: String } |  |  |  |  |


### src/reconstitution.rs

| `FinalDisposition` | enum | FinalDisposition { Preserved, Subsumed, Replaced, Archived, Refused } |  |  |  |  |

| `admission_digest` | function | admission_digest(&self) -> &str |  |  |  |  |

| `admit_empire_reconstitution_for_construct` | function | admit_empire_reconstitution_for_construct( document: &str, ) -> Result<EmpireReconstitutionAdmission, ReconstitutionRefusal> |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `authority_id` | function | authority_id(&self) -> &str |  |  |  |  |

| `capabilities` | function | capabilities(&self) -> &[ReconstitutedCapability] |  |  |  |  |

| `disposition` | function | disposition(&self) -> FinalDisposition |  |  |  |  |

| `evidence_ids` | function | evidence_ids(&self) -> &[String] |  |  |  |  |

| `id` | function | id(&self) -> &str |  |  |  |  |

| `may_actuate` | function | may_actuate(&self) -> bool |  |  |  |  |

| `observable_surfaces` | function | observable_surfaces(&self) -> &[String] |  |  |  |  |

| `observation_receipt_digest` | function | observation_receipt_digest(&self) -> &str |  |  |  |  |

| `study_id` | function | study_id(&self) -> &str |  |  |  |  |

| `to_o_star_value` | function | to_o_star_value(&self) -> Value |  |  |  |  |

| `OSTAR-EMPIRE-001` | str_key | STUDY_ID = "OSTAR-EMPIRE-001" |  |  |  |  |

| `diagnostics` | str_key | OBSERVABLE_SURFACES = "diagnostics" |  |  |  |  |

| `ggen.legacy.authority-vacuum.admission.v1` | str_key | ADMISSION_SCHEMA = "ggen.legacy.authority-vacuum.admission.v1" |  |  |  |  |

| `ggen.legacy.authority-vacuum.receipt.v1` | str_key | RECEIPT_SCHEMA = "ggen.legacy.authority-vacuum.receipt.v1" |  |  |  |  |

| `ontostar-admission-manufacture` | str_key | REQUIRED_CAPABILITIES = "ontostar-admission-manufacture" |  |  |  |  |

| `AdmissionBrand` | struct |  |  |  |  |  |

| `EmpireReconstitutionAdmission` | struct | EmpireReconstitutionAdmission { study_id: String, admission_digest: String, observation_receipt_digest: String, authority_id: String, authority_digest: String, capabilities: Vec<ReconstitutedCapability>, _brand: sealed::AdmissionBrand } |  |  |  |  |

| `ReconstitutedCapability` | struct | ReconstitutedCapability { id: String, disposition: FinalDisposition, evidence_ids: Vec<String>, observable_surfaces: Vec<String> } |  |  |  |  |

| `ReconstitutionRefusal` | struct | ReconstitutionRefusal { pub code: &'static str, pub detail: String } |  |  |  |  |


### src/reference_stack/feedback.rs

| `project_beam4pm_feedback` | function | project_beam4pm_feedback( log: &ReceiptedOcelLog, subject: &str, replay_identity: &str, ) -> Result<Beam4PmFeedback, String> |  |  |  |  |

| `Beam4PmFeedback` | struct | Beam4PmFeedback { pub subject: String, pub replay_identity: String, pub construct_digest: String, pub outcome_receipt_digest: String, pub ocel_event_count: usize } |  |  |  |  |


### src/reference_stack/fibo.rs

| `FIBO_PAYMENT_CAPABILITY` | const | FIBO_PAYMENT_CAPABILITY: &str |  |  |  |  |

| `GRAPHLAW_HOOK_CONTRACT` | const | GRAPHLAW_HOOK_CONTRACT: &str |  |  |  |  |

| `bind_verification_receipt` | function | bind_verification_receipt( projection: &FiboPaymentProjection, receipt: &VerificationReceipt, ) -> Result<AuthorityEvidence, String> |  |  |  |  |

| `project_fibo_payment` | function | project_fibo_payment(candidate: KnowledgeHookCandidate) -> Result<FiboPaymentProjection, String> |  |  |  |  |

| `fibo:PaymentExecution` | str_key | FIBO_PAYMENT_CAPABILITY = "fibo:PaymentExecution" |  |  |  |  |

| `https://graphlaw.dev/knowledge-hook#HookCandidate` | str_key | GRAPHLAW_HOOK_CONTRACT = "https://graphlaw.dev/knowledge-hook#HookCandidate" |  |  |  |  |

| `AuthorityEvidence` | struct | AuthorityEvidence { pub effect_digest: String, pub principal: String, pub policy_epoch: u64, pub revocation_epoch: u64, pub generation: u64, pub audience: String, pub signer_count: usize, pub custodian_count: usize } |  |  |  |  |

| `FiboPaymentProjection` | struct | FiboPaymentProjection { pub hook_contract: &'static str, pub source_digest: String, pub replay_identity: String, pub prepared_effect: PreparedEffect, pub effect_digest: String } |  |  |  |  |

| `KnowledgeHookCandidate` | struct | KnowledgeHookCandidate { pub version: u32, pub exact_subject: Value, pub source_digest: String, pub replay_identity: String, pub authority: String, pub capability: String, pub principal: String, pub payload: Value } |  |  |  |  |


### src/reference_stack/mod.rs

| `feedback::{project_beam4pm_feedback, Beam4PmFeedback}` | use | feedback::{project_beam4pm_feedback, Beam4PmFeedback} |  |  |  |  |

| `fibo::{ bind_verification_receipt, project_fibo_payment, AuthorityEvidence, FiboPaymentProjection, KnowledgeHookCandidate, FIBO_PAYMENT_CAPABILITY, GRAPHLAW_HOOK_CONTRACT, }` | use | fibo::{ bind_verification_receipt, project_fibo_payment, AuthorityEvidence, FiboPaymentProjection, KnowledgeHookCandidate, FIBO_PAYMENT_CAPABILITY, GRAPHLAW_HOOK_CONTRACT, } |  |  |  |  |

| `ownership::{ canonical_owners, ReferenceOwner, ReferenceStage, ECOSYSTEM_REFERENCE_HEAD, GRAPHLAW_REFERENCE_HEAD, INHERITED_COURTS, REFERENCE_STAGES, }` | use | ownership::{ canonical_owners, ReferenceOwner, ReferenceStage, ECOSYSTEM_REFERENCE_HEAD, GRAPHLAW_REFERENCE_HEAD, INHERITED_COURTS, REFERENCE_STAGES, } |  |  |  |  |


### src/reference_stack/ownership.rs

| `ECOSYSTEM_REFERENCE_HEAD` | const | ECOSYSTEM_REFERENCE_HEAD: &str |  |  |  |  |

| `GRAPHLAW_REFERENCE_HEAD` | const | GRAPHLAW_REFERENCE_HEAD: &str |  |  |  |  |

| `INHERITED_COURTS` | const | INHERITED_COURTS: &[&str] |  |  |  |  |

| `REFERENCE_STAGES` | const | REFERENCE_STAGES: &[ReferenceStage] |  |  |  |  |

| `ReferenceStage` | enum | ReferenceStage { SemanticState, GraphLaw, KnowledgeHook, Sa2aIntent, IndependentAuthorityEvidence, ReactorCommandBus, CastleBrce, ExternalOrSyntheticDo, IndependentPostcondition, ReceiptOcel, Beam4PmFeedback, XaasRuntimeComposition } |  |  |  |  |

| `canonical_owners` | function | canonical_owners() -> &'static [ReferenceOwner] |  |  |  |  |

| `48a7bbd801b8df1d7ffab879b10d58d7f14ef7bc` | str_key | GRAPHLAW_REFERENCE_HEAD = "48a7bbd801b8df1d7ffab879b10d58d7f14ef7bc" |  |  |  |  |

| `bebffdeb2ea4dd6eace31d69298bfb51a073b265` | str_key | ECOSYSTEM_REFERENCE_HEAD = "bebffdeb2ea4dd6eace31d69298bfb51a073b265" |  |  |  |  |

| `graphlaw:knowledge_hook:authority-none` | str_key | INHERITED_COURTS = "graphlaw:knowledge_hook:authority-none" |  |  |  |  |

| `ReferenceOwner` | struct | ReferenceOwner { pub layer: &'static str, pub owner: &'static str } |  |  |  |  |


### src/refusal_conformance.rs

| `verify_refusal` | function | verify_refusal(observation: RefusalObservation<'_>) -> Result<(), &'static str> |  |  |  |  |

| `RefusalObservation` | struct | RefusalObservation { pub expected: &'a str, pub actual: Result<(), &'a str>, pub observed_world_change: bool } |  |  |  |  |


### src/sa2a/mod.rs

| `algorithm::*` | use | algorithm::* |  |  |  |  |

| `certificate::*` | use | certificate::* |  |  |  |  |

| `key_registry::*` | use | key_registry::* |  |  |  |  |

| `policy::*` | use | policy::* |  |  |  |  |

| `refusal::*` | use | refusal::* |  |  |  |  |

| `threshold::*` | use | threshold::* |  |  |  |  |

| `verifier::*` | use | verifier::* |  |  |  |  |


### src/sa2a_authority.rs

| `Refusal` | enum | Refusal { EffectDigestMismatch, PrincipalMismatch, PolicyEpochMismatch, RevocationEpochStale, ExecutionGenerationStale, EmptyQuorum, DuplicateSigner, DuplicateIndependenceDomain, UnknownKey, KeyPrincipalMismatch, AlgorithmMismatch, AlgorithmNotAllowed, KeyNotYetValid, KeyRevoked, SignatureInvalid, QuorumNotMet } |  |  |  |  |

| `SignatureAlgorithm` | enum | SignatureAlgorithm { Ed25519, MlDsa65, SlhDsaShake128f } |  |  |  |  |

| `certificate_message` | function | certificate_message(c:&ActuationCertificate) -> Vec<u8> |  |  |  |  |

| `new` | function | new(provider:P,keys:impl IntoIterator<Item=KeyRecord>) -> Self |  |  |  |  |

| `verify` | function | verify(&self,effect:[u8;32],principal:&str,c:&ActuationCertificate,p:&VerificationPolicy) -> Result<(),Refusal> |  |  |  |  |

| `ActuationCertificate` | struct | ActuationCertificate { pub effect_digest:[u8;32], pub principal:String, pub policy_epoch:u64, pub revocation_epoch:u64, pub execution_generation:u64, pub signatures:Vec<CertificateSignature> } |  |  |  |  |

| `AuthorityVerifier` | struct | AuthorityVerifier { provider:P, keys:BTreeMap<String,KeyRecord> } |  |  |  |  |

| `CertificateSignature` | struct | CertificateSignature { pub key_id:String, pub algorithm:SignatureAlgorithm, pub signature:Vec<u8> } |  |  |  |  |

| `KeyRecord` | struct | KeyRecord { pub key_id:String, pub principal:String, pub algorithm:SignatureAlgorithm, pub public_key:Vec<u8>, pub valid_from_epoch:u64, pub revoked_at_epoch:Option<u64>, pub independence_domain:String } |  |  |  |  |

| `VerificationPolicy` | struct | VerificationPolicy { pub current_policy_epoch:u64, pub current_revocation_epoch:u64, pub minimum_execution_generation:u64, pub quorum:usize, pub allowed_algorithms:BTreeSet<SignatureAlgorithm> } |  |  |  |  |

| `CryptoProvider` | trait |  |  |  |  |  |


### src/sa2a_security/algorithm.rs

| `SignatureAlgorithm` | enum | SignatureAlgorithm { Ed25519, MlDsa65, SlhDsaShake128f } |  |  |  |  |

| `id` | function | id(self) -> &'static str |  |  |  |  |


### src/sa2a_security/certificate.rs

| `signing_message` | function | signing_message(&self) -> Result<Vec<u8>, SecurityRefusal> |  |  |  |  |

| `ActuationCertificate` | struct | ActuationCertificate { pub version: u32, pub effect_digest: String, pub principal: String, pub policy_epoch: u64, pub revocation_epoch: u64, pub generation: u64, pub nonce: String, pub not_before_ms: u64, pub expires_at_ms: u64, pub audience: String, pub threshold: u16, pub signatures: Vec<CertificateSignature> } |  |  |  |  |


### src/sa2a_security/crypto.rs

| `verify_signature` | function | verify_signature( algorithm: SignatureAlgorithm, public_key: &[u8], message: &[u8], signature: &[u8], ) -> Result<(), SecurityRefusal> |  |  |  |  |


### src/sa2a_security/encoding.rs

| `CERTIFICATE_DOMAIN` | const | CERTIFICATE_DOMAIN: &[u8] |  |  |  |  |

| `PREPARED_EFFECT_DOMAIN` | const | PREPARED_EFFECT_DOMAIN: &[u8] |  |  |  |  |

| `push_field` | function | push_field(out: &mut Vec<u8>, value: &[u8]) |  |  |  |  |

| `sha256_tagged` | function | sha256_tagged(domain: &[u8], body: &[u8]) -> String |  |  |  |  |

| `valid_sha256_tag` | function | valid_sha256_tag(value: &str) -> bool |  |  |  |  |

| `SA2A-C2-ACTUATION-CERTIFICATE-V1` | str_key | CERTIFICATE_DOMAIN = "SA2A-C2-ACTUATION-CERTIFICATE-V1" |  |  |  |  |

| `SA2A-PREPARED-EFFECT-V1` | str_key | PREPARED_EFFECT_DOMAIN = "SA2A-PREPARED-EFFECT-V1" |  |  |  |  |


### src/sa2a_security/epoch.rs

| `admit_epochs` | function | admit_epochs(expected: SecurityEpochs, certificate: SecurityEpochs) -> Result<(), SecurityRefusal> |  |  |  |  |

| `SecurityEpochs` | struct | SecurityEpochs { pub policy: u64, pub revocation: u64, pub generation: u64 } |  |  |  |  |


### src/sa2a_security/error.rs

| `SecurityRefusal` | enum | SecurityRefusal { InvalidDigest, InvalidKey, InvalidSignature, AlgorithmMismatch, UnknownKey, KeyRevoked, KeyNotYetValid, KeyExpired, PolicyEpochMismatch, RevocationEpochMismatch, GenerationMismatch, PrincipalMismatch, AudienceMismatch, CertificateExpired, CertificateNotYetValid, EffectDigestMismatch, DuplicateSigner, InsufficientQuorum, InsufficientCustodianIndependence, NonceReplay, ResourceAmplification, InvalidResourceEnvelope } |  |  |  |  |


### src/sa2a_security/key_registry.rs

| `KeyState` | enum | KeyState { Active, Revoked } |  |  |  |  |

| `from_records` | function | from_records(records: impl IntoIterator<Item = KeyRecord>) -> Self |  |  |  |  |

| `records` | function | records(&self) -> Vec<KeyRecord> |  |  |  |  |

| `resolve` | function | resolve(&self, key_id: &str, now_ms: u64, revocation_epoch: u64) -> Result<&KeyRecord, SecurityRefusal> |  |  |  |  |

| `KeyRecord` | struct | KeyRecord { pub key_id: String, pub custodian_id: String, pub algorithm: SignatureAlgorithm, pub public_key: Vec<u8>, pub state: KeyState, pub not_before_ms: u64, pub expires_at_ms: u64, pub revocation_epoch: u64 } |  |  |  |  |

| `KeyRegistry` | struct | KeyRegistry { keys: BTreeMap<String, KeyRecord> } |  |  |  |  |


### src/sa2a_security/mod.rs

| `algorithm::SignatureAlgorithm` | use | algorithm::SignatureAlgorithm |  |  |  |  |

| `certificate::ActuationCertificate` | use | certificate::ActuationCertificate |  |  |  |  |

| `error::SecurityRefusal` | use | error::SecurityRefusal |  |  |  |  |

| `key_registry::{KeyRecord, KeyRegistry, KeyState}` | use | key_registry::{KeyRecord, KeyRegistry, KeyState} |  |  |  |  |

| `prepared_effect::PreparedEffect` | use | prepared_effect::PreparedEffect |  |  |  |  |

| `resource::{BudgetLedger, ResourceEnvelope}` | use | resource::{BudgetLedger, ResourceEnvelope} |  |  |  |  |

| `signature::CertificateSignature` | use | signature::CertificateSignature |  |  |  |  |

| `verifier::{CertificateVerifier, VerificationReceipt}` | use | verifier::{CertificateVerifier, VerificationReceipt} |  |  |  |  |


### src/sa2a_security/nonce.rs

| `claim` | function | claim(&mut self, key_id: &str, nonce: &str) -> Result<(), SecurityRefusal> |  |  |  |  |

| `NonceFence` | struct | NonceFence { seen: BTreeSet<(String, String)> } |  |  |  |  |


### src/sa2a_security/prepared_effect.rs

| `digest` | function | digest(&self) -> Result<String, SecurityRefusal> |  |  |  |  |

| `PreparedEffect` | struct | PreparedEffect { pub version: u32, pub principal: String, pub capability: String, pub subject: Value, pub payload: Value } |  |  |  |  |


### src/sa2a_security/principal.rs

| `preserve_principal` | function | preserve_principal(expected: &str, observed: &str) -> Result<(), SecurityRefusal> |  |  |  |  |


### src/sa2a_security/quorum.rs

| `admit_distinct_quorum` | function | admit_distinct_quorum( threshold: u16, verified: &[(CertificateSignature, KeyRecord)], ) -> Result<Vec<VerifiedSigner>, SecurityRefusal> |  |  |  |  |

| `VerifiedSigner` | struct | VerifiedSigner { pub key_id: String, pub custodian_id: String } |  |  |  |  |


### src/sa2a_security/receipt.rs

| `from_signers` | function | from_signers( effect_digest: String, principal: String, policy_epoch: u64, revocation_epoch: u64, generation: u64, audience: String, signers: &[VerifiedSigner], ) -> Self |  |  |  |  |

| `VerificationReceipt` | struct | VerificationReceipt { pub effect_digest: String, pub principal: String, pub policy_epoch: u64, pub revocation_epoch: u64, pub generation: u64, pub audience: String, pub verified_key_ids: Vec<String>, pub verified_custodian_ids: Vec<String> } |  |  |  |  |


### src/sa2a_security/resource.rs

| `allocate` | function | allocate(&mut self, child: ResourceEnvelope) -> Result<(), SecurityRefusal> |  |  |  |  |

| `committed` | function | committed(&self) -> ResourceEnvelope |  |  |  |  |

| `contains` | function | contains(self, child: Self) -> bool |  |  |  |  |

| `new` | function | new(root: ResourceEnvelope) -> Result<Self, SecurityRefusal> |  |  |  |  |

| `validate` | function | validate(self) -> Result<Self, SecurityRefusal> |  |  |  |  |

| `BudgetLedger` | struct | BudgetLedger { root: ResourceEnvelope, committed: ResourceEnvelope } |  |  |  |  |

| `ResourceEnvelope` | struct | ResourceEnvelope { pub compute_units: u64, pub io_bytes: u64, pub effects: u64 } |  |  |  |  |


### src/sa2a_security/signature.rs

| `CertificateSignature` | struct | CertificateSignature { pub key_id: String, pub algorithm: SignatureAlgorithm, pub signature: Vec<u8> } |  |  |  |  |


### src/sa2a_security/verifier.rs

| `verify` | function | verify( &self, effect: &PreparedEffect, certificate: &ActuationCertificate, ) -> Result<VerificationReceipt, SecurityRefusal> |  |  |  |  |

| `CertificateVerifier` | struct | CertificateVerifier { pub registry: &'a KeyRegistry, pub expected_epochs: SecurityEpochs, pub expected_audience: &'a str, pub now_ms: u64 } |  |  |  |  |

| `super::receipt::VerificationReceipt` | use | super::receipt::VerificationReceipt |  |  |  |  |


### src/security_universe/admission.inc.rs

| `EvidenceSurface` | enum | EvidenceSurface { Oscal, Stix21, Taxii21, Ocsf, Sarif, CycloneDx, Spdx, Csaf, OpenTelemetry, Ocel, Json, JsonLd, Rdf, Csv, Xml, Syslog, NativeOpaque } |  |  |  |  |

| `MappingRelation` | enum | MappingRelation { Related, InformativeReference, Narrows, Broadens, Implements, Assesses, Mitigates, Detects, Observes, Evidences, Translates, Contradicts, Equivalent } |  |  |  |  |

| `SecurityStanding` | enum | SecurityStanding { Unknown, PartialAlive, Alive, Refused } |  |  |  |  |

| `admit_external_tool_evidence` | function | admit_external_tool_evidence( evidence: ExternalToolEvidence<'a>, ) -> Result<AdmittedExternalToolEvidence<'a>, String> |  |  |  |  |

| `admit_security_mapping` | function | admit_security_mapping(mapping: SecurityMapping<'a>) -> Result<AdmittedSecurityMapping<'a>, String> |  |  |  |  |

| `find_source` | function | find_source(id: &str) -> Option<&'static SecuritySource> |  |  |  |  |

| `find_tool` | function | find_tool(id: &str) -> Option<&'static SecurityToolIntegration> |  |  |  |  |

| `manufacture_security_intent` | function | manufacture_security_intent( tool_id: &'a str, target_subject: &'a str, operation: &'a str, parent_evidence_digest: &'a str, ) -> Result<SecurityIntent<'a>, String> |  |  |  |  |

| `qualify_federated_fortune5` | function | qualify_federated_fortune5( base: &crate::fortune5::Fortune5Qualification, evidence: &[CoverageEvidence<'_>], ) -> Result<FederatedFortune5Qualification, String> |  |  |  |  |

| `qualify_fortune5_security_universe` | function | qualify_fortune5_security_universe( evidence: &[CoverageEvidence<'_>], ) -> Result<SecurityQualification, String> |  |  |  |  |

| `validate_security_catalog` | function | validate_security_catalog() -> Result<(), String> |  |  |  |  |

| `validate_tool_catalog` | function | validate_tool_catalog() -> Result<(), String> |  |  |  |  |

| `AdmittedExternalToolEvidence` | struct | AdmittedExternalToolEvidence { pub evidence: ExternalToolEvidence<'a>, pub direct_actuation_authority: bool } |  |  |  |  |

| `AdmittedSecurityMapping` | struct | AdmittedSecurityMapping { pub mapping: SecurityMapping<'a> } |  |  |  |  |

| `CoverageEvidence` | struct | CoverageEvidence { pub source_id: &'a str, pub source_digest: &'a str, pub imported_objects: u64, pub mapped_objects: u64, pub verified_objects: u64, pub receipt_digest: Option<&'a str> } |  |  |  |  |

| `ExternalToolEvidence` | struct | ExternalToolEvidence { pub tool_id: &'a str, pub authority: &'a str, pub native_version: &'a str, pub native_object_id: &'a str, pub surface: EvidenceSurface, pub adapter_identity_digest: &'a str, pub payload_digest: &'a str, pub receipt_digest: &'a str } |  |  |  |  |

| `FederatedFortune5Qualification` | struct | FederatedFortune5Qualification { pub standing: SecurityStanding, pub base_standing: crate::fortune5::Standing, pub security: SecurityQualification } |  |  |  |  |

| `SecurityIntent` | struct | SecurityIntent { pub tool_id: &'a str, pub target_subject: &'a str, pub operation: &'a str, pub parent_evidence_digest: &'a str, pub direct_actuation_authority: bool } |  |  |  |  |

| `SecurityMapping` | struct | SecurityMapping { pub left_source_id: &'a str, pub left_object_id: &'a str, pub right_source_id: &'a str, pub right_object_id: &'a str, pub relation: MappingRelation, pub equivalence_proof_receipt: Option<&'a str> } |  |  |  |  |

| `SecurityQualification` | struct | SecurityQualification { pub standing: SecurityStanding, pub required_sources: usize, pub observed_sources: usize, pub fully_verified_sources: usize, pub missing_sources: Vec<&'static str>, pub partial_sources: Vec<&'static str> } |  |  |  |  |


### src/security_universe/catalog.inc.rs

| `SECURITY_SOURCES` | const | SECURITY_SOURCES: &[SecuritySource] |  |  |  |  |

| `SecurityKind` | enum | SecurityKind { GovernanceFramework, ControlCatalog, Assurance, Regulation, CloudBaseline, ApplicationSecurity, AiSecurity, IndustrialControl, ThreatKnowledge, VulnerabilityKnowledge, DetectionLanguage, TelemetrySchema, SupplyChain, EvidenceExchange, SecurityOntology, PolicyLanguage } |  |  |  |  |

| `VersionPolicy` | enum | VersionPolicy { Pinned, Rolling } |  |  |  |  |

| `nist-csf` | str_key | SECURITY_SOURCES = "nist-csf" |  |  |  |  |

| `SecuritySource` | struct | SecuritySource { pub id: &'static str, pub authority: &'static str, pub version: &'static str, pub version_policy: VersionPolicy, pub kind: SecurityKind, pub machine_surface: &'static str, pub source_uri: &'static str } |  |  |  |  |


### src/security_universe/tests.inc.rs

| `0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef` | str_key | D = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef" |  |  |  |  |


### src/security_universe/tools.inc.rs

| `FORTUNE5_SECURITY_CORE` | const | FORTUNE5_SECURITY_CORE: &[&str] |  |  |  |  |

| `SECURITY_TOOLS` | const | SECURITY_TOOLS: &[SecurityToolIntegration] |  |  |  |  |

| `ToolBoundary` | enum | ToolBoundary { ObserveOnly, AssessOnly, DetectOnly, ConstructIntentOnly, ExternalEnforcerBehindCastleAdmission } |  |  |  |  |

| `aws-security-hub` | str_key | SECURITY_TOOLS = "aws-security-hub" |  |  |  |  |

| `nist-csf` | str_key | FORTUNE5_SECURITY_CORE = "nist-csf" |  |  |  |  |

| `SecurityToolIntegration` | struct | SecurityToolIntegration { pub id: &'static str, pub ecosystem: &'static str, pub native_surface: &'static str, pub normalized_output: &'static str, pub boundary: ToolBoundary, pub direct_actuation_authority: bool } |  |  |  |  |


### src/strategic_board.rs

| `assess_board_reentry` | function | assess_board_reentry( constitution: &BoardConstitution, strategic_receipt: &BoardStrategicReceipt, event: &crate::board::MaterialityEvent, policy: &crate::board::MaterialityPolicy, ) -> Result<BoardReentryDecision, String> |  |  |  |  |

| `assess_counterstrategies` | function | assess_counterstrategies( constitution: &BoardConstitution, partition: &StrategyPartition, candidate: &CampaignCandidate, scenarios: &[CounterstrategyScenario], ) -> Result<CounterstrategyAssessment, String> |  |  |  |  |

| `build_strategic_board_package` | function | build_strategic_board_package( base: &crate::board::BoardPackage, constitution: &BoardConstitution, mandate: &StrategicMandatePacket, receipt: &BoardStrategicReceipt, portfolio: &CampaignPortfolioAnalysis, counterstrategy: &CounterstrategyAssessment, twin: &StrategicTwinSnapshot, reentry: &BoardReentryDecision, generated_at: &str, ) -> Result<StrategicBoardPackage, String> |  |  |  |  |

| `build_strategic_twin_snapshot` | function | build_strategic_twin_snapshot( constitution: &BoardConstitution, mandate: &StrategicMandatePacket, receipt: &BoardStrategicReceipt, portfolio: &CampaignPortfolioAnalysis, counterstrategy: &CounterstrategyAssessment, ) -> Result<StrategicTwinSnapshot, String> |  |  |  |  |

| `diff_strategic_twins` | function | diff_strategic_twins( previous: &StrategicTwinSnapshot, current: &StrategicTwinSnapshot, ) -> Result<StrategicBoardDelta, String> |  |  |  |  |

| `judge_counterstrategy` | function | judge_counterstrategy( constitution: &BoardConstitution, partition: &StrategyPartition, candidate: &CampaignCandidate, scenario: &CounterstrategyScenario, ) -> Result<CounterstrategyVerdict, String> |  |  |  |  |

| `qualify_campaign_portfolio` | function | qualify_campaign_portfolio( candidates: &[CampaignCandidate], verdicts: &[CampaignVerdict], policy: &CampaignPortfolioPolicy, ) -> Result<CampaignPortfolioAnalysis, String> |  |  |  |  |

| `verify_strategic_board_package_offline` | function | verify_strategic_board_package_offline( package: &StrategicBoardPackage, ) -> OfflineBoardPackageVerification |  |  |  |  |

| `BoardReentryDecision` | struct | BoardReentryDecision { pub required: bool, pub reasons: Vec<String>, pub escalate_by_epoch_ms: Option<i64>, pub materiality_score_bps: i64, pub triggering_dimensions: Vec<crate::board::MaterialityDimension> } |  |  |  |  |

| `CampaignPortfolioAnalysis` | struct | CampaignPortfolioAnalysis { pub standing: StrategicStanding, pub subject: String, pub constitution_digest: String, pub campaign_ids: Vec<String>, pub aggregate_capital_committed: u64, pub aggregate_reversible_capital: u64, pub reversible_capital_bps: u64, pub max_single_campaign_concentration_bps: u64, pub reasons: Vec<String>, pub analysis_digest: String } |  |  |  |  |

| `CampaignPortfolioPolicy` | struct | CampaignPortfolioPolicy { pub aggregate_capital_at_risk_limit: u64, pub max_single_campaign_concentration_bps: u64, pub min_reversible_capital_bps: u64 } |  |  |  |  |

| `CounterstrategyAssessment` | struct | CounterstrategyAssessment { pub campaign_id: String, pub campaign_digest: String, pub standing: StrategicStanding, pub verdicts: Vec<CounterstrategyVerdict>, pub assessment_digest: String } |  |  |  |  |

| `CounterstrategyScenario` | struct | CounterstrategyScenario { pub scenario_id: String, pub local_premise_mutations: BTreeMap<String, String>, pub falsifier_triggered: bool, pub prohibited_outcomes_reached: Vec<String>, pub authority_expansion_attempts: Vec<String>, pub additional_capital_required: u64, pub remaining_options: u32 } |  |  |  |  |

| `CounterstrategyVerdict` | struct | CounterstrategyVerdict { pub scenario_id: String, pub standing: StrategicStanding, pub refusals: Vec<String>, pub verdict_digest: String } |  |  |  |  |

| `OfflineBoardPackageVerification` | struct | OfflineBoardPackageVerification { pub standing: StrategicStanding, pub reasons: Vec<String> } |  |  |  |  |

| `StrategicBoardDelta` | struct | StrategicBoardDelta { pub subject: String, pub mandate_id: String, pub changed_dimensions: Vec<String>, pub requires_board_attention: bool, pub previous_snapshot_digest: String, pub current_snapshot_digest: String } |  |  |  |  |

| `StrategicBoardPackage` | struct | StrategicBoardPackage { pub profile: &'static str, pub subject: String, pub generated_at: String, pub fortune5_board_package_digest: String, pub constitution_digest: String, pub mandate_packet_digest: String, pub strategic_receipt_digest: String, pub portfolio_analysis_digest: String, pub counterstrategy_assessment_digest: String, pub twin_snapshot_digest: String, pub strategic_standing: StrategicStanding, pub board_reentry_required: bool, pub authority_ceiling: &'static str, pub actuation: &'static str, pub package_digest: String } |  |  |  |  |

| `StrategicTwinSnapshot` | struct | StrategicTwinSnapshot { pub subject: String, pub mandate_id: String, pub campaign_id: String, pub standing: StrategicStanding, pub aggregate_capital_committed: u64, pub aggregate_reversible_capital: u64, pub options_remaining: u32, pub falsified_premises: Vec<String>, pub failed_counterstrategy_scenarios: Vec<String>, pub prohibited_outcome_witnesses: Vec<String>, pub authority_expansions: Vec<String>, pub snapshot_digest: String } |  |  |  |  |


### src/strategic_command.rs

| `BOARD_LENSES` | const | BOARD_LENSES: [BoardLens; 8] |  |  |  |  |

| `STRATEGIC_ACTUATION` | const | STRATEGIC_ACTUATION: &str |  |  |  |  |

| `STRATEGIC_AUTHORITY_CEILING` | const | STRATEGIC_AUTHORITY_CEILING: &str |  |  |  |  |

| `STRATEGIC_SUCCESSOR_BOUNDARY` | const | STRATEGIC_SUCCESSOR_BOUNDARY: &str |  |  |  |  |

| `STRATEGY_OPERATORS` | const | STRATEGY_OPERATORS: &[StrategyOperator] |  |  |  |  |

| `BoardAvatar` | enum | BoardAvatar { Audit, Risk, CapitalAllocation, Resilience, Governance, Safety, CompetitiveStrategy, LeadIndependent } |  |  |  |  |

| `RecompileScope` | enum | RecompileScope { None, Campaign(Vec<String>), Strategic } |  |  |  |  |

| `ReplanLevel` | enum | ReplanLevel { PolicyBranch, SuffixReuse, FollowBiasedTailRepair, BoundedFullReplan, HierarchyRecompile, StrategicRecompile } |  |  |  |  |

| `StrategicAxis` | enum | StrategicAxis { Position, Topology, OptionSpace, Belief, Tempo, Objective, Authority, Reversibility } |  |  |  |  |

| `StrategicStanding` | enum | StrategicStanding { Alive, Refused } |  |  |  |  |

| `admit_board_selection` | function | admit_board_selection( constitution: &BoardConstitution, candidates: &[CampaignCandidate], verdicts: &[CampaignVerdict], request: BoardSelectionRequest, ) -> Result<StrategicMandatePacket, String> |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `bind_strategic_mandate_construct_request` | function | bind_strategic_mandate_construct_request( mut request: crate::castle::ConstructRequest, mandate: &StrategicMandatePacket, ) -> Result<crate::castle::ConstructRequest, String> |  |  |  |  |

| `candidate_digest` | function | candidate_digest(&self) -> Result<String, String> |  |  |  |  |

| `compile_board_constitution` | function | compile_board_constitution(input: ConstitutionInput) -> Result<BoardConstitution, String> |  |  |  |  |

| `compile_board_strategic_receipt` | function | compile_board_strategic_receipt( constitution: &BoardConstitution, candidate: &CampaignCandidate, mandate: &StrategicMandatePacket, input: BoardReceiptInput, ) -> Result<BoardStrategicReceipt, String> |  |  |  |  |

| `compile_strategy_doctrine` | function | compile_strategy_doctrine( constitution: &BoardConstitution, premise_digests: BTreeMap<String, String>, capability_ids: Vec<String>, provenance: Vec<String>, ) -> Result<StrategyDoctrine, String> |  |  |  |  |

| `construct_campaign_candidate` | function | construct_campaign_candidate( constitution: &BoardConstitution, doctrine: &StrategyDoctrine, partition: &StrategyPartition, candidate_id: impl Into<String>, assumptions: Vec<String>, falsifier: impl Into<String>, objectives: BTreeMap<String, i64>, expected_outcomes: Vec<String>, capital_committed: u64, reversible_capital: u64, ) -> Result<CampaignCandidate, String> |  |  |  |  |

| `determine_recompile_scope` | function | determine_recompile_scope( doctrine: &StrategyDoctrine, partitions: &[StrategyPartition], current_global_premises: &BTreeMap<String, String>, current_local_premises: &BTreeMap<String, String>, ) -> RecompileScope |  |  |  |  |

| `judge_campaign_candidate` | function | judge_campaign_candidate( constitution: &BoardConstitution, doctrine: &StrategyDoctrine, partition: &StrategyPartition, candidate: &CampaignCandidate, current_local_premises: &BTreeMap<String, String>, ) -> CampaignVerdict |  |  |  |  |

| `partition_strategy` | function | partition_strategy( doctrine: &StrategyDoctrine, strategy_id: impl Into<String>, strategy: impl Into<String>, local_premise_digests: BTreeMap<String, String>, local_constraints: Vec<String>, operator_ids: Vec<String>, ) -> Result<StrategyPartition, String> |  |  |  |  |

| `route_replan` | function | route_replan(evidence: DivergenceEvidence) -> ReplanLevel |  |  |  |  |

| `to_json` | function | to_json(&self) -> Value |  |  |  |  |

| `BRCE` | str_key | STRATEGIC_SUCCESSOR_BOUNDARY = "BRCE" |  |  |  |  |

| `CONSTRUCT` | str_key | STRATEGIC_AUTHORITY_CEILING = "CONSTRUCT" |  |  |  |  |

| `Can we prove the corporation did what the board authorized?` | str_key | BOARD_LENSES = "Can we prove the corporation did what the board authorized?" |  |  |  |  |

| `NONE` | str_key | STRATEGIC_ACTUATION = "NONE" |  |  |  |  |

| `OBSERVE` | str_key | ALLOWED_ACTIONS = "OBSERVE" |  |  |  |  |

| `refuse-last-war` | str_key | STRATEGY_OPERATORS = "refuse-last-war" |  |  |  |  |

| `BoardConstitution` | struct | BoardConstitution { pub subject: String, pub mandate_id: String, pub objectives: Vec<String>, pub prohibited_outcomes: Vec<String>, pub invariants: Vec<String>, pub delegated_authority: Vec<String>, pub nondelegable_decisions: Vec<String>, pub capital_at_risk_limit: u64, pub escalation_conditions: Vec<String>, pub withdrawal_conditions: Vec<String>, pub evidence_requirements: Vec<String>, pub authority_ceiling: &'static str, pub constitution_digest: String } |  |  |  |  |

| `BoardLens` | struct | BoardLens { pub avatar: BoardAvatar, pub question: &'static str, pub governed_surface: &'static str } |  |  |  |  |

| `BoardReceiptInput` | struct | BoardReceiptInput { pub falsified_premises: Vec<String>, pub material_exceptions: Vec<String>, pub options_remaining: u32, pub prohibited_outcome_witnesses: Vec<String>, pub authority_expansions: Vec<String>, pub next_board_decision: Option<String>, pub evidence_digest: String } |  |  |  |  |

| `BoardSelectionRequest` | struct | BoardSelectionRequest { pub candidate_id: String, pub selection_authority_digest: String, pub selected_by: String, pub selected_at: String } |  |  |  |  |

| `BoardStrategicReceipt` | struct | BoardStrategicReceipt { pub standing: StrategicStanding, pub mandate_id: String, pub constitution_digest: String, pub campaign_id: String, pub campaign_digest: String, pub falsified_premises: Vec<String>, pub material_exceptions: Vec<String>, pub options_remaining: u32, pub prohibited_outcome_witnesses: Vec<String>, pub authority_expansions: Vec<String>, pub next_board_decision: Option<String>, pub evidence_digest: String, pub authority_ceiling: &'static str, pub actuation: &'static str, pub receipt_digest: String } |  |  |  |  |

| `CampaignCandidate` | struct | CampaignCandidate { pub candidate_id: String, pub subject: String, pub strategy_id: String, pub constitution_digest: String, pub doctrine_digest: String, pub partition_digest: String, pub preserved_invariants: Vec<String>, pub capabilities: Vec<String>, pub actions: Vec<String>, pub assumptions: Vec<String>, pub falsifier: String, pub objectives: BTreeMap<String, i64>, pub expected_outcomes: Vec<String>, pub capital_committed: u64, pub reversible_capital: u64, pub authority_ceiling: String, pub observed_partition_digests: Vec<String> } |  |  |  |  |

| `CampaignVerdict` | struct | CampaignVerdict { pub standing: StrategicStanding, pub candidate_id: String, pub candidate_digest: String, pub refusals: Vec<String> } |  |  |  |  |

| `ConstitutionInput` | struct | ConstitutionInput { pub subject: String, pub mandate_id: String, pub objectives: Vec<String>, pub prohibited_outcomes: Vec<String>, pub invariants: Vec<String>, pub delegated_authority: Vec<String>, pub nondelegable_decisions: Vec<String>, pub capital_at_risk_limit: u64, pub escalation_conditions: Vec<String>, pub withdrawal_conditions: Vec<String>, pub evidence_requirements: Vec<String> } |  |  |  |  |

| `DivergenceEvidence` | struct | DivergenceEvidence { pub policy_branch_available: bool, pub valid_suffix_available: bool, pub tail_repair_available: bool, pub hierarchy_premise_broken: bool, pub strategic_premise_broken: bool } |  |  |  |  |

| `StrategicMandatePacket` | struct | StrategicMandatePacket { pub subject: String, pub mandate_id: String, pub constitution_digest: String, pub candidate_id: String, pub candidate_digest: String, pub selection_authority_digest: String, pub selected_by: String, pub selected_at: String, pub evidence_requirements: Vec<String>, pub authority_ceiling: &'static str, pub actuation: &'static str, pub successor_boundary: &'static str, pub packet_digest: String } |  |  |  |  |

| `StrategyDoctrine` | struct | StrategyDoctrine { pub subject: String, pub constitution_digest: String, pub premise_digests: BTreeMap<String, String>, pub invariants: Vec<String>, pub capability_ids: Vec<String>, pub provenance: Vec<String>, pub authority_ceiling: &'static str, pub doctrine_digest: String } |  |  |  |  |

| `StrategyOperator` | struct | StrategyOperator { pub id: &'static str, pub provenance: &'static str, pub reads: &'static [StrategicAxis], pub writes: &'static [StrategicAxis] } |  |  |  |  |

| `StrategyPartition` | struct | StrategyPartition { pub strategy_id: String, pub strategy: String, pub doctrine_digest: String, pub local_premise_digests: BTreeMap<String, String>, pub local_constraints: Vec<String>, pub operator_ids: Vec<String>, pub partition_digest: String } |  |  |  |  |

| `crate::strategic_board::{ assess_board_reentry, assess_counterstrategies, build_strategic_board_package, build_strategic_twin_snapshot, diff_strategic_twins, judge_counterstrategy, qualify_campaign_portfolio, verify_strategic_board_package_offline, BoardReentryDecision, CampaignPortfolioAnalysis, CampaignPortfolioPolicy, CounterstrategyAssessment, CounterstrategyScenario, CounterstrategyVerdict, OfflineBoardPackageVerification, StrategicBoardDelta, StrategicBoardPackage, StrategicTwinSnapshot, }` | use | crate::strategic_board::{ assess_board_reentry, assess_counterstrategies, build_strategic_board_package, build_strategic_twin_snapshot, diff_strategic_twins, judge_counterstrategy, qualify_campaign_portfolio, verify_strategic_board_package_offline, BoardReentryDecision, CampaignPortfolioAnalysis, CampaignPortfolioPolicy, CounterstrategyAssessment, CounterstrategyScenario, CounterstrategyVerdict, OfflineBoardPackageVerification, StrategicBoardDelta, StrategicBoardPackage, StrategicTwinSnapshot, } |  |  |  |  |


### src/v26_8_18/airgap.rs

| `admit_airgap_result` | function | admit_airgap_result(bundle: &AirgapBundle, result: &AirgapResult) -> AirgapAdmission |  |  |  |  |

| `manufacture_airgap_bundle` | function | manufacture_airgap_bundle( bundle_id: String, constitution: &GlobalConstitution, o_star_snapshot: Value, construct_graph: Value, prohibited_goals: Value, ) -> Result<AirgapBundle, String> |  |  |  |  |

| `AirgapAdmission` | struct | AirgapAdmission { pub standing: ReleaseStanding, pub reason: String } |  |  |  |  |

| `AirgapBundle` | struct | AirgapBundle { pub kind: String, pub release: String, pub bundle_id: String, pub constitution_id: String, pub ontology_version: String, pub provider_semantics: BTreeMap<String, String>, pub invariant_set_digest: String, pub o_star_snapshot: Value, pub construct_graph: Value, pub prohibited_goals: Value, pub network_dependencies: Vec<String>, pub secret_dependencies: Vec<String>, pub bundle_digest: String } |  |  |  |  |

| `AirgapResult` | struct | AirgapResult { pub bundle_id: String, pub input_bundle_digest: String, pub result_digest: String, pub network_used: bool, pub secret_material_used: bool } |  |  |  |  |


### src/v26_8_18/brce.rs

| `execute_command_process` | function | execute_command_process( process: &PowlProcess, state: &WorldState, envelope: &TestEnvelope, admission: &ConstructAdmission, policy: CommandAdapterPolicy, blake3: &dyn Blake3Provider, signer: &dyn ReceiptSigner, now: impl Fn() -> i64, ) -> Result<(ReceiptedOcelLog, Vec<BrceTransitionRecord>), String> |  |  |  |  |

| `execute_command_process_durable` | function | execute_command_process_durable( process: &PowlProcess, state: &WorldState, envelope: &TestEnvelope, admission: &ConstructAdmission, policy: CommandAdapterPolicy, durable_journal_root: impl AsRef<Path>, blake3: &dyn Blake3Provider, signer: &dyn ReceiptSigner, now: impl Fn() -> i64, ) -> Result<(ReceiptedOcelLog, Vec<BrceTransitionRecord>), String> |  |  |  |  |

| `fortune5_adapter_catalog` | function | fortune5_adapter_catalog() -> Vec<ProviderAdapterDescriptor> |  |  |  |  |

| `journal` | function | journal(&self) -> Vec<BrceTransitionRecord> |  |  |  |  |

| `new` | function | new(inner: &'a dyn GymActAdapter, blake3: &'a dyn Blake3Provider, signer: &'a dyn ReceiptSigner) -> Self |  |  |  |  |

| `new_durable` | function | new_durable( inner: &'a dyn GymActAdapter, blake3: &'a dyn Blake3Provider, signer: &'a dyn ReceiptSigner, durable_journal_root: PathBuf, ) -> Self |  |  |  |  |

| `validate_command_adapter_policy` | function | validate_command_adapter_policy(policy: &CommandAdapterPolicy) -> ReleaseStanding |  |  |  |  |

| `BrceGymActAdapter` | struct | BrceGymActAdapter { inner: &'a dyn GymActAdapter, blake3: &'a dyn Blake3Provider, signer: &'a dyn ReceiptSigner, journal: Mutex<Vec<BrceTransitionRecord>>, durable_journal_root: Option<PathBuf> } |  |  |  |  |

| `BrceTransitionRecord` | struct | BrceTransitionRecord { pub transition_id: String, pub prepare_receipt: Receipt, pub outcome_receipt: Option<Receipt>, pub standing: ReleaseStanding, pub reason: String } |  |  |  |  |

| `CommandAdapterPolicy` | struct | CommandAdapterPolicy { pub adapter_id: String, pub provider: String, pub workload_identity: String, pub commands: BTreeMap<String, CommandSpec> } |  |  |  |  |

| `CommandSpec` | struct | CommandSpec { pub transition_id: String, pub program: String, pub args: Vec<String>, pub allowed_exit_codes: BTreeSet<i32>, pub max_output_bytes: usize, pub timeout_ms: u64 } |  |  |  |  |

| `DurableBrceOutcomeRecord` | struct | DurableBrceOutcomeRecord { pub kind: String, pub release: String, pub transition_id: String, pub prepare_receipt_digest: String, pub outcome_receipt: DurableBrceReceipt, pub provider_status: String } |  |  |  |  |

| `DurableBrcePrepareRecord` | struct | DurableBrcePrepareRecord { pub kind: String, pub release: String, pub transition_id: String, pub subject: String, pub construct_digest: String, pub process_digest: String, pub prepare_receipt: DurableBrceReceipt } |  |  |  |  |

| `DurableBrceReceipt` | struct | DurableBrceReceipt { pub algorithm: String, pub artifact_digest: String, pub receipt_digest: String, pub epistemic_class: String, pub subject: String, pub parent_digests: Vec<String>, pub origin_key_id: String, pub origin_signature: String } |  |  |  |  |

| `ProviderAdapterDescriptor` | struct | ProviderAdapterDescriptor { pub kind: String, pub program: String, pub authority_model: String, pub ambient_credentials_allowed: bool } |  |  |  |  |


### src/v26_8_18/chaos.rs

| `ChaosScenario` | enum | ChaosScenario { RegionLoss, WanPartition, StalePolicy, RevokedAuthority, ClockSkew, ProviderThrottle, PartialActuation, ReceiptStoreLoss } |  |  |  |  |

| `qualify_chaos` | function | qualify_chaos(evidence: &[ChaosEvidence]) -> ChaosQualification |  |  |  |  |

| `required_chaos_scenarios` | function | required_chaos_scenarios() -> Vec<ChaosScenario> |  |  |  |  |

| `ChaosEvidence` | struct | ChaosEvidence { pub scenario: ChaosScenario, pub exercised: bool, pub failed_closed: bool, pub receipt_digest: String, pub detail: String } |  |  |  |  |

| `ChaosQualification` | struct | ChaosQualification { pub standing: ReleaseStanding, pub reasons: Vec<String>, pub scenarios: BTreeMap<ChaosScenario, ReleaseStanding> } |  |  |  |  |


### src/v26_8_18/crypto.rs

| `SignatureSuite` | enum | SignatureSuite { Ed25519, EcdsaP256, RsaPssSha256, MlDsa, SlhDsa } |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `dual_artifact_identity` | function | dual_artifact_identity(bytes: &[u8]) -> ArtifactIdentity |  |  |  |  |

| `implemented_signature_suites` | function | implemented_signature_suites() -> BTreeSet<SignatureSuite> |  |  |  |  |

| `qualify_crypto_profile` | function | qualify_crypto_profile(profile: &CryptoProfile) -> CryptoQualification |  |  |  |  |

| `qualify_pqc_runtime` | function | qualify_pqc_runtime() -> PqcRuntimeQualification |  |  |  |  |

| `sign_pqc_message` | function | sign_pqc_message( suite: SignatureSuite, seed: [u8; 32], message: &[u8], ) -> Result<PqcSignatureProof, String> |  |  |  |  |

| `verify_pqc_message` | function | verify_pqc_message(proof: &PqcSignatureProof, message: &[u8]) -> bool |  |  |  |  |

| `ArtifactIdentity` | struct | ArtifactIdentity { pub blake3_256: String, pub sha256: String } |  |  |  |  |

| `CryptoProfile` | struct | CryptoProfile { pub required_identity_hashes: BTreeSet<String>, pub accepted_signature_suites: BTreeSet<SignatureSuite>, pub require_post_quantum: bool } |  |  |  |  |

| `CryptoQualification` | struct | CryptoQualification { pub standing: ReleaseStanding, pub reasons: Vec<String>, pub implemented_signature_suites: BTreeSet<SignatureSuite> } |  |  |  |  |

| `PqcRuntimeQualification` | struct | PqcRuntimeQualification { pub standing: ReleaseStanding, pub ml_dsa_65: bool, pub slh_dsa_shake_128f: bool, pub reasons: Vec<String> } |  |  |  |  |

| `PqcSignatureProof` | struct | PqcSignatureProof { pub suite: SignatureSuite, pub parameter_set: String, pub message_blake3: String, pub public_key_hex: String, pub signature_hex: String } |  |  |  |  |


### src/v26_8_18/dfcm.rs

| `ProbePurpose` | enum | ProbePurpose { Version, AuthorityContext, SelfTest } |  |  |  |  |

| `ReadOnlyProbeKind` | enum | ReadOnlyProbeKind { AwsCliVersion, AwsAuthorityContext, AzureCliVersion, AzureAuthorityContext, GcpCliVersion, GcpAuthorityContext, KubernetesCliVersion, KubernetesAuthorityContext, GitHubCliVersion, GitHubAuthorityContext, LocalSelfTest } |  |  |  |  |

| `args` | function | args(self) -> Vec<&'static str> |  |  |  |  |

| `bind_command_policy` | function | bind_command_policy( binding: &AdapterBinding, policy: &CommandAdapterPolicy, ) -> Result<CommandAdapterPolicy, String> |  |  |  |  |

| `default_program` | function | default_program(self) -> &'static str |  |  |  |  |

| `live_observation_index` | function | live_observation_index( observations: &[ProviderProbeObservation], ) -> BTreeMap<String, BTreeSet<String>> |  |  |  |  |

| `manufacture_live_probe_plan` | function | manufacture_live_probe_plan(manifest: &DeploymentManifest) -> Vec<ReadOnlyProbeSpec> |  |  |  |  |

| `provider_kind` | function | provider_kind(self) -> &'static str |  |  |  |  |

| `purpose` | function | purpose(self) -> ProbePurpose |  |  |  |  |

| `qualify_dfcm_closure` | function | qualify_dfcm_closure( manifest: &DeploymentManifest, observations: &[ProviderProbeObservation], crypto_profile: &CryptoProfile, now_epoch_ms: i64, max_live_evidence_age_ms: i64, ) -> DfcmClosureQualification |  |  |  |  |

| `qualify_live_deployment` | function | qualify_live_deployment( manifest: &DeploymentManifest, observations: &[ProviderProbeObservation], now_epoch_ms: i64, max_age_ms: i64, ) -> LiveDeploymentQualification |  |  |  |  |

| `qualify_protocol_fence` | function | qualify_protocol_fence() -> ProtocolFenceQualification |  |  |  |  |

| `run_read_only_probe` | function | run_read_only_probe( spec: &ReadOnlyProbeSpec, observed_at_epoch_ms: i64, ) -> Result<ProviderProbeObservation, String> |  |  |  |  |

| `DfcmClosureQualification` | struct | DfcmClosureQualification { pub standing: ReleaseStanding, pub static_deployment: ReleaseStanding, pub live_deployment: ReleaseStanding, pub crypto_profile: ReleaseStanding, pub pqc_runtime: PqcRuntimeQualification, pub protocol_fence: ProtocolFenceQualification, pub findings: Vec<String> } |  |  |  |  |

| `LiveDeploymentQualification` | struct | LiveDeploymentQualification { pub standing: ReleaseStanding, pub static_manifest_digest: String, pub cells: usize, pub adapters_expected: usize, pub adapters_alive: usize, pub findings: Vec<String> } |  |  |  |  |

| `ProtocolDispatchSummary` | struct | ProtocolDispatchSummary { pub origin: InterfaceOrigin, pub select_standing: ReleaseStanding, pub construct_standing: ReleaseStanding, pub do_standing: ReleaseStanding } |  |  |  |  |

| `ProtocolFenceQualification` | struct | ProtocolFenceQualification { pub standing: ReleaseStanding, pub dispatches: Vec<ProtocolDispatchSummary>, pub findings: Vec<String> } |  |  |  |  |

| `ProviderProbeObservation` | struct | ProviderProbeObservation { pub cell_id: String, pub adapter_id: String, pub provider_kind: String, pub workload_identity: String, pub provider_semantics_version: String, pub kind: ReadOnlyProbeKind, pub purpose: ProbePurpose, pub observed_at_epoch_ms: i64, pub exit_code: i32, pub stdout_blake3: String, pub stderr_blake3: String, pub identity_binding_verified: bool, pub observation_digest: String, pub standing: ReleaseStanding, pub reason: String } |  |  |  |  |

| `ReadOnlyProbeSpec` | struct | ReadOnlyProbeSpec { pub cell_id: String, pub adapter_id: String, pub workload_identity: String, pub provider_semantics_version: String, pub kind: ReadOnlyProbeKind, pub program_override: Option<String>, pub expected_identity_marker: Option<String>, pub max_output_bytes: usize, pub timeout_ms: u64 } |  |  |  |  |


### src/v26_8_18/evidence.rs

| `persist_evidence` | function | persist_evidence(root: impl AsRef<Path>, record: &DurableEvidenceRecord) -> Result<EvidenceCommit, String> |  |  |  |  |

| `verify_evidence_file` | function | verify_evidence_file(path: impl AsRef<Path>) -> Result<EvidenceVerification, String> |  |  |  |  |

| `DurableEvidenceRecord` | struct | DurableEvidenceRecord { pub cell_id: String, pub subject: String, pub construct_digest: String, pub ocel_receipt_digest: String, pub brce_prepare_receipt_digests: Vec<String>, pub brce_outcome_receipt_digests: Vec<String>, pub event_count: usize } |  |  |  |  |

| `EvidenceCommit` | struct | EvidenceCommit { pub standing: ReleaseStanding, pub record_identity: ArtifactIdentity, pub path: String, pub reason: String } |  |  |  |  |

| `EvidenceVerification` | struct | EvidenceVerification { pub standing: ReleaseStanding, pub record_identity: ArtifactIdentity, pub path: String, pub record: DurableEvidenceRecord, pub reason: String } |  |  |  |  |


### src/v26_8_18/mod.rs

| `RELEASE_KIND` | const | RELEASE_KIND: &str |  |  |  |  |

| `RELEASE_VERSION` | const | RELEASE_VERSION: &str |  |  |  |  |

| `26.10.8+dfcm.1` | str_key | RELEASE_VERSION = "26.10.8+dfcm.1" |  |  |  |  |

| `CASTLE_FORTUNE5_GLOBAL_V1` | str_key | RELEASE_KIND = "CASTLE_FORTUNE5_GLOBAL_V1" |  |  |  |  |

| `airgap::*` | use | airgap::* |  |  |  |  |

| `brce::*` | use | brce::* |  |  |  |  |

| `chaos::*` | use | chaos::* |  |  |  |  |

| `crypto::*` | use | crypto::* |  |  |  |  |

| `dfcm::*` | use | dfcm::* |  |  |  |  |

| `evidence::*` | use | evidence::* |  |  |  |  |

| `protocol::*` | use | protocol::* |  |  |  |  |

| `replication::*` | use | replication::* |  |  |  |  |

| `runtime::*` | use | runtime::* |  |  |  |  |

| `topology::*` | use | topology::* |  |  |  |  |


### src/v26_8_18/protocol.rs

| `IntentMode` | enum | IntentMode { Select, Construct, Do } |  |  |  |  |

| `InterfaceOrigin` | enum | InterfaceOrigin { Cli, Api, Mcp, A2a, Human, Planner, Replay } |  |  |  |  |

| `a2a_agent_card` | function | a2a_agent_card() -> A2aAgentCard |  |  |  |  |

| `admit_interface_intent` | function | admit_interface_intent(intent: &InterfaceIntent) -> InterfaceAdmission |  |  |  |  |

| `admit_observation` | function | admit_observation( observation: &ObservationEnvelope, allowed_sources: &BTreeSet<String>, now_epoch_ms: i64, max_age_ms: i64, ) -> AdmittedObservation |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `dispatch_interface_intent` | function | dispatch_interface_intent(intent: &InterfaceIntent) -> ProtocolDispatch |  |  |  |  |

| `mcp_tool_catalog` | function | mcp_tool_catalog() -> Vec<McpToolDescriptor> |  |  |  |  |

| `A2aAgentCard` | struct | A2aAgentCard { pub name: String, pub version: String, pub capabilities: Vec<String>, pub default_authority: String } |  |  |  |  |

| `AdmittedObservation` | struct | AdmittedObservation { pub standing: ReleaseStanding, pub observation_id: String, pub subject: String, pub source: String, pub payload_digest: String, pub reason: String } |  |  |  |  |

| `InterfaceAdmission` | struct | InterfaceAdmission { pub standing: ReleaseStanding, pub request_id: String, pub reason: String, pub normalized_intent_digest: String } |  |  |  |  |

| `InterfaceIntent` | struct | InterfaceIntent { pub request_id: String, pub origin: InterfaceOrigin, pub mode: IntentMode, pub subject: String, pub operation: String, pub payload: Value, pub construct_admission_digest: Option<String>, pub prepare_receipt_digest: Option<String> } |  |  |  |  |

| `McpToolDescriptor` | struct | McpToolDescriptor { pub name: String, pub mode: IntentMode, pub consequential: bool } |  |  |  |  |

| `ObservationEnvelope` | struct | ObservationEnvelope { pub observation_id: String, pub source: String, pub subject: String, pub observed_at_epoch_ms: i64, pub epistemic_class: String, pub payload: Value } |  |  |  |  |

| `ProtocolDispatch` | struct | ProtocolDispatch { pub standing: ReleaseStanding, pub reason: String, pub request_id: String, pub normalized_intent_digest: String, pub result: Value } |  |  |  |  |


### src/v26_8_18/replication.rs

| `admit_receipt_checkpoint` | function | admit_receipt_checkpoint(state: &mut ReplicaState, checkpoint: ReceiptCheckpoint) -> ReplicationAdmission |  |  |  |  |

| `load_durable_replica` | function | load_durable_replica(path: impl AsRef<Path>) -> Result<ReplicaState, String> |  |  |  |  |

| `persist_receipt_checkpoint` | function | persist_receipt_checkpoint( path: impl AsRef<Path>, receiver_id: &str, checkpoint: ReceiptCheckpoint, ) -> Result<DurableReplicaCommit, String> |  |  |  |  |

| `reconcile_transition` | function | reconcile_transition(evidence: &ReconciliationEvidence) -> ReconciliationDecision |  |  |  |  |

| `DurableReplicaCommit` | struct | DurableReplicaCommit { pub standing: ReleaseStanding, pub reason: String, pub path: String, pub state_blake3: String, pub admission: ReplicationAdmission } |  |  |  |  |

| `ReceiptCheckpoint` | struct | ReceiptCheckpoint { pub cell_id: String, pub sequence: u64, pub head_digest: String, pub constitution_id: String, pub observed_at_epoch_ms: i64 } |  |  |  |  |

| `ReconciliationDecision` | struct | ReconciliationDecision { pub standing: ReleaseStanding, pub reason: String, pub replay_allowed: bool } |  |  |  |  |

| `ReconciliationEvidence` | struct | ReconciliationEvidence { pub transition_id: String, pub prepare_receipt_digest: String, pub outcome_receipt_digest: Option<String>, pub provider_observed: Option<bool>, pub retry_is_proven_idempotent: bool } |  |  |  |  |

| `ReplicaState` | struct | ReplicaState { pub receiver_id: String, pub checkpoints: BTreeMap<String, ReceiptCheckpoint> } |  |  |  |  |

| `ReplicationAdmission` | struct | ReplicationAdmission { pub standing: ReleaseStanding, pub reason: String, pub cell_id: String, pub sequence: u64 } |  |  |  |  |


### src/v26_8_18/runtime.rs

| `decode_seed_hex` | function | decode_seed_hex(value: &str) -> Result<[u8; 32], String> |  |  |  |  |

| `execute_runtime_request` | function | execute_runtime_request( request: &RuntimeExecutionRequest, key_id: String, seed: [u8; 32], expected_construct_digest: &str, now_epoch_ms: i64, ) -> Result<RuntimeDoSummary, String> |  |  |  |  |

| `from_seed` | function | from_seed(key_id: String, seed: [u8; 32]) -> Result<Self, String> |  |  |  |  |

| `manufacture_runtime_construct` | function | manufacture_runtime_construct( request: &RuntimeExecutionRequest, key_id: String, seed: [u8; 32], ) -> Result<ConstructManufactureSummary, String> |  |  |  |  |

| `to_envelope` | function | to_envelope(&self) -> TestEnvelope |  |  |  |  |

| `to_powl` | function | to_powl(&self) -> PowlProcess |  |  |  |  |

| `verifier` | function | verifier(&self) -> Ed25519RuntimeVerifier |  |  |  |  |

| `ConstructManufactureSummary` | struct | ConstructManufactureSummary { pub standing: ReleaseStanding, pub construct_digest: String, pub construct_receipt_digest: String, pub process_digest: String, pub replay_identity_digest: String, pub subject: String, pub authority: String } |  |  |  |  |

| `Ed25519RuntimeSigner` | struct | Ed25519RuntimeSigner { key_id: String, signing_key: SigningKey } |  |  |  |  |

| `Ed25519RuntimeVerifier` | struct | Ed25519RuntimeVerifier { key_id: String, verifying_key: VerifyingKey } |  |  |  |  |

| `NativeBlake3` | struct |  |  |  |  |  |

| `PortableActivity` | struct | PortableActivity { pub id: String, pub transition_id: String, pub predecessors: Vec<String> } |  |  |  |  |

| `PortableEnvelope` | struct | PortableEnvelope { pub system_id: String, pub allowed_transition_ids: BTreeSet<String>, pub max_steps: u32, pub expires_at_epoch_ms: i64 } |  |  |  |  |

| `PortableProcess` | struct | PortableProcess { pub id: String, pub goal_id: String, pub activities: Vec<PortableActivity> } |  |  |  |  |

| `RuntimeDoSummary` | struct | RuntimeDoSummary { pub standing: ReleaseStanding, pub construct: ConstructManufactureSummary, pub ocel_receipt_digest: String, pub event_count: usize, pub brce_prepare_receipt_digests: Vec<String>, pub brce_outcome_receipt_digests: Vec<String>, pub evidence_commit: EvidenceCommit } |  |  |  |  |

| `RuntimeExecutionRequest` | struct | RuntimeExecutionRequest { pub cell_id: String, pub evidence_dir: String, pub subject: String, pub authority: String, pub o_star: Value, pub config_graph: Value, pub ontology: Value, pub process: PortableProcess, pub envelope: PortableEnvelope, pub allowed_authorities: BTreeSet<String>, pub adapter_policy: CommandAdapterPolicy } |  |  |  |  |


### src/v26_8_18/topology.rs

| `CloudProvider` | enum | CloudProvider { Aws, Azure, Gcp, PrivateCloud, Edge } |  |  |  |  |

| `ReleaseStanding` | enum | ReleaseStanding { Unknown, PartialAlive, Alive, Blocked, BuildBroken, Unsupported, Refused } |  |  |  |  |

| `aggregate_global_standing` | function | aggregate_global_standing(mut cells: Vec<CellStandingRow>) -> GlobalStanding |  |  |  |  |

| `alive` | function | alive(&self) -> bool |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `qualify_deployment` | function | qualify_deployment(manifest: &DeploymentManifest, now_epoch_ms: i64) -> DeploymentQualification |  |  |  |  |

| `AdapterBinding` | struct | AdapterBinding { pub adapter_id: String, pub kind: String, pub provider_semantics_key: String, pub provider_semantics_version: String, pub allowed_transition_ids: BTreeSet<String>, pub workload_identity: String, pub ambient_credentials: bool } |  |  |  |  |

| `CastleCellManifest` | struct | CastleCellManifest { pub cell_id: String, pub region: String, pub provider: CloudProvider, pub authority_domain: String, pub residency: String, pub subject_prefixes: Vec<String>, pub local_receipt_store: String, pub local_ocel_store: String, pub max_parallel_do: u32, pub adapters: Vec<AdapterBinding> } |  |  |  |  |

| `CellStandingRow` | struct | CellStandingRow { pub cell_id: String, pub observation: ReleaseStanding, pub construct: ReleaseStanding, pub do_standing: ReleaseStanding, pub replay: ReleaseStanding } |  |  |  |  |

| `DeploymentManifest` | struct | DeploymentManifest { pub kind: String, pub release: String, pub constitution: GlobalConstitution, pub cells: Vec<CastleCellManifest>, pub protocol_surfaces: Vec<ProtocolSurface>, pub required_providers: BTreeSet<CloudProvider>, pub required_adapter_kinds: BTreeSet<String> } |  |  |  |  |

| `DeploymentQualification` | struct | DeploymentQualification { pub standing: ReleaseStanding, pub manifest_digest: String, pub findings: Vec<String>, pub cells: usize, pub providers: BTreeSet<CloudProvider>, pub adapter_kinds: BTreeSet<String> } |  |  |  |  |

| `GlobalConstitution` | struct | GlobalConstitution { pub constitution_id: String, pub ontology_version: String, pub invariant_set_digest: String, pub trust_root_ids: BTreeSet<String>, pub provider_semantics: BTreeMap<String, String>, pub issued_at_epoch_ms: i64, pub expires_at_epoch_ms: i64 } |  |  |  |  |

| `GlobalStanding` | struct | GlobalStanding { pub standing: ReleaseStanding, pub cells: Vec<CellStandingRow> } |  |  |  |  |

| `ProtocolSurface` | struct | ProtocolSurface { pub surface: InterfaceOrigin, pub endpoint: String, pub modes: BTreeSet<IntentMode> } |  |  |  |  |


### src/v26_9_28/mod.rs

| `ECOSYSTEM_EPOCH` | const | ECOSYSTEM_EPOCH: &str |  |  |  |  |

| `MAX_EXTERNAL_WITNESS_BYTES` | const | MAX_EXTERNAL_WITNESS_BYTES: u64 |  |  |  |  |

| `MAX_EXTERNAL_WITNESS_DEADLINE_MS` | const | MAX_EXTERNAL_WITNESS_DEADLINE_MS: u64 |  |  |  |  |

| `MAX_EXTERNAL_WITNESS_STEPS` | const | MAX_EXTERNAL_WITNESS_STEPS: u64 |  |  |  |  |

| `SA2A_REPLAN_CONTRACT_DIGEST` | const | SA2A_REPLAN_CONTRACT_DIGEST: &str |  |  |  |  |

| `SA2A_REPLAN_SCHEMA_ID` | const | SA2A_REPLAN_SCHEMA_ID: &str |  |  |  |  |

| `EvidenceStanding` | enum | EvidenceStanding { Alive, Refused(String) } |  |  |  |  |

| `WitnessKind` | enum | WitnessKind { Semantic, Receipt, Process, Planner, Recovery, Federation, ModelCompute } |  |  |  |  |

| `admit_external_witness` | function | admit_external_witness( manifest: &EcosystemManifest, witness: &ExternalWitness, ) -> EvidenceStanding |  |  |  |  |

| `admit_fond_differential` | function | admit_fond_differential(check: &FondDifferentialCheck) -> EvidenceStanding |  |  |  |  |

| `admit_independent_plan` | function | admit_independent_plan(check: &IndependentPlanCheck) -> EvidenceStanding |  |  |  |  |

| `admit_sa2a_replan_envelope` | function | admit_sa2a_replan_envelope( expected_subject: &str, envelope: &PortableReplanEnvelope, ) -> EvidenceStanding |  |  |  |  |

| `bind_v26_9_28_construct_request` | function | bind_v26_9_28_construct_request( mut request: crate::castle::ConstructRequest, manifest: &EcosystemManifest, witnesses: &[ExternalWitness], ) -> Result<crate::castle::ConstructRequest, String> |  |  |  |  |

| `ecosystem_manifest` | function | ecosystem_manifest() -> Result<EcosystemManifest, String> |  |  |  |  |

| `id` | function | id(self) -> &'static str |  |  |  |  |

| `is_alive` | function | is_alive(&self) -> bool |  |  |  |  |

| `qualify_portable_runtime` | function | qualify_portable_runtime(witnesses: &[PortableRuntimeWitness]) -> EvidenceStanding |  |  |  |  |

| `qualify_v26_9_28_upgrade` | function | qualify_v26_9_28_upgrade(manifest: &EcosystemManifest) -> UpgradeQualification |  |  |  |  |

| `route_edge_local_recovery` | function | route_edge_local_recovery( subject: &str, ordered_providers: &[String], failed_providers: &BTreeSet<String>, ) -> Result<RecoveryDecision, String> |  |  |  |  |

| `executed` | str_key | CONSEQUENCES = "executed" |  |  |  |  |

| `ggen-marketplace-v26.9.29` | str_key | REQUIRED = "ggen-marketplace-v26.9.29" |  |  |  |  |

| `sa2a/replan-envelope/v1` | str_key | SA2A_REPLAN_SCHEMA_ID = "sa2a/replan-envelope/v1" |  |  |  |  |

| `sha256:ff7643034ed101930e9c80df716df863b6ee6d14f3b29aff764209ad11dab80e` | str_key | SA2A_REPLAN_CONTRACT_DIGEST = "sha256:ff7643034ed101930e9c80df716df863b6ee6d14f3b29aff764209ad11dab80e" |  |  |  |  |

| `stop` | str_key | DECISIONS = "stop" |  |  |  |  |

| `v26.9.28` | str_key | ECOSYSTEM_EPOCH = "v26.9.28" |  |  |  |  |

| `EcosystemManifest` | struct | EcosystemManifest { pub kind: String, pub release_epoch: String, pub reviewed_local_date: String, pub timezone: String, pub castle_base_sha: String, pub review_window: ReviewWindow, pub marketplace_pack: MarketplacePack, pub subjects: Vec<SourceSubject>, pub excluded_open_prs: Vec<ExcludedPr> } |  |  |  |  |

| `ExcludedPr` | struct | ExcludedPr { pub repo: String, pub pr: u64, pub reason: String } |  |  |  |  |

| `ExternalWitness` | struct | ExternalWitness { pub source_id: String, pub source_sha: String, pub subject: String, pub kind: WitnessKind, pub input_digest: String, pub output_digest: String, pub direct_do_authority: bool, pub limits: WitnessLimits } |  |  |  |  |

| `FondDifferentialCheck` | struct | FondDifferentialCheck { pub policy_digest: String, pub model_digest: String, pub agreed: bool, pub counterexample: Option<String> } |  |  |  |  |

| `IndependentPlanCheck` | struct | IndependentPlanCheck { pub plan_digest: String, pub valid_under_independent_dynamics: bool, pub claimed_cost: i64, pub independently_verified_cost: i64 } |  |  |  |  |

| `MarketplacePack` | struct | MarketplacePack { pub repo: String, pub r pub commit_sha: String, pub path: String, pub ontology_sha: String, pub security_universe_sha: String, pub security_tools_sha: String } |  |  |  |  |

| `PortableReplanDecision` | struct | PortableReplanDecision { pub kind: String, pub reason: String, pub authority: String } |  |  |  |  |

| `PortableReplanEnvelope` | struct | PortableReplanEnvelope { pub schema: String, pub contract_digest: String, pub exact_subject: Value, pub receipt_id: String, pub consequence: String, pub decision: PortableReplanDecision, pub provider: Option<String>, pub projection_digest: Option<String>, pub source_replay_key: Option<String> } |  |  |  |  |

| `PortableRuntimeWitness` | struct | PortableRuntimeWitness { pub engine_family: String, pub module_digest: String, pub input_digest: String, pub output_digest: String } |  |  |  |  |

| `RecoveryDecision` | struct | RecoveryDecision { pub subject: String, pub selected_provider: String, pub excluded_failed_providers: Vec<String> } |  |  |  |  |

| `ReviewWindow` | struct | ReviewWindow { pub timezone: String, pub local_start: String, pub task_cutoff: String, pub utc_start: String, pub utc_cutoff: String, pub pr_count: u64, pub merged_pr_count: u64, pub open_pr_count: u64, pub closed_unmerged_pr_count: u64 } |  |  |  |  |

| `SourceSubject` | struct | SourceSubject { pub id: String, pub repo: String, pub source_kind: String, pub reference: String, pub sha: String, pub pr: Option<u64>, pub role: String, pub authority_ceiling: String, pub allowed_witness_kinds: Vec<String> } |  |  |  |  |

| `UpgradeQualification` | struct | UpgradeQualification { pub standing: EvidenceStanding, pub missing_subjects: Vec<String> } |  |  |  |  |

| `WitnessLimits` | struct | WitnessLimits { pub max_steps: u64, pub max_bytes: u64, pub deadline_ms: u64 } |  |  |  |  |


### tests/board.rs

| `castle:board-test` | str_key | SUBJECT = "castle:board-test" |  |  |  |  |


### tests/capability_intake.rs

| `../docs/sjira/v26.9.28/capability-intake.ttl` | str_key | GRAPH = "../docs/sjira/v26.9.28/capability-intake.ttl" |  |  |  |  |


### tests/castle.rs

| `DOCKER_BIN` | env_key | std::env::var("DOCKER_BIN") |  |  |  |  |

| `GYMACT_BIN` | env_key | std::env::var("GYMACT_BIN") |  |  |  |  |

| `HOME` | env_key | std::env::var("HOME") |  |  |  |  |


### tests/cli_payments.rs

| `6363636363636363636363636363636363636363636363636363636363636363` | str_key | RECEIPT_SEED_HEX = "6363636363636363636363636363636363636363636363636363636363636363" |  |  |  |  |

| `acct:supplier-9821` | str_key | PAYEE = "acct:supplier-9821" |  |  |  |  |

| `acct:treasury` | str_key | PAYER = "acct:treasury" |  |  |  |  |

| `actuator:payments` | str_key | AUDIENCE = "actuator:payments" |  |  |  |  |

| `principal:procurement-agent` | str_key | PRINCIPAL = "principal:procurement-agent" |  |  |  |  |


### tests/cli_payments_rail.rs

| `6363636363636363636363636363636363636363636363636363636363636363` | str_key | RECEIPT_SEED_HEX = "6363636363636363636363636363636363636363636363636363636363636363" |  |  |  |  |

| `acct:supplier-9821` | str_key | PAYEE = "acct:supplier-9821" |  |  |  |  |

| `acct:treasury` | str_key | PAYER = "acct:treasury" |  |  |  |  |

| `actuator:payments` | str_key | AUDIENCE = "actuator:payments" |  |  |  |  |

| `principal:procurement-agent` | str_key | PRINCIPAL = "principal:procurement-agent" |  |  |  |  |


### tests/common/payments.rs

| `AUDIENCE` | const | AUDIENCE: &str |  |  |  |  |

| `NOW_MS` | const | NOW_MS: u64 |  |  |  |  |

| `PAYEE` | const | PAYEE: &str |  |  |  |  |

| `PAYER` | const | PAYER: &str |  |  |  |  |

| `PRINCIPAL` | const | PRINCIPAL: &str |  |  |  |  |

| `admission_ctx` | function | admission_ctx(&self) -> AdmissionContext<'_> |  |  |  |  |

| `admit` | function | admit(&self, effect: PreparedEffect, nonce: &str) -> Result<PaymentAdmission, String> |  |  |  |  |

| `cert` | function | cert(&self, effect: &PreparedEffect, nonce: &str, signers: &[&str]) -> ActuationCertificate |  |  |  |  |

| `effect` | function | effect(&self, amount: &str, obligation: &str) -> PreparedEffect |  |  |  |  |

| `exec_ctx` | function | exec_ctx(&self) -> ExecutionContext<'_> |  |  |  |  |

| `new` | function | new(tag: &str) -> Self |  |  |  |  |

| `token_for` | function | token_for(&self, admission: &PaymentAdmission) -> ActuationToken |  |  |  |  |

| `unique_dir` | function | unique_dir(tag: &str) -> PathBuf |  |  |  |  |

| `with` | function | with(tag: &str, treasury_minor: u64, per_effect_cap: u64, epoch_cap: u64) -> Self |  |  |  |  |

| `acct:supplier-9821` | str_key | PAYEE = "acct:supplier-9821" |  |  |  |  |

| `acct:treasury` | str_key | PAYER = "acct:treasury" |  |  |  |  |

| `actuator:payments` | str_key | AUDIENCE = "actuator:payments" |  |  |  |  |

| `principal:procurement-agent` | str_key | PRINCIPAL = "principal:procurement-agent" |  |  |  |  |

| `Fixture` | struct | Fixture { pub dir: PathBuf, pub registry: KeyRegistry, pub epochs: SecurityEpochs, pub policy: SpendPolicy, pub claims: ClaimStore, pub nonces: DurableNonceFence, pub ledger: FileJournalLedger, pub signer: ReceiptKey, pub verifier: ReceiptCheck, custodians: Vec<(String, SigningKey)> } |  |  |  |  |

| `RealBlake3` | struct |  |  |  |  |  |

| `ReceiptCheck` | struct | ReceiptCheck { id: String, key: VerifyingKey } |  |  |  |  |

| `ReceiptKey` | struct | ReceiptKey { id: String, key: SigningKey } |  |  |  |  |


### tests/common/scripted_rail.rs

| `authoritative` | function | authoritative(mut self) -> Self |  |  |  |  |

| `new` | function | new(script: Vec<RailStatus>) -> Self |  |  |  |  |

| `push` | function | push(&self, s: RailStatus) |  |  |  |  |

| `rejected` | function | rejected() -> RailStatus |  |  |  |  |

| `returned` | function | returned() -> RailStatus |  |  |  |  |

| `settled` | function | settled() -> RailStatus |  |  |  |  |

| `ScriptedRail` | struct | ScriptedRail { script: Mutex<VecDeque<RailStatus>>, last: Mutex<RailStatus>, authoritative: bool } |  |  |  |  |


### tests/fortune5.rs

| `castle:enterprise:test` | str_key | SUBJECT = "castle:enterprise:test" |  |  |  |  |


### tests/payments_compliance.rs

| `5493001KJTIIGC8Y1R12` | str_key | REAL_LEI = "5493001KJTIIGC8Y1R12" |  |  |  |  |


### tests/payments_fibo.rs

| `CASTLE_FIBO_CORPUS` | env_key | std::env::var("CASTLE_FIBO_CORPUS") |  |  |  |  |


### tests/payments_hardening.rs

| `2026-09-29T12:34:56Z` | str_key | TS = "2026-09-29T12:34:56Z" |  |  |  |  |


### tests/payments_holds.rs

| `admit_submitted` | function | admit_submitted(fx: &Fixture, amount: &str, obligation: &str, nonce: &str) -> PaymentAdmission |  |  |  |  |


### tests/payments_iso20022.rs

| `CASTLE_WRITE_GOLDEN` | env_key | std::env::var("CASTLE_WRITE_GOLDEN") |  |  |  |  |

| `2026-09-29T12:34:56Z` | str_key | TS = "2026-09-29T12:34:56Z" |  |  |  |  |

| `CASTUS33` | str_key | DAGT = "CASTUS33" |  |  |  |  |

| `SUPPGB2LXXX` | str_key | CAGT = "SUPPGB2LXXX" |  |  |  |  |

| `acct:supplier-2` | str_key | PAYEE2 = "acct:supplier-2" |  |  |  |  |


### tests/payments_iso20022_official.rs

| `CASTLE_ISO20022_XSD_DIR` | env_key | std::env::var("CASTLE_ISO20022_XSD_DIR") |  |  |  |  |

| `/private/tmp/claude-501/-Users-sac-castle/62dd6f54-50cc-4146-9377-c4ca3cfee3bd/scratchpad/iso/xsd` | str_key | DEFAULT_DIR = "/private/tmp/claude-501/-Users-sac-castle/62dd6f54-50cc-4146-9377-c4ca3cfee3bd/scratchpad/iso/xsd" |  |  |  |  |

| `2026-09-29T12:34:56Z` | str_key | TS = "2026-09-29T12:34:56Z" |  |  |  |  |


### tests/payments_money.rs

| `REFUSED:PAYMENT_AMOUNT_NOT_INTEGER` | str_key | NOT_INT = "REFUSED:PAYMENT_AMOUNT_NOT_INTEGER" |  |  |  |  |

| `REFUSED:PAYMENT_AMOUNT_ZERO` | str_key | ZERO = "REFUSED:PAYMENT_AMOUNT_ZERO" |  |  |  |  |


### tests/payments_rail_sim.rs

| `sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef` | str_key | EFF = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef" |  |  |  |  |


### tests/prop_dfcm.rs

| `p0` | str_key | ALPHABET = "p0" |  |  |  |  |


### tests/sjira_castle.rs

| `../docs/sjira/v26.9.28/courts/exclusions.rq` | str_key | EXCLUSIONS = "../docs/sjira/v26.9.28/courts/exclusions.rq" |  |  |  |  |

| `../docs/sjira/v26.9.28/courts/ownership.rq` | str_key | OWNERSHIP = "../docs/sjira/v26.9.28/courts/ownership.rq" |  |  |  |  |

| `../docs/sjira/v26.9.28/courts/runtime_crown.rq` | str_key | RUNTIME_CROWN = "../docs/sjira/v26.9.28/courts/runtime_crown.rq" |  |  |  |  |

| `../docs/sjira/v26.9.28/courts/stop.rq` | str_key | STOP = "../docs/sjira/v26.9.28/courts/stop.rq" |  |  |  |  |

| `../docs/sjira/v26.9.28/goal.ttl` | str_key | GOAL = "../docs/sjira/v26.9.28/goal.ttl" |  |  |  |  |

| `../docs/sjira/v26.9.28/repos.ttl` | str_key | REPOS = "../docs/sjira/v26.9.28/repos.ttl" |  |  |  |  |



<!-- AGENT-FORBIDDEN-END -->

## Signature/type/default/errors table

<!-- RIGID table: header order is fixed; rows come only from the query. -->

| Item | Type | Signature | Params | Defaults | Errors | Invariants |
|------|------|-----------|--------|----------|--------|------------|

| `test` | script | node --experimental-strip-types --test test/*.test.ts |  |  |  |  |

| `admission_provider` | function | admission_provider/0 |  |  |  |  |

| `kernel` | function | kernel/0 |  |  |  |  |

| `receipt_verifier` | function | receipt_verifier/0 |  |  |  |  |

| `semantic_bundle` | function | semantic_bundle/0 |  |  |  |  |

| `class_iri` | str_key | class_iri: "http: |  |  |  |  |

| `table` | str_key | table: "castle_admissions" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: true |  |  |  |  |

| `admit` | function | admit/3 |  |  |  |  |

| `BLOCKED_ADMISSION_PROVIDER_NOT_CONFIGURED` | str_key | {:error, :BLOCKED_ADMISSION_PROVIDER_NOT_CONFIGURED} |  |  |  |  |

| `external_id` | function | external_id/1 |  |  |  |  |

| `verify` | function | verify/3 |  |  |  |  |

| `REFUSED_ADMISSION_WITNESS_MISMATCH` | str_key | {:error, :REFUSED_ADMISSION_WITNESS_MISMATCH} |  |  |  |  |

| `REFUSED_INCOMPLETE_ADMISSION_WITNESS` | str_key | {:error, :REFUSED_INCOMPLETE_ADMISSION_WITNESS} |  |  |  |  |

| `REFUSED_INVALID_ADMISSION_WITNESS` | str_key | {:error, :REFUSED_INVALID_ADMISSION_WITNESS} |  |  |  |  |

| `external_id` | str_key | external_id: value |  |  |  |  |

| `external_id` | str_key | "external_id" => _ |  |  |  |  |

| `id` | str_key | id: value |  |  |  |  |

| `id` | str_key | "id" => _ |  |  |  |  |

| `start` | function | start/2 |  |  |  |  |

| `name` | str_key | name: CastlePaaS.Supervisor |  |  |  |  |

| `strategy` | str_key | strategy: :one_for_one |  |  |  |  |

| `sha256` | function | sha256/1 |  |  |  |  |

| `case` | str_key | case: :lower |  |  |  |  |

| `class_iri` | str_key | class_iri: "http: |  |  |  |  |

| `table` | str_key | table: "castle_capabilities" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: false |  |  |  |  |

| `extensions` | str_key | extensions: [AshJsonApi.Domain] |  |  |  |  |

| `class_iri` | str_key | class_iri: "http: |  |  |  |  |

| `table` | str_key | table: "castle_evidence" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: true |  |  |  |  |

| `class_iri` | str_key | class_iri: "http: |  |  |  |  |

| `table` | str_key | table: "castle_execution_intents" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: true |  |  |  |  |

| `CastlePaaS.Generated.Resource` | ash_resource |  |  |  |  |  |

| `allow_nil?` | str_key | allow_nil?: false |  |  |  |  |

| `allow_nil?` | str_key | allow_nil?: true |  |  |  |  |

| `class_iri` | str_key | Keyword.fetch!("class_iri") |  |  |  |  |

| `constraints` | str_key | constraints: [ one_of: [ :UNKNOWN, :PARTIAL_ALIVE, :ALIVE, :BLOCKED, :BUILD_BROKEN, :UNSUPPORTED, :REFUSED ] ] |  |  |  |  |

| `data_layer` | str_key | data_layer: AshPostgres.DataLayer |  |  |  |  |

| `default` | str_key | default: :UNKNOWN |  |  |  |  |

| `default` | str_key | default: %{} |  |  |  |  |

| `domain` | str_key | domain: CastlePaaS.Domain |  |  |  |  |

| `extensions` | str_key | extensions: [AshR2RML, AshJsonApi.Resource] |  |  |  |  |

| `one_of` | str_key | one_of: [ :UNKNOWN, :PARTIAL_ALIVE, :ALIVE, :BLOCKED, :BUILD_BROKEN, :UNSUPPORTED, :REFUSED ] |  |  |  |  |

| `public?` | str_key | public?: false |  |  |  |  |

| `public?` | str_key | public?: true |  |  |  |  |

| `table` | str_key | Keyword.fetch!("table") |  |  |  |  |

| `tenant_scoped?` | str_key | Keyword.fetch!("tenant_scoped?") |  |  |  |  |

| `result` | type | @type result :: {:ok, map()} | {:error, term()} |  |  |  |  |

| `execute` | function | execute/3 |  |  |  |  |

| `manufacture` | function | manufacture/1 |  |  |  |  |

| `release_info` | function | release_info/0 |  |  |  |  |

| `BLOCKED_CASTLE_BINARY` | str_key | {:error, {:BLOCKED_CASTLE_BINARY, reason}} |  |  |  |  |

| `BLOCKED_CASTLE_RUNTIME_CONFIGURATION` | str_key | {:error, :BLOCKED_CASTLE_RUNTIME_CONFIGURATION} |  |  |  |  |

| `BLOCKED_KERNEL_TRANSPORT` | str_key | {:error, {:BLOCKED_KERNEL_TRANSPORT, Exception.message(error)}} |  |  |  |  |

| `REFUSED_AMBIENT_COMMAND_POLICY` | str_key | {:error, :REFUSED_AMBIENT_COMMAND_POLICY} |  |  |  |  |

| `REFUSED_CASTLE_RUNTIME_CONFIGURATION` | str_key | {:error, :REFUSED_CASTLE_RUNTIME_CONFIGURATION} |  |  |  |  |

| `REFUSED_CASTLE_RUNTIME_IDENTITY` | str_key | {:error, :REFUSED_CASTLE_RUNTIME_IDENTITY} |  |  |  |  |

| `REFUSED_CONSTRUCT_NOT_ALIVE` | str_key | {:error, :REFUSED_CONSTRUCT_NOT_ALIVE} |  |  |  |  |

| `REFUSED_INVALID_ADAPTER_PROFILE` | str_key | {:error, :REFUSED_INVALID_ADAPTER_PROFILE} |  |  |  |  |

| `REFUSED_INVALID_BRCE_RECEIPT_DIGEST` | str_key | {:error, :REFUSED_INVALID_BRCE_RECEIPT_DIGEST} |  |  |  |  |

| `REFUSED_INVALID_DIGEST` | str_key | {:error, :REFUSED_INVALID_DIGEST} |  |  |  |  |

| `REFUSED_KERNEL_DO` | str_key | {:error, {:REFUSED_KERNEL_DO, other}} |  |  |  |  |

| `REFUSED_KERNEL_EXIT` | str_key | {:error, {:REFUSED_KERNEL_EXIT, status, output}} |  |  |  |  |

| `REFUSED_KERNEL_MANUFACTURE` | str_key | {:error, {:REFUSED_KERNEL_MANUFACTURE, other}} |  |  |  |  |

| `REFUSED_NON_JSON_KERNEL_RESPONSE` | str_key | {:error, {:REFUSED_NON_JSON_KERNEL_RESPONSE, output}} |  |  |  |  |

| `REFUSED_O_STAR_REQUIRED` | str_key | {:error, :REFUSED_O_STAR_REQUIRED} |  |  |  |  |

| `REFUSED_REQUIRED_FIELD` | str_key | {:error, {:REFUSED_REQUIRED_FIELD, key}} |  |  |  |  |

| `REFUSED_UNKNOWN_ADAPTER_PROFILE` | str_key | {:error, :REFUSED_UNKNOWN_ADAPTER_PROFILE} |  |  |  |  |

| `REFUSED_UNRECEIPTED_DO` | str_key | {:error, :REFUSED_UNRECEIPTED_DO} |  |  |  |  |

| `REFUSED_WRONG_KERNEL_IDENTITY` | str_key | {:error, :REFUSED_WRONG_KERNEL_IDENTITY} |  |  |  |  |

| `adapter_policy` | str_key | Map.get("adapter_policy") |  |  |  |  |

| `adapter_policy` | str_key | "adapter_policy" => _ |  |  |  |  |

| `adapter_profile_id` | str_key | Map.get("adapter_profile_id") |  |  |  |  |

| `admitted` | str_key | Map.get("admitted") |  |  |  |  |

| `allowed_authorities` | str_key | Map.get("allowed_authorities") |  |  |  |  |

| `allowed_authorities` | str_key | "allowed_authorities" => _ |  |  |  |  |

| `authority` | str_key | "authority" => _ |  |  |  |  |

| `bin` | str_key | bin: bin |  |  |  |  |

| `bin_sha256` | str_key | bin_sha256: actual |  |  |  |  |

| `brce_outcome_receipt_digests` | str_key | "brce_outcome_receipt_digests" => _ |  |  |  |  |

| `brce_prepare_receipt_digests` | str_key | "brce_prepare_receipt_digests" => _ |  |  |  |  |

| `case` | str_key | case: :lower |  |  |  |  |

| `cell_id` | str_key | "cell_id" => _ |  |  |  |  |

| `cell_id` | str_key | Map.get("cell_id") |  |  |  |  |

| `config_graph` | str_key | "config_graph" => _ |  |  |  |  |

| `config_graph` | str_key | Map.get("config_graph") |  |  |  |  |

| `construct_digest` | str_key | "construct_digest" => _ |  |  |  |  |

| `envelope` | str_key | "envelope" => _ |  |  |  |  |

| `evidence_commit` | str_key | "evidence_commit" => _ |  |  |  |  |

| `evidence_dir` | str_key | "evidence_dir" => _ |  |  |  |  |

| `key_id` | str_key | key_id: key_id |  |  |  |  |

| `name` | str_key | "name" => _ |  |  |  |  |

| `o_star` | str_key | "o_star" => _ |  |  |  |  |

| `ontology` | str_key | "ontology" => _ |  |  |  |  |

| `ontology` | str_key | Map.get("ontology") |  |  |  |  |

| `process` | str_key | "process" => _ |  |  |  |  |

| `release` | str_key | "release" => _ |  |  |  |  |

| `signing_key_path` | str_key | signing_key_path: signing_key_path |  |  |  |  |

| `standing` | str_key | "standing" => _ |  |  |  |  |

| `stderr_to_stdout` | str_key | stderr_to_stdout: true |  |  |  |  |

| `subject` | str_key | "subject" => _ |  |  |  |  |

| `version` | str_key | "version" => _ |  |  |  |  |

| `zeroUnreceiptedActuation` | str_key | "zeroUnreceiptedActuation" => _ |  |  |  |  |

| `class_iri` | str_key | class_iri: "http: |  |  |  |  |

| `table` | str_key | table: "castle_observations" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: true |  |  |  |  |

| `class_iri` | str_key | class_iri: "https: |  |  |  |  |

| `table` | str_key | table: "castle_organizations" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: false |  |  |  |  |

| `record` | function | record/3 |  |  |  |  |

| `class_iri` | str_key | class_iri: "http: |  |  |  |  |

| `table` | str_key | table: "castle_plans" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: true |  |  |  |  |

| `class_iri` | str_key | class_iri: "http: |  |  |  |  |

| `table` | str_key | table: "castle_platform_services" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: true |  |  |  |  |

| `admission` | str_key | admission: args.admission |  |  |  |  |

| `digest` | str_key | digest: args.witness["witness_digest"] |  |  |  |  |

| `external_id` | str_key | external_id: "admission:#{CastlePaaS.AdmissionWitness.external_id(args.subject)}" |  |  |  |  |

| `label` | str_key | label: "CASTLE O* admission" |  |  |  |  |

| `metadata` | str_key | metadata: args.witness |  |  |  |  |

| `provider` | str_key | provider: provider |  |  |  |  |

| `standing` | str_key | standing: :ALIVE |  |  |  |  |

| `witness` | str_key | witness: args.witness |  |  |  |  |

| `admission_result` | str_key | admission_result: %{witness: witness} |  |  |  |  |

| `admission_witness` | str_key | admission_witness: args.witness |  |  |  |  |

| `digest` | str_key | digest: digest |  |  |  |  |

| `external_id` | str_key | external_id: Map.get(process, :id) || Map.get(process, "id") || "plan:unidentified" |  |  |  |  |

| `external_id` | str_key | external_id: "intent:#{subject}:#{digest}" |  |  |  |  |

| `id` | str_key | Map.get("id") |  |  |  |  |

| `intent` | str_key | intent: intent |  |  |  |  |

| `intent` | str_key | intent: args.intent |  |  |  |  |

| `label` | str_key | label: "CASTLE inert plan" |  |  |  |  |

| `label` | str_key | label: "CASTLE inert execution intent" |  |  |  |  |

| `metadata` | str_key | metadata: %{process: process} |  |  |  |  |

| `metadata` | str_key | metadata: %{admission_witness: args.witness, intent: args.intent} |  |  |  |  |

| `now_epoch_ms` | str_key | now_epoch_ms: now |  |  |  |  |

| `plan` | str_key | plan: args.plan |  |  |  |  |

| `process` | str_key | Map.get("process") |  |  |  |  |

| `process` | str_key | process: process |  |  |  |  |

| `record` | str_key | record: record |  |  |  |  |

| `runtime_intent` | str_key | runtime_intent: Map.put(args.intent, :o_star, args.witness) |  |  |  |  |

| `standing` | str_key | standing: :PARTIAL_ALIVE |  |  |  |  |

| `subject` | str_key | Map.get("subject") |  |  |  |  |

| `witness` | str_key | witness: witness |  |  |  |  |

| `digest` | str_key | Map.get("digest") |  |  |  |  |

| `digest` | str_key | digest: digest |  |  |  |  |

| `evidence` | str_key | evidence: evidence |  |  |  |  |

| `external_id` | str_key | external_id: Map.get(evidence, :external_id) || Map.get(evidence, "external_id") || "evidence:#{digest}" |  |  |  |  |

| `external_id` | str_key | Map.get("external_id") |  |  |  |  |

| `label` | str_key | label: Map.get(evidence, :label) || Map.get(evidence, "label") || "CASTLE evidence" |  |  |  |  |

| `label` | str_key | Map.get("label") |  |  |  |  |

| `metadata` | str_key | metadata: evidence |  |  |  |  |

| `standing` | str_key | Map.get("standing") |  |  |  |  |

| `standing` | str_key | standing: standing |  |  |  |  |

| `tenant` | str_key | tenant: tenant |  |  |  |  |

| `attrs` | str_key | attrs: attrs |  |  |  |  |

| `tenant` | str_key | tenant: tenant |  |  |  |  |

| `digest` | str_key | digest: digest |  |  |  |  |

| `external_id` | str_key | external_id: "replay:#{digest}" |  |  |  |  |

| `label` | str_key | label: "CASTLE receipt replay" |  |  |  |  |

| `metadata` | str_key | metadata: args.verification |  |  |  |  |

| `receipt_digest` | str_key | Map.get("receipt_digest") |  |  |  |  |

| `standing` | str_key | standing: :ALIVE |  |  |  |  |

| `class_iri` | str_key | class_iri: "http: |  |  |  |  |

| `table` | str_key | table: "castle_receipts" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: true |  |  |  |  |

| `verify` | function | verify/1 |  |  |  |  |

| `BLOCKED_RECEIPT_VERIFIER_NOT_CONFIGURED` | str_key | {:error, :BLOCKED_RECEIPT_VERIFIER_NOT_CONFIGURED} |  |  |  |  |

| `class_iri` | str_key | class_iri: "http: |  |  |  |  |

| `table` | str_key | table: "castle_replays" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: true |  |  |  |  |

| `installed_extensions` | function | installed_extensions/0 |  |  |  |  |

| `min_pg_version` | function | min_pg_version/0 |  |  |  |  |

| `major` | str_key | major: 14 |  |  |  |  |

| `minor` | str_key | minor: 0 |  |  |  |  |

| `otp_app` | str_key | otp_app: :castle_paas |  |  |  |  |

| `patch` | str_key | patch: 0 |  |  |  |  |

| `parse` | function | parse/1 |  |  |  |  |

| `ALIVE` | str_key | "ALIVE" => _ |  |  |  |  |

| `BLOCKED` | str_key | "BLOCKED" => _ |  |  |  |  |

| `BUILD_BROKEN` | str_key | "BUILD_BROKEN" => _ |  |  |  |  |

| `PARTIAL_ALIVE` | str_key | "PARTIAL_ALIVE" => _ |  |  |  |  |

| `REFUSED` | str_key | "REFUSED" => _ |  |  |  |  |

| `REFUSED_INVALID_STANDING` | str_key | {:error, :REFUSED_INVALID_STANDING} |  |  |  |  |

| `UNKNOWN` | str_key | "UNKNOWN" => _ |  |  |  |  |

| `UNSUPPORTED` | str_key | "UNSUPPORTED" => _ |  |  |  |  |

| `class_iri` | str_key | class_iri: "http: |  |  |  |  |

| `table` | str_key | table: "castle_subjects" |  |  |  |  |

| `tenant_scoped?` | str_key | tenant_scoped?: true |  |  |  |  |

| `162e466d8f07d0a75a468b4441b4bc8b1aad369b` | str_key | GGEN_COMMIT = "162e466d8f07d0a75a468b4441b4bc8b1aad369b" |  |  |  |  |

| `26.8.15` | str_key | GGEN_VERSION = "26.8.15" |  |  |  |  |

| `FIBO_MAPPINGS` | const | FIBO_MAPPINGS: &[FiboMapping] |  |  |  |  |

| `IRI_CURRENCY` | const | IRI_CURRENCY: &str |  |  |  |  |

| `IRI_CURRENCY_EUR` | const | IRI_CURRENCY_EUR: &str |  |  |  |  |

| `IRI_CURRENCY_GBP` | const | IRI_CURRENCY_GBP: &str |  |  |  |  |

| `IRI_CURRENCY_JPY` | const | IRI_CURRENCY_JPY: &str |  |  |  |  |

| `IRI_CURRENCY_KWD` | const | IRI_CURRENCY_KWD: &str |  |  |  |  |

| `IRI_CURRENCY_USD` | const | IRI_CURRENCY_USD: &str |  |  |  |  |

| `IRI_HAS_AMOUNT` | const | IRI_HAS_AMOUNT: &str |  |  |  |  |

| `IRI_HAS_CURRENCY` | const | IRI_HAS_CURRENCY: &str |  |  |  |  |

| `IRI_HAS_PAYMENT_AMOUNT` | const | IRI_HAS_PAYMENT_AMOUNT: &str |  |  |  |  |

| `IRI_LEGAL_ENTITY_IDENTIFIER` | const | IRI_LEGAL_ENTITY_IDENTIFIER: &str |  |  |  |  |

| `IRI_MONETARY_AMOUNT` | const | IRI_MONETARY_AMOUNT: &str |  |  |  |  |

| `IRI_PAYEE` | const | IRI_PAYEE: &str |  |  |  |  |

| `IRI_PAYER` | const | IRI_PAYER: &str |  |  |  |  |

| `IRI_PAYMENT` | const | IRI_PAYMENT: &str |  |  |  |  |

| `IRI_PAYMENT_OBLIGATION` | const | IRI_PAYMENT_OBLIGATION: &str |  |  |  |  |

| `IRI_SETTLEMENT` | const | IRI_SETTLEMENT: &str |  |  |  |  |

| `currency` | str_key | FIBO_MAPPINGS = "currency" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/BE/LegalEntities/LEIEntities/LegalEntityIdentifier` | str_key | IRI_LEGAL_ENTITY_IDENTIFIER = "https://spec.edmcouncil.org/fibo/ontology/BE/LegalEntities/LEIEntities/LegalEntityIdentifier" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FBC/FinancialInstruments/Settlement/Settlement` | str_key | IRI_SETTLEMENT = "https://spec.edmcouncil.org/fibo/ontology/FBC/FinancialInstruments/Settlement/Settlement" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/CurrencyAmount/Currency` | str_key | IRI_CURRENCY = "https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/CurrencyAmount/Currency" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/CurrencyAmount/MonetaryAmount` | str_key | IRI_MONETARY_AMOUNT = "https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/CurrencyAmount/MonetaryAmount" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/CurrencyAmount/hasAmount` | str_key | IRI_HAS_AMOUNT = "https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/CurrencyAmount/hasAmount" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/CurrencyAmount/hasCurrency` | str_key | IRI_HAS_CURRENCY = "https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/CurrencyAmount/hasCurrency" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/ISO4217-CurrencyCodes/EUR` | str_key | IRI_CURRENCY_EUR = "https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/ISO4217-CurrencyCodes/EUR" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/ISO4217-CurrencyCodes/GBP` | str_key | IRI_CURRENCY_GBP = "https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/ISO4217-CurrencyCodes/GBP" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/ISO4217-CurrencyCodes/JPY` | str_key | IRI_CURRENCY_JPY = "https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/ISO4217-CurrencyCodes/JPY" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/ISO4217-CurrencyCodes/KWD` | str_key | IRI_CURRENCY_KWD = "https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/ISO4217-CurrencyCodes/KWD" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/ISO4217-CurrencyCodes/USD` | str_key | IRI_CURRENCY_USD = "https://spec.edmcouncil.org/fibo/ontology/FND/Accounting/ISO4217-CurrencyCodes/USD" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/PaymentsAndSchedules/Payee` | str_key | IRI_PAYEE = "https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/PaymentsAndSchedules/Payee" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/PaymentsAndSchedules/Payer` | str_key | IRI_PAYER = "https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/PaymentsAndSchedules/Payer" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/PaymentsAndSchedules/Payment` | str_key | IRI_PAYMENT = "https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/PaymentsAndSchedules/Payment" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/PaymentsAndSchedules/PaymentObligation` | str_key | IRI_PAYMENT_OBLIGATION = "https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/PaymentsAndSchedules/PaymentObligation" |  |  |  |  |

| `https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/PaymentsAndSchedules/hasPaymentAmount` | str_key | IRI_HAS_PAYMENT_AMOUNT = "https://spec.edmcouncil.org/fibo/ontology/FND/ProductsAndServices/PaymentsAndSchedules/hasPaymentAmount" |  |  |  |  |

| `FiboMapping` | struct | FiboMapping { pub id: &'static str, pub castle_term: &'static str, pub fibo_iri: &'static str, pub fibo_kind: &'static str, pub match_kind: &'static str, pub status: &'static str, pub fibo_module: &'static str, pub version_iri: &'static str, pub corpus_file: &'static str, pub corpus_sha256: &'static str } |  |  |  |  |

| `crypto_capabilities_handler` | function | crypto_capabilities_handler() -> Result<Value> |  |  |  |  |

| `deployment_bind_policy_handler` | function | deployment_bind_policy_handler( binding_path: String, policy_path: String, ) -> Result<Value> |  |  |  |  |

| `dfcm_qualify_handler` | function | dfcm_qualify_handler( manifest_path: String, evidence_path: String, now_epoch_ms: i64, max_evidence_age_ms: i64, ) -> Result<Value> |  |  |  |  |

| `dfcm_verify_handler` | function | dfcm_verify_handler() -> Result<Value> |  |  |  |  |

| `live_check_plan_handler` | function | live_check_plan_handler(manifest_path: String) -> Result<Value> |  |  |  |  |

| `live_check_qualify_handler` | function | live_check_qualify_handler( manifest_path: String, evidence_path: String, now_epoch_ms: i64, max_evidence_age_ms: i64, ) -> Result<Value> |  |  |  |  |

| `live_check_run_handler` | function | live_check_run_handler(spec_path: String, observed_at_epoch_ms: i64) -> Result<Value> |  |  |  |  |

| `protocol_dispatch_handler` | function | protocol_dispatch_handler(intent_path: String) -> Result<Value> |  |  |  |  |

| `replication_admit_handler` | function | replication_admit_handler( state_path: String, checkpoint_path: String, receiver_id: String, ) -> Result<Value> |  |  |  |  |

| `evidence_verify_handler` | function | evidence_verify_handler(evidence_path: String) -> Result<Value> |  |  |  |  |

| `chaos_qualify_handler` | function | chaos_qualify_handler(evidence_path: String) -> Result<Value> |  |  |  |  |

| `construct_manufacture_handler` | function | construct_manufacture_handler(request_path: String, signing_key_path: String, key_id: String) -> Result<Value> |  |  |  |  |

| `crypto_capabilities_handler` | function | crypto_capabilities_handler() -> Result<Value> |  |  |  |  |

| `deployment_adapters_handler` | function | deployment_adapters_handler() -> Result<Value> |  |  |  |  |

| `deployment_qualify_handler` | function | deployment_qualify_handler(manifest_path: String, now_epoch_ms: i64) -> Result<Value> |  |  |  |  |

| `do_execute_handler` | function | do_execute_handler(request_path: String, signing_key_path: String, key_id: String, expected_construct_digest: String, now_epoch_ms: i64) -> Result<Value> |  |  |  |  |

| `fortune5_qualify_handler` | function | fortune5_qualify_handler(subject: String, evidence_path: String, now_epoch_ms: Option<i64>, max_evidence_age_ms: Option<i64>) -> Result<Value> |  |  |  |  |

| `fortune5_requirements_handler` | function | fortune5_requirements_handler() -> Result<Value> |  |  |  |  |

| `impact_coverage_handler` | function | impact_coverage_handler(classes_path: String, target_coverage_bps: Option<i64>) -> Result<Value> |  |  |  |  |

| `inventory_components_handler` | function | inventory_components_handler() -> Result<Value> |  |  |  |  |

| `inventory_goals_handler` | function | inventory_goals_handler() -> Result<Value> |  |  |  |  |

| `protocol_a2a_handler` | function | protocol_a2a_handler() -> Result<Value> |  |  |  |  |

| `protocol_mcp_handler` | function | protocol_mcp_handler() -> Result<Value> |  |  |  |  |

| `release_info_handler` | function | release_info_handler() -> Result<Value> |  |  |  |  |

| `replay_admit_handler` | function | replay_admit_handler(replay_class_id: String, structural_signature: String, ontology_version: String, provider_semantics_version: String, invariant_set_digest: String, process_digest: String, invariants_hold: bool) -> Result<Value> |  |  |  |  |

| `execute_handler` | function | execute_handler(a: ExecuteArgs) -> Result<Value> |  |  |  |  |

| `explain_handler` | function | explain_handler(effect_digest: String, state_dir: String, ledger_dir: String, reference_ledger: bool) -> Result<Value> |  |  |  |  |

| `finalize_handler` | function | finalize_handler( effect_digest: String, state_dir: String, ledger_dir: String, rail_dir: String, rail_mode: Option<String>, reference_ledger: bool, reference_rail: bool, ) -> Result<Value> |  |  |  |  |

| `prepare_handler` | function | prepare_handler( principal: String, payer: String, payee: String, amount_minor: String, currency: String, obligation_id: String, purpose: String, reverses: Option<String>, ) -> Result<Value> |  |  |  |  |

| `rail_submit_handler` | function | rail_submit_handler(a: RailSubmitArgs) -> Result<Value> |  |  |  |  |

| `receipt_handler` | function | receipt_handler( effect_digest: String, state_dir: String, ledger_dir: String, rail_dir: String, journal_dir: String, reference_ledger: bool, reference_rail: bool, ) -> Result<Value> |  |  |  |  |

| `reconcile_handler` | function | reconcile_handler(effect_digest: String, state_dir: String, ledger_dir: String, reference_ledger: bool) -> Result<Value> |  |  |  |  |

| `recover_handler` | function | recover_handler(state_dir: String) -> Result<Value> |  |  |  |  |

| `replay_handler` | function | replay_handler(journal_dir: String, effect_digest: String) -> Result<Value> |  |  |  |  |

| `CASTLE-CLI-FINALITY-OBSERVATION-V1` | str_key | FINALITY_DOMAIN = "CASTLE-CLI-FINALITY-OBSERVATION-V1" |  |  |  |  |

| `REFUSED:PAYMENT_CLAIM_NOT_FOUND` | str_key | CLAIM_NOT_FOUND = "REFUSED:PAYMENT_CLAIM_NOT_FOUND" |  |  |  |  |

| `REFUSED:PAYMENT_JOURNAL_WITH_SCREENING_UNSUPPORTED` | str_key | JOURNAL_WITH_SCREENING_UNSUPPORTED = "REFUSED:PAYMENT_JOURNAL_WITH_SCREENING_UNSUPPORTED" |  |  |  |  |

| `REFUSED:PAYMENT_LEDGER_NOT_FOUND` | str_key | LEDGER_NOT_FOUND = "REFUSED:PAYMENT_LEDGER_NOT_FOUND" |  |  |  |  |

| `REFUSED:PAYMENT_LEDGER_UNAVAILABLE` | str_key | LEDGER_UNAVAILABLE = "REFUSED:PAYMENT_LEDGER_UNAVAILABLE" |  |  |  |  |

| `REFUSED:PAYMENT_RAIL_MODE_INVALID` | str_key | RAIL_MODE_INVALID = "REFUSED:PAYMENT_RAIL_MODE_INVALID" |  |  |  |  |

| `REFUSED:PAYMENT_RAIL_NOT_FOUND` | str_key | RAIL_NOT_FOUND = "REFUSED:PAYMENT_RAIL_NOT_FOUND" |  |  |  |  |

| `REFUSED:PAYMENT_RECONCILE_RAIL_HOLD_LIVE` | str_key | RECONCILE_RAIL_HOLD_LIVE = "REFUSED:PAYMENT_RECONCILE_RAIL_HOLD_LIVE" |  |  |  |  |

| `REFUSED:PAYMENT_SCREENING_INPUTS_INCOMPLETE` | str_key | SCREENING_INPUTS_INCOMPLETE = "REFUSED:PAYMENT_SCREENING_INPUTS_INCOMPLETE" |  |  |  |  |

| `REFUSED:RECEIPT_RAIL_HISTORY_MISSING_SETTLEMENT` | str_key | RAIL_HISTORY_MISSING_SETTLEMENT = "REFUSED:RECEIPT_RAIL_HISTORY_MISSING_SETTLEMENT" |  |  |  |  |

| `REFUSED:REFERENCE_LEDGER_NOT_CONFIRMED` | str_key | REFERENCE_LEDGER_NOT_CONFIRMED = "REFUSED:REFERENCE_LEDGER_NOT_CONFIRMED" |  |  |  |  |

| `REFUSED:REFERENCE_RAIL_NOT_CONFIRMED` | str_key | REFERENCE_RAIL_NOT_CONFIRMED = "REFUSED:REFERENCE_RAIL_NOT_CONFIRMED" |  |  |  |  |

| `ExecuteArgs` | struct | ExecuteArgs { pub effect_path: String, pub certificate_path: String, pub registry_path: String, pub policy_path: String, pub state_dir: String, pub ledger_dir: String, pub opening_path: String, pub reference_ledger: bool, pub audience: String, pub policy_epoch: i64, pub revocation_epoch: i64, pub generation: i64, pub now_ms: i64, pub receipt_key_id: String, pub receipt_seed_hex: String, pub allowed_authority: String } |  |  |  |  |

| `RailSubmitArgs` | struct | RailSubmitArgs { pub base: ExecuteArgs, pub reference_rail: bool, pub rail_dir: String, pub rail_mode: String, pub sanctions_path: Option<String>, pub counterparties_path: Option<String>, pub journal_dir: Option<String>, pub debtor_agent_bic: String, pub creditor_agent_bic: String, pub created_at_iso: String, pub debtor_name: Option<String>, pub creditor_name: Option<String>, pub effect_expires_at_ms: Option<String> } |  |  |  |  |

| `BLAKE3_EMPTY_HEX` | const | BLAKE3_EMPTY_HEX: &str |  |  |  |  |

| `assert_blake3_self_test` | function | assert_blake3_self_test() -> Result<(), String> |  |  |  |  |

| `blake3_hex_utf8` | function | blake3_hex_utf8(input: &str) -> String |  |  |  |  |

| `af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262` | str_key | BLAKE3_EMPTY_HEX = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262" |  |  |  |  |

| `SIGNATURE_ALGORITHM` | const | SIGNATURE_ALGORITHM: &str |  |  |  |  |

| `FailureMode` | enum | FailureMode { FailClosed, SafeDegrade, LocalCapability, Defer } |  |  |  |  |

| `MaterialityDimension` | enum | MaterialityDimension { Financial, Operational, Customer, Legal, Regulatory, Reputational, Systemic } |  |  |  |  |

| `admit_evidence` | function | admit_evidence(bundle: &EvidenceBundle, trust: &TrustStore, store: &HashMap<String, ReceiptV2>) -> EvidenceAdmission |  |  |  |  |

| `admit_failure_semantics` | function | admit_failure_semantics(input: &FailureSemanticsInput) -> FailureSemanticsDecision |  |  |  |  |

| `assess_materiality` | function | assess_materiality(event: &MaterialityEvent, policy: &MaterialityPolicy) -> Result<MaterialityAssessment, String> |  |  |  |  |

| `board_requirements` | function | board_requirements() -> &'static [Fortune5Requirement] |  |  |  |  |

| `build_board_package` | function | build_board_package(admission: &BoardAdmission, generated_at: &str, material_refused_subjects: u32, risk_appetite_breaches: u32) -> Result<BoardPackage, String> |  |  |  |  |

| `classify_icfr_subject` | function | classify_icfr_subject(input: &IcfrSubject) -> IcfrClassification |  |  |  |  |

| `decode` | function | decode(input: &str) -> Result<Vec<u8>, String> |  |  |  |  |

| `detect_segregation_of_duty_violations` | function | detect_segregation_of_duty_violations(assignments: &[RoleAssignment], incompatible_role_pairs: &[(String, String)]) -> Vec<SodViolation> |  |  |  |  |

| `encode` | function | encode(input: &[u8]) -> String |  |  |  |  |

| `issue_evidence_receipt` | function | issue_evidence_receipt(input: &EvidenceInput, context: &ReceiptIssueContext, signing_key: &SigningKey) -> Result<EvidenceBundle, String> |  |  |  |  |

| `qualify_fortune5_board` | function | qualify_fortune5_board(input: BoardAdmissionInput) -> BoardAdmission |  |  |  |  |

| `qualify_verified_fortune5` | function | qualify_verified_fortune5( bundles: &[EvidenceBundle], context: &QualificationContext, trust: &TrustStore, receipt_store: &HashMap<String, ReceiptV2>, requirements: &[Fortune5Requirement], ) -> VerifiedQualification |  |  |  |  |

| `verify_receipt_dag` | function | verify_receipt_dag(root: &ReceiptV2, trust: &TrustStore, store: &HashMap<String, ReceiptV2>) -> ReceiptVerification |  |  |  |  |

| `ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/` | str_key | ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/" |  |  |  |  |

| `Ed25519` | str_key | SIGNATURE_ALGORITHM = "Ed25519" |  |  |  |  |

| `alpha` | str_key | KEYS = "alpha" |  |  |  |  |

| `BoardAdmission` | struct | BoardAdmission { pub standing: Standing, pub enterprise: VerifiedQualification, pub castle: VerifiedQualification, pub reasons: Vec<String> } |  |  |  |  |

| `BoardAdmissionInput` | struct | BoardAdmissionInput { pub enterprise_context: QualificationContext, pub enterprise_evidence: Vec<EvidenceBundle>, pub castle_context: QualificationContext, pub castle_evidence: Vec<EvidenceBundle>, pub trust: &'a TrustStore, pub receipt_store: HashMap<String, ReceiptV2>, pub castle_assurance_domain: String, pub independent_assurance_domain: String } |  |  |  |  |

| `BoardPackage` | struct | BoardPackage { pub profile: &'static str, pub enterprise_subject: String, pub castle_subject: String, pub generated_at: String, pub enterprise_standing: Standing, pub castle_standing: Standing, pub material_refused_subjects: u32, pub risk_appetite_breaches: u32, pub control_count: usize, pub evidence_digest: String } |  |  |  |  |

| `EvidenceAdmission` | struct | EvidenceAdmission { pub standing: Standing, pub reasons: Vec<String>, pub observation: Option<MetricObservation> } |  |  |  |  |

| `EvidenceBundle` | struct | EvidenceBundle { pub observation: MetricObservation, pub receipt: ReceiptV2 } |  |  |  |  |

| `EvidenceInput` | struct | EvidenceInput { pub metric: String, pub value: crate::fortune5::MetricValue, pub subject: String, pub observed_at: String, pub epistemic_class: crate::fortune5::EvidenceEpistemicClass } |  |  |  |  |

| `FailureSemanticsDecision` | struct | FailureSemanticsDecision { pub standing: Standing, pub may_actuate: bool, pub reason: String } |  |  |  |  |

| `FailureSemanticsInput` | struct | FailureSemanticsInput { pub mode: FailureMode, pub castle_available: bool, pub local_capability_verified: bool, pub receipt_channel_available: bool } |  |  |  |  |

| `IcfrClassification` | struct | IcfrClassification { pub subject: String, pub in_scope: bool, pub reasons: Vec<String> } |  |  |  |  |

| `IcfrSubject` | struct | IcfrSubject { pub subject: String, pub processes: Vec<String>, pub material_accounts: Vec<String>, pub affects_financial_reporting: bool } |  |  |  |  |

| `MaterialityAssessment` | struct | MaterialityAssessment { pub material: bool, pub score_bps: i64, pub triggering_dimensions: Vec<MaterialityDimension>, pub escalate_by_epoch_ms: Option<i64>, pub policy_digest: String, pub authority_digest: String } |  |  |  |  |

| `MaterialityEvent` | struct | MaterialityEvent { pub id: String, pub subject: String, pub occurred_at_epoch_ms: i64, pub impact_bps: BTreeMap<MaterialityDimension, i64> } |  |  |  |  |

| `MaterialityPolicy` | struct | MaterialityPolicy { pub policy_digest: String, pub authority_digest: String, pub per_dimension_threshold_bps: BTreeMap<MaterialityDimension, i64>, pub aggregate_threshold_bps: i64, pub escalation_within_ms: i64 } |  |  |  |  |

| `ReceiptCoreV2` | struct | ReceiptCoreV2 { pub version: &'static str, pub algorithm: &'static str, pub signature_algorithm: &'static str, pub payload_digest: String, pub subject: String, pub metric: String, pub policy_digest: String, pub authority_digest: String, pub parent_digests: Vec<String>, pub key_id: String, pub trust_epoch: i64, pub issued_at: String, pub assurance_domain: String } |  |  |  |  |

| `ReceiptIssueContext` | struct | ReceiptIssueContext { pub policy_digest: String, pub authority_digest: String, pub parent_digests: Vec<String>, pub key_id: String, pub trust_epoch: i64, pub assurance_domain: String } |  |  |  |  |

| `ReceiptV2` | struct | ReceiptV2 { pub core: ReceiptCoreV2, pub receipt_digest: String, pub signature: String } |  |  |  |  |

| `ReceiptVerification` | struct | ReceiptVerification { pub standing: Standing, pub reasons: Vec<String>, pub verified_digests: Vec<String> } |  |  |  |  |

| `RoleAssignment` | struct | RoleAssignment { pub principal: String, pub role: String } |  |  |  |  |

| `SodViolation` | struct | SodViolation { pub principal: String, pub roles: (String, String) } |  |  |  |  |

| `TrustKey` | struct | TrustKey { pub key_id: String, pub verifying_key: VerifyingKey, pub valid_from_epoch: i64, pub revoked_at_epoch: Option<i64>, pub assurance_domain: String } |  |  |  |  |

| `TrustStore` | struct | TrustStore { pub current_epoch: i64, pub keys: HashMap<String, TrustKey> } |  |  |  |  |

| `VerifiedQualification` | struct | VerifiedQualification { pub standing: Standing, pub qualification: Fortune5Qualification, pub evidence_refusals: Vec<String> } |  |  |  |  |

| `AUTHORITY_CEILING` | const | AUTHORITY_CEILING: &str |  |  |  |  |

| `DONORS` | const | DONORS: &[CapabilityDonor] |  |  |  |  |

| `OWNER_CAPABILITY` | const | OWNER_CAPABILITY: &str |  |  |  |  |

| `PROJECTION_SOURCE` | const | PROJECTION_SOURCE: &str |  |  |  |  |

| `RUNTIME_CORE` | const | RUNTIME_CORE: &str |  |  |  |  |

| `do_authority` | function | do_authority(_repository: &str) -> bool |  |  |  |  |

| `donor` | function | donor(repository: &str) -> Option<&'static CapabilityDonor> |  |  |  |  |

| `CONSEQUENTIAL_ADMISSIBILITY` | str_key | OWNER_CAPABILITY = "CONSEQUENTIAL_ADMISSIBILITY" |  |  |  |  |

| `CONSTRUCT` | str_key | AUTHORITY_CEILING = "CONSTRUCT" |  |  |  |  |

| `seanchatmangpt/ash_surface` | str_key | DONORS = "seanchatmangpt/ash_surface" |  |  |  |  |

| `seanchatmangpt/ggen-ecosystem@50fdfa20c84205a80c6eb94e916cffbedc4b816e` | str_key | PROJECTION_SOURCE = "seanchatmangpt/ggen-ecosystem@50fdfa20c84205a80c6eb94e916cffbedc4b816e" |  |  |  |  |

| `seanchatmangpt/xaas` | str_key | RUNTIME_CORE = "seanchatmangpt/xaas" |  |  |  |  |

| `CapabilityDonor` | struct | CapabilityDonor { pub repository: &'static str, pub sha: &'static str, pub capability: &'static str, pub disposition: &'static str } |  |  |  |  |

| `EpistemicClass` | enum | EpistemicClass { Constructed, Counterfactual, Replayed, Observed, Inferred } |  |  |  |  |

| `GymActStatus` | enum | GymActStatus { Observed, Refused } |  |  |  |  |

| `admit_construct_for_do` | function | admit_construct_for_do( capability: &ConstructCapability, process: &PowlProcess, envelope: &TestEnvelope, blake3: &dyn Blake3Provider, verifier: &dyn ReceiptVerifier, policy: &ConstructTrustPolicy, now: impl Fn() -> i64, ) -> Result<ConstructAdmission, String> |  |  |  |  |

| `apply_zero_day_observation` | function | apply_zero_day_observation(graph: &DependencyGraph, observation: ZeroDayObservation) -> Result<ZeroDayImpact, String> |  |  |  |  |

| `as_str` | function | as_str(&self) -> &'static str |  |  |  |  |

| `compile_adversarial_classes` | function | compile_adversarial_classes(goals: &[AdversarialGoal], rules: &[TransitionRule], planners: &[Box<dyn Planner>]) -> Vec<CompiledAdversarialClass> |  |  |  |  |

| `compile_witness_to_powl` | function | compile_witness_to_powl(id: &str, vulnerability: &VulnerabilityCondition, rules: &[TransitionRule]) -> PowlProcess |  |  |  |  |

| `construct_compromise` | function | construct_compromise(&self, dependency_id: &str, capability: &str) -> Result<ConstructedCompromise, String> |  |  |  |  |

| `create_receipt` | function | create_receipt( artifact: &Value, epistemic_class: EpistemicClass, subject: &str, parent_digests: &[String], blake3: &dyn Blake3Provider, signer: &dyn ReceiptSigner, ) -> Result<Receipt, String> |  |  |  |  |

| `derive_vulnerabilities` | function | derive_vulnerabilities(goal: &AdversarialGoal, rules: &[TransitionRule], max_depth: u32) -> Vec<VulnerabilityCondition> |  |  |  |  |

| `enabled_activities` | function | enabled_activities(process: &PowlProcess, completed: &BTreeSet<String>) -> Vec<PowlActivity> |  |  |  |  |

| `execute_powl_with_gym_act` | function | execute_powl_with_gym_act( process: &PowlProcess, state: &WorldState, envelope: &TestEnvelope, gymact: &dyn GymActAdapter, authorization: DoAuthorizationContext<'_>, ) -> Result<ReceiptedOcelLog, String> |  |  |  |  |

| `impacted_closure` | function | impacted_closure(&self, changed_ids: I) -> Vec<String> |  |  |  |  |

| `local_docker_default` | function | local_docker_default(docker_bin: impl Into<String>) -> Self |  |  |  |  |

| `manufacture_construct_capability` | function | manufacture_construct_capability(request: ConstructRequest, blake3: &dyn Blake3Provider, signer: &dyn ReceiptSigner) -> Result<ConstructCapability, String> |  |  |  |  |

| `match_compiled_classes` | function | match_compiled_classes(classes: &'a [CompiledAdversarialClass], facts: &BTreeSet<Predicate>) -> Vec<&'a CompiledAdversarialClass> |  |  |  |  |

| `new` | function | new(nodes: Vec<DependencyNode>, edges: Vec<DependencyEdge>) -> Result<Self, String> |  |  |  |  |

| `platform_eng_colima_default` | function | platform_eng_colima_default() -> Self |  |  |  |  |

| `run_planner_ensemble` | function | run_planner_ensemble(problem: &PlanningProblem<'_>, planners: &[Box<dyn Planner>]) -> Vec<PlanCandidate> |  |  |  |  |

| `verify_receipt` | function | verify_receipt(artifact: &Value, receipt: &Receipt, blake3: &dyn Blake3Provider, verifier: &dyn ReceiptVerifier, trusted_origin_key_ids: &BTreeSet<String>) -> bool |  |  |  |  |

| `alpha` | str_key | KEYS = "alpha" |  |  |  |  |

| `ActuationPermit` | struct | ActuationPermit { pub construct_digest: String, pub process_digest: String, pub subject: String, pub authority: String, pub transition_id: String, pub expires_at_epoch_ms: i64 } |  |  |  |  |

| `AdmissionBrand` | struct |  |  |  |  |  |

| `AdversarialGoal` | struct | AdversarialGoal { pub id: String, pub predicate: Predicate, pub consequence: i64 } |  |  |  |  |

| `AutofdeLabPlanner` | struct | AutofdeLabPlanner { pub id: String, pub script_path: std::path::PathBuf, pub python_bin: String } |  |  |  |  |

| `CompiledAdversarialClass` | struct | CompiledAdversarialClass { pub key: String, pub goal: AdversarialGoal, pub vulnerability: VulnerabilityCondition, pub process: PowlProcess } |  |  |  |  |

| `ConstructAdmission` | struct | ConstructAdmission { pub standing: &'static str, pub construct_digest: String, pub process_digest: String, pub o_star_digest: String, pub config_graph_digest: String, pub ontology_digest: String, pub replay_identity_digest: String, pub subject: String, pub authority: String, pub allowed_transition_ids: Vec<String>, pub max_steps: u32, pub expires_at_epoch_ms: i64, _brand: sealed::AdmissionBrand } |  |  |  |  |

| `ConstructArtifact` | struct | ConstructArtifact { pub kind: &'static str, pub algorithm: &'static str, pub subject: String, pub authority: String, pub o_star_digest: String, pub config_graph_digest: String, pub ontology_digest: String, pub process_digest: String, pub replay_identity_digest: String, pub allowed_transition_ids: Vec<String>, pub max_steps: u32, pub expires_at_epoch_ms: i64 } |  |  |  |  |

| `ConstructCapability` | struct | ConstructCapability { pub sources: ConstructSources, pub artifact: ConstructArtifact, pub source_receipts: ConstructSourceReceipts, pub receipt: Receipt } |  |  |  |  |

| `ConstructRequest` | struct | ConstructRequest { pub subject: String, pub authority: String, pub o_star: Value, pub config_graph: Value, pub ontology: Value, pub process: PowlProcess, pub envelope: TestEnvelope } |  |  |  |  |

| `ConstructSourceReceipts` | struct | ConstructSourceReceipts { pub o_star: Receipt, pub config_graph: Receipt, pub ontology: Receipt, pub process: Receipt } |  |  |  |  |

| `ConstructSources` | struct | ConstructSources { pub o_star: Value, pub config_graph: Value, pub ontology: Value } |  |  |  |  |

| `ConstructTrustPolicy` | struct | ConstructTrustPolicy { pub trusted_origin_key_ids: BTreeSet<String>, pub allowed_authorities: BTreeSet<String> } |  |  |  |  |

| `ConstructedCompromise` | struct | ConstructedCompromise { pub dependency_id: String, pub capability: String, pub facts: Vec<Predicate>, pub impacted: Vec<String>, pub epistemic_class: &'static str } |  |  |  |  |

| `ContainerGymActAdapter` | struct | ContainerGymActAdapter { pub docker_bin: String, pub now_ms: fn() -> i64, pub allowed_images: BTreeMap<String, String> } |  |  |  |  |

| `CostMinimizingPlanner` | struct | CostMinimizingPlanner { pub id: String } |  |  |  |  |

| `DependencyEdge` | struct | DependencyEdge { pub from: String, pub to: String, pub relation: String } |  |  |  |  |

| `DependencyGraph` | struct | DependencyGraph { pub nodes: BTreeMap<String, DependencyNode>, pub edges: Vec<DependencyEdge>, dependents: BTreeMap<String, BTreeSet<String>> } |  |  |  |  |

| `DependencyNode` | struct | DependencyNode { pub id: String, pub kind: String } |  |  |  |  |

| `DoAuthorizationContext` | struct | DoAuthorizationContext { pub admission: &'a ConstructAdmission, pub blake3: &'a dyn Blake3Provider, pub receipt_signer: &'a dyn ReceiptSigner, pub now: Box<dyn Fn() -> i64 + 'a> } |  |  |  |  |

| `GymActResult` | struct | GymActResult { pub transition_id: String, pub status: GymActStatus, pub objects: Vec<OcelObject>, pub attributes: BTreeMap<String, Value> } |  |  |  |  |

| `KindClusterReadOnlyGymAct` | struct | KindClusterReadOnlyGymAct { pub kube_context: String, pub allowed_read_only_queries: BTreeMap<String, Vec<String>> } |  |  |  |  |

| `OcelEvent` | struct | OcelEvent { pub id: String, pub kind: String, pub time: String, pub attributes: BTreeMap<String, Value>, pub object_ids: Vec<String> } |  |  |  |  |

| `OcelLog` | struct | OcelLog { pub version: &'static str, pub objects: Vec<OcelObject>, pub events: Vec<OcelEvent> } |  |  |  |  |

| `OcelObject` | struct | OcelObject { pub id: String, pub kind: String } |  |  |  |  |

| `PlanCandidate` | struct | PlanCandidate { pub planner_id: String, pub process: PowlProcess, pub score: i64 } |  |  |  |  |

| `PlanningProblem` | struct | PlanningProblem { pub goal: &'a AdversarialGoal, pub vulnerability: &'a VulnerabilityCondition, pub rules: &'a [TransitionRule] } |  |  |  |  |

| `PowlActivity` | struct | PowlActivity { pub id: String, pub transition_id: String, pub predecessors: Vec<String> } |  |  |  |  |

| `PowlProcess` | struct | PowlProcess { pub id: String, pub goal_id: String, pub activities: Vec<PowlActivity> } |  |  |  |  |

| `ProcessGymActAdapter` | struct | ProcessGymActAdapter { pub gymact_bin: String, pub kube_context: String, pub allowed_verifications: BTreeMap<String, (String, Value)> } |  |  |  |  |

| `Receipt` | struct | Receipt { pub algorithm: &'static str, pub artifact_digest: String, pub receipt_digest: String, pub epistemic_class: EpistemicClass, pub subject: String, pub parent_digests: Vec<String>, pub origin_key_id: String, pub origin_signature: String } |  |  |  |  |

| `ReceiptedOcelLog` | struct | ReceiptedOcelLog { pub log: OcelLog, pub construct_digest: String, pub receipt: Receipt } |  |  |  |  |

| `TestEnvelope` | struct | TestEnvelope { pub system_id: String, pub allowed_transition_ids: BTreeSet<String>, pub max_steps: u32, pub expires_at_epoch_ms: i64 } |  |  |  |  |

| `TransitionRule` | struct | TransitionRule { pub id: String, pub preconditions: Vec<Predicate>, pub effects: Vec<Predicate>, pub cost: Option<f64>, pub planner_hint: Option<String> } |  |  |  |  |

| `VulnerabilityCondition` | struct | VulnerabilityCondition { pub goal_id: String, pub predicates: Vec<Predicate>, pub witness_transitions: Vec<String> } |  |  |  |  |

| `WitnessPlanner` | struct | WitnessPlanner { pub id: String } |  |  |  |  |

| `WorldState` | struct | WorldState { pub system_id: String, pub facts: BTreeSet<Predicate> } |  |  |  |  |

| `ZeroDayImpact` | struct | ZeroDayImpact { pub observation: ZeroDayObservation, pub impacted_dependencies: Vec<String>, pub newly_admitted_fact: Predicate } |  |  |  |  |

| `ZeroDayObservation` | struct | ZeroDayObservation { pub dependency_id: String, pub capability: String } |  |  |  |  |

| `Blake3Provider` | trait |  |  |  |  |  |

| `GymActAdapter` | trait |  |  |  |  |  |

| `Planner` | trait |  |  |  |  |  |

| `ReceiptSigner` | trait |  |  |  |  |  |

| `ReceiptVerifier` | trait |  |  |  |  |  |

| `DdUiRefusal` | enum | DdUiRefusal { IrreversiblePresentationSelection, RenderAuthorityEscalation, DirectDoFromUi, UnadmittedConstruct, NonBrceDo, UnreplayablePresentation } |  |  |  |  |

| `admit` | function | admit(&self) -> Result<(), DdUiRefusal> |  |  |  |  |

| `PresentationAuthority` | struct | PresentationAuthority { pub irreversible_selections: u32, pub render_actuation_authority: bool, pub output_kind: &'a str, pub construct_admitted: bool, pub do_route: &'a str, pub grammar_digest: Option<&'a str>, pub world_digest: Option<&'a str>, pub frontier_digest: Option<&'a str>, pub screen_digest: Option<&'a str> } |  |  |  |  |

| `EvidenceEpistemicClass` | enum | EvidenceEpistemicClass { Observed, Replayed, Inferred } |  |  |  |  |

| `MetricValue` | enum | MetricValue { Number(f64), Bool(bool), Str(String) } |  |  |  |  |

| `Standing` | enum | Standing { Alive, Refused, Unknown } |  |  |  |  |

| `admit_replay` | function | admit_replay(manifest: &ReplayManifest, subject: &ReplaySubject) -> ReplayAdmission |  |  |  |  |

| `as_str` | function | as_str(&self) -> &'static str |  |  |  |  |

| `minimum_impact_coverage` | function | minimum_impact_coverage(classes: &[AdversarialImpactClass], target_coverage_bps: i64) -> Result<ImpactCoverageSelection, String> |  |  |  |  |

| `qualify_fortune5` | function | qualify_fortune5( observations: &[MetricObservation], context: &QualificationContext, requirements: &[Fortune5Requirement], ) -> Fortune5Qualification |  |  |  |  |

| `qualify_fortune5_default` | function | qualify_fortune5_default(observations: &[MetricObservation], context: &QualificationContext) -> Fortune5Qualification |  |  |  |  |

| `AdversarialImpactClass` | struct | AdversarialImpactClass { pub key: String, pub impact: f64 } |  |  |  |  |

| `ControlEvaluation` | struct | ControlEvaluation { pub control_id: String, pub category: String, pub metric: String, pub standing: Standing, pub observed: Option<MetricValue>, pub expected: String, pub receipt_digest: Option<String>, pub reason: String } |  |  |  |  |

| `Fortune5Qualification` | struct | Fortune5Qualification { pub standing: Standing, pub subject: String, pub profile: &'static str, pub controls: Vec<ControlEvaluation>, pub alive: usize, pub refused: usize, pub unknown: usize, pub categories: BTreeMap<String, Standing> } |  |  |  |  |

| `ImpactCoverageSelection` | struct | ImpactCoverageSelection { pub selected: Vec<AdversarialImpactClass>, pub coverage_bps: i64, pub total_impact: f64, pub selected_impact: f64 } |  |  |  |  |

| `MetricObservation` | struct | MetricObservation { pub metric: String, pub value: MetricValue, pub receipt_digest: String, pub subject: String, pub observed_at: String, pub epistemic_class: EvidenceEpistemicClass } |  |  |  |  |

| `QualificationContext` | struct | QualificationContext { pub subject: String, pub now_epoch_ms: Option<i64>, pub max_evidence_age_ms: Option<i64> } |  |  |  |  |

| `ReplayAdmission` | struct | ReplayAdmission { pub standing: Standing, pub replay_class_id: String, pub reasons: Vec<String> } |  |  |  |  |

| `ReplayManifest` | struct | ReplayManifest { pub replay_class_id: String, pub structural_signature: String, pub ontology_version: String, pub provider_semantics_version: String, pub invariant_set_digest: String, pub process_digest: String } |  |  |  |  |

| `ReplaySubject` | struct | ReplaySubject { pub structural_signature: String, pub ontology_version: String, pub provider_semantics_version: String, pub invariant_set_digest: String, pub invariants_hold: bool } |  |  |  |  |

| `FORTUNE5_REQUIREMENTS` | const | FORTUNE5_REQUIREMENTS: &[Fortune5Requirement] |  |  |  |  |

| `F5-AUTH-001` | str_key | FORTUNE5_REQUIREMENTS = "F5-AUTH-001" |  |  |  |  |

| `Fortune5Requirement` | struct | Fortune5Requirement { pub order: u32, pub control_id: &'static str, pub category: &'static str, pub description: &'static str, pub metric: &'static str, pub comparator: &'static str, pub target: &'static str, pub authority: &'static str } |  |  |  |  |

| `GENERATED_BINDINGS` | const | GENERATED_BINDINGS: &[GeneratedBinding] |  |  |  |  |

| `default_adversarial_goals` | function | default_adversarial_goals() -> Vec<DefaultAdversarialGoal> |  |  |  |  |

| `generated_components` | function | generated_components() -> impl Iterator<Item = &'static GeneratedBinding> |  |  |  |  |

| `component` | str_key | GENERATED_BINDINGS = "component" |  |  |  |  |

| `DefaultAdversarialGoal` | struct | DefaultAdversarialGoal { pub id: &'static str, pub predicate: &'static str, pub consequence: i64 } |  |  |  |  |

| `GeneratedBinding` | struct | GeneratedBinding { pub kind: &'static str, pub order: u32, pub identifier: &'static str, pub slug: &'static str, pub role: &'static str, pub authority: &'static str, pub predicate: &'static str, pub consequence: i64 } |  |  |  |  |

| `DOCKER_BIN` | env_key | std::env::var("DOCKER_BIN") |  |  |  |  |

| `execute_default_container_observation` | function | execute_default_container_observation( process: &PowlProcess, state: &WorldState, envelope: &TestEnvelope, authorization: DoAuthorizationContext<'_>, docker_bin: impl Into<String>, ) -> Result<ReceiptedOcelLog, String> |  |  |  |  |

| `alpine:3.20` | str_key | DEFAULT_IMAGE = "alpine:3.20" |  |  |  |  |

| `observe-alpine-container` | str_key | DEFAULT_TRANSITION = "observe-alpine-container" |  |  |  |  |

| `castle::*` | use | castle::* |  |  |  |  |

| `dd_ui::{DdUiRefusal, PresentationAuthority}` | use | dd_ui::{DdUiRefusal, PresentationAuthority} |  |  |  |  |

| `generated::{ default_adversarial_goals, generated_components, DefaultAdversarialGoal, GeneratedBinding, GENERATED_BINDINGS, }` | use | generated::{ default_adversarial_goals, generated_components, DefaultAdversarialGoal, GeneratedBinding, GENERATED_BINDINGS, } |  |  |  |  |

| `gymact_container::execute_default_container_observation` | use | gymact_container::execute_default_container_observation |  |  |  |  |

| `planner_minimal::MinimalActionPlanner` | use | planner_minimal::MinimalActionPlanner |  |  |  |  |

| `reconstitution::{ admit_empire_reconstitution_for_construct, EmpireReconstitutionAdmission, FinalDisposition, ReconstitutedCapability, ReconstitutionRefusal, }` | use | reconstitution::{ admit_empire_reconstitution_for_construct, EmpireReconstitutionAdmission, FinalDisposition, ReconstitutedCapability, ReconstitutionRefusal, } |  |  |  |  |

| `refusal_conformance::{verify_refusal, RefusalObservation}` | use | refusal_conformance::{verify_refusal, RefusalObservation} |  |  |  |  |

| `EffectClass` | enum | EffectClass { Observe, Select, Construct, Do } |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `digest` | function | digest(&self) -> String |  |  |  |  |

| `to_json` | function | to_json(&self) -> Value |  |  |  |  |

| `OperationEnvelope` | struct | OperationEnvelope { pub operation_id: String, pub subject: String, pub intent: String, pub capability: String, pub context: Value, pub ontology_refs: Vec<String>, pub input: Value, pub actor: String, pub authority: String, pub effect_class: EffectClass, pub standing_requirements: Vec<String>, pub prior_receipts: Vec<String>, pub correlation: String } |  |  |  |  |

| `T_POST` | const | T_POST: &str |  |  |  |  |

| `T_RESERVE` | const | T_RESERVE: &str |  |  |  |  |

| `StepOutcome` | enum | StepOutcome { Reserved, Posted(LedgerEntry), Refused { code: &'static str, definite: bool } } |  |  |  |  |

| `outcomes` | function | outcomes(&self) -> Vec<StepOutcome> |  |  |  |  |

| `payments.post` | str_key | T_POST = "payments.post" |  |  |  |  |

| `payments.reserve` | str_key | T_RESERVE = "payments.reserve" |  |  |  |  |

| `PaymentGymActAdapter` | struct | PaymentGymActAdapter { ledger: &'a dyn LedgerPort, admission: &'a PaymentAdmission, subject: String, token: ActuationToken, outcomes: Mutex<Vec<StepOutcome>> } |  |  |  |  |

| `PAYMENT_SCREENING_REQUIRED` | const | PAYMENT_SCREENING_REQUIRED: &str |  |  |  |  |

| `admit_payment` | function | admit_payment( prepared: PreparedEffect, certificate: &ActuationCertificate, ctx: &AdmissionContext<'_>, ) -> PayResult<PaymentAdmission> |  |  |  |  |

| `admit_payment_screened` | function | admit_payment_screened( prepared: PreparedEffect, certificate: &ActuationCertificate, ctx: &AdmissionContext<'_>, screening: &Screening<'_>, ) -> PayResult<(PaymentAdmission, ScreeningEvidence)> |  |  |  |  |

| `effect` | function | effect(&self) -> &PaymentEffect |  |  |  |  |

| `generation` | function | generation(&self) -> u64 |  |  |  |  |

| `nonce` | function | nonce(&self) -> &str |  |  |  |  |

| `screening` | function | screening(&self) -> Option<&ScreeningEvidence> |  |  |  |  |

| `verification` | function | verification(&self) -> &VerificationReceipt |  |  |  |  |

| `REFUSED:PAYMENT_SCREENING_REQUIRED` | str_key | PAYMENT_SCREENING_REQUIRED = "REFUSED:PAYMENT_SCREENING_REQUIRED" |  |  |  |  |

| `AdmissionContext` | struct | AdmissionContext { pub registry: &'a KeyRegistry, pub epochs: SecurityEpochs, pub audience: &'a str, pub now_ms: u64, pub policy: &'a SpendPolicy, pub claims: &'a ClaimStore, pub nonces: &'a DurableNonceFence } |  |  |  |  |

| `PaymentAdmission` | struct | PaymentAdmission { effect: PaymentEffect, receipt: VerificationReceipt, generation: u64, nonce: String, screening: Option<ScreeningEvidence>, _seal: () } |  |  |  |  |

| `Screening` | struct | Screening { pub controls: &'a [&'a dyn ComplianceControl], pub counterparties: &'a CounterpartyRegistry } |  |  |  |  |

| `ScreeningEvidence` | struct | ScreeningEvidence { pub compliance_bundle_digest: String, pub counterparty_evidence_digest: String } |  |  |  |  |

| `ClaimState` | enum | ClaimState { Reserved, Executed, Refused, UnknownOutcome, Submitted, Final, Returned } |  |  |  |  |

| `get` | function | get(&self, digest: &str) -> PayResult<Option<Claim>> |  |  |  |  |

| `list` | function | list(&self) -> PayResult<Vec<Claim>> |  |  |  |  |

| `open` | function | open(root: impl Into<PathBuf>) -> PayResult<Self> |  |  |  |  |

| `reserve` | function | reserve(&self, claim: &Claim, epoch_cap: Option<u64>) -> PayResult<()> |  |  |  |  |

| `reversed_total` | function | reversed_total(&self, original: &str) -> PayResult<u64> |  |  |  |  |

| `transition` | function | transition( &self, digest: &str, from: &[ClaimState], to: ClaimState, construct_digest: Option<&str>, detail: &str, ) -> PayResult<Claim> |  |  |  |  |

| `Claim` | struct | Claim { pub effect_digest: String, pub principal: String, pub payer: String, pub payee: String, pub amount_minor: u64, pub currency: Currency, pub generation: u64, pub state: ClaimState, pub reverses: Option<String>, pub construct_digest: Option<String>, pub detail: String, pub obligation_id: String, pub purpose: String, pub audience: String, pub verified_custodian_ids: Vec<String> } |  |  |  |  |

| `ClaimStore` | struct | ClaimStore { root: PathBuf, lock: Mutex<()> } |  |  |  |  |

| `COMPLIANCE_CONTROL_DUPLICATE` | const | COMPLIANCE_CONTROL_DUPLICATE: &str |  |  |  |  |

| `NO_COMPLIANCE_CONTROLS` | const | NO_COMPLIANCE_CONTROLS: &str |  |  |  |  |

| `SANCTIONS_HIT` | const | SANCTIONS_HIT: &str |  |  |  |  |

| `SANCTIONS_LIST_INVALID` | const | SANCTIONS_LIST_INVALID: &str |  |  |  |  |

| `from_json` | function | from_json(bytes: &[u8]) -> PayResult<Self> |  |  |  |  |

| `normalize` | function | normalize(text: &str) -> String |  |  |  |  |

| `run_controls` | function | run_controls(controls: &[&dyn ComplianceControl], effect: &PaymentEffect) -> PayResult<ComplianceBundle> |  |  |  |  |

| `with_aliases` | function | with_aliases(mut self, aliases: &[(&str, &str)]) -> Self |  |  |  |  |

| `CASTLE-COMPLIANCE-BUNDLE-V1` | str_key | BUNDLE_DOMAIN = "CASTLE-COMPLIANCE-BUNDLE-V1" |  |  |  |  |

| `CASTLE-CONTROL-EVIDENCE-V1` | str_key | EVIDENCE_DOMAIN = "CASTLE-CONTROL-EVIDENCE-V1" |  |  |  |  |

| `CASTLE-SANCTIONS-LIST-V1` | str_key | LIST_DOMAIN = "CASTLE-SANCTIONS-LIST-V1" |  |  |  |  |

| `REFUSED:COMPLIANCE_CONTROL_DUPLICATE` | str_key | COMPLIANCE_CONTROL_DUPLICATE = "REFUSED:COMPLIANCE_CONTROL_DUPLICATE" |  |  |  |  |

| `REFUSED:NO_COMPLIANCE_CONTROLS` | str_key | NO_COMPLIANCE_CONTROLS = "REFUSED:NO_COMPLIANCE_CONTROLS" |  |  |  |  |

| `REFUSED:PAYMENT_SANCTIONS_HIT` | str_key | SANCTIONS_HIT = "REFUSED:PAYMENT_SANCTIONS_HIT" |  |  |  |  |

| `REFUSED:SANCTIONS_LIST_INVALID` | str_key | SANCTIONS_LIST_INVALID = "REFUSED:SANCTIONS_LIST_INVALID" |  |  |  |  |

| `ComplianceBundle` | struct | ComplianceBundle { pub evidence: Vec<ControlEvidence>, pub bundle_digest: String } |  |  |  |  |

| `ControlEvidence` | struct | ControlEvidence { pub control_id: String, pub version: String, pub outcome: String, pub evidence_digest: String } |  |  |  |  |

| `SanctionsList` | struct | SanctionsList { pub list_id: String, pub version: String, pub entries: BTreeSet<String>, pub digest: String, aliases: BTreeMap<String, BTreeSet<String>> } |  |  |  |  |

| `ComplianceControl` | trait |  |  |  |  |  |

| `ACCOUNT_AMBIGUOUS` | const | ACCOUNT_AMBIGUOUS: &str |  |  |  |  |

| `COUNTERPARTY_UNVERIFIED` | const | COUNTERPARTY_UNVERIFIED: &str |  |  |  |  |

| `LEI_INVALID` | const | LEI_INVALID: &str |  |  |  |  |

| `REGISTRY_INVALID` | const | REGISTRY_INVALID: &str |  |  |  |  |

| `as_str` | function | as_str(&self) -> &str |  |  |  |  |

| `evidence_digest` | function | evidence_digest(&self, payer: &Counterparty, payee: &Counterparty) -> String |  |  |  |  |

| `fibo_party_json` | function | fibo_party_json(cp: &Counterparty) -> Value |  |  |  |  |

| `from_json` | function | from_json(bytes: &[u8]) -> PayResult<Self> |  |  |  |  |

| `parse` | function | parse(text: &str) -> PayResult<Self> |  |  |  |  |

| `resolve` | function | resolve(&self, account: &str) -> PayResult<&Counterparty> |  |  |  |  |

| `with_check_digits` | function | with_check_digits(prefix18: &str) -> PayResult<Self> |  |  |  |  |

| `with_require_lei` | function | with_require_lei(mut self, require: bool) -> Self |  |  |  |  |

| `CASTLE-COUNTERPARTY-EVIDENCE-V1` | str_key | EVIDENCE_DOMAIN = "CASTLE-COUNTERPARTY-EVIDENCE-V1" |  |  |  |  |

| `REFUSED:COUNTERPARTY_ACCOUNT_AMBIGUOUS` | str_key | ACCOUNT_AMBIGUOUS = "REFUSED:COUNTERPARTY_ACCOUNT_AMBIGUOUS" |  |  |  |  |

| `REFUSED:COUNTERPARTY_LEI_INVALID` | str_key | LEI_INVALID = "REFUSED:COUNTERPARTY_LEI_INVALID" |  |  |  |  |

| `REFUSED:COUNTERPARTY_REGISTRY_INVALID` | str_key | REGISTRY_INVALID = "REFUSED:COUNTERPARTY_REGISTRY_INVALID" |  |  |  |  |

| `REFUSED:COUNTERPARTY_UNVERIFIED` | str_key | COUNTERPARTY_UNVERIFIED = "REFUSED:COUNTERPARTY_UNVERIFIED" |  |  |  |  |

| `Counterparty` | struct | Counterparty { pub party_id: String, pub legal_name: String, pub lei: Option<Lei>, pub accounts: BTreeSet<String> } |  |  |  |  |

| `CounterpartyRegistry` | struct | CounterpartyRegistry { parties: Vec<Counterparty>, by_account: BTreeMap<String, usize>, require_lei: bool } |  |  |  |  |

| `Lei` | struct | Lei { String } |  |  |  |  |

| `acquire` | function | acquire(root: &Path) -> PayResult<Self> |  |  |  |  |

| `publish_new` | function | publish_new(root: &Path, final_path: &Path, bytes: &[u8]) -> PayResult<bool> |  |  |  |  |

| `publish_replace` | function | publish_replace(root: &Path, final_path: &Path, bytes: &[u8]) -> PayResult<()> |  |  |  |  |

| `sync_dir` | function | sync_dir(root: &Path) -> PayResult<()> |  |  |  |  |

| `DirLock` | struct | DirLock { path: PathBuf } |  |  |  |  |

| `PAYMENT_CAPABILITY` | const | PAYMENT_CAPABILITY: &str |  |  |  |  |

| `digest` | function | digest(&self) -> &str |  |  |  |  |

| `from_prepared` | function | from_prepared(prepared: PreparedEffect) -> PayResult<Self> |  |  |  |  |

| `invoice_ref` | function | invoice_ref(&self) -> Option<&str> |  |  |  |  |

| `money` | function | money(&self) -> Money |  |  |  |  |

| `obligation_id` | function | obligation_id(&self) -> &str |  |  |  |  |

| `payee` | function | payee(&self) -> &str |  |  |  |  |

| `payer` | function | payer(&self) -> &str |  |  |  |  |

| `prepare` | function | prepare( principal: &str, payer: &str, payee: &str, amount_minor: &str, currency: Currency, obligation_id: &str, purpose: &str, reverses: Option<&str>, ) -> PayResult<PreparedEffect> |  |  |  |  |

| `prepare_with_invoice` | function | prepare_with_invoice( principal: &str, payer: &str, payee: &str, amount_minor: &str, currency: Currency, obligation_id: &str, purpose: &str, invoice_ref: Option<&str>, reverses: Option<&str>, ) -> PayResult<PreparedEffect> |  |  |  |  |

| `prepared` | function | prepared(&self) -> &PreparedEffect |  |  |  |  |

| `principal` | function | principal(&self) -> &str |  |  |  |  |

| `purpose` | function | purpose(&self) -> &str |  |  |  |  |

| `reverses` | function | reverses(&self) -> Option<&str> |  |  |  |  |

| `account_transfer` | str_key | SUBJECT_KIND = "account_transfer" |  |  |  |  |

| `payments.transfer.v1` | str_key | PAYMENT_CAPABILITY = "payments.transfer.v1" |  |  |  |  |

| `PaymentEffect` | struct | PaymentEffect { prepared: PreparedEffect, digest: String, payer: String, payee: String, money: Money, obligation_id: String, purpose: String, reverses: Option<String>, invoice_ref: Option<String> } |  |  |  |  |

| `EVENT_RECEIPT_INCOMPLETE` | const | EVENT_RECEIPT_INCOMPLETE: &str |  |  |  |  |

| `EVENT_RECEIPT_TAMPERED` | const | EVENT_RECEIPT_TAMPERED: &str |  |  |  |  |

| `explain` | function | explain(receipt: &EventReceipt) -> Value |  |  |  |  |

| `ledger_entry_digest` | function | ledger_entry_digest(entry: &LedgerEntry) -> String |  |  |  |  |

| `seal_event_receipt` | function | seal_event_receipt(inputs: &EventInputs) -> PayResult<EventReceipt> |  |  |  |  |

| `verify_event_receipt` | function | verify_event_receipt(receipt: &EventReceipt) -> PayResult<()> |  |  |  |  |

| `CASTLE-EVENT-RECEIPT-V1` | str_key | RECEIPT_DOMAIN = "CASTLE-EVENT-RECEIPT-V1" |  |  |  |  |

| `CASTLE-LEDGER-ENTRY-V1` | str_key | LEDGER_DOMAIN = "CASTLE-LEDGER-ENTRY-V1" |  |  |  |  |

| `REFUSED:EVENT_RECEIPT_INCOMPLETE` | str_key | EVENT_RECEIPT_INCOMPLETE = "REFUSED:EVENT_RECEIPT_INCOMPLETE" |  |  |  |  |

| `REFUSED:EVENT_RECEIPT_TAMPERED` | str_key | EVENT_RECEIPT_TAMPERED = "REFUSED:EVENT_RECEIPT_TAMPERED" |  |  |  |  |

| `EventInputs` | struct | EventInputs { pub effect_id: String, pub obligation_id: String, pub admission_decision_digest: String, pub pee_effect_id: Option<String>, pub construct_digest: Option<String>, pub brce_prepare_digests: Vec<String>, pub brce_outcome_digests: Vec<String>, pub rail_correlation_id: Option<String>, pub rail_payload_digest: Option<String>, pub finality_evidence_digest: Option<String>, pub ledger_entry_digest: Option<String>, pub claim_state: String, pub compliance_bundle_digest: Option<String>, pub counterparty_evidence_digest: Option<String>, pub settled_at_ms: Option<u64> } |  |  |  |  |

| `EventReceipt` | struct | EventReceipt { pub inputs: EventInputs, pub receipt_digest: String } |  |  |  |  |

| `PaymentStanding` | enum | PaymentStanding { Settled, Refused, UnknownOutcome } |  |  |  |  |

| `build_construct` | function | build_construct( admission: &PaymentAdmission, ctx: &ExecutionContext<'_>, ) -> PayResult<(ConstructAdmission, PowlProcess, TestEnvelope)> |  |  |  |  |

| `build_construct_with` | function | build_construct_with( admission: &PaymentAdmission, ctx: &ExecutionContext<'_>, process: PowlProcess, allowed_transitions: BTreeSet<String>, ) -> PayResult<(ConstructAdmission, PowlProcess, TestEnvelope)> |  |  |  |  |

| `execute_payment` | function | execute_payment(admission: PaymentAdmission, ctx: &ExecutionContext<'_>) -> PayResult<PaymentExecution> |  |  |  |  |

| `ExecutionContext` | struct | ExecutionContext { pub blake3: &'a dyn Blake3Provider, pub signer: &'a dyn ReceiptSigner, pub verifier: &'a dyn ReceiptVerifier, pub allowed_authorities: BTreeSet<String>, pub journal_root: PathBuf, pub now_epoch_ms: i64, pub ledger: &'a dyn LedgerPort, pub claims: &'a ClaimStore, pub policy: &'a SpendPolicy } |  |  |  |  |

| `PaymentExecution` | struct | PaymentExecution { pub standing: PaymentStanding, pub effect_digest: String, pub construct_digest: Option<String>, pub ocel_receipt_digest: Option<String>, pub brce_prepare_receipt_digests: Vec<String>, pub brce_outcome_receipt_digests: Vec<String>, pub ledger_entry: Option<LedgerEntry>, pub detail: String } |  |  |  |  |

| `NOT_ABANDONABLE` | const | NOT_ABANDONABLE: &str |  |  |  |  |

| `RAIL_HOLD_UNAVAILABLE` | const | RAIL_HOLD_UNAVAILABLE: &str |  |  |  |  |

| `RAIL_NEVER_SAW_EFFECT` | const | RAIL_NEVER_SAW_EFFECT: &str |  |  |  |  |

| `RAIL_NEVER_SUBMITTED` | const | RAIL_NEVER_SUBMITTED: &str |  |  |  |  |

| `RAIL_TIME_INVALID` | const | RAIL_TIME_INVALID: &str |  |  |  |  |

| `T_HOLD` | const | T_HOLD: &str |  |  |  |  |

| `T_RAIL_SUBMIT` | const | T_RAIL_SUBMIT: &str |  |  |  |  |

| `FinalizeResult` | enum | FinalizeResult { Pending, Applied { outcome: FinalityOutcome, evidence_digest: String }, StillUnknown, ProvenAbsent } |  |  |  |  |

| `RailStandingAfterSubmit` | enum | RailStandingAfterSubmit { Submitted, Refused, UnknownOutcome } |  |  |  |  |

| `abandon_unsubmitted` | function | abandon_unsubmitted( claims: &ClaimStore, ledger: &dyn LedgerPort, rail: &dyn RailActuator, effect_digest: &str, ) -> PayResult<FinalizeResult> |  |  |  |  |

| `finalize_via_rail` | function | finalize_via_rail( effect_digest: &str, claims: &ClaimStore, ledger: &dyn LedgerPort, rail: &dyn RailActuator, ) -> PayResult<FinalizeResult> |  |  |  |  |

| `submit_via_rail` | function | submit_via_rail( admission: PaymentAdmission, params: &RailExecutionParams, ctx: &ExecutionContext<'_>, rail: &dyn RailActuator, ) -> PayResult<RailSubmission> |  |  |  |  |

| `CASTLE-RAIL-LOCAL-EVIDENCE-V1` | str_key | LOCAL_EVIDENCE_DOMAIN = "CASTLE-RAIL-LOCAL-EVIDENCE-V1" |  |  |  |  |

| `RAIL_NEVER_SAW_EFFECT` | str_key | RAIL_NEVER_SAW_EFFECT = "RAIL_NEVER_SAW_EFFECT" |  |  |  |  |

| `RAIL_NEVER_SUBMITTED` | str_key | RAIL_NEVER_SUBMITTED = "RAIL_NEVER_SUBMITTED" |  |  |  |  |

| `REFUSED:PAYMENT_NOT_ABANDONABLE` | str_key | NOT_ABANDONABLE = "REFUSED:PAYMENT_NOT_ABANDONABLE" |  |  |  |  |

| `REFUSED:PAYMENT_RAIL_HOLD_UNAVAILABLE` | str_key | RAIL_HOLD_UNAVAILABLE = "REFUSED:PAYMENT_RAIL_HOLD_UNAVAILABLE" |  |  |  |  |

| `REFUSED:PAYMENT_RAIL_TIME_INVALID` | str_key | RAIL_TIME_INVALID = "REFUSED:PAYMENT_RAIL_TIME_INVALID" |  |  |  |  |

| `payments.hold` | str_key | T_HOLD = "payments.hold" |  |  |  |  |

| `payments.rail_submit` | str_key | T_RAIL_SUBMIT = "payments.rail_submit" |  |  |  |  |

| `RailExecutionParams` | struct | RailExecutionParams { pub bindings: EffectBindings, pub created_at_iso: String, pub debtor_name: String, pub creditor_name: String, pub debtor_agent_bic: String, pub creditor_agent_bic: String, pub rail_profile: String } |  |  |  |  |

| `RailSubmission` | struct | RailSubmission { pub effect_id: String, pub correlation_id: String, pub standing: RailStandingAfterSubmit, pub construct_digest: Option<String>, pub brce_prepare_receipt_digests: Vec<String>, pub brce_outcome_receipt_digests: Vec<String>, pub pee: PreparedEconomicEffect, pub payload_digest: String, pub detail: String } |  |  |  |  |

| `Knowledge` | enum | Knowledge { Known(ClassRule), Unknown } |  |  |  |  |

| `classify` | function | classify(&self, effect: &PaymentEffect) -> PayResult<Knowledge> |  |  |  |  |

| `key` | function | key(&self) -> String |  |  |  |  |

| `of` | function | of(effect: &PaymentEffect) -> Self |  |  |  |  |

| `open` | function | open(root: impl Into<PathBuf>) -> PayResult<Self> |  |  |  |  |

| `record_settled` | function | record_settled(&self, effect: &PaymentEffect, exec: &PaymentExecution) -> PayResult<()> |  |  |  |  |

| `requires_intelligence` | function | requires_intelligence(k: &Knowledge) -> bool |  |  |  |  |

| `BLOCKED:EXPERIENCE_STORE_FAILED` | str_key | STORE_FAILED = "BLOCKED:EXPERIENCE_STORE_FAILED" |  |  |  |  |

| `CASTLE-PAYMENT-CLASS-V1` | str_key | CLASS_DOMAIN = "CASTLE-PAYMENT-CLASS-V1" |  |  |  |  |

| `REFUSED:EXPERIENCE_REQUIRES_SETTLED_RECEIPT` | str_key | REQUIRES_SETTLED = "REFUSED:EXPERIENCE_REQUIRES_SETTLED_RECEIPT" |  |  |  |  |

| `ClassRule` | struct | ClassRule { pub class_key: String, pub per_effect_cap_minor: u64, pub settled_count: u64, pub last_construct_digest: String, pub last_ocel_receipt_digest: String } |  |  |  |  |

| `ExperienceStore` | struct | ExperienceStore { root: PathBuf } |  |  |  |  |

| `PaymentClass` | struct | PaymentClass { principal: String, payer: String, payee: String, currency: Currency, purpose: String } |  |  |  |  |

| `CASTLE_NS` | const | CASTLE_NS: &str |  |  |  |  |

| `CASTLE_PAYMENT_EFFECT` | const | CASTLE_PAYMENT_EFFECT: &str |  |  |  |  |

| `claim_jsonld` | function | claim_jsonld(claim: &Claim, entry: Option<&LedgerEntry>) -> Value |  |  |  |  |

| `claim_type_iri` | function | claim_type_iri(claim: &Claim) -> &'static str |  |  |  |  |

| `currency_iri` | function | currency_iri(currency: Currency) -> &'static str |  |  |  |  |

| `mapping_for` | function | mapping_for(id: &str) -> Option<&'static FiboMapping> |  |  |  |  |

| `monetary_amount_json` | function | monetary_amount_json(money: &Money) -> Value |  |  |  |  |

| `unverified_terms` | function | unverified_terms() -> Vec<&'static FiboMapping> |  |  |  |  |

| `https://chatmangpt.com/ontology/castle#` | str_key | CASTLE_NS = "https://chatmangpt.com/ontology/castle#" |  |  |  |  |

| `https://chatmangpt.com/ontology/castle#PaymentEffect` | str_key | CASTLE_PAYMENT_EFFECT = "https://chatmangpt.com/ontology/castle#PaymentEffect" |  |  |  |  |

| `https://chatmangpt.com/ontology/castle#constructDigest` | str_key | CASTLE_CONSTRUCT_DIGEST = "https://chatmangpt.com/ontology/castle#constructDigest" |  |  |  |  |

| `https://chatmangpt.com/ontology/castle#effectDigest` | str_key | CASTLE_EFFECT_DIGEST = "https://chatmangpt.com/ontology/castle#effectDigest" |  |  |  |  |

| `https://chatmangpt.com/ontology/castle#minorUnits` | str_key | CASTLE_MINOR_UNITS = "https://chatmangpt.com/ontology/castle#minorUnits" |  |  |  |  |

| `https://chatmangpt.com/ontology/castle#obligation` | str_key | CASTLE_OBLIGATION = "https://chatmangpt.com/ontology/castle#obligation" |  |  |  |  |

| `https://chatmangpt.com/ontology/castle#obligationId` | str_key | CASTLE_OBLIGATION_ID = "https://chatmangpt.com/ontology/castle#obligationId" |  |  |  |  |

| `https://chatmangpt.com/ontology/castle#payeeAccount` | str_key | CASTLE_PAYEE = "https://chatmangpt.com/ontology/castle#payeeAccount" |  |  |  |  |

| `https://chatmangpt.com/ontology/castle#payerAccount` | str_key | CASTLE_PAYER = "https://chatmangpt.com/ontology/castle#payerAccount" |  |  |  |  |

| `ISO20022_PROFILE` | const | ISO20022_PROFILE: &str |  |  |  |  |

| `PROJECTION_FIELD_INVALID` | const | PROJECTION_FIELD_INVALID: &str |  |  |  |  |

| `message_profile_version` | function | message_profile_version() -> &'static str |  |  |  |  |

| `pacs008_fi_credit_transfer` | function | pacs008_fi_credit_transfer( admission: &PaymentAdmission, created_at_iso: &str, instructing_agent_bic: &str, instructed_agent_bic: &str, debtor_agent_bic: &str, creditor_agent_bic: &str, ) -> PayResult<String> |  |  |  |  |

| `pain001_customer_credit_transfer` | function | pain001_customer_credit_transfer( admission: &PaymentAdmission, created_at_iso: &str, debtor_name: &str, creditor_name: &str, debtor_agent_bic: &str, creditor_agent_bic: &str, ) -> PayResult<String> |  |  |  |  |

| `project_effect_digest_from_pain001` | function | project_effect_digest_from_pain001(xml: &str) -> Option<String> |  |  |  |  |

| `project_obligation_id_from_pain001` | function | project_obligation_id_from_pain001(xml: &str) -> Option<String> |  |  |  |  |

| `REFUSED:PROJECTION_FIELD_INVALID` | str_key | PROJECTION_FIELD_INVALID = "REFUSED:PROJECTION_FIELD_INVALID" |  |  |  |  |

| `castle:effect-digest=` | str_key | DIGEST_MARKER = "castle:effect-digest=" |  |  |  |  |

| `castle:obligation-id=` | str_key | OBLIGATION_MARKER = "castle:obligation-id=" |  |  |  |  |

| `{` | str_key | ISO20022_PROFILE = "{" |  |  |  |  |

| `LedgerError` | enum | LedgerError { InsufficientFunds, Unavailable(String) } |  |  |  |  |

| `conserves` | function | conserves(&self, currency: Currency) -> Result<bool, LedgerError> |  |  |  |  |

| `construct_digest` | function | construct_digest(&self) -> &str |  |  |  |  |

| `entries` | function | entries(&self) -> Result<Vec<LedgerEntry>, LedgerError> |  |  |  |  |

| `from_construct` | function | from_construct(admission: &ConstructAdmission) -> Self |  |  |  |  |

| `open` | function | open(root: impl Into<PathBuf>, opening: &[(&str, Currency, u64)]) -> Result<Self, LedgerError> |  |  |  |  |

| `returns` | function | returns(&self) -> Result<Vec<LedgerEntry>, LedgerError> |  |  |  |  |

| `ActuationToken` | struct | ActuationToken { construct_digest: String } |  |  |  |  |

| `FileJournalLedger` | struct | FileJournalLedger { root: PathBuf, lock: Mutex<()> } |  |  |  |  |

| `LedgerEntry` | struct | LedgerEntry { pub seq: u64, pub effect_digest: String, pub debit_account: String, pub credit_account: String, pub amount_minor: u64, pub currency: Currency, pub construct_digest: String } |  |  |  |  |

| `LedgerHold` | struct | LedgerHold { pub effect_digest: String, pub account: String, pub currency: Currency, pub amount_minor: u64, pub construct_digest: String } |  |  |  |  |

| `LedgerPort` | trait |  |  |  |  |  |

| `VERSION` | const | VERSION: &str |  |  |  |  |

| `26.9.29` | str_key | VERSION = "26.9.29" |  |  |  |  |

| `admission::{admit_payment, AdmissionContext, PaymentAdmission}` | use | admission::{admit_payment, AdmissionContext, PaymentAdmission} |  |  |  |  |

| `admission::{admit_payment_screened, Screening, ScreeningEvidence}` | use | admission::{admit_payment_screened, Screening, ScreeningEvidence} |  |  |  |  |

| `claim_store::{Claim, ClaimState, ClaimStore}` | use | claim_store::{Claim, ClaimState, ClaimStore} |  |  |  |  |

| `effect::{PaymentEffect, PAYMENT_CAPABILITY}` | use | effect::{PaymentEffect, PAYMENT_CAPABILITY} |  |  |  |  |

| `event_receipt::{explain, seal_event_receipt, verify_event_receipt, EventInputs, EventReceipt}` | use | event_receipt::{explain, seal_event_receipt, verify_event_receipt, EventInputs, EventReceipt} |  |  |  |  |

| `execute::{build_construct, execute_payment, ExecutionContext, PaymentExecution, PaymentStanding}` | use | execute::{build_construct, execute_payment, ExecutionContext, PaymentExecution, PaymentStanding} |  |  |  |  |

| `execute_rail::{abandon_unsubmitted, finalize_via_rail, submit_via_rail, FinalizeResult, RailExecutionParams, RailStandingAfterSubmit, RailSubmission}` | use | execute_rail::{abandon_unsubmitted, finalize_via_rail, submit_via_rail, FinalizeResult, RailExecutionParams, RailStandingAfterSubmit, RailSubmission} |  |  |  |  |

| `experience::{ClassRule, ExperienceStore, Knowledge, PaymentClass}` | use | experience::{ClassRule, ExperienceStore, Knowledge, PaymentClass} |  |  |  |  |

| `ledger::{ActuationToken, FileJournalLedger, LedgerEntry, LedgerError, LedgerHold, LedgerPort}` | use | ledger::{ActuationToken, FileJournalLedger, LedgerEntry, LedgerError, LedgerHold, LedgerPort} |  |  |  |  |

| `money::{Currency, Money}` | use | money::{Currency, Money} |  |  |  |  |

| `nonce::DurableNonceFence` | use | nonce::DurableNonceFence |  |  |  |  |

| `pee::{EffectBindings, PreparedEconomicEffect}` | use | pee::{EffectBindings, PreparedEconomicEffect} |  |  |  |  |

| `policy::{PrincipalPolicy, SpendPolicy}` | use | policy::{PrincipalPolicy, SpendPolicy} |  |  |  |  |

| `rail::{RailAck, RailActuator, RailError, RailInstruction, RailStatus}` | use | rail::{RailAck, RailActuator, RailError, RailInstruction, RailStatus} |  |  |  |  |

| `rail_sim::{SimMode, SimRail}` | use | rail_sim::{SimMode, SimRail} |  |  |  |  |

| `reconcile::{reconcile, recover_journal, JournalRecovery, ReconcileResolution}` | use | reconcile::{reconcile, recover_journal, JournalRecovery, ReconcileResolution} |  |  |  |  |

| `replay::{admit_payment_journaled, admit_payment_screened_journaled, replay_admission, replay_admission_anchored, AdmissionJournal, AdmissionRecord, ReplayVerdict}` | use | replay::{admit_payment_journaled, admit_payment_screened_journaled, replay_admission, replay_admission_anchored, AdmissionJournal, AdmissionRecord, ReplayVerdict} |  |  |  |  |

| `settlement::{apply_rail_report, observe_rail, FinalityKind, FinalityOutcome, RailReport}` | use | settlement::{apply_rail_report, observe_rail, FinalityKind, FinalityOutcome, RailReport} |  |  |  |  |

| `Currency` | enum | Currency { USD, EUR, GBP, JPY, KWD } |  |  |  |  |

| `code` | function | code(self) -> &'static str |  |  |  |  |

| `exponent` | function | exponent(self) -> u32 |  |  |  |  |

| `parse_minor` | function | parse_minor(text: &str, currency: Currency) -> PayResult<Self> |  |  |  |  |

| `to_decimal_string` | function | to_decimal_string(self) -> String |  |  |  |  |

| `Money` | struct | Money { pub minor: u64, pub currency: Currency } |  |  |  |  |

| `claim` | function | claim(&self, principal: &str, nonce: &str) -> PayResult<()> |  |  |  |  |

| `open` | function | open(root: impl Into<PathBuf>) -> PayResult<Self> |  |  |  |  |

| `DurableNonceFence` | struct | DurableNonceFence { root: PathBuf } |  |  |  |  |

| `OBLIGATION_ID_NOT_DERIVED` | const | OBLIGATION_ID_NOT_DERIVED: &str |  |  |  |  |

| `derive_obligation_id` | function | derive_obligation_id(payer: &str, payee: &str, purpose: &str, invoice_ref: &str) -> String |  |  |  |  |

| `prepare_for_invoice` | function | prepare_for_invoice( principal: &str, payer: &str, payee: &str, amount_minor: &str, currency: Currency, purpose: &str, invoice_ref: &str, reverses: Option<&str>, ) -> PayResult<PreparedEffect> |  |  |  |  |

| `CASTLE-OBLIGATION-V1` | str_key | OBLIGATION_DOMAIN = "CASTLE-OBLIGATION-V1" |  |  |  |  |

| `REFUSED:PAYMENT_OBLIGATION_ID_NOT_DERIVED` | str_key | OBLIGATION_ID_NOT_DERIVED = "REFUSED:PAYMENT_OBLIGATION_ID_NOT_DERIVED" |  |  |  |  |

| `PEE_BINDING_INCOMPLETE` | const | PEE_BINDING_INCOMPLETE: &str |  |  |  |  |

| `PEE_EXPIRED` | const | PEE_EXPIRED: &str |  |  |  |  |

| `PEE_IDENTITY_MISMATCH` | const | PEE_IDENTITY_MISMATCH: &str |  |  |  |  |

| `PEE_NOT_YET_VALID` | const | PEE_NOT_YET_VALID: &str |  |  |  |  |

| `check_fresh` | function | check_fresh(&self, now_ms: u64) -> PayResult<()> |  |  |  |  |

| `reseal_identity` | function | reseal_identity(&mut self) -> PayResult<()> |  |  |  |  |

| `seal` | function | seal(admission: &PaymentAdmission, b: &EffectBindings) -> PayResult<Self> |  |  |  |  |

| `verify_against` | function | verify_against(&self, admission: &PaymentAdmission) -> PayResult<()> |  |  |  |  |

| `verify_identity` | function | verify_identity(&self) -> PayResult<()> |  |  |  |  |

| `CASTLE-PEE-AUTHORITY-V1` | str_key | AUTHORITY_DOMAIN = "CASTLE-PEE-AUTHORITY-V1" |  |  |  |  |

| `CASTLE-PEE-V1` | str_key | PEE_DOMAIN = "CASTLE-PEE-V1" |  |  |  |  |

| `REFUSED:EFFECT_BINDING_INCOMPLETE` | str_key | PEE_BINDING_INCOMPLETE = "REFUSED:EFFECT_BINDING_INCOMPLETE" |  |  |  |  |

| `REFUSED:EFFECT_EXPIRED` | str_key | PEE_EXPIRED = "REFUSED:EFFECT_EXPIRED" |  |  |  |  |

| `REFUSED:EFFECT_IDENTITY_MISMATCH` | str_key | PEE_IDENTITY_MISMATCH = "REFUSED:EFFECT_IDENTITY_MISMATCH" |  |  |  |  |

| `REFUSED:EFFECT_NOT_YET_VALID` | str_key | PEE_NOT_YET_VALID = "REFUSED:EFFECT_NOT_YET_VALID" |  |  |  |  |

| `effect_id` | str_key | EXCLUDED = "effect_id" |  |  |  |  |

| `EffectBindings` | struct | EffectBindings { pub policy_profile_id: String, pub law_state_digest: String, pub counterparty_evidence_digest: String, pub funding_source_id: String, pub resource_reservation_id: String, pub rail_profile_id: String, pub message_profile_version: String, pub parent_receipt: String, pub created_at_ms: u64, pub expires_at_ms: u64 } |  |  |  |  |

| `PreparedEconomicEffect` | struct | PreparedEconomicEffect { pub version: u32, pub effect_id: String, pub obligation_id: String, pub principal_id: String, pub payer_account: String, pub beneficiary_account: String, pub amount_minor: String, pub currency: Currency, pub purpose: String, pub authority_grant_id: String, pub authority_digest: String, pub policy_profile_id: String, pub law_state_digest: String, pub counterparty_evidence_digest: String, pub funding_source_id: String, pub resource_reservation_id: String, pub rail_profile_id: String, pub message_profile_version: String, pub idempotency_key: String, pub created_at_ms: u64, pub expires_at_ms: u64, pub nonce: String, pub parent_receipt: String, pub canonical_payload_digest: String } |  |  |  |  |

| `INSUFFICIENT_QUORUM` | const | INSUFFICIENT_QUORUM: &str |  |  |  |  |

| `check_quorum` | function | check_quorum(&self, certificate_threshold: u16) -> PayResult<()> |  |  |  |  |

| `check_static` | function | check_static(&self, effect: &PaymentEffect) -> PayResult<&PrincipalPolicy> |  |  |  |  |

| `default_min_quorum` | function | default_min_quorum() -> u16 |  |  |  |  |

| `to_json` | function | to_json(&self) -> serde_json::Value |  |  |  |  |

| `REFUSED:InsufficientQuorum` | str_key | INSUFFICIENT_QUORUM = "REFUSED:InsufficientQuorum" |  |  |  |  |

| `PrincipalPolicy` | struct | PrincipalPolicy { pub allowed_payers: BTreeSet<String>, pub allowed_payees: BTreeSet<String>, pub per_effect_cap: BTreeMap<Currency, u64>, pub epoch_cap: BTreeMap<Currency, u64>, pub min_quorum: u16 } |  |  |  |  |

| `SpendPolicy` | struct | SpendPolicy { pub principals: BTreeMap<String, PrincipalPolicy>, pub require_derived_obligation: bool, pub require_screening: bool } |  |  |  |  |

| `PAYLOAD_DOMAIN` | const | PAYLOAD_DOMAIN: &[u8] |  |  |  |  |

| `RailAck` | enum | RailAck { Accepted { correlation_id: String }, Rejected { correlation_id: String, reason_code: String } } |  |  |  |  |

| `RailError` | enum | RailError { Timeout, Unavailable(String) } |  |  |  |  |

| `RailStatus` | enum | RailStatus { Unknown, Accepted, Settled { final_ref: String }, Rejected { reason: String }, Returned { reason: String, return_ref: String } } |  |  |  |  |

| `correlation_id_for` | function | correlation_id_for(effect_id: &str) -> String |  |  |  |  |

| `payload_digest` | function | payload_digest(&self) -> String |  |  |  |  |

| `CASTLE-RAIL-PAYLOAD-V1` | str_key | PAYLOAD_DOMAIN = "CASTLE-RAIL-PAYLOAD-V1" |  |  |  |  |

| `RailInstruction` | struct | RailInstruction { pub effect_id: String, pub correlation_id: String, pub message_profile: String, pub payload: String, pub amount_minor: u64, pub currency: Currency, pub payer: String, pub payee: String } |  |  |  |  |

| `RailActuator` | trait |  |  |  |  |  |

| `SIM_REJECT_AFTER_DELAY` | const | SIM_REJECT_AFTER_DELAY: &str |  |  |  |  |

| `SIM_REJECT_DUPLICATE_DIFFERENT_PAYLOAD` | const | SIM_REJECT_DUPLICATE_DIFFERENT_PAYLOAD: &str |  |  |  |  |

| `SIM_RETURN_REASON` | const | SIM_RETURN_REASON: &str |  |  |  |  |

| `SimMode` | enum | SimMode { Honest, DropAckAfterAccept, RejectAfterPolls(u32), SettleAfterPolls(u32), ReturnAfterSettle, Down, DuplicateReports } |  |  |  |  |

| `accepted_correlations` | function | accepted_correlations(&self) -> Vec<String> |  |  |  |  |

| `open` | function | open(root: &Path, mode: SimMode) -> Result<Self, String> |  |  |  |  |

| `set_mode` | function | set_mode(&self, mode: SimMode) -> Result<(), String> |  |  |  |  |

| `settlement_count` | function | settlement_count(&self, correlation: &str) -> usize |  |  |  |  |

| `status_history` | function | status_history(&self, correlation: &str) -> Vec<RailStatus> |  |  |  |  |

| `submissions_seen` | function | submissions_seen(&self, correlation: &str) -> usize |  |  |  |  |

| `CASTLE-SIMRAIL-V1` | str_key | SIM_DOMAIN = "CASTLE-SIMRAIL-V1" |  |  |  |  |

| `DUPLICATE_CORRELATION_DIFFERENT_PAYLOAD` | str_key | SIM_REJECT_DUPLICATE_DIFFERENT_PAYLOAD = "DUPLICATE_CORRELATION_DIFFERENT_PAYLOAD" |  |  |  |  |

| `REJECTED_BY_RAIL_AFTER_DELAY` | str_key | SIM_REJECT_AFTER_DELAY = "REJECTED_BY_RAIL_AFTER_DELAY" |  |  |  |  |

| `RETURNED_BY_BENEFICIARY_BANK` | str_key | SIM_RETURN_REASON = "RETURNED_BY_BENEFICIARY_BANK" |  |  |  |  |

| `SimRail` | struct | SimRail { root: PathBuf } |  |  |  |  |

| `NOT_RECONCILABLE` | const | NOT_RECONCILABLE: &str |  |  |  |  |

| `ReconcileResolution` | enum | ReconcileResolution { Settled(LedgerEntry), ProvenAbsent, StillUnknown } |  |  |  |  |

| `reconcile` | function | reconcile(effect_digest: &str, claims: &ClaimStore, ledger: &dyn LedgerPort) -> PayResult<ReconcileResolution> |  |  |  |  |

| `recover_journal` | function | recover_journal(journal_root: &Path) -> PayResult<JournalRecovery> |  |  |  |  |

| `REFUSED:PAYMENT_NOT_RECONCILABLE` | str_key | NOT_RECONCILABLE = "REFUSED:PAYMENT_NOT_RECONCILABLE" |  |  |  |  |

| `JournalRecovery` | struct | JournalRecovery { pub prepared_without_outcome: Vec<String>, pub complete: Vec<String> } |  |  |  |  |

| `ACCOUNT_NOT_ALLOWED` | const | ACCOUNT_NOT_ALLOWED: &str |  |  |  |  |

| `ALREADY_SETTLED` | const | ALREADY_SETTLED: &str |  |  |  |  |

| `AMOUNT_EXCEEDS_CAP` | const | AMOUNT_EXCEEDS_CAP: &str |  |  |  |  |

| `AMOUNT_NOT_INTEGER` | const | AMOUNT_NOT_INTEGER: &str |  |  |  |  |

| `AMOUNT_ZERO` | const | AMOUNT_ZERO: &str |  |  |  |  |

| `BUDGET_EXCEEDED` | const | BUDGET_EXCEEDED: &str |  |  |  |  |

| `CAPABILITY_NOT_ADMITTED` | const | CAPABILITY_NOT_ADMITTED: &str |  |  |  |  |

| `CLAIM_NOT_RESERVED` | const | CLAIM_NOT_RESERVED: &str |  |  |  |  |

| `CLAIM_STORE_FAILED` | const | CLAIM_STORE_FAILED: &str |  |  |  |  |

| `CURRENCY_UNSUPPORTED` | const | CURRENCY_UNSUPPORTED: &str |  |  |  |  |

| `INSUFFICIENT_FUNDS` | const | INSUFFICIENT_FUNDS: &str |  |  |  |  |

| `IN_FLIGHT` | const | IN_FLIGHT: &str |  |  |  |  |

| `LEDGER_UNAVAILABLE` | const | LEDGER_UNAVAILABLE: &str |  |  |  |  |

| `NO_SPEND_POLICY` | const | NO_SPEND_POLICY: &str |  |  |  |  |

| `OBLIGATION_ALREADY_CLAIMED` | const | OBLIGATION_ALREADY_CLAIMED: &str |  |  |  |  |

| `OUTCOME_UNKNOWN` | const | OUTCOME_UNKNOWN: &str |  |  |  |  |

| `PAYLOAD_INVALID` | const | PAYLOAD_INVALID: &str |  |  |  |  |

| `RECONCILIATION_DRIFT` | const | RECONCILIATION_DRIFT: &str |  |  |  |  |

| `REVERSAL_EXCEEDS_ORIGINAL` | const | REVERSAL_EXCEEDS_ORIGINAL: &str |  |  |  |  |

| `REVERSAL_MISMATCH` | const | REVERSAL_MISMATCH: &str |  |  |  |  |

| `REVERSAL_ORIGINAL_NOT_EXECUTED` | const | REVERSAL_ORIGINAL_NOT_EXECUTED: &str |  |  |  |  |

| `SAME_ACCOUNT` | const | SAME_ACCOUNT: &str |  |  |  |  |

| `refuse` | function | refuse(code: &str) -> PayResult<T> |  |  |  |  |

| `BLOCKED:PAYMENT_CLAIM_STORE_FAILED` | str_key | CLAIM_STORE_FAILED = "BLOCKED:PAYMENT_CLAIM_STORE_FAILED" |  |  |  |  |

| `REFUSED:NO_SPEND_POLICY_FOR_PRINCIPAL` | str_key | NO_SPEND_POLICY = "REFUSED:NO_SPEND_POLICY_FOR_PRINCIPAL" |  |  |  |  |

| `REFUSED:PAYMENT_ACCOUNT_NOT_ALLOWED` | str_key | ACCOUNT_NOT_ALLOWED = "REFUSED:PAYMENT_ACCOUNT_NOT_ALLOWED" |  |  |  |  |

| `REFUSED:PAYMENT_ALREADY_SETTLED` | str_key | ALREADY_SETTLED = "REFUSED:PAYMENT_ALREADY_SETTLED" |  |  |  |  |

| `REFUSED:PAYMENT_AMOUNT_EXCEEDS_CAP` | str_key | AMOUNT_EXCEEDS_CAP = "REFUSED:PAYMENT_AMOUNT_EXCEEDS_CAP" |  |  |  |  |

| `REFUSED:PAYMENT_AMOUNT_NOT_INTEGER` | str_key | AMOUNT_NOT_INTEGER = "REFUSED:PAYMENT_AMOUNT_NOT_INTEGER" |  |  |  |  |

| `REFUSED:PAYMENT_AMOUNT_ZERO` | str_key | AMOUNT_ZERO = "REFUSED:PAYMENT_AMOUNT_ZERO" |  |  |  |  |

| `REFUSED:PAYMENT_BUDGET_EXCEEDED` | str_key | BUDGET_EXCEEDED = "REFUSED:PAYMENT_BUDGET_EXCEEDED" |  |  |  |  |

| `REFUSED:PAYMENT_CAPABILITY_NOT_ADMITTED` | str_key | CAPABILITY_NOT_ADMITTED = "REFUSED:PAYMENT_CAPABILITY_NOT_ADMITTED" |  |  |  |  |

| `REFUSED:PAYMENT_CLAIM_NOT_RESERVED` | str_key | CLAIM_NOT_RESERVED = "REFUSED:PAYMENT_CLAIM_NOT_RESERVED" |  |  |  |  |

| `REFUSED:PAYMENT_CURRENCY_UNSUPPORTED` | str_key | CURRENCY_UNSUPPORTED = "REFUSED:PAYMENT_CURRENCY_UNSUPPORTED" |  |  |  |  |

| `REFUSED:PAYMENT_INSUFFICIENT_FUNDS` | str_key | INSUFFICIENT_FUNDS = "REFUSED:PAYMENT_INSUFFICIENT_FUNDS" |  |  |  |  |

| `REFUSED:PAYMENT_IN_FLIGHT` | str_key | IN_FLIGHT = "REFUSED:PAYMENT_IN_FLIGHT" |  |  |  |  |

| `REFUSED:PAYMENT_LEDGER_UNAVAILABLE` | str_key | LEDGER_UNAVAILABLE = "REFUSED:PAYMENT_LEDGER_UNAVAILABLE" |  |  |  |  |

| `REFUSED:PAYMENT_OBLIGATION_ALREADY_CLAIMED` | str_key | OBLIGATION_ALREADY_CLAIMED = "REFUSED:PAYMENT_OBLIGATION_ALREADY_CLAIMED" |  |  |  |  |

| `REFUSED:PAYMENT_OUTCOME_UNKNOWN` | str_key | OUTCOME_UNKNOWN = "REFUSED:PAYMENT_OUTCOME_UNKNOWN" |  |  |  |  |

| `REFUSED:PAYMENT_PAYLOAD_INVALID` | str_key | PAYLOAD_INVALID = "REFUSED:PAYMENT_PAYLOAD_INVALID" |  |  |  |  |

| `REFUSED:PAYMENT_REVERSAL_EXCEEDS_ORIGINAL` | str_key | REVERSAL_EXCEEDS_ORIGINAL = "REFUSED:PAYMENT_REVERSAL_EXCEEDS_ORIGINAL" |  |  |  |  |

| `REFUSED:PAYMENT_REVERSAL_MISMATCH` | str_key | REVERSAL_MISMATCH = "REFUSED:PAYMENT_REVERSAL_MISMATCH" |  |  |  |  |

| `REFUSED:PAYMENT_REVERSAL_ORIGINAL_NOT_EXECUTED` | str_key | REVERSAL_ORIGINAL_NOT_EXECUTED = "REFUSED:PAYMENT_REVERSAL_ORIGINAL_NOT_EXECUTED" |  |  |  |  |

| `REFUSED:PAYMENT_SAME_ACCOUNT` | str_key | SAME_ACCOUNT = "REFUSED:PAYMENT_SAME_ACCOUNT" |  |  |  |  |

| `REFUSED:RECONCILIATION_DRIFT` | str_key | RECONCILIATION_DRIFT = "REFUSED:RECONCILIATION_DRIFT" |  |  |  |  |

| `REPLAY_DIGEST_MISMATCH` | const | REPLAY_DIGEST_MISMATCH: &str |  |  |  |  |

| `REPLAY_RECORD_CORRUPT` | const | REPLAY_RECORD_CORRUPT: &str |  |  |  |  |

| `REPLAY_RECORD_MISSING` | const | REPLAY_RECORD_MISSING: &str |  |  |  |  |

| `REPLAY_REGISTRY_UNTRUSTED` | const | REPLAY_REGISTRY_UNTRUSTED: &str |  |  |  |  |

| `ReplayVerdict` | enum | ReplayVerdict { Reproduced { effect_digest: String, decision_digest: String }, Diverged { reason: String } } |  |  |  |  |

| `admit_payment_journaled` | function | admit_payment_journaled( prepared: PreparedEffect, certificate: &ActuationCertificate, ctx: &AdmissionContext<'_>, journal: &AdmissionJournal, ) -> PayResult<PaymentAdmission> |  |  |  |  |

| `admit_payment_screened_journaled` | function | admit_payment_screened_journaled( prepared: PreparedEffect, certificate: &ActuationCertificate, ctx: &AdmissionContext<'_>, screening: &Screening<'_>, journal: &AdmissionJournal, ) -> PayResult<(PaymentAdmission, ScreeningEvidence)> |  |  |  |  |

| `load` | function | load(&self, effect_digest: &str) -> PayResult<AdmissionRecord> |  |  |  |  |

| `open` | function | open(root: impl Into<PathBuf>) -> PayResult<Self> |  |  |  |  |

| `replay_admission` | function | replay_admission(journal: &AdmissionJournal, effect_digest: &str) -> PayResult<ReplayVerdict> |  |  |  |  |

| `replay_admission_anchored` | function | replay_admission_anchored( journal: &AdmissionJournal, effect_digest: &str, trusted_registry: &KeyRegistry, ) -> PayResult<ReplayVerdict> |  |  |  |  |

| `CASTLE-REPLAY-DECISION-V1` | str_key | DECISION_DOMAIN = "CASTLE-REPLAY-DECISION-V1" |  |  |  |  |

| `REFUSED:REPLAY_EFFECT_DIGEST_MISMATCH` | str_key | REPLAY_DIGEST_MISMATCH = "REFUSED:REPLAY_EFFECT_DIGEST_MISMATCH" |  |  |  |  |

| `REFUSED:REPLAY_RECORD_CORRUPT` | str_key | REPLAY_RECORD_CORRUPT = "REFUSED:REPLAY_RECORD_CORRUPT" |  |  |  |  |

| `REFUSED:REPLAY_RECORD_MISSING` | str_key | REPLAY_RECORD_MISSING = "REFUSED:REPLAY_RECORD_MISSING" |  |  |  |  |

| `REFUSED:REPLAY_REGISTRY_UNTRUSTED` | str_key | REPLAY_REGISTRY_UNTRUSTED = "REFUSED:REPLAY_REGISTRY_UNTRUSTED" |  |  |  |  |

| `AdmissionJournal` | struct | AdmissionJournal { root: PathBuf } |  |  |  |  |

| `AdmissionRecord` | struct | AdmissionRecord { pub prepared: PreparedEffect, pub certificate: ActuationCertificate, pub registry: Vec<KeyRecord>, pub policy_epoch: u64, pub revocation_epoch: u64, pub generation: u64, pub audience: String, pub now_ms: u64, pub policy: SpendPolicy } |  |  |  |  |

| `EVIDENCE_CONFLICT` | const | EVIDENCE_CONFLICT: &str |  |  |  |  |

| `PAYMENT_NOT_FINAL` | const | PAYMENT_NOT_FINAL: &str |  |  |  |  |

| `PAYMENT_NOT_HELD` | const | PAYMENT_NOT_HELD: &str |  |  |  |  |

| `PAYMENT_NOT_SUBMITTED` | const | PAYMENT_NOT_SUBMITTED: &str |  |  |  |  |

| `REPORT_CORRELATION_MISMATCH` | const | REPORT_CORRELATION_MISMATCH: &str |  |  |  |  |

| `REPORT_NOT_TERMINAL` | const | REPORT_NOT_TERMINAL: &str |  |  |  |  |

| `FinalityKind` | enum | FinalityKind { Final, Rejected, Returned } |  |  |  |  |

| `FinalityOutcome` | enum | FinalityOutcome { Settled(LedgerEntry), AlreadyFinal, Released, Returned(LedgerEntry) } |  |  |  |  |

| `apply_finality` | function | apply_finality(claims: &ClaimStore, ledger: &dyn LedgerPort, ev: &FinalityEvidence) -> PayResult<FinalityOutcome> |  |  |  |  |

| `apply_rail_report` | function | apply_rail_report(claims: &ClaimStore, ledger: &dyn LedgerPort, report: &RailReport) -> PayResult<FinalityOutcome> |  |  |  |  |

| `correlation_id` | function | correlation_id(&self) -> &str |  |  |  |  |

| `effect_digest` | function | effect_digest(&self) -> &str |  |  |  |  |

| `evidence_digest` | function | evidence_digest(&self) -> &str |  |  |  |  |

| `kind` | function | kind(&self) -> FinalityKind |  |  |  |  |

| `observe_rail` | function | observe_rail(rail: &dyn RailActuator, effect_digest: &str) -> Result<RailReport, RailError> |  |  |  |  |

| `payee` | function | payee(&self) -> &str |  |  |  |  |

| `reason` | function | reason(&self) -> &str |  |  |  |  |

| `status` | function | status(&self) -> &RailStatus |  |  |  |  |

| `CASTLE-RAIL-STATUS-EVIDENCE-V1` | str_key | STATUS_EVIDENCE_DOMAIN = "CASTLE-RAIL-STATUS-EVIDENCE-V1" |  |  |  |  |

| `REFUSED:PAYMENT_EVIDENCE_CONFLICT` | str_key | EVIDENCE_CONFLICT = "REFUSED:PAYMENT_EVIDENCE_CONFLICT" |  |  |  |  |

| `REFUSED:PAYMENT_NOT_FINAL` | str_key | PAYMENT_NOT_FINAL = "REFUSED:PAYMENT_NOT_FINAL" |  |  |  |  |

| `REFUSED:PAYMENT_NOT_HELD` | str_key | PAYMENT_NOT_HELD = "REFUSED:PAYMENT_NOT_HELD" |  |  |  |  |

| `REFUSED:PAYMENT_NOT_SUBMITTED` | str_key | PAYMENT_NOT_SUBMITTED = "REFUSED:PAYMENT_NOT_SUBMITTED" |  |  |  |  |

| `REFUSED:PAYMENT_REPORT_CORRELATION_MISMATCH` | str_key | REPORT_CORRELATION_MISMATCH = "REFUSED:PAYMENT_REPORT_CORRELATION_MISMATCH" |  |  |  |  |

| `REFUSED:PAYMENT_REPORT_NOT_TERMINAL` | str_key | REPORT_NOT_TERMINAL = "REFUSED:PAYMENT_REPORT_NOT_TERMINAL" |  |  |  |  |

| `FinalityEvidence` | struct | FinalityEvidence { effect_digest: String, correlation_id: String, evidence_digest: String, kind: FinalityKind, reason: String } |  |  |  |  |

| `RailReport` | struct | RailReport { effect_digest: String, correlation_id: String, status: RailStatus, evidence_digest: String, _seal: () } |  |  |  |  |

| `MinimalActionPlanner` | struct | MinimalActionPlanner { pub id: String } |  |  |  |  |

| `FinalDisposition` | enum | FinalDisposition { Preserved, Subsumed, Replaced, Archived, Refused } |  |  |  |  |

| `admission_digest` | function | admission_digest(&self) -> &str |  |  |  |  |

| `admit_empire_reconstitution_for_construct` | function | admit_empire_reconstitution_for_construct( document: &str, ) -> Result<EmpireReconstitutionAdmission, ReconstitutionRefusal> |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `authority_id` | function | authority_id(&self) -> &str |  |  |  |  |

| `capabilities` | function | capabilities(&self) -> &[ReconstitutedCapability] |  |  |  |  |

| `disposition` | function | disposition(&self) -> FinalDisposition |  |  |  |  |

| `evidence_ids` | function | evidence_ids(&self) -> &[String] |  |  |  |  |

| `id` | function | id(&self) -> &str |  |  |  |  |

| `may_actuate` | function | may_actuate(&self) -> bool |  |  |  |  |

| `observable_surfaces` | function | observable_surfaces(&self) -> &[String] |  |  |  |  |

| `observation_receipt_digest` | function | observation_receipt_digest(&self) -> &str |  |  |  |  |

| `study_id` | function | study_id(&self) -> &str |  |  |  |  |

| `to_o_star_value` | function | to_o_star_value(&self) -> Value |  |  |  |  |

| `OSTAR-EMPIRE-001` | str_key | STUDY_ID = "OSTAR-EMPIRE-001" |  |  |  |  |

| `diagnostics` | str_key | OBSERVABLE_SURFACES = "diagnostics" |  |  |  |  |

| `ggen.legacy.authority-vacuum.admission.v1` | str_key | ADMISSION_SCHEMA = "ggen.legacy.authority-vacuum.admission.v1" |  |  |  |  |

| `ggen.legacy.authority-vacuum.receipt.v1` | str_key | RECEIPT_SCHEMA = "ggen.legacy.authority-vacuum.receipt.v1" |  |  |  |  |

| `ontostar-admission-manufacture` | str_key | REQUIRED_CAPABILITIES = "ontostar-admission-manufacture" |  |  |  |  |

| `AdmissionBrand` | struct |  |  |  |  |  |

| `EmpireReconstitutionAdmission` | struct | EmpireReconstitutionAdmission { study_id: String, admission_digest: String, observation_receipt_digest: String, authority_id: String, authority_digest: String, capabilities: Vec<ReconstitutedCapability>, _brand: sealed::AdmissionBrand } |  |  |  |  |

| `ReconstitutedCapability` | struct | ReconstitutedCapability { id: String, disposition: FinalDisposition, evidence_ids: Vec<String>, observable_surfaces: Vec<String> } |  |  |  |  |

| `ReconstitutionRefusal` | struct | ReconstitutionRefusal { pub code: &'static str, pub detail: String } |  |  |  |  |

| `project_beam4pm_feedback` | function | project_beam4pm_feedback( log: &ReceiptedOcelLog, subject: &str, replay_identity: &str, ) -> Result<Beam4PmFeedback, String> |  |  |  |  |

| `Beam4PmFeedback` | struct | Beam4PmFeedback { pub subject: String, pub replay_identity: String, pub construct_digest: String, pub outcome_receipt_digest: String, pub ocel_event_count: usize } |  |  |  |  |

| `FIBO_PAYMENT_CAPABILITY` | const | FIBO_PAYMENT_CAPABILITY: &str |  |  |  |  |

| `GRAPHLAW_HOOK_CONTRACT` | const | GRAPHLAW_HOOK_CONTRACT: &str |  |  |  |  |

| `bind_verification_receipt` | function | bind_verification_receipt( projection: &FiboPaymentProjection, receipt: &VerificationReceipt, ) -> Result<AuthorityEvidence, String> |  |  |  |  |

| `project_fibo_payment` | function | project_fibo_payment(candidate: KnowledgeHookCandidate) -> Result<FiboPaymentProjection, String> |  |  |  |  |

| `fibo:PaymentExecution` | str_key | FIBO_PAYMENT_CAPABILITY = "fibo:PaymentExecution" |  |  |  |  |

| `https://graphlaw.dev/knowledge-hook#HookCandidate` | str_key | GRAPHLAW_HOOK_CONTRACT = "https://graphlaw.dev/knowledge-hook#HookCandidate" |  |  |  |  |

| `AuthorityEvidence` | struct | AuthorityEvidence { pub effect_digest: String, pub principal: String, pub policy_epoch: u64, pub revocation_epoch: u64, pub generation: u64, pub audience: String, pub signer_count: usize, pub custodian_count: usize } |  |  |  |  |

| `FiboPaymentProjection` | struct | FiboPaymentProjection { pub hook_contract: &'static str, pub source_digest: String, pub replay_identity: String, pub prepared_effect: PreparedEffect, pub effect_digest: String } |  |  |  |  |

| `KnowledgeHookCandidate` | struct | KnowledgeHookCandidate { pub version: u32, pub exact_subject: Value, pub source_digest: String, pub replay_identity: String, pub authority: String, pub capability: String, pub principal: String, pub payload: Value } |  |  |  |  |

| `feedback::{project_beam4pm_feedback, Beam4PmFeedback}` | use | feedback::{project_beam4pm_feedback, Beam4PmFeedback} |  |  |  |  |

| `fibo::{ bind_verification_receipt, project_fibo_payment, AuthorityEvidence, FiboPaymentProjection, KnowledgeHookCandidate, FIBO_PAYMENT_CAPABILITY, GRAPHLAW_HOOK_CONTRACT, }` | use | fibo::{ bind_verification_receipt, project_fibo_payment, AuthorityEvidence, FiboPaymentProjection, KnowledgeHookCandidate, FIBO_PAYMENT_CAPABILITY, GRAPHLAW_HOOK_CONTRACT, } |  |  |  |  |

| `ownership::{ canonical_owners, ReferenceOwner, ReferenceStage, ECOSYSTEM_REFERENCE_HEAD, GRAPHLAW_REFERENCE_HEAD, INHERITED_COURTS, REFERENCE_STAGES, }` | use | ownership::{ canonical_owners, ReferenceOwner, ReferenceStage, ECOSYSTEM_REFERENCE_HEAD, GRAPHLAW_REFERENCE_HEAD, INHERITED_COURTS, REFERENCE_STAGES, } |  |  |  |  |

| `ECOSYSTEM_REFERENCE_HEAD` | const | ECOSYSTEM_REFERENCE_HEAD: &str |  |  |  |  |

| `GRAPHLAW_REFERENCE_HEAD` | const | GRAPHLAW_REFERENCE_HEAD: &str |  |  |  |  |

| `INHERITED_COURTS` | const | INHERITED_COURTS: &[&str] |  |  |  |  |

| `REFERENCE_STAGES` | const | REFERENCE_STAGES: &[ReferenceStage] |  |  |  |  |

| `ReferenceStage` | enum | ReferenceStage { SemanticState, GraphLaw, KnowledgeHook, Sa2aIntent, IndependentAuthorityEvidence, ReactorCommandBus, CastleBrce, ExternalOrSyntheticDo, IndependentPostcondition, ReceiptOcel, Beam4PmFeedback, XaasRuntimeComposition } |  |  |  |  |

| `canonical_owners` | function | canonical_owners() -> &'static [ReferenceOwner] |  |  |  |  |

| `48a7bbd801b8df1d7ffab879b10d58d7f14ef7bc` | str_key | GRAPHLAW_REFERENCE_HEAD = "48a7bbd801b8df1d7ffab879b10d58d7f14ef7bc" |  |  |  |  |

| `bebffdeb2ea4dd6eace31d69298bfb51a073b265` | str_key | ECOSYSTEM_REFERENCE_HEAD = "bebffdeb2ea4dd6eace31d69298bfb51a073b265" |  |  |  |  |

| `graphlaw:knowledge_hook:authority-none` | str_key | INHERITED_COURTS = "graphlaw:knowledge_hook:authority-none" |  |  |  |  |

| `ReferenceOwner` | struct | ReferenceOwner { pub layer: &'static str, pub owner: &'static str } |  |  |  |  |

| `verify_refusal` | function | verify_refusal(observation: RefusalObservation<'_>) -> Result<(), &'static str> |  |  |  |  |

| `RefusalObservation` | struct | RefusalObservation { pub expected: &'a str, pub actual: Result<(), &'a str>, pub observed_world_change: bool } |  |  |  |  |

| `algorithm::*` | use | algorithm::* |  |  |  |  |

| `certificate::*` | use | certificate::* |  |  |  |  |

| `key_registry::*` | use | key_registry::* |  |  |  |  |

| `policy::*` | use | policy::* |  |  |  |  |

| `refusal::*` | use | refusal::* |  |  |  |  |

| `threshold::*` | use | threshold::* |  |  |  |  |

| `verifier::*` | use | verifier::* |  |  |  |  |

| `Refusal` | enum | Refusal { EffectDigestMismatch, PrincipalMismatch, PolicyEpochMismatch, RevocationEpochStale, ExecutionGenerationStale, EmptyQuorum, DuplicateSigner, DuplicateIndependenceDomain, UnknownKey, KeyPrincipalMismatch, AlgorithmMismatch, AlgorithmNotAllowed, KeyNotYetValid, KeyRevoked, SignatureInvalid, QuorumNotMet } |  |  |  |  |

| `SignatureAlgorithm` | enum | SignatureAlgorithm { Ed25519, MlDsa65, SlhDsaShake128f } |  |  |  |  |

| `certificate_message` | function | certificate_message(c:&ActuationCertificate) -> Vec<u8> |  |  |  |  |

| `new` | function | new(provider:P,keys:impl IntoIterator<Item=KeyRecord>) -> Self |  |  |  |  |

| `verify` | function | verify(&self,effect:[u8;32],principal:&str,c:&ActuationCertificate,p:&VerificationPolicy) -> Result<(),Refusal> |  |  |  |  |

| `ActuationCertificate` | struct | ActuationCertificate { pub effect_digest:[u8;32], pub principal:String, pub policy_epoch:u64, pub revocation_epoch:u64, pub execution_generation:u64, pub signatures:Vec<CertificateSignature> } |  |  |  |  |

| `AuthorityVerifier` | struct | AuthorityVerifier { provider:P, keys:BTreeMap<String,KeyRecord> } |  |  |  |  |

| `CertificateSignature` | struct | CertificateSignature { pub key_id:String, pub algorithm:SignatureAlgorithm, pub signature:Vec<u8> } |  |  |  |  |

| `KeyRecord` | struct | KeyRecord { pub key_id:String, pub principal:String, pub algorithm:SignatureAlgorithm, pub public_key:Vec<u8>, pub valid_from_epoch:u64, pub revoked_at_epoch:Option<u64>, pub independence_domain:String } |  |  |  |  |

| `VerificationPolicy` | struct | VerificationPolicy { pub current_policy_epoch:u64, pub current_revocation_epoch:u64, pub minimum_execution_generation:u64, pub quorum:usize, pub allowed_algorithms:BTreeSet<SignatureAlgorithm> } |  |  |  |  |

| `CryptoProvider` | trait |  |  |  |  |  |

| `SignatureAlgorithm` | enum | SignatureAlgorithm { Ed25519, MlDsa65, SlhDsaShake128f } |  |  |  |  |

| `id` | function | id(self) -> &'static str |  |  |  |  |

| `signing_message` | function | signing_message(&self) -> Result<Vec<u8>, SecurityRefusal> |  |  |  |  |

| `ActuationCertificate` | struct | ActuationCertificate { pub version: u32, pub effect_digest: String, pub principal: String, pub policy_epoch: u64, pub revocation_epoch: u64, pub generation: u64, pub nonce: String, pub not_before_ms: u64, pub expires_at_ms: u64, pub audience: String, pub threshold: u16, pub signatures: Vec<CertificateSignature> } |  |  |  |  |

| `verify_signature` | function | verify_signature( algorithm: SignatureAlgorithm, public_key: &[u8], message: &[u8], signature: &[u8], ) -> Result<(), SecurityRefusal> |  |  |  |  |

| `CERTIFICATE_DOMAIN` | const | CERTIFICATE_DOMAIN: &[u8] |  |  |  |  |

| `PREPARED_EFFECT_DOMAIN` | const | PREPARED_EFFECT_DOMAIN: &[u8] |  |  |  |  |

| `push_field` | function | push_field(out: &mut Vec<u8>, value: &[u8]) |  |  |  |  |

| `sha256_tagged` | function | sha256_tagged(domain: &[u8], body: &[u8]) -> String |  |  |  |  |

| `valid_sha256_tag` | function | valid_sha256_tag(value: &str) -> bool |  |  |  |  |

| `SA2A-C2-ACTUATION-CERTIFICATE-V1` | str_key | CERTIFICATE_DOMAIN = "SA2A-C2-ACTUATION-CERTIFICATE-V1" |  |  |  |  |

| `SA2A-PREPARED-EFFECT-V1` | str_key | PREPARED_EFFECT_DOMAIN = "SA2A-PREPARED-EFFECT-V1" |  |  |  |  |

| `admit_epochs` | function | admit_epochs(expected: SecurityEpochs, certificate: SecurityEpochs) -> Result<(), SecurityRefusal> |  |  |  |  |

| `SecurityEpochs` | struct | SecurityEpochs { pub policy: u64, pub revocation: u64, pub generation: u64 } |  |  |  |  |

| `SecurityRefusal` | enum | SecurityRefusal { InvalidDigest, InvalidKey, InvalidSignature, AlgorithmMismatch, UnknownKey, KeyRevoked, KeyNotYetValid, KeyExpired, PolicyEpochMismatch, RevocationEpochMismatch, GenerationMismatch, PrincipalMismatch, AudienceMismatch, CertificateExpired, CertificateNotYetValid, EffectDigestMismatch, DuplicateSigner, InsufficientQuorum, InsufficientCustodianIndependence, NonceReplay, ResourceAmplification, InvalidResourceEnvelope } |  |  |  |  |

| `KeyState` | enum | KeyState { Active, Revoked } |  |  |  |  |

| `from_records` | function | from_records(records: impl IntoIterator<Item = KeyRecord>) -> Self |  |  |  |  |

| `records` | function | records(&self) -> Vec<KeyRecord> |  |  |  |  |

| `resolve` | function | resolve(&self, key_id: &str, now_ms: u64, revocation_epoch: u64) -> Result<&KeyRecord, SecurityRefusal> |  |  |  |  |

| `KeyRecord` | struct | KeyRecord { pub key_id: String, pub custodian_id: String, pub algorithm: SignatureAlgorithm, pub public_key: Vec<u8>, pub state: KeyState, pub not_before_ms: u64, pub expires_at_ms: u64, pub revocation_epoch: u64 } |  |  |  |  |

| `KeyRegistry` | struct | KeyRegistry { keys: BTreeMap<String, KeyRecord> } |  |  |  |  |

| `algorithm::SignatureAlgorithm` | use | algorithm::SignatureAlgorithm |  |  |  |  |

| `certificate::ActuationCertificate` | use | certificate::ActuationCertificate |  |  |  |  |

| `error::SecurityRefusal` | use | error::SecurityRefusal |  |  |  |  |

| `key_registry::{KeyRecord, KeyRegistry, KeyState}` | use | key_registry::{KeyRecord, KeyRegistry, KeyState} |  |  |  |  |

| `prepared_effect::PreparedEffect` | use | prepared_effect::PreparedEffect |  |  |  |  |

| `resource::{BudgetLedger, ResourceEnvelope}` | use | resource::{BudgetLedger, ResourceEnvelope} |  |  |  |  |

| `signature::CertificateSignature` | use | signature::CertificateSignature |  |  |  |  |

| `verifier::{CertificateVerifier, VerificationReceipt}` | use | verifier::{CertificateVerifier, VerificationReceipt} |  |  |  |  |

| `claim` | function | claim(&mut self, key_id: &str, nonce: &str) -> Result<(), SecurityRefusal> |  |  |  |  |

| `NonceFence` | struct | NonceFence { seen: BTreeSet<(String, String)> } |  |  |  |  |

| `digest` | function | digest(&self) -> Result<String, SecurityRefusal> |  |  |  |  |

| `PreparedEffect` | struct | PreparedEffect { pub version: u32, pub principal: String, pub capability: String, pub subject: Value, pub payload: Value } |  |  |  |  |

| `preserve_principal` | function | preserve_principal(expected: &str, observed: &str) -> Result<(), SecurityRefusal> |  |  |  |  |

| `admit_distinct_quorum` | function | admit_distinct_quorum( threshold: u16, verified: &[(CertificateSignature, KeyRecord)], ) -> Result<Vec<VerifiedSigner>, SecurityRefusal> |  |  |  |  |

| `VerifiedSigner` | struct | VerifiedSigner { pub key_id: String, pub custodian_id: String } |  |  |  |  |

| `from_signers` | function | from_signers( effect_digest: String, principal: String, policy_epoch: u64, revocation_epoch: u64, generation: u64, audience: String, signers: &[VerifiedSigner], ) -> Self |  |  |  |  |

| `VerificationReceipt` | struct | VerificationReceipt { pub effect_digest: String, pub principal: String, pub policy_epoch: u64, pub revocation_epoch: u64, pub generation: u64, pub audience: String, pub verified_key_ids: Vec<String>, pub verified_custodian_ids: Vec<String> } |  |  |  |  |

| `allocate` | function | allocate(&mut self, child: ResourceEnvelope) -> Result<(), SecurityRefusal> |  |  |  |  |

| `committed` | function | committed(&self) -> ResourceEnvelope |  |  |  |  |

| `contains` | function | contains(self, child: Self) -> bool |  |  |  |  |

| `new` | function | new(root: ResourceEnvelope) -> Result<Self, SecurityRefusal> |  |  |  |  |

| `validate` | function | validate(self) -> Result<Self, SecurityRefusal> |  |  |  |  |

| `BudgetLedger` | struct | BudgetLedger { root: ResourceEnvelope, committed: ResourceEnvelope } |  |  |  |  |

| `ResourceEnvelope` | struct | ResourceEnvelope { pub compute_units: u64, pub io_bytes: u64, pub effects: u64 } |  |  |  |  |

| `CertificateSignature` | struct | CertificateSignature { pub key_id: String, pub algorithm: SignatureAlgorithm, pub signature: Vec<u8> } |  |  |  |  |

| `verify` | function | verify( &self, effect: &PreparedEffect, certificate: &ActuationCertificate, ) -> Result<VerificationReceipt, SecurityRefusal> |  |  |  |  |

| `CertificateVerifier` | struct | CertificateVerifier { pub registry: &'a KeyRegistry, pub expected_epochs: SecurityEpochs, pub expected_audience: &'a str, pub now_ms: u64 } |  |  |  |  |

| `super::receipt::VerificationReceipt` | use | super::receipt::VerificationReceipt |  |  |  |  |

| `EvidenceSurface` | enum | EvidenceSurface { Oscal, Stix21, Taxii21, Ocsf, Sarif, CycloneDx, Spdx, Csaf, OpenTelemetry, Ocel, Json, JsonLd, Rdf, Csv, Xml, Syslog, NativeOpaque } |  |  |  |  |

| `MappingRelation` | enum | MappingRelation { Related, InformativeReference, Narrows, Broadens, Implements, Assesses, Mitigates, Detects, Observes, Evidences, Translates, Contradicts, Equivalent } |  |  |  |  |

| `SecurityStanding` | enum | SecurityStanding { Unknown, PartialAlive, Alive, Refused } |  |  |  |  |

| `admit_external_tool_evidence` | function | admit_external_tool_evidence( evidence: ExternalToolEvidence<'a>, ) -> Result<AdmittedExternalToolEvidence<'a>, String> |  |  |  |  |

| `admit_security_mapping` | function | admit_security_mapping(mapping: SecurityMapping<'a>) -> Result<AdmittedSecurityMapping<'a>, String> |  |  |  |  |

| `find_source` | function | find_source(id: &str) -> Option<&'static SecuritySource> |  |  |  |  |

| `find_tool` | function | find_tool(id: &str) -> Option<&'static SecurityToolIntegration> |  |  |  |  |

| `manufacture_security_intent` | function | manufacture_security_intent( tool_id: &'a str, target_subject: &'a str, operation: &'a str, parent_evidence_digest: &'a str, ) -> Result<SecurityIntent<'a>, String> |  |  |  |  |

| `qualify_federated_fortune5` | function | qualify_federated_fortune5( base: &crate::fortune5::Fortune5Qualification, evidence: &[CoverageEvidence<'_>], ) -> Result<FederatedFortune5Qualification, String> |  |  |  |  |

| `qualify_fortune5_security_universe` | function | qualify_fortune5_security_universe( evidence: &[CoverageEvidence<'_>], ) -> Result<SecurityQualification, String> |  |  |  |  |

| `validate_security_catalog` | function | validate_security_catalog() -> Result<(), String> |  |  |  |  |

| `validate_tool_catalog` | function | validate_tool_catalog() -> Result<(), String> |  |  |  |  |

| `AdmittedExternalToolEvidence` | struct | AdmittedExternalToolEvidence { pub evidence: ExternalToolEvidence<'a>, pub direct_actuation_authority: bool } |  |  |  |  |

| `AdmittedSecurityMapping` | struct | AdmittedSecurityMapping { pub mapping: SecurityMapping<'a> } |  |  |  |  |

| `CoverageEvidence` | struct | CoverageEvidence { pub source_id: &'a str, pub source_digest: &'a str, pub imported_objects: u64, pub mapped_objects: u64, pub verified_objects: u64, pub receipt_digest: Option<&'a str> } |  |  |  |  |

| `ExternalToolEvidence` | struct | ExternalToolEvidence { pub tool_id: &'a str, pub authority: &'a str, pub native_version: &'a str, pub native_object_id: &'a str, pub surface: EvidenceSurface, pub adapter_identity_digest: &'a str, pub payload_digest: &'a str, pub receipt_digest: &'a str } |  |  |  |  |

| `FederatedFortune5Qualification` | struct | FederatedFortune5Qualification { pub standing: SecurityStanding, pub base_standing: crate::fortune5::Standing, pub security: SecurityQualification } |  |  |  |  |

| `SecurityIntent` | struct | SecurityIntent { pub tool_id: &'a str, pub target_subject: &'a str, pub operation: &'a str, pub parent_evidence_digest: &'a str, pub direct_actuation_authority: bool } |  |  |  |  |

| `SecurityMapping` | struct | SecurityMapping { pub left_source_id: &'a str, pub left_object_id: &'a str, pub right_source_id: &'a str, pub right_object_id: &'a str, pub relation: MappingRelation, pub equivalence_proof_receipt: Option<&'a str> } |  |  |  |  |

| `SecurityQualification` | struct | SecurityQualification { pub standing: SecurityStanding, pub required_sources: usize, pub observed_sources: usize, pub fully_verified_sources: usize, pub missing_sources: Vec<&'static str>, pub partial_sources: Vec<&'static str> } |  |  |  |  |

| `SECURITY_SOURCES` | const | SECURITY_SOURCES: &[SecuritySource] |  |  |  |  |

| `SecurityKind` | enum | SecurityKind { GovernanceFramework, ControlCatalog, Assurance, Regulation, CloudBaseline, ApplicationSecurity, AiSecurity, IndustrialControl, ThreatKnowledge, VulnerabilityKnowledge, DetectionLanguage, TelemetrySchema, SupplyChain, EvidenceExchange, SecurityOntology, PolicyLanguage } |  |  |  |  |

| `VersionPolicy` | enum | VersionPolicy { Pinned, Rolling } |  |  |  |  |

| `nist-csf` | str_key | SECURITY_SOURCES = "nist-csf" |  |  |  |  |

| `SecuritySource` | struct | SecuritySource { pub id: &'static str, pub authority: &'static str, pub version: &'static str, pub version_policy: VersionPolicy, pub kind: SecurityKind, pub machine_surface: &'static str, pub source_uri: &'static str } |  |  |  |  |

| `0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef` | str_key | D = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef" |  |  |  |  |

| `FORTUNE5_SECURITY_CORE` | const | FORTUNE5_SECURITY_CORE: &[&str] |  |  |  |  |

| `SECURITY_TOOLS` | const | SECURITY_TOOLS: &[SecurityToolIntegration] |  |  |  |  |

| `ToolBoundary` | enum | ToolBoundary { ObserveOnly, AssessOnly, DetectOnly, ConstructIntentOnly, ExternalEnforcerBehindCastleAdmission } |  |  |  |  |

| `aws-security-hub` | str_key | SECURITY_TOOLS = "aws-security-hub" |  |  |  |  |

| `nist-csf` | str_key | FORTUNE5_SECURITY_CORE = "nist-csf" |  |  |  |  |

| `SecurityToolIntegration` | struct | SecurityToolIntegration { pub id: &'static str, pub ecosystem: &'static str, pub native_surface: &'static str, pub normalized_output: &'static str, pub boundary: ToolBoundary, pub direct_actuation_authority: bool } |  |  |  |  |

| `assess_board_reentry` | function | assess_board_reentry( constitution: &BoardConstitution, strategic_receipt: &BoardStrategicReceipt, event: &crate::board::MaterialityEvent, policy: &crate::board::MaterialityPolicy, ) -> Result<BoardReentryDecision, String> |  |  |  |  |

| `assess_counterstrategies` | function | assess_counterstrategies( constitution: &BoardConstitution, partition: &StrategyPartition, candidate: &CampaignCandidate, scenarios: &[CounterstrategyScenario], ) -> Result<CounterstrategyAssessment, String> |  |  |  |  |

| `build_strategic_board_package` | function | build_strategic_board_package( base: &crate::board::BoardPackage, constitution: &BoardConstitution, mandate: &StrategicMandatePacket, receipt: &BoardStrategicReceipt, portfolio: &CampaignPortfolioAnalysis, counterstrategy: &CounterstrategyAssessment, twin: &StrategicTwinSnapshot, reentry: &BoardReentryDecision, generated_at: &str, ) -> Result<StrategicBoardPackage, String> |  |  |  |  |

| `build_strategic_twin_snapshot` | function | build_strategic_twin_snapshot( constitution: &BoardConstitution, mandate: &StrategicMandatePacket, receipt: &BoardStrategicReceipt, portfolio: &CampaignPortfolioAnalysis, counterstrategy: &CounterstrategyAssessment, ) -> Result<StrategicTwinSnapshot, String> |  |  |  |  |

| `diff_strategic_twins` | function | diff_strategic_twins( previous: &StrategicTwinSnapshot, current: &StrategicTwinSnapshot, ) -> Result<StrategicBoardDelta, String> |  |  |  |  |

| `judge_counterstrategy` | function | judge_counterstrategy( constitution: &BoardConstitution, partition: &StrategyPartition, candidate: &CampaignCandidate, scenario: &CounterstrategyScenario, ) -> Result<CounterstrategyVerdict, String> |  |  |  |  |

| `qualify_campaign_portfolio` | function | qualify_campaign_portfolio( candidates: &[CampaignCandidate], verdicts: &[CampaignVerdict], policy: &CampaignPortfolioPolicy, ) -> Result<CampaignPortfolioAnalysis, String> |  |  |  |  |

| `verify_strategic_board_package_offline` | function | verify_strategic_board_package_offline( package: &StrategicBoardPackage, ) -> OfflineBoardPackageVerification |  |  |  |  |

| `BoardReentryDecision` | struct | BoardReentryDecision { pub required: bool, pub reasons: Vec<String>, pub escalate_by_epoch_ms: Option<i64>, pub materiality_score_bps: i64, pub triggering_dimensions: Vec<crate::board::MaterialityDimension> } |  |  |  |  |

| `CampaignPortfolioAnalysis` | struct | CampaignPortfolioAnalysis { pub standing: StrategicStanding, pub subject: String, pub constitution_digest: String, pub campaign_ids: Vec<String>, pub aggregate_capital_committed: u64, pub aggregate_reversible_capital: u64, pub reversible_capital_bps: u64, pub max_single_campaign_concentration_bps: u64, pub reasons: Vec<String>, pub analysis_digest: String } |  |  |  |  |

| `CampaignPortfolioPolicy` | struct | CampaignPortfolioPolicy { pub aggregate_capital_at_risk_limit: u64, pub max_single_campaign_concentration_bps: u64, pub min_reversible_capital_bps: u64 } |  |  |  |  |

| `CounterstrategyAssessment` | struct | CounterstrategyAssessment { pub campaign_id: String, pub campaign_digest: String, pub standing: StrategicStanding, pub verdicts: Vec<CounterstrategyVerdict>, pub assessment_digest: String } |  |  |  |  |

| `CounterstrategyScenario` | struct | CounterstrategyScenario { pub scenario_id: String, pub local_premise_mutations: BTreeMap<String, String>, pub falsifier_triggered: bool, pub prohibited_outcomes_reached: Vec<String>, pub authority_expansion_attempts: Vec<String>, pub additional_capital_required: u64, pub remaining_options: u32 } |  |  |  |  |

| `CounterstrategyVerdict` | struct | CounterstrategyVerdict { pub scenario_id: String, pub standing: StrategicStanding, pub refusals: Vec<String>, pub verdict_digest: String } |  |  |  |  |

| `OfflineBoardPackageVerification` | struct | OfflineBoardPackageVerification { pub standing: StrategicStanding, pub reasons: Vec<String> } |  |  |  |  |

| `StrategicBoardDelta` | struct | StrategicBoardDelta { pub subject: String, pub mandate_id: String, pub changed_dimensions: Vec<String>, pub requires_board_attention: bool, pub previous_snapshot_digest: String, pub current_snapshot_digest: String } |  |  |  |  |

| `StrategicBoardPackage` | struct | StrategicBoardPackage { pub profile: &'static str, pub subject: String, pub generated_at: String, pub fortune5_board_package_digest: String, pub constitution_digest: String, pub mandate_packet_digest: String, pub strategic_receipt_digest: String, pub portfolio_analysis_digest: String, pub counterstrategy_assessment_digest: String, pub twin_snapshot_digest: String, pub strategic_standing: StrategicStanding, pub board_reentry_required: bool, pub authority_ceiling: &'static str, pub actuation: &'static str, pub package_digest: String } |  |  |  |  |

| `StrategicTwinSnapshot` | struct | StrategicTwinSnapshot { pub subject: String, pub mandate_id: String, pub campaign_id: String, pub standing: StrategicStanding, pub aggregate_capital_committed: u64, pub aggregate_reversible_capital: u64, pub options_remaining: u32, pub falsified_premises: Vec<String>, pub failed_counterstrategy_scenarios: Vec<String>, pub prohibited_outcome_witnesses: Vec<String>, pub authority_expansions: Vec<String>, pub snapshot_digest: String } |  |  |  |  |

| `BOARD_LENSES` | const | BOARD_LENSES: [BoardLens; 8] |  |  |  |  |

| `STRATEGIC_ACTUATION` | const | STRATEGIC_ACTUATION: &str |  |  |  |  |

| `STRATEGIC_AUTHORITY_CEILING` | const | STRATEGIC_AUTHORITY_CEILING: &str |  |  |  |  |

| `STRATEGIC_SUCCESSOR_BOUNDARY` | const | STRATEGIC_SUCCESSOR_BOUNDARY: &str |  |  |  |  |

| `STRATEGY_OPERATORS` | const | STRATEGY_OPERATORS: &[StrategyOperator] |  |  |  |  |

| `BoardAvatar` | enum | BoardAvatar { Audit, Risk, CapitalAllocation, Resilience, Governance, Safety, CompetitiveStrategy, LeadIndependent } |  |  |  |  |

| `RecompileScope` | enum | RecompileScope { None, Campaign(Vec<String>), Strategic } |  |  |  |  |

| `ReplanLevel` | enum | ReplanLevel { PolicyBranch, SuffixReuse, FollowBiasedTailRepair, BoundedFullReplan, HierarchyRecompile, StrategicRecompile } |  |  |  |  |

| `StrategicAxis` | enum | StrategicAxis { Position, Topology, OptionSpace, Belief, Tempo, Objective, Authority, Reversibility } |  |  |  |  |

| `StrategicStanding` | enum | StrategicStanding { Alive, Refused } |  |  |  |  |

| `admit_board_selection` | function | admit_board_selection( constitution: &BoardConstitution, candidates: &[CampaignCandidate], verdicts: &[CampaignVerdict], request: BoardSelectionRequest, ) -> Result<StrategicMandatePacket, String> |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `bind_strategic_mandate_construct_request` | function | bind_strategic_mandate_construct_request( mut request: crate::castle::ConstructRequest, mandate: &StrategicMandatePacket, ) -> Result<crate::castle::ConstructRequest, String> |  |  |  |  |

| `candidate_digest` | function | candidate_digest(&self) -> Result<String, String> |  |  |  |  |

| `compile_board_constitution` | function | compile_board_constitution(input: ConstitutionInput) -> Result<BoardConstitution, String> |  |  |  |  |

| `compile_board_strategic_receipt` | function | compile_board_strategic_receipt( constitution: &BoardConstitution, candidate: &CampaignCandidate, mandate: &StrategicMandatePacket, input: BoardReceiptInput, ) -> Result<BoardStrategicReceipt, String> |  |  |  |  |

| `compile_strategy_doctrine` | function | compile_strategy_doctrine( constitution: &BoardConstitution, premise_digests: BTreeMap<String, String>, capability_ids: Vec<String>, provenance: Vec<String>, ) -> Result<StrategyDoctrine, String> |  |  |  |  |

| `construct_campaign_candidate` | function | construct_campaign_candidate( constitution: &BoardConstitution, doctrine: &StrategyDoctrine, partition: &StrategyPartition, candidate_id: impl Into<String>, assumptions: Vec<String>, falsifier: impl Into<String>, objectives: BTreeMap<String, i64>, expected_outcomes: Vec<String>, capital_committed: u64, reversible_capital: u64, ) -> Result<CampaignCandidate, String> |  |  |  |  |

| `determine_recompile_scope` | function | determine_recompile_scope( doctrine: &StrategyDoctrine, partitions: &[StrategyPartition], current_global_premises: &BTreeMap<String, String>, current_local_premises: &BTreeMap<String, String>, ) -> RecompileScope |  |  |  |  |

| `judge_campaign_candidate` | function | judge_campaign_candidate( constitution: &BoardConstitution, doctrine: &StrategyDoctrine, partition: &StrategyPartition, candidate: &CampaignCandidate, current_local_premises: &BTreeMap<String, String>, ) -> CampaignVerdict |  |  |  |  |

| `partition_strategy` | function | partition_strategy( doctrine: &StrategyDoctrine, strategy_id: impl Into<String>, strategy: impl Into<String>, local_premise_digests: BTreeMap<String, String>, local_constraints: Vec<String>, operator_ids: Vec<String>, ) -> Result<StrategyPartition, String> |  |  |  |  |

| `route_replan` | function | route_replan(evidence: DivergenceEvidence) -> ReplanLevel |  |  |  |  |

| `to_json` | function | to_json(&self) -> Value |  |  |  |  |

| `BRCE` | str_key | STRATEGIC_SUCCESSOR_BOUNDARY = "BRCE" |  |  |  |  |

| `CONSTRUCT` | str_key | STRATEGIC_AUTHORITY_CEILING = "CONSTRUCT" |  |  |  |  |

| `Can we prove the corporation did what the board authorized?` | str_key | BOARD_LENSES = "Can we prove the corporation did what the board authorized?" |  |  |  |  |

| `NONE` | str_key | STRATEGIC_ACTUATION = "NONE" |  |  |  |  |

| `OBSERVE` | str_key | ALLOWED_ACTIONS = "OBSERVE" |  |  |  |  |

| `refuse-last-war` | str_key | STRATEGY_OPERATORS = "refuse-last-war" |  |  |  |  |

| `BoardConstitution` | struct | BoardConstitution { pub subject: String, pub mandate_id: String, pub objectives: Vec<String>, pub prohibited_outcomes: Vec<String>, pub invariants: Vec<String>, pub delegated_authority: Vec<String>, pub nondelegable_decisions: Vec<String>, pub capital_at_risk_limit: u64, pub escalation_conditions: Vec<String>, pub withdrawal_conditions: Vec<String>, pub evidence_requirements: Vec<String>, pub authority_ceiling: &'static str, pub constitution_digest: String } |  |  |  |  |

| `BoardLens` | struct | BoardLens { pub avatar: BoardAvatar, pub question: &'static str, pub governed_surface: &'static str } |  |  |  |  |

| `BoardReceiptInput` | struct | BoardReceiptInput { pub falsified_premises: Vec<String>, pub material_exceptions: Vec<String>, pub options_remaining: u32, pub prohibited_outcome_witnesses: Vec<String>, pub authority_expansions: Vec<String>, pub next_board_decision: Option<String>, pub evidence_digest: String } |  |  |  |  |

| `BoardSelectionRequest` | struct | BoardSelectionRequest { pub candidate_id: String, pub selection_authority_digest: String, pub selected_by: String, pub selected_at: String } |  |  |  |  |

| `BoardStrategicReceipt` | struct | BoardStrategicReceipt { pub standing: StrategicStanding, pub mandate_id: String, pub constitution_digest: String, pub campaign_id: String, pub campaign_digest: String, pub falsified_premises: Vec<String>, pub material_exceptions: Vec<String>, pub options_remaining: u32, pub prohibited_outcome_witnesses: Vec<String>, pub authority_expansions: Vec<String>, pub next_board_decision: Option<String>, pub evidence_digest: String, pub authority_ceiling: &'static str, pub actuation: &'static str, pub receipt_digest: String } |  |  |  |  |

| `CampaignCandidate` | struct | CampaignCandidate { pub candidate_id: String, pub subject: String, pub strategy_id: String, pub constitution_digest: String, pub doctrine_digest: String, pub partition_digest: String, pub preserved_invariants: Vec<String>, pub capabilities: Vec<String>, pub actions: Vec<String>, pub assumptions: Vec<String>, pub falsifier: String, pub objectives: BTreeMap<String, i64>, pub expected_outcomes: Vec<String>, pub capital_committed: u64, pub reversible_capital: u64, pub authority_ceiling: String, pub observed_partition_digests: Vec<String> } |  |  |  |  |

| `CampaignVerdict` | struct | CampaignVerdict { pub standing: StrategicStanding, pub candidate_id: String, pub candidate_digest: String, pub refusals: Vec<String> } |  |  |  |  |

| `ConstitutionInput` | struct | ConstitutionInput { pub subject: String, pub mandate_id: String, pub objectives: Vec<String>, pub prohibited_outcomes: Vec<String>, pub invariants: Vec<String>, pub delegated_authority: Vec<String>, pub nondelegable_decisions: Vec<String>, pub capital_at_risk_limit: u64, pub escalation_conditions: Vec<String>, pub withdrawal_conditions: Vec<String>, pub evidence_requirements: Vec<String> } |  |  |  |  |

| `DivergenceEvidence` | struct | DivergenceEvidence { pub policy_branch_available: bool, pub valid_suffix_available: bool, pub tail_repair_available: bool, pub hierarchy_premise_broken: bool, pub strategic_premise_broken: bool } |  |  |  |  |

| `StrategicMandatePacket` | struct | StrategicMandatePacket { pub subject: String, pub mandate_id: String, pub constitution_digest: String, pub candidate_id: String, pub candidate_digest: String, pub selection_authority_digest: String, pub selected_by: String, pub selected_at: String, pub evidence_requirements: Vec<String>, pub authority_ceiling: &'static str, pub actuation: &'static str, pub successor_boundary: &'static str, pub packet_digest: String } |  |  |  |  |

| `StrategyDoctrine` | struct | StrategyDoctrine { pub subject: String, pub constitution_digest: String, pub premise_digests: BTreeMap<String, String>, pub invariants: Vec<String>, pub capability_ids: Vec<String>, pub provenance: Vec<String>, pub authority_ceiling: &'static str, pub doctrine_digest: String } |  |  |  |  |

| `StrategyOperator` | struct | StrategyOperator { pub id: &'static str, pub provenance: &'static str, pub reads: &'static [StrategicAxis], pub writes: &'static [StrategicAxis] } |  |  |  |  |

| `StrategyPartition` | struct | StrategyPartition { pub strategy_id: String, pub strategy: String, pub doctrine_digest: String, pub local_premise_digests: BTreeMap<String, String>, pub local_constraints: Vec<String>, pub operator_ids: Vec<String>, pub partition_digest: String } |  |  |  |  |

| `crate::strategic_board::{ assess_board_reentry, assess_counterstrategies, build_strategic_board_package, build_strategic_twin_snapshot, diff_strategic_twins, judge_counterstrategy, qualify_campaign_portfolio, verify_strategic_board_package_offline, BoardReentryDecision, CampaignPortfolioAnalysis, CampaignPortfolioPolicy, CounterstrategyAssessment, CounterstrategyScenario, CounterstrategyVerdict, OfflineBoardPackageVerification, StrategicBoardDelta, StrategicBoardPackage, StrategicTwinSnapshot, }` | use | crate::strategic_board::{ assess_board_reentry, assess_counterstrategies, build_strategic_board_package, build_strategic_twin_snapshot, diff_strategic_twins, judge_counterstrategy, qualify_campaign_portfolio, verify_strategic_board_package_offline, BoardReentryDecision, CampaignPortfolioAnalysis, CampaignPortfolioPolicy, CounterstrategyAssessment, CounterstrategyScenario, CounterstrategyVerdict, OfflineBoardPackageVerification, StrategicBoardDelta, StrategicBoardPackage, StrategicTwinSnapshot, } |  |  |  |  |

| `admit_airgap_result` | function | admit_airgap_result(bundle: &AirgapBundle, result: &AirgapResult) -> AirgapAdmission |  |  |  |  |

| `manufacture_airgap_bundle` | function | manufacture_airgap_bundle( bundle_id: String, constitution: &GlobalConstitution, o_star_snapshot: Value, construct_graph: Value, prohibited_goals: Value, ) -> Result<AirgapBundle, String> |  |  |  |  |

| `AirgapAdmission` | struct | AirgapAdmission { pub standing: ReleaseStanding, pub reason: String } |  |  |  |  |

| `AirgapBundle` | struct | AirgapBundle { pub kind: String, pub release: String, pub bundle_id: String, pub constitution_id: String, pub ontology_version: String, pub provider_semantics: BTreeMap<String, String>, pub invariant_set_digest: String, pub o_star_snapshot: Value, pub construct_graph: Value, pub prohibited_goals: Value, pub network_dependencies: Vec<String>, pub secret_dependencies: Vec<String>, pub bundle_digest: String } |  |  |  |  |

| `AirgapResult` | struct | AirgapResult { pub bundle_id: String, pub input_bundle_digest: String, pub result_digest: String, pub network_used: bool, pub secret_material_used: bool } |  |  |  |  |

| `execute_command_process` | function | execute_command_process( process: &PowlProcess, state: &WorldState, envelope: &TestEnvelope, admission: &ConstructAdmission, policy: CommandAdapterPolicy, blake3: &dyn Blake3Provider, signer: &dyn ReceiptSigner, now: impl Fn() -> i64, ) -> Result<(ReceiptedOcelLog, Vec<BrceTransitionRecord>), String> |  |  |  |  |

| `execute_command_process_durable` | function | execute_command_process_durable( process: &PowlProcess, state: &WorldState, envelope: &TestEnvelope, admission: &ConstructAdmission, policy: CommandAdapterPolicy, durable_journal_root: impl AsRef<Path>, blake3: &dyn Blake3Provider, signer: &dyn ReceiptSigner, now: impl Fn() -> i64, ) -> Result<(ReceiptedOcelLog, Vec<BrceTransitionRecord>), String> |  |  |  |  |

| `fortune5_adapter_catalog` | function | fortune5_adapter_catalog() -> Vec<ProviderAdapterDescriptor> |  |  |  |  |

| `journal` | function | journal(&self) -> Vec<BrceTransitionRecord> |  |  |  |  |

| `new` | function | new(inner: &'a dyn GymActAdapter, blake3: &'a dyn Blake3Provider, signer: &'a dyn ReceiptSigner) -> Self |  |  |  |  |

| `new_durable` | function | new_durable( inner: &'a dyn GymActAdapter, blake3: &'a dyn Blake3Provider, signer: &'a dyn ReceiptSigner, durable_journal_root: PathBuf, ) -> Self |  |  |  |  |

| `validate_command_adapter_policy` | function | validate_command_adapter_policy(policy: &CommandAdapterPolicy) -> ReleaseStanding |  |  |  |  |

| `BrceGymActAdapter` | struct | BrceGymActAdapter { inner: &'a dyn GymActAdapter, blake3: &'a dyn Blake3Provider, signer: &'a dyn ReceiptSigner, journal: Mutex<Vec<BrceTransitionRecord>>, durable_journal_root: Option<PathBuf> } |  |  |  |  |

| `BrceTransitionRecord` | struct | BrceTransitionRecord { pub transition_id: String, pub prepare_receipt: Receipt, pub outcome_receipt: Option<Receipt>, pub standing: ReleaseStanding, pub reason: String } |  |  |  |  |

| `CommandAdapterPolicy` | struct | CommandAdapterPolicy { pub adapter_id: String, pub provider: String, pub workload_identity: String, pub commands: BTreeMap<String, CommandSpec> } |  |  |  |  |

| `CommandSpec` | struct | CommandSpec { pub transition_id: String, pub program: String, pub args: Vec<String>, pub allowed_exit_codes: BTreeSet<i32>, pub max_output_bytes: usize, pub timeout_ms: u64 } |  |  |  |  |

| `DurableBrceOutcomeRecord` | struct | DurableBrceOutcomeRecord { pub kind: String, pub release: String, pub transition_id: String, pub prepare_receipt_digest: String, pub outcome_receipt: DurableBrceReceipt, pub provider_status: String } |  |  |  |  |

| `DurableBrcePrepareRecord` | struct | DurableBrcePrepareRecord { pub kind: String, pub release: String, pub transition_id: String, pub subject: String, pub construct_digest: String, pub process_digest: String, pub prepare_receipt: DurableBrceReceipt } |  |  |  |  |

| `DurableBrceReceipt` | struct | DurableBrceReceipt { pub algorithm: String, pub artifact_digest: String, pub receipt_digest: String, pub epistemic_class: String, pub subject: String, pub parent_digests: Vec<String>, pub origin_key_id: String, pub origin_signature: String } |  |  |  |  |

| `ProviderAdapterDescriptor` | struct | ProviderAdapterDescriptor { pub kind: String, pub program: String, pub authority_model: String, pub ambient_credentials_allowed: bool } |  |  |  |  |

| `ChaosScenario` | enum | ChaosScenario { RegionLoss, WanPartition, StalePolicy, RevokedAuthority, ClockSkew, ProviderThrottle, PartialActuation, ReceiptStoreLoss } |  |  |  |  |

| `qualify_chaos` | function | qualify_chaos(evidence: &[ChaosEvidence]) -> ChaosQualification |  |  |  |  |

| `required_chaos_scenarios` | function | required_chaos_scenarios() -> Vec<ChaosScenario> |  |  |  |  |

| `ChaosEvidence` | struct | ChaosEvidence { pub scenario: ChaosScenario, pub exercised: bool, pub failed_closed: bool, pub receipt_digest: String, pub detail: String } |  |  |  |  |

| `ChaosQualification` | struct | ChaosQualification { pub standing: ReleaseStanding, pub reasons: Vec<String>, pub scenarios: BTreeMap<ChaosScenario, ReleaseStanding> } |  |  |  |  |

| `SignatureSuite` | enum | SignatureSuite { Ed25519, EcdsaP256, RsaPssSha256, MlDsa, SlhDsa } |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `dual_artifact_identity` | function | dual_artifact_identity(bytes: &[u8]) -> ArtifactIdentity |  |  |  |  |

| `implemented_signature_suites` | function | implemented_signature_suites() -> BTreeSet<SignatureSuite> |  |  |  |  |

| `qualify_crypto_profile` | function | qualify_crypto_profile(profile: &CryptoProfile) -> CryptoQualification |  |  |  |  |

| `qualify_pqc_runtime` | function | qualify_pqc_runtime() -> PqcRuntimeQualification |  |  |  |  |

| `sign_pqc_message` | function | sign_pqc_message( suite: SignatureSuite, seed: [u8; 32], message: &[u8], ) -> Result<PqcSignatureProof, String> |  |  |  |  |

| `verify_pqc_message` | function | verify_pqc_message(proof: &PqcSignatureProof, message: &[u8]) -> bool |  |  |  |  |

| `ArtifactIdentity` | struct | ArtifactIdentity { pub blake3_256: String, pub sha256: String } |  |  |  |  |

| `CryptoProfile` | struct | CryptoProfile { pub required_identity_hashes: BTreeSet<String>, pub accepted_signature_suites: BTreeSet<SignatureSuite>, pub require_post_quantum: bool } |  |  |  |  |

| `CryptoQualification` | struct | CryptoQualification { pub standing: ReleaseStanding, pub reasons: Vec<String>, pub implemented_signature_suites: BTreeSet<SignatureSuite> } |  |  |  |  |

| `PqcRuntimeQualification` | struct | PqcRuntimeQualification { pub standing: ReleaseStanding, pub ml_dsa_65: bool, pub slh_dsa_shake_128f: bool, pub reasons: Vec<String> } |  |  |  |  |

| `PqcSignatureProof` | struct | PqcSignatureProof { pub suite: SignatureSuite, pub parameter_set: String, pub message_blake3: String, pub public_key_hex: String, pub signature_hex: String } |  |  |  |  |

| `ProbePurpose` | enum | ProbePurpose { Version, AuthorityContext, SelfTest } |  |  |  |  |

| `ReadOnlyProbeKind` | enum | ReadOnlyProbeKind { AwsCliVersion, AwsAuthorityContext, AzureCliVersion, AzureAuthorityContext, GcpCliVersion, GcpAuthorityContext, KubernetesCliVersion, KubernetesAuthorityContext, GitHubCliVersion, GitHubAuthorityContext, LocalSelfTest } |  |  |  |  |

| `args` | function | args(self) -> Vec<&'static str> |  |  |  |  |

| `bind_command_policy` | function | bind_command_policy( binding: &AdapterBinding, policy: &CommandAdapterPolicy, ) -> Result<CommandAdapterPolicy, String> |  |  |  |  |

| `default_program` | function | default_program(self) -> &'static str |  |  |  |  |

| `live_observation_index` | function | live_observation_index( observations: &[ProviderProbeObservation], ) -> BTreeMap<String, BTreeSet<String>> |  |  |  |  |

| `manufacture_live_probe_plan` | function | manufacture_live_probe_plan(manifest: &DeploymentManifest) -> Vec<ReadOnlyProbeSpec> |  |  |  |  |

| `provider_kind` | function | provider_kind(self) -> &'static str |  |  |  |  |

| `purpose` | function | purpose(self) -> ProbePurpose |  |  |  |  |

| `qualify_dfcm_closure` | function | qualify_dfcm_closure( manifest: &DeploymentManifest, observations: &[ProviderProbeObservation], crypto_profile: &CryptoProfile, now_epoch_ms: i64, max_live_evidence_age_ms: i64, ) -> DfcmClosureQualification |  |  |  |  |

| `qualify_live_deployment` | function | qualify_live_deployment( manifest: &DeploymentManifest, observations: &[ProviderProbeObservation], now_epoch_ms: i64, max_age_ms: i64, ) -> LiveDeploymentQualification |  |  |  |  |

| `qualify_protocol_fence` | function | qualify_protocol_fence() -> ProtocolFenceQualification |  |  |  |  |

| `run_read_only_probe` | function | run_read_only_probe( spec: &ReadOnlyProbeSpec, observed_at_epoch_ms: i64, ) -> Result<ProviderProbeObservation, String> |  |  |  |  |

| `DfcmClosureQualification` | struct | DfcmClosureQualification { pub standing: ReleaseStanding, pub static_deployment: ReleaseStanding, pub live_deployment: ReleaseStanding, pub crypto_profile: ReleaseStanding, pub pqc_runtime: PqcRuntimeQualification, pub protocol_fence: ProtocolFenceQualification, pub findings: Vec<String> } |  |  |  |  |

| `LiveDeploymentQualification` | struct | LiveDeploymentQualification { pub standing: ReleaseStanding, pub static_manifest_digest: String, pub cells: usize, pub adapters_expected: usize, pub adapters_alive: usize, pub findings: Vec<String> } |  |  |  |  |

| `ProtocolDispatchSummary` | struct | ProtocolDispatchSummary { pub origin: InterfaceOrigin, pub select_standing: ReleaseStanding, pub construct_standing: ReleaseStanding, pub do_standing: ReleaseStanding } |  |  |  |  |

| `ProtocolFenceQualification` | struct | ProtocolFenceQualification { pub standing: ReleaseStanding, pub dispatches: Vec<ProtocolDispatchSummary>, pub findings: Vec<String> } |  |  |  |  |

| `ProviderProbeObservation` | struct | ProviderProbeObservation { pub cell_id: String, pub adapter_id: String, pub provider_kind: String, pub workload_identity: String, pub provider_semantics_version: String, pub kind: ReadOnlyProbeKind, pub purpose: ProbePurpose, pub observed_at_epoch_ms: i64, pub exit_code: i32, pub stdout_blake3: String, pub stderr_blake3: String, pub identity_binding_verified: bool, pub observation_digest: String, pub standing: ReleaseStanding, pub reason: String } |  |  |  |  |

| `ReadOnlyProbeSpec` | struct | ReadOnlyProbeSpec { pub cell_id: String, pub adapter_id: String, pub workload_identity: String, pub provider_semantics_version: String, pub kind: ReadOnlyProbeKind, pub program_override: Option<String>, pub expected_identity_marker: Option<String>, pub max_output_bytes: usize, pub timeout_ms: u64 } |  |  |  |  |

| `persist_evidence` | function | persist_evidence(root: impl AsRef<Path>, record: &DurableEvidenceRecord) -> Result<EvidenceCommit, String> |  |  |  |  |

| `verify_evidence_file` | function | verify_evidence_file(path: impl AsRef<Path>) -> Result<EvidenceVerification, String> |  |  |  |  |

| `DurableEvidenceRecord` | struct | DurableEvidenceRecord { pub cell_id: String, pub subject: String, pub construct_digest: String, pub ocel_receipt_digest: String, pub brce_prepare_receipt_digests: Vec<String>, pub brce_outcome_receipt_digests: Vec<String>, pub event_count: usize } |  |  |  |  |

| `EvidenceCommit` | struct | EvidenceCommit { pub standing: ReleaseStanding, pub record_identity: ArtifactIdentity, pub path: String, pub reason: String } |  |  |  |  |

| `EvidenceVerification` | struct | EvidenceVerification { pub standing: ReleaseStanding, pub record_identity: ArtifactIdentity, pub path: String, pub record: DurableEvidenceRecord, pub reason: String } |  |  |  |  |

| `RELEASE_KIND` | const | RELEASE_KIND: &str |  |  |  |  |

| `RELEASE_VERSION` | const | RELEASE_VERSION: &str |  |  |  |  |

| `26.10.8+dfcm.1` | str_key | RELEASE_VERSION = "26.10.8+dfcm.1" |  |  |  |  |

| `CASTLE_FORTUNE5_GLOBAL_V1` | str_key | RELEASE_KIND = "CASTLE_FORTUNE5_GLOBAL_V1" |  |  |  |  |

| `airgap::*` | use | airgap::* |  |  |  |  |

| `brce::*` | use | brce::* |  |  |  |  |

| `chaos::*` | use | chaos::* |  |  |  |  |

| `crypto::*` | use | crypto::* |  |  |  |  |

| `dfcm::*` | use | dfcm::* |  |  |  |  |

| `evidence::*` | use | evidence::* |  |  |  |  |

| `protocol::*` | use | protocol::* |  |  |  |  |

| `replication::*` | use | replication::* |  |  |  |  |

| `runtime::*` | use | runtime::* |  |  |  |  |

| `topology::*` | use | topology::* |  |  |  |  |

| `IntentMode` | enum | IntentMode { Select, Construct, Do } |  |  |  |  |

| `InterfaceOrigin` | enum | InterfaceOrigin { Cli, Api, Mcp, A2a, Human, Planner, Replay } |  |  |  |  |

| `a2a_agent_card` | function | a2a_agent_card() -> A2aAgentCard |  |  |  |  |

| `admit_interface_intent` | function | admit_interface_intent(intent: &InterfaceIntent) -> InterfaceAdmission |  |  |  |  |

| `admit_observation` | function | admit_observation( observation: &ObservationEnvelope, allowed_sources: &BTreeSet<String>, now_epoch_ms: i64, max_age_ms: i64, ) -> AdmittedObservation |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `dispatch_interface_intent` | function | dispatch_interface_intent(intent: &InterfaceIntent) -> ProtocolDispatch |  |  |  |  |

| `mcp_tool_catalog` | function | mcp_tool_catalog() -> Vec<McpToolDescriptor> |  |  |  |  |

| `A2aAgentCard` | struct | A2aAgentCard { pub name: String, pub version: String, pub capabilities: Vec<String>, pub default_authority: String } |  |  |  |  |

| `AdmittedObservation` | struct | AdmittedObservation { pub standing: ReleaseStanding, pub observation_id: String, pub subject: String, pub source: String, pub payload_digest: String, pub reason: String } |  |  |  |  |

| `InterfaceAdmission` | struct | InterfaceAdmission { pub standing: ReleaseStanding, pub request_id: String, pub reason: String, pub normalized_intent_digest: String } |  |  |  |  |

| `InterfaceIntent` | struct | InterfaceIntent { pub request_id: String, pub origin: InterfaceOrigin, pub mode: IntentMode, pub subject: String, pub operation: String, pub payload: Value, pub construct_admission_digest: Option<String>, pub prepare_receipt_digest: Option<String> } |  |  |  |  |

| `McpToolDescriptor` | struct | McpToolDescriptor { pub name: String, pub mode: IntentMode, pub consequential: bool } |  |  |  |  |

| `ObservationEnvelope` | struct | ObservationEnvelope { pub observation_id: String, pub source: String, pub subject: String, pub observed_at_epoch_ms: i64, pub epistemic_class: String, pub payload: Value } |  |  |  |  |

| `ProtocolDispatch` | struct | ProtocolDispatch { pub standing: ReleaseStanding, pub reason: String, pub request_id: String, pub normalized_intent_digest: String, pub result: Value } |  |  |  |  |

| `admit_receipt_checkpoint` | function | admit_receipt_checkpoint(state: &mut ReplicaState, checkpoint: ReceiptCheckpoint) -> ReplicationAdmission |  |  |  |  |

| `load_durable_replica` | function | load_durable_replica(path: impl AsRef<Path>) -> Result<ReplicaState, String> |  |  |  |  |

| `persist_receipt_checkpoint` | function | persist_receipt_checkpoint( path: impl AsRef<Path>, receiver_id: &str, checkpoint: ReceiptCheckpoint, ) -> Result<DurableReplicaCommit, String> |  |  |  |  |

| `reconcile_transition` | function | reconcile_transition(evidence: &ReconciliationEvidence) -> ReconciliationDecision |  |  |  |  |

| `DurableReplicaCommit` | struct | DurableReplicaCommit { pub standing: ReleaseStanding, pub reason: String, pub path: String, pub state_blake3: String, pub admission: ReplicationAdmission } |  |  |  |  |

| `ReceiptCheckpoint` | struct | ReceiptCheckpoint { pub cell_id: String, pub sequence: u64, pub head_digest: String, pub constitution_id: String, pub observed_at_epoch_ms: i64 } |  |  |  |  |

| `ReconciliationDecision` | struct | ReconciliationDecision { pub standing: ReleaseStanding, pub reason: String, pub replay_allowed: bool } |  |  |  |  |

| `ReconciliationEvidence` | struct | ReconciliationEvidence { pub transition_id: String, pub prepare_receipt_digest: String, pub outcome_receipt_digest: Option<String>, pub provider_observed: Option<bool>, pub retry_is_proven_idempotent: bool } |  |  |  |  |

| `ReplicaState` | struct | ReplicaState { pub receiver_id: String, pub checkpoints: BTreeMap<String, ReceiptCheckpoint> } |  |  |  |  |

| `ReplicationAdmission` | struct | ReplicationAdmission { pub standing: ReleaseStanding, pub reason: String, pub cell_id: String, pub sequence: u64 } |  |  |  |  |

| `decode_seed_hex` | function | decode_seed_hex(value: &str) -> Result<[u8; 32], String> |  |  |  |  |

| `execute_runtime_request` | function | execute_runtime_request( request: &RuntimeExecutionRequest, key_id: String, seed: [u8; 32], expected_construct_digest: &str, now_epoch_ms: i64, ) -> Result<RuntimeDoSummary, String> |  |  |  |  |

| `from_seed` | function | from_seed(key_id: String, seed: [u8; 32]) -> Result<Self, String> |  |  |  |  |

| `manufacture_runtime_construct` | function | manufacture_runtime_construct( request: &RuntimeExecutionRequest, key_id: String, seed: [u8; 32], ) -> Result<ConstructManufactureSummary, String> |  |  |  |  |

| `to_envelope` | function | to_envelope(&self) -> TestEnvelope |  |  |  |  |

| `to_powl` | function | to_powl(&self) -> PowlProcess |  |  |  |  |

| `verifier` | function | verifier(&self) -> Ed25519RuntimeVerifier |  |  |  |  |

| `ConstructManufactureSummary` | struct | ConstructManufactureSummary { pub standing: ReleaseStanding, pub construct_digest: String, pub construct_receipt_digest: String, pub process_digest: String, pub replay_identity_digest: String, pub subject: String, pub authority: String } |  |  |  |  |

| `Ed25519RuntimeSigner` | struct | Ed25519RuntimeSigner { key_id: String, signing_key: SigningKey } |  |  |  |  |

| `Ed25519RuntimeVerifier` | struct | Ed25519RuntimeVerifier { key_id: String, verifying_key: VerifyingKey } |  |  |  |  |

| `NativeBlake3` | struct |  |  |  |  |  |

| `PortableActivity` | struct | PortableActivity { pub id: String, pub transition_id: String, pub predecessors: Vec<String> } |  |  |  |  |

| `PortableEnvelope` | struct | PortableEnvelope { pub system_id: String, pub allowed_transition_ids: BTreeSet<String>, pub max_steps: u32, pub expires_at_epoch_ms: i64 } |  |  |  |  |

| `PortableProcess` | struct | PortableProcess { pub id: String, pub goal_id: String, pub activities: Vec<PortableActivity> } |  |  |  |  |

| `RuntimeDoSummary` | struct | RuntimeDoSummary { pub standing: ReleaseStanding, pub construct: ConstructManufactureSummary, pub ocel_receipt_digest: String, pub event_count: usize, pub brce_prepare_receipt_digests: Vec<String>, pub brce_outcome_receipt_digests: Vec<String>, pub evidence_commit: EvidenceCommit } |  |  |  |  |

| `RuntimeExecutionRequest` | struct | RuntimeExecutionRequest { pub cell_id: String, pub evidence_dir: String, pub subject: String, pub authority: String, pub o_star: Value, pub config_graph: Value, pub ontology: Value, pub process: PortableProcess, pub envelope: PortableEnvelope, pub allowed_authorities: BTreeSet<String>, pub adapter_policy: CommandAdapterPolicy } |  |  |  |  |

| `CloudProvider` | enum | CloudProvider { Aws, Azure, Gcp, PrivateCloud, Edge } |  |  |  |  |

| `ReleaseStanding` | enum | ReleaseStanding { Unknown, PartialAlive, Alive, Blocked, BuildBroken, Unsupported, Refused } |  |  |  |  |

| `aggregate_global_standing` | function | aggregate_global_standing(mut cells: Vec<CellStandingRow>) -> GlobalStanding |  |  |  |  |

| `alive` | function | alive(&self) -> bool |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `qualify_deployment` | function | qualify_deployment(manifest: &DeploymentManifest, now_epoch_ms: i64) -> DeploymentQualification |  |  |  |  |

| `AdapterBinding` | struct | AdapterBinding { pub adapter_id: String, pub kind: String, pub provider_semantics_key: String, pub provider_semantics_version: String, pub allowed_transition_ids: BTreeSet<String>, pub workload_identity: String, pub ambient_credentials: bool } |  |  |  |  |

| `CastleCellManifest` | struct | CastleCellManifest { pub cell_id: String, pub region: String, pub provider: CloudProvider, pub authority_domain: String, pub residency: String, pub subject_prefixes: Vec<String>, pub local_receipt_store: String, pub local_ocel_store: String, pub max_parallel_do: u32, pub adapters: Vec<AdapterBinding> } |  |  |  |  |

| `CellStandingRow` | struct | CellStandingRow { pub cell_id: String, pub observation: ReleaseStanding, pub construct: ReleaseStanding, pub do_standing: ReleaseStanding, pub replay: ReleaseStanding } |  |  |  |  |

| `DeploymentManifest` | struct | DeploymentManifest { pub kind: String, pub release: String, pub constitution: GlobalConstitution, pub cells: Vec<CastleCellManifest>, pub protocol_surfaces: Vec<ProtocolSurface>, pub required_providers: BTreeSet<CloudProvider>, pub required_adapter_kinds: BTreeSet<String> } |  |  |  |  |

| `DeploymentQualification` | struct | DeploymentQualification { pub standing: ReleaseStanding, pub manifest_digest: String, pub findings: Vec<String>, pub cells: usize, pub providers: BTreeSet<CloudProvider>, pub adapter_kinds: BTreeSet<String> } |  |  |  |  |

| `GlobalConstitution` | struct | GlobalConstitution { pub constitution_id: String, pub ontology_version: String, pub invariant_set_digest: String, pub trust_root_ids: BTreeSet<String>, pub provider_semantics: BTreeMap<String, String>, pub issued_at_epoch_ms: i64, pub expires_at_epoch_ms: i64 } |  |  |  |  |

| `GlobalStanding` | struct | GlobalStanding { pub standing: ReleaseStanding, pub cells: Vec<CellStandingRow> } |  |  |  |  |

| `ProtocolSurface` | struct | ProtocolSurface { pub surface: InterfaceOrigin, pub endpoint: String, pub modes: BTreeSet<IntentMode> } |  |  |  |  |

| `ECOSYSTEM_EPOCH` | const | ECOSYSTEM_EPOCH: &str |  |  |  |  |

| `MAX_EXTERNAL_WITNESS_BYTES` | const | MAX_EXTERNAL_WITNESS_BYTES: u64 |  |  |  |  |

| `MAX_EXTERNAL_WITNESS_DEADLINE_MS` | const | MAX_EXTERNAL_WITNESS_DEADLINE_MS: u64 |  |  |  |  |

| `MAX_EXTERNAL_WITNESS_STEPS` | const | MAX_EXTERNAL_WITNESS_STEPS: u64 |  |  |  |  |

| `SA2A_REPLAN_CONTRACT_DIGEST` | const | SA2A_REPLAN_CONTRACT_DIGEST: &str |  |  |  |  |

| `SA2A_REPLAN_SCHEMA_ID` | const | SA2A_REPLAN_SCHEMA_ID: &str |  |  |  |  |

| `EvidenceStanding` | enum | EvidenceStanding { Alive, Refused(String) } |  |  |  |  |

| `WitnessKind` | enum | WitnessKind { Semantic, Receipt, Process, Planner, Recovery, Federation, ModelCompute } |  |  |  |  |

| `admit_external_witness` | function | admit_external_witness( manifest: &EcosystemManifest, witness: &ExternalWitness, ) -> EvidenceStanding |  |  |  |  |

| `admit_fond_differential` | function | admit_fond_differential(check: &FondDifferentialCheck) -> EvidenceStanding |  |  |  |  |

| `admit_independent_plan` | function | admit_independent_plan(check: &IndependentPlanCheck) -> EvidenceStanding |  |  |  |  |

| `admit_sa2a_replan_envelope` | function | admit_sa2a_replan_envelope( expected_subject: &str, envelope: &PortableReplanEnvelope, ) -> EvidenceStanding |  |  |  |  |

| `bind_v26_9_28_construct_request` | function | bind_v26_9_28_construct_request( mut request: crate::castle::ConstructRequest, manifest: &EcosystemManifest, witnesses: &[ExternalWitness], ) -> Result<crate::castle::ConstructRequest, String> |  |  |  |  |

| `ecosystem_manifest` | function | ecosystem_manifest() -> Result<EcosystemManifest, String> |  |  |  |  |

| `id` | function | id(self) -> &'static str |  |  |  |  |

| `is_alive` | function | is_alive(&self) -> bool |  |  |  |  |

| `qualify_portable_runtime` | function | qualify_portable_runtime(witnesses: &[PortableRuntimeWitness]) -> EvidenceStanding |  |  |  |  |

| `qualify_v26_9_28_upgrade` | function | qualify_v26_9_28_upgrade(manifest: &EcosystemManifest) -> UpgradeQualification |  |  |  |  |

| `route_edge_local_recovery` | function | route_edge_local_recovery( subject: &str, ordered_providers: &[String], failed_providers: &BTreeSet<String>, ) -> Result<RecoveryDecision, String> |  |  |  |  |

| `executed` | str_key | CONSEQUENCES = "executed" |  |  |  |  |

| `ggen-marketplace-v26.9.29` | str_key | REQUIRED = "ggen-marketplace-v26.9.29" |  |  |  |  |

| `sa2a/replan-envelope/v1` | str_key | SA2A_REPLAN_SCHEMA_ID = "sa2a/replan-envelope/v1" |  |  |  |  |

| `sha256:ff7643034ed101930e9c80df716df863b6ee6d14f3b29aff764209ad11dab80e` | str_key | SA2A_REPLAN_CONTRACT_DIGEST = "sha256:ff7643034ed101930e9c80df716df863b6ee6d14f3b29aff764209ad11dab80e" |  |  |  |  |

| `stop` | str_key | DECISIONS = "stop" |  |  |  |  |

| `v26.9.28` | str_key | ECOSYSTEM_EPOCH = "v26.9.28" |  |  |  |  |

| `EcosystemManifest` | struct | EcosystemManifest { pub kind: String, pub release_epoch: String, pub reviewed_local_date: String, pub timezone: String, pub castle_base_sha: String, pub review_window: ReviewWindow, pub marketplace_pack: MarketplacePack, pub subjects: Vec<SourceSubject>, pub excluded_open_prs: Vec<ExcludedPr> } |  |  |  |  |

| `ExcludedPr` | struct | ExcludedPr { pub repo: String, pub pr: u64, pub reason: String } |  |  |  |  |

| `ExternalWitness` | struct | ExternalWitness { pub source_id: String, pub source_sha: String, pub subject: String, pub kind: WitnessKind, pub input_digest: String, pub output_digest: String, pub direct_do_authority: bool, pub limits: WitnessLimits } |  |  |  |  |

| `FondDifferentialCheck` | struct | FondDifferentialCheck { pub policy_digest: String, pub model_digest: String, pub agreed: bool, pub counterexample: Option<String> } |  |  |  |  |

| `IndependentPlanCheck` | struct | IndependentPlanCheck { pub plan_digest: String, pub valid_under_independent_dynamics: bool, pub claimed_cost: i64, pub independently_verified_cost: i64 } |  |  |  |  |

| `MarketplacePack` | struct | MarketplacePack { pub repo: String, pub r pub commit_sha: String, pub path: String, pub ontology_sha: String, pub security_universe_sha: String, pub security_tools_sha: String } |  |  |  |  |

| `PortableReplanDecision` | struct | PortableReplanDecision { pub kind: String, pub reason: String, pub authority: String } |  |  |  |  |

| `PortableReplanEnvelope` | struct | PortableReplanEnvelope { pub schema: String, pub contract_digest: String, pub exact_subject: Value, pub receipt_id: String, pub consequence: String, pub decision: PortableReplanDecision, pub provider: Option<String>, pub projection_digest: Option<String>, pub source_replay_key: Option<String> } |  |  |  |  |

| `PortableRuntimeWitness` | struct | PortableRuntimeWitness { pub engine_family: String, pub module_digest: String, pub input_digest: String, pub output_digest: String } |  |  |  |  |

| `RecoveryDecision` | struct | RecoveryDecision { pub subject: String, pub selected_provider: String, pub excluded_failed_providers: Vec<String> } |  |  |  |  |

| `ReviewWindow` | struct | ReviewWindow { pub timezone: String, pub local_start: String, pub task_cutoff: String, pub utc_start: String, pub utc_cutoff: String, pub pr_count: u64, pub merged_pr_count: u64, pub open_pr_count: u64, pub closed_unmerged_pr_count: u64 } |  |  |  |  |

| `SourceSubject` | struct | SourceSubject { pub id: String, pub repo: String, pub source_kind: String, pub reference: String, pub sha: String, pub pr: Option<u64>, pub role: String, pub authority_ceiling: String, pub allowed_witness_kinds: Vec<String> } |  |  |  |  |

| `UpgradeQualification` | struct | UpgradeQualification { pub standing: EvidenceStanding, pub missing_subjects: Vec<String> } |  |  |  |  |

| `WitnessLimits` | struct | WitnessLimits { pub max_steps: u64, pub max_bytes: u64, pub deadline_ms: u64 } |  |  |  |  |

| `castle:board-test` | str_key | SUBJECT = "castle:board-test" |  |  |  |  |

| `../docs/sjira/v26.9.28/capability-intake.ttl` | str_key | GRAPH = "../docs/sjira/v26.9.28/capability-intake.ttl" |  |  |  |  |

| `DOCKER_BIN` | env_key | std::env::var("DOCKER_BIN") |  |  |  |  |

| `GYMACT_BIN` | env_key | std::env::var("GYMACT_BIN") |  |  |  |  |

| `HOME` | env_key | std::env::var("HOME") |  |  |  |  |

| `6363636363636363636363636363636363636363636363636363636363636363` | str_key | RECEIPT_SEED_HEX = "6363636363636363636363636363636363636363636363636363636363636363" |  |  |  |  |

| `acct:supplier-9821` | str_key | PAYEE = "acct:supplier-9821" |  |  |  |  |

| `acct:treasury` | str_key | PAYER = "acct:treasury" |  |  |  |  |

| `actuator:payments` | str_key | AUDIENCE = "actuator:payments" |  |  |  |  |

| `principal:procurement-agent` | str_key | PRINCIPAL = "principal:procurement-agent" |  |  |  |  |

| `6363636363636363636363636363636363636363636363636363636363636363` | str_key | RECEIPT_SEED_HEX = "6363636363636363636363636363636363636363636363636363636363636363" |  |  |  |  |

| `acct:supplier-9821` | str_key | PAYEE = "acct:supplier-9821" |  |  |  |  |

| `acct:treasury` | str_key | PAYER = "acct:treasury" |  |  |  |  |

| `actuator:payments` | str_key | AUDIENCE = "actuator:payments" |  |  |  |  |

| `principal:procurement-agent` | str_key | PRINCIPAL = "principal:procurement-agent" |  |  |  |  |

| `AUDIENCE` | const | AUDIENCE: &str |  |  |  |  |

| `NOW_MS` | const | NOW_MS: u64 |  |  |  |  |

| `PAYEE` | const | PAYEE: &str |  |  |  |  |

| `PAYER` | const | PAYER: &str |  |  |  |  |

| `PRINCIPAL` | const | PRINCIPAL: &str |  |  |  |  |

| `admission_ctx` | function | admission_ctx(&self) -> AdmissionContext<'_> |  |  |  |  |

| `admit` | function | admit(&self, effect: PreparedEffect, nonce: &str) -> Result<PaymentAdmission, String> |  |  |  |  |

| `cert` | function | cert(&self, effect: &PreparedEffect, nonce: &str, signers: &[&str]) -> ActuationCertificate |  |  |  |  |

| `effect` | function | effect(&self, amount: &str, obligation: &str) -> PreparedEffect |  |  |  |  |

| `exec_ctx` | function | exec_ctx(&self) -> ExecutionContext<'_> |  |  |  |  |

| `new` | function | new(tag: &str) -> Self |  |  |  |  |

| `token_for` | function | token_for(&self, admission: &PaymentAdmission) -> ActuationToken |  |  |  |  |

| `unique_dir` | function | unique_dir(tag: &str) -> PathBuf |  |  |  |  |

| `with` | function | with(tag: &str, treasury_minor: u64, per_effect_cap: u64, epoch_cap: u64) -> Self |  |  |  |  |

| `acct:supplier-9821` | str_key | PAYEE = "acct:supplier-9821" |  |  |  |  |

| `acct:treasury` | str_key | PAYER = "acct:treasury" |  |  |  |  |

| `actuator:payments` | str_key | AUDIENCE = "actuator:payments" |  |  |  |  |

| `principal:procurement-agent` | str_key | PRINCIPAL = "principal:procurement-agent" |  |  |  |  |

| `Fixture` | struct | Fixture { pub dir: PathBuf, pub registry: KeyRegistry, pub epochs: SecurityEpochs, pub policy: SpendPolicy, pub claims: ClaimStore, pub nonces: DurableNonceFence, pub ledger: FileJournalLedger, pub signer: ReceiptKey, pub verifier: ReceiptCheck, custodians: Vec<(String, SigningKey)> } |  |  |  |  |

| `RealBlake3` | struct |  |  |  |  |  |

| `ReceiptCheck` | struct | ReceiptCheck { id: String, key: VerifyingKey } |  |  |  |  |

| `ReceiptKey` | struct | ReceiptKey { id: String, key: SigningKey } |  |  |  |  |

| `authoritative` | function | authoritative(mut self) -> Self |  |  |  |  |

| `new` | function | new(script: Vec<RailStatus>) -> Self |  |  |  |  |

| `push` | function | push(&self, s: RailStatus) |  |  |  |  |

| `rejected` | function | rejected() -> RailStatus |  |  |  |  |

| `returned` | function | returned() -> RailStatus |  |  |  |  |

| `settled` | function | settled() -> RailStatus |  |  |  |  |

| `ScriptedRail` | struct | ScriptedRail { script: Mutex<VecDeque<RailStatus>>, last: Mutex<RailStatus>, authoritative: bool } |  |  |  |  |

| `castle:enterprise:test` | str_key | SUBJECT = "castle:enterprise:test" |  |  |  |  |

| `5493001KJTIIGC8Y1R12` | str_key | REAL_LEI = "5493001KJTIIGC8Y1R12" |  |  |  |  |

| `CASTLE_FIBO_CORPUS` | env_key | std::env::var("CASTLE_FIBO_CORPUS") |  |  |  |  |

| `2026-09-29T12:34:56Z` | str_key | TS = "2026-09-29T12:34:56Z" |  |  |  |  |

| `admit_submitted` | function | admit_submitted(fx: &Fixture, amount: &str, obligation: &str, nonce: &str) -> PaymentAdmission |  |  |  |  |

| `CASTLE_WRITE_GOLDEN` | env_key | std::env::var("CASTLE_WRITE_GOLDEN") |  |  |  |  |

| `2026-09-29T12:34:56Z` | str_key | TS = "2026-09-29T12:34:56Z" |  |  |  |  |

| `CASTUS33` | str_key | DAGT = "CASTUS33" |  |  |  |  |

| `SUPPGB2LXXX` | str_key | CAGT = "SUPPGB2LXXX" |  |  |  |  |

| `acct:supplier-2` | str_key | PAYEE2 = "acct:supplier-2" |  |  |  |  |

| `CASTLE_ISO20022_XSD_DIR` | env_key | std::env::var("CASTLE_ISO20022_XSD_DIR") |  |  |  |  |

| `/private/tmp/claude-501/-Users-sac-castle/62dd6f54-50cc-4146-9377-c4ca3cfee3bd/scratchpad/iso/xsd` | str_key | DEFAULT_DIR = "/private/tmp/claude-501/-Users-sac-castle/62dd6f54-50cc-4146-9377-c4ca3cfee3bd/scratchpad/iso/xsd" |  |  |  |  |

| `2026-09-29T12:34:56Z` | str_key | TS = "2026-09-29T12:34:56Z" |  |  |  |  |

| `REFUSED:PAYMENT_AMOUNT_NOT_INTEGER` | str_key | NOT_INT = "REFUSED:PAYMENT_AMOUNT_NOT_INTEGER" |  |  |  |  |

| `REFUSED:PAYMENT_AMOUNT_ZERO` | str_key | ZERO = "REFUSED:PAYMENT_AMOUNT_ZERO" |  |  |  |  |

| `sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef` | str_key | EFF = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef" |  |  |  |  |

| `p0` | str_key | ALPHABET = "p0" |  |  |  |  |

| `../docs/sjira/v26.9.28/courts/exclusions.rq` | str_key | EXCLUSIONS = "../docs/sjira/v26.9.28/courts/exclusions.rq" |  |  |  |  |

| `../docs/sjira/v26.9.28/courts/ownership.rq` | str_key | OWNERSHIP = "../docs/sjira/v26.9.28/courts/ownership.rq" |  |  |  |  |

| `../docs/sjira/v26.9.28/courts/runtime_crown.rq` | str_key | RUNTIME_CROWN = "../docs/sjira/v26.9.28/courts/runtime_crown.rq" |  |  |  |  |

| `../docs/sjira/v26.9.28/courts/stop.rq` | str_key | STOP = "../docs/sjira/v26.9.28/courts/stop.rq" |  |  |  |  |

| `../docs/sjira/v26.9.28/goal.ttl` | str_key | GOAL = "../docs/sjira/v26.9.28/goal.ttl" |  |  |  |  |

| `../docs/sjira/v26.9.28/repos.ttl` | str_key | REPOS = "../docs/sjira/v26.9.28/repos.ttl" |  |  |  |  |


<!-- ============================================================= -->
<!-- AGENT-FORBIDDEN-END: nothing below this line may describe     -->
<!-- code behavior.                                                -->
<!-- ============================================================= -->
