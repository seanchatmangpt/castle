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

| test | script | node --experimental-strip-types --test test/*.test.ts |  |  |  |  |


### CastlePaaS

| admission_provider | function | admission_provider/0 |  |  |  |  |

| kernel | function | kernel/0 |  |  |  |  |

| receipt_verifier | function | receipt_verifier/0 |  |  |  |  |

| semantic_bundle | function | semantic_bundle/0 |  |  |  |  |


### CastlePaaS.Admission


### CastlePaaS.AdmissionProvider


### CastlePaaS.AdmissionProvider.Refuse

| admit | function | admit/3 |  |  |  |  |


### CastlePaaS.AdmissionWitness

| atom_key | function | atom_key/1 |  |  |  |  |

| atom_key | function | atom_key/1 |  |  |  |  |

| atom_key | function | atom_key/1 |  |  |  |  |

| atom_key | function | atom_key/1 |  |  |  |  |

| atom_key | function | atom_key/1 |  |  |  |  |

| atom_key | function | atom_key/1 |  |  |  |  |

| atom_key | function | atom_key/1 |  |  |  |  |

| atom_key | function | atom_key/1 |  |  |  |  |

| digest? | function | digest?/1 |  |  |  |  |

| digest? | function | digest?/1 |  |  |  |  |

| external_id | function | external_id/1 |  |  |  |  |

| external_id | function | external_id/1 |  |  |  |  |

| external_id | function | external_id/1 |  |  |  |  |

| external_id | function | external_id/1 |  |  |  |  |

| external_id | function | external_id/1 |  |  |  |  |

| external_id | function | external_id/1 |  |  |  |  |

| get | function | get/2 |  |  |  |  |

| stringify | function | stringify/1 |  |  |  |  |

| stringify | function | stringify/1 |  |  |  |  |

| stringify | function | stringify/1 |  |  |  |  |

| verify | function | verify/3 |  |  |  |  |

| verify | function | verify/3 |  |  |  |  |


### CastlePaaS.Application

| start | function | start/2 |  |  |  |  |


### CastlePaaS.Canonical

| encode | function | encode/1 |  |  |  |  |

| encode | function | encode/1 |  |  |  |  |

| encode | function | encode/1 |  |  |  |  |

| encode | function | encode/1 |  |  |  |  |

| encode | function | encode/1 |  |  |  |  |

| sha256 | function | sha256/1 |  |  |  |  |


### CastlePaaS.Capability


### CastlePaaS.Domain


### CastlePaaS.Evidence


### CastlePaaS.ExecutionIntent


### CastlePaaS.Generated.Resource

| CastlePaaS.Generated.Resource | ash_resource |  |  |  |  |  |


### CastlePaaS.Kernel


### CastlePaaS.Kernel.CLI

| build_request | function | build_request/1 |  |  |  |  |

| digest? | function | digest?/1 |  |  |  |  |

| digest? | function | digest?/1 |  |  |  |  |

| execute | function | execute/3 |  |  |  |  |

| manufacture | function | manufacture/1 |  |  |  |  |

| release_info | function | release_info/0 |  |  |  |  |

| require_alive_construct | function | require_alive_construct/1 |  |  |  |  |

| require_alive_construct | function | require_alive_construct/1 |  |  |  |  |

| require_castle_identity | function | require_castle_identity/1 |  |  |  |  |

| require_castle_identity | function | require_castle_identity/1 |  |  |  |  |

| require_receipted_do | function | require_receipted_do/0 |  |  |  |  |

| require_receipted_do | function | require_receipted_do/1 |  |  |  |  |

| required_map | function | required_map/2 |  |  |  |  |

| required_string | function | required_string/2 |  |  |  |  |

| run | function | run/2 |  |  |  |  |

| runtime | function | runtime/0 |  |  |  |  |

| stringify | function | stringify/1 |  |  |  |  |

| stringify | function | stringify/1 |  |  |  |  |

| stringify | function | stringify/1 |  |  |  |  |

| write_request | function | write_request/1 |  |  |  |  |


### CastlePaaS.Observation


### CastlePaaS.Organization


### CastlePaaS.Persistence

| record | function | record/3 |  |  |  |  |


### CastlePaaS.Plan


### CastlePaaS.PlatformService


### CastlePaaS.Reactors.AdmitSubject


### CastlePaaS.Reactors.ConstructIntent


### CastlePaaS.Reactors.ExecuteIntent


### CastlePaaS.Reactors.PublishProjection


### CastlePaaS.Reactors.QualifyEvidence


### CastlePaaS.Reactors.RegisterSubject


### CastlePaaS.Reactors.ReplayReceipt


### CastlePaaS.Receipt


### CastlePaaS.ReceiptVerifier


### CastlePaaS.ReceiptVerifier.Refuse

| verify | function | verify/1 |  |  |  |  |


### CastlePaaS.Replay


### CastlePaaS.Repo

| installed_extensions | function | installed_extensions/0 |  |  |  |  |

| min_pg_version | function | min_pg_version/0 |  |  |  |  |


### CastlePaaS.Standing

| parse | function | parse/1 |  |  |  |  |

| parse | function | parse/1 |  |  |  |  |

| parse | function | parse/1 |  |  |  |  |


### CastlePaaS.Subject


### ontology/payments-fibo/generated/fibo_generated.rs

| FiboMapping | struct |  |  |  |  |  |


### src/bin/castle/verbs/dfcm_handlers.rs

| crypto_capabilities_handler | function | crypto_capabilities_handler() |  |  |  |  |

| deployment_bind_policy_handler | function | deployment_bind_policy_handler(
    binding_path: String,
    policy_path: String,
) |  |  |  |  |

| dfcm_qualify_handler | function | dfcm_qualify_handler(
    manifest_path: String,
    evidence_path: String,
    now_epoch_ms: i64,
    max_evidence_age_ms: i64,
) |  |  |  |  |

| dfcm_verify_handler | function | dfcm_verify_handler() |  |  |  |  |

| live_check_plan_handler | function | live_check_plan_handler(manifest_path: String) |  |  |  |  |

| live_check_qualify_handler | function | live_check_qualify_handler(
    manifest_path: String,
    evidence_path: String,
    now_epoch_ms: i64,
    max_evidence_age_ms: i64,
) |  |  |  |  |

| live_check_run_handler | function | live_check_run_handler(spec_path: String, observed_at_epoch_ms: i64) |  |  |  |  |

| protocol_dispatch_handler | function | protocol_dispatch_handler(intent_path: String) |  |  |  |  |

| replication_admit_handler | function | replication_admit_handler(
    state_path: String,
    checkpoint_path: String,
    receiver_id: String,
) |  |  |  |  |


### src/bin/castle/verbs/evidence_handlers.rs

| evidence_verify_handler | function | evidence_verify_handler(evidence_path: String) |  |  |  |  |


### src/bin/castle/verbs/handlers.rs

| chaos_qualify_handler | function | chaos_qualify_handler(evidence_path: String) |  |  |  |  |

| construct_manufacture_handler | function | construct_manufacture_handler(request_path: String, signing_key_path: String, key_id: String) |  |  |  |  |

| crypto_capabilities_handler | function | crypto_capabilities_handler() |  |  |  |  |

| deployment_adapters_handler | function | deployment_adapters_handler() |  |  |  |  |

| deployment_qualify_handler | function | deployment_qualify_handler(manifest_path: String, now_epoch_ms: i64) |  |  |  |  |

| do_execute_handler | function | do_execute_handler(request_path: String, signing_key_path: String, key_id: String, expected_construct_digest: String, now_epoch_ms: i64) |  |  |  |  |

| fortune5_qualify_handler | function | fortune5_qualify_handler(subject: String, evidence_path: String, now_epoch_ms: Option<i64>, max_evidence_age_ms: Option<i64>) |  |  |  |  |

| fortune5_requirements_handler | function | fortune5_requirements_handler() |  |  |  |  |

| impact_coverage_handler | function | impact_coverage_handler(classes_path: String, target_coverage_bps: Option<i64>) |  |  |  |  |

| inventory_components_handler | function | inventory_components_handler() |  |  |  |  |

| inventory_goals_handler | function | inventory_goals_handler() |  |  |  |  |

| protocol_a2a_handler | function | protocol_a2a_handler() |  |  |  |  |

| protocol_mcp_handler | function | protocol_mcp_handler() |  |  |  |  |

| release_info_handler | function | release_info_handler() |  |  |  |  |

| replay_admit_handler | function | replay_admit_handler(replay_class_id: String, structural_signature: String, ontology_version: String, provider_semantics_version: String, invariant_set_digest: String, process_digest: String, invariants_hold: bool) |  |  |  |  |


### src/bin/castle/verbs/payments_handlers.rs

| execute_handler | function | execute_handler(a: ExecuteArgs) |  |  |  |  |

| explain_handler | function | explain_handler(effect_digest: String, state_dir: String, ledger_dir: String, reference_ledger: bool) |  |  |  |  |

| finalize_handler | function | finalize_handler(
    effect_digest: String,
    state_dir: String,
    ledger_dir: String,
    rail_dir: String,
    rail_mode: Option<String>,
    reference_ledger: bool,
    reference_rail: bool,
) |  |  |  |  |

| prepare_handler | function | prepare_handler(
    principal: String,
    payer: String,
    payee: String,
    amount_minor: String,
    currency: String,
    obligation_id: String,
    purpose: String,
    reverses: Option<String>,
) |  |  |  |  |

| rail_submit_handler | function | rail_submit_handler(a: RailSubmitArgs) |  |  |  |  |

| receipt_handler | function | receipt_handler(
    effect_digest: String,
    state_dir: String,
    ledger_dir: String,
    rail_dir: String,
    journal_dir: String,
    reference_ledger: bool,
    reference_rail: bool,
) |  |  |  |  |

| reconcile_handler | function | reconcile_handler(effect_digest: String, state_dir: String, ledger_dir: String, reference_ledger: bool) |  |  |  |  |

| recover_handler | function | recover_handler(state_dir: String) |  |  |  |  |

| replay_handler | function | replay_handler(journal_dir: String, effect_digest: String) |  |  |  |  |

| ExecuteArgs | struct |  |  |  |  |  |

| RailSubmitArgs | struct |  |  |  |  |  |


### src/blake3.rs

| assert_blake3_self_test | function | assert_blake3_self_test() |  |  |  |  |

| blake3_hex_utf8 | function | blake3_hex_utf8(input: &str) |  |  |  |  |


### src/board.rs

| FailureMode | enum |  |  |  |  |  |

| MaterialityDimension | enum |  |  |  |  |  |

| admit_evidence | function | admit_evidence(bundle: &EvidenceBundle, trust: &TrustStore, store: &HashMap<String, ReceiptV2>) |  |  |  |  |

| admit_failure_semantics | function | admit_failure_semantics(input: &FailureSemanticsInput) |  |  |  |  |

| assess_materiality | function | assess_materiality(event: &MaterialityEvent, policy: &MaterialityPolicy) |  |  |  |  |

| board_requirements | function | board_requirements() |  |  |  |  |

| build_board_package | function | build_board_package(admission: &BoardAdmission, generated_at: &str, material_refused_subjects: u32, risk_appetite_breaches: u32) |  |  |  |  |

| classify_icfr_subject | function | classify_icfr_subject(input: &IcfrSubject) |  |  |  |  |

| decode | function | decode(input: &str) |  |  |  |  |

| detect_segregation_of_duty_violations | function | detect_segregation_of_duty_violations(assignments: &[RoleAssignment], incompatible_role_pairs: &[(String, String) |  |  |  |  |

| encode | function | encode(input: &[u8]) |  |  |  |  |

| issue_evidence_receipt | function | issue_evidence_receipt(input: &EvidenceInput, context: &ReceiptIssueContext, signing_key: &SigningKey) |  |  |  |  |

| qualify_fortune5_board | function | qualify_fortune5_board(input: BoardAdmissionInput) |  |  |  |  |

| qualify_verified_fortune5 | function | qualify_verified_fortune5(
    bundles: &[EvidenceBundle],
    context: &QualificationContext,
    trust: &TrustStore,
    receipt_store: &HashMap<String, ReceiptV2>,
    requirements: &[Fortune5Requirement],
) |  |  |  |  |

| verify_receipt_dag | function | verify_receipt_dag(root: &ReceiptV2, trust: &TrustStore, store: &HashMap<String, ReceiptV2>) |  |  |  |  |

| BoardAdmission | struct |  |  |  |  |  |

| BoardAdmissionInput | struct |  |  |  |  |  |

| BoardPackage | struct |  |  |  |  |  |

| EvidenceAdmission | struct |  |  |  |  |  |

| EvidenceBundle | struct |  |  |  |  |  |

| EvidenceInput | struct |  |  |  |  |  |

| FailureSemanticsDecision | struct |  |  |  |  |  |

| FailureSemanticsInput | struct |  |  |  |  |  |

| IcfrClassification | struct |  |  |  |  |  |

| IcfrSubject | struct |  |  |  |  |  |

| MaterialityAssessment | struct |  |  |  |  |  |

| MaterialityEvent | struct |  |  |  |  |  |

| MaterialityPolicy | struct |  |  |  |  |  |

| ReceiptCoreV2 | struct |  |  |  |  |  |

| ReceiptIssueContext | struct |  |  |  |  |  |

| ReceiptV2 | struct |  |  |  |  |  |

| ReceiptVerification | struct |  |  |  |  |  |

| RoleAssignment | struct |  |  |  |  |  |

| SodViolation | struct |  |  |  |  |  |

| TrustKey | struct |  |  |  |  |  |

| TrustStore | struct |  |  |  |  |  |

| VerifiedQualification | struct |  |  |  |  |  |


### src/capability_intake.rs

| do_authority | function | do_authority(_repository: &str) |  |  |  |  |

| donor | function | donor(repository: &str) |  |  |  |  |

| CapabilityDonor | struct |  |  |  |  |  |


### src/castle.rs

| EpistemicClass | enum |  |  |  |  |  |

| GymActStatus | enum |  |  |  |  |  |

| admit_construct_for_do | function | admit_construct_for_do(
    capability: &ConstructCapability,
    process: &PowlProcess,
    envelope: &TestEnvelope,
    blake3: &dyn Blake3Provider,
    verifier: &dyn ReceiptVerifier,
    policy: &ConstructTrustPolicy,
    now: impl Fn() |  |  |  |  |

| apply_zero_day_observation | function | apply_zero_day_observation(graph: &DependencyGraph, observation: ZeroDayObservation) |  |  |  |  |

| as_str | function | as_str(&self) |  |  |  |  |

| compile_adversarial_classes | function | compile_adversarial_classes(goals: &[AdversarialGoal], rules: &[TransitionRule], planners: &[Box<dyn Planner>]) |  |  |  |  |

| compile_witness_to_powl | function | compile_witness_to_powl(id: &str, vulnerability: &VulnerabilityCondition, rules: &[TransitionRule]) |  |  |  |  |

| construct_compromise | function | construct_compromise(&self, dependency_id: &str, capability: &str) |  |  |  |  |

| create_receipt | function | create_receipt(
    artifact: &Value,
    epistemic_class: EpistemicClass,
    subject: &str,
    parent_digests: &[String],
    blake3: &dyn Blake3Provider,
    signer: &dyn ReceiptSigner,
) |  |  |  |  |

| derive_vulnerabilities | function | derive_vulnerabilities(goal: &AdversarialGoal, rules: &[TransitionRule], max_depth: u32) |  |  |  |  |

| enabled_activities | function | enabled_activities(process: &PowlProcess, completed: &BTreeSet<String>) |  |  |  |  |

| execute_powl_with_gym_act | function | execute_powl_with_gym_act(
    process: &PowlProcess,
    state: &WorldState,
    envelope: &TestEnvelope,
    gymact: &dyn GymActAdapter,
    authorization: DoAuthorizationContext<'_>,
) |  |  |  |  |

| local_docker_default | function | local_docker_default(docker_bin: impl Into<String>) |  |  |  |  |

| manufacture_construct_capability | function | manufacture_construct_capability(request: ConstructRequest, blake3: &dyn Blake3Provider, signer: &dyn ReceiptSigner) |  |  |  |  |

| new | function | new(nodes: Vec<DependencyNode>, edges: Vec<DependencyEdge>) |  |  |  |  |

| platform_eng_colima_default | function | platform_eng_colima_default() |  |  |  |  |

| run_planner_ensemble | function | run_planner_ensemble(problem: &PlanningProblem<'_>, planners: &[Box<dyn Planner>]) |  |  |  |  |

| verify_receipt | function | verify_receipt(artifact: &Value, receipt: &Receipt, blake3: &dyn Blake3Provider, verifier: &dyn ReceiptVerifier, trusted_origin_key_ids: &BTreeSet<String>) |  |  |  |  |

| ActuationPermit | struct |  |  |  |  |  |

| AdmissionBrand | struct |  |  |  |  |  |

| AdversarialGoal | struct |  |  |  |  |  |

| AutofdeLabPlanner | struct |  |  |  |  |  |

| CompiledAdversarialClass | struct |  |  |  |  |  |

| ConstructAdmission | struct |  |  |  |  |  |

| ConstructArtifact | struct |  |  |  |  |  |

| ConstructCapability | struct |  |  |  |  |  |

| ConstructRequest | struct |  |  |  |  |  |

| ConstructSourceReceipts | struct |  |  |  |  |  |

| ConstructSources | struct |  |  |  |  |  |

| ConstructTrustPolicy | struct |  |  |  |  |  |

| ConstructedCompromise | struct |  |  |  |  |  |

| ContainerGymActAdapter | struct |  |  |  |  |  |

| CostMinimizingPlanner | struct |  |  |  |  |  |

| DependencyEdge | struct |  |  |  |  |  |

| DependencyGraph | struct |  |  |  |  |  |

| DependencyNode | struct |  |  |  |  |  |

| DoAuthorizationContext | struct |  |  |  |  |  |

| GymActResult | struct |  |  |  |  |  |

| KindClusterReadOnlyGymAct | struct |  |  |  |  |  |

| OcelEvent | struct |  |  |  |  |  |

| OcelLog | struct |  |  |  |  |  |

| OcelObject | struct |  |  |  |  |  |

| PlanCandidate | struct |  |  |  |  |  |

| PlanningProblem | struct |  |  |  |  |  |

| PowlActivity | struct |  |  |  |  |  |

| PowlProcess | struct |  |  |  |  |  |

| ProcessGymActAdapter | struct |  |  |  |  |  |

| Receipt | struct |  |  |  |  |  |

| ReceiptedOcelLog | struct |  |  |  |  |  |

| TestEnvelope | struct |  |  |  |  |  |

| TransitionRule | struct |  |  |  |  |  |

| VulnerabilityCondition | struct |  |  |  |  |  |

| WitnessPlanner | struct |  |  |  |  |  |

| WorldState | struct |  |  |  |  |  |

| ZeroDayImpact | struct |  |  |  |  |  |

| ZeroDayObservation | struct |  |  |  |  |  |

| Blake3Provider | trait |  |  |  |  |  |

| GymActAdapter | trait |  |  |  |  |  |

| Planner | trait |  |  |  |  |  |

| ReceiptSigner | trait |  |  |  |  |  |

| ReceiptVerifier | trait |  |  |  |  |  |


### src/dd_ui.rs

| DdUiRefusal | enum |  |  |  |  |  |

| admit | function | admit(&self) |  |  |  |  |

| PresentationAuthority | struct |  |  |  |  |  |


### src/fortune5.rs

| EvidenceEpistemicClass | enum |  |  |  |  |  |

| MetricValue | enum |  |  |  |  |  |

| Standing | enum |  |  |  |  |  |

| admit_replay | function | admit_replay(manifest: &ReplayManifest, subject: &ReplaySubject) |  |  |  |  |

| as_str | function | as_str(&self) |  |  |  |  |

| minimum_impact_coverage | function | minimum_impact_coverage(classes: &[AdversarialImpactClass], target_coverage_bps: i64) |  |  |  |  |

| qualify_fortune5 | function | qualify_fortune5(
    observations: &[MetricObservation],
    context: &QualificationContext,
    requirements: &[Fortune5Requirement],
) |  |  |  |  |

| qualify_fortune5_default | function | qualify_fortune5_default(observations: &[MetricObservation], context: &QualificationContext) |  |  |  |  |

| AdversarialImpactClass | struct |  |  |  |  |  |

| ControlEvaluation | struct |  |  |  |  |  |

| Fortune5Qualification | struct |  |  |  |  |  |

| ImpactCoverageSelection | struct |  |  |  |  |  |

| MetricObservation | struct |  |  |  |  |  |

| QualificationContext | struct |  |  |  |  |  |

| ReplayAdmission | struct |  |  |  |  |  |

| ReplayManifest | struct |  |  |  |  |  |

| ReplaySubject | struct |  |  |  |  |  |


### src/fortune5_generated.rs

| Fortune5Requirement | struct |  |  |  |  |  |


### src/generated.rs

| default_adversarial_goals | function | default_adversarial_goals() |  |  |  |  |

| generated_components | function | generated_components() |  |  |  |  |

| DefaultAdversarialGoal | struct |  |  |  |  |  |

| GeneratedBinding | struct |  |  |  |  |  |


### src/gymact_container.rs

| execute_default_container_observation | function | execute_default_container_observation(
    process: &PowlProcess,
    state: &WorldState,
    envelope: &TestEnvelope,
    authorization: DoAuthorizationContext<'_>,
    docker_bin: impl Into<String>,
) |  |  |  |  |


### src/operation_envelope.rs

| EffectClass | enum |  |  |  |  |  |

| as_str | function | as_str(self) |  |  |  |  |

| digest | function | digest(&self) |  |  |  |  |

| to_json | function | to_json(&self) |  |  |  |  |

| OperationEnvelope | struct |  |  |  |  |  |


### src/payments/adapter.rs

| StepOutcome | enum |  |  |  |  |  |

| outcomes | function | outcomes(&self) |  |  |  |  |

| PaymentGymActAdapter | struct |  |  |  |  |  |


### src/payments/admission.rs

| admit_payment | function | admit_payment(
    prepared: PreparedEffect,
    certificate: &ActuationCertificate,
    ctx: &AdmissionContext<'_>,
) |  |  |  |  |

| admit_payment_screened | function | admit_payment_screened(
    prepared: PreparedEffect,
    certificate: &ActuationCertificate,
    ctx: &AdmissionContext<'_>,
    screening: &Screening<'_>,
) |  |  |  |  |

| effect | function | effect(&self) |  |  |  |  |

| generation | function | generation(&self) |  |  |  |  |

| nonce | function | nonce(&self) |  |  |  |  |

| screening | function | screening(&self) |  |  |  |  |

| verification | function | verification(&self) |  |  |  |  |

| AdmissionContext | struct |  |  |  |  |  |

| PaymentAdmission | struct |  |  |  |  |  |

| Screening | struct |  |  |  |  |  |

| ScreeningEvidence | struct |  |  |  |  |  |


### src/payments/claim_store.rs

| ClaimState | enum |  |  |  |  |  |

| get | function | get(&self, digest: &str) |  |  |  |  |

| list | function | list(&self) |  |  |  |  |

| open | function | open(root: impl Into<PathBuf>) |  |  |  |  |

| reserve | function | reserve(&self, claim: &Claim, epoch_cap: Option<u64>) |  |  |  |  |

| reversed_total | function | reversed_total(&self, original: &str) |  |  |  |  |

| transition | function | transition(
        &self,
        digest: &str,
        from: &[ClaimState],
        to: ClaimState,
        construct_digest: Option<&str>,
        detail: &str,
    ) |  |  |  |  |

| Claim | struct |  |  |  |  |  |

| ClaimStore | struct |  |  |  |  |  |


### src/payments/compliance.rs

| from_json | function | from_json(bytes: &[u8]) |  |  |  |  |

| normalize | function | normalize(text: &str) |  |  |  |  |

| run_controls | function | run_controls(controls: &[&dyn ComplianceControl], effect: &PaymentEffect) |  |  |  |  |

| with_aliases | function | with_aliases(mut self, aliases: &[(&str, &str) |  |  |  |  |

| ComplianceBundle | struct |  |  |  |  |  |

| ControlEvidence | struct |  |  |  |  |  |

| SanctionsList | struct |  |  |  |  |  |

| ComplianceControl | trait |  |  |  |  |  |


### src/payments/counterparty.rs

| as_str | function | as_str(&self) |  |  |  |  |

| evidence_digest | function | evidence_digest(&self, payer: &Counterparty, payee: &Counterparty) |  |  |  |  |

| fibo_party_json | function | fibo_party_json(cp: &Counterparty) |  |  |  |  |

| from_json | function | from_json(bytes: &[u8]) |  |  |  |  |

| parse | function | parse(text: &str) |  |  |  |  |

| resolve | function | resolve(&self, account: &str) |  |  |  |  |

| with_check_digits | function | with_check_digits(prefix18: &str) |  |  |  |  |

| with_require_lei | function | with_require_lei(mut self, require: bool) |  |  |  |  |

| Counterparty | struct |  |  |  |  |  |

| CounterpartyRegistry | struct |  |  |  |  |  |

| Lei | struct |  |  |  |  |  |


### src/payments/dirlock.rs

| acquire | function | acquire(root: &Path) |  |  |  |  |

| publish_new | function | publish_new(root: &Path, final_path: &Path, bytes: &[u8]) |  |  |  |  |

| publish_replace | function | publish_replace(root: &Path, final_path: &Path, bytes: &[u8]) |  |  |  |  |

| sync_dir | function | sync_dir(root: &Path) |  |  |  |  |

| DirLock | struct |  |  |  |  |  |


### src/payments/effect.rs

| digest | function | digest(&self) |  |  |  |  |

| from_prepared | function | from_prepared(prepared: PreparedEffect) |  |  |  |  |

| invoice_ref | function | invoice_ref(&self) |  |  |  |  |

| money | function | money(&self) |  |  |  |  |

| obligation_id | function | obligation_id(&self) |  |  |  |  |

| payee | function | payee(&self) |  |  |  |  |

| payer | function | payer(&self) |  |  |  |  |

| prepare | function | prepare(
        principal: &str,
        payer: &str,
        payee: &str,
        amount_minor: &str,
        currency: Currency,
        obligation_id: &str,
        purpose: &str,
        reverses: Option<&str>,
    ) |  |  |  |  |

| prepare_with_invoice | function | prepare_with_invoice(
        principal: &str,
        payer: &str,
        payee: &str,
        amount_minor: &str,
        currency: Currency,
        obligation_id: &str,
        purpose: &str,
        invoice_ref: Option<&str>,
        reverses: Option<&str>,
    ) |  |  |  |  |

| prepared | function | prepared(&self) |  |  |  |  |

| principal | function | principal(&self) |  |  |  |  |

| purpose | function | purpose(&self) |  |  |  |  |

| reverses | function | reverses(&self) |  |  |  |  |

| PaymentEffect | struct |  |  |  |  |  |


### src/payments/event_receipt.rs

| explain | function | explain(receipt: &EventReceipt) |  |  |  |  |

| ledger_entry_digest | function | ledger_entry_digest(entry: &LedgerEntry) |  |  |  |  |

| seal_event_receipt | function | seal_event_receipt(inputs: &EventInputs) |  |  |  |  |

| verify_event_receipt | function | verify_event_receipt(receipt: &EventReceipt) |  |  |  |  |

| EventInputs | struct |  |  |  |  |  |

| EventReceipt | struct |  |  |  |  |  |


### src/payments/execute.rs

| PaymentStanding | enum |  |  |  |  |  |

| build_construct | function | build_construct(
    admission: &PaymentAdmission,
    ctx: &ExecutionContext<'_>,
) |  |  |  |  |

| build_construct_with | function | build_construct_with(
    admission: &PaymentAdmission,
    ctx: &ExecutionContext<'_>,
    process: PowlProcess,
    allowed_transitions: BTreeSet<String>,
) |  |  |  |  |

| execute_payment | function | execute_payment(admission: PaymentAdmission, ctx: &ExecutionContext<'_>) |  |  |  |  |

| ExecutionContext | struct |  |  |  |  |  |

| PaymentExecution | struct |  |  |  |  |  |


### src/payments/execute_rail.rs

| FinalizeResult | enum |  |  |  |  |  |

| RailStandingAfterSubmit | enum |  |  |  |  |  |

| abandon_unsubmitted | function | abandon_unsubmitted(
    claims: &ClaimStore,
    ledger: &dyn LedgerPort,
    rail: &dyn RailActuator,
    effect_digest: &str,
) |  |  |  |  |

| finalize_via_rail | function | finalize_via_rail(
    effect_digest: &str,
    claims: &ClaimStore,
    ledger: &dyn LedgerPort,
    rail: &dyn RailActuator,
) |  |  |  |  |

| submit_via_rail | function | submit_via_rail(
    admission: PaymentAdmission,
    params: &RailExecutionParams,
    ctx: &ExecutionContext<'_>,
    rail: &dyn RailActuator,
) |  |  |  |  |

| RailExecutionParams | struct |  |  |  |  |  |

| RailSubmission | struct |  |  |  |  |  |


### src/payments/experience.rs

| Knowledge | enum |  |  |  |  |  |

| classify | function | classify(&self, effect: &PaymentEffect) |  |  |  |  |

| key | function | key(&self) |  |  |  |  |

| of | function | of(effect: &PaymentEffect) |  |  |  |  |

| open | function | open(root: impl Into<PathBuf>) |  |  |  |  |

| record_settled | function | record_settled(&self, effect: &PaymentEffect, exec: &PaymentExecution) |  |  |  |  |

| requires_intelligence | function | requires_intelligence(k: &Knowledge) |  |  |  |  |

| ClassRule | struct |  |  |  |  |  |

| ExperienceStore | struct |  |  |  |  |  |

| PaymentClass | struct |  |  |  |  |  |


### src/payments/fibo.rs

| claim_jsonld | function | claim_jsonld(claim: &Claim, entry: Option<&LedgerEntry>) |  |  |  |  |

| claim_type_iri | function | claim_type_iri(claim: &Claim) |  |  |  |  |

| currency_iri | function | currency_iri(currency: Currency) |  |  |  |  |

| mapping_for | function | mapping_for(id: &str) |  |  |  |  |

| monetary_amount_json | function | monetary_amount_json(money: &Money) |  |  |  |  |

| unverified_terms | function | unverified_terms() |  |  |  |  |


### src/payments/iso20022.rs

| message_profile_version | function | message_profile_version() |  |  |  |  |

| pacs008_fi_credit_transfer | function | pacs008_fi_credit_transfer(
    admission: &PaymentAdmission,
    created_at_iso: &str,
    instructing_agent_bic: &str,
    instructed_agent_bic: &str,
    debtor_agent_bic: &str,
    creditor_agent_bic: &str,
) |  |  |  |  |

| pain001_customer_credit_transfer | function | pain001_customer_credit_transfer(
    admission: &PaymentAdmission,
    created_at_iso: &str,
    debtor_name: &str,
    creditor_name: &str,
    debtor_agent_bic: &str,
    creditor_agent_bic: &str,
) |  |  |  |  |

| project_effect_digest_from_pain001 | function | project_effect_digest_from_pain001(xml: &str) |  |  |  |  |

| project_obligation_id_from_pain001 | function | project_obligation_id_from_pain001(xml: &str) |  |  |  |  |


### src/payments/ledger.rs

| LedgerError | enum |  |  |  |  |  |

| conserves | function | conserves(&self, currency: Currency) |  |  |  |  |

| construct_digest | function | construct_digest(&self) |  |  |  |  |

| entries | function | entries(&self) |  |  |  |  |

| from_construct | function | from_construct(admission: &ConstructAdmission) |  |  |  |  |

| open | function | open(root: impl Into<PathBuf>, opening: &[(&str, Currency, u64) |  |  |  |  |

| returns | function | returns(&self) |  |  |  |  |

| ActuationToken | struct |  |  |  |  |  |

| FileJournalLedger | struct |  |  |  |  |  |

| LedgerEntry | struct |  |  |  |  |  |

| LedgerHold | struct |  |  |  |  |  |

| LedgerPort | trait |  |  |  |  |  |


### src/payments/money.rs

| Currency | enum |  |  |  |  |  |

| code | function | code(self) |  |  |  |  |

| exponent | function | exponent(self) |  |  |  |  |

| parse_minor | function | parse_minor(text: &str, currency: Currency) |  |  |  |  |

| to_decimal_string | function | to_decimal_string(self) |  |  |  |  |

| Money | struct |  |  |  |  |  |


### src/payments/nonce.rs

| claim | function | claim(&self, principal: &str, nonce: &str) |  |  |  |  |

| open | function | open(root: impl Into<PathBuf>) |  |  |  |  |

| DurableNonceFence | struct |  |  |  |  |  |


### src/payments/obligation.rs

| derive_obligation_id | function | derive_obligation_id(payer: &str, payee: &str, purpose: &str, invoice_ref: &str) |  |  |  |  |

| prepare_for_invoice | function | prepare_for_invoice(
    principal: &str,
    payer: &str,
    payee: &str,
    amount_minor: &str,
    currency: Currency,
    purpose: &str,
    invoice_ref: &str,
    reverses: Option<&str>,
) |  |  |  |  |


### src/payments/pee.rs

| check_fresh | function | check_fresh(&self, now_ms: u64) |  |  |  |  |

| reseal_identity | function | reseal_identity(&mut self) |  |  |  |  |

| seal | function | seal(admission: &PaymentAdmission, b: &EffectBindings) |  |  |  |  |

| verify_against | function | verify_against(&self, admission: &PaymentAdmission) |  |  |  |  |

| verify_identity | function | verify_identity(&self) |  |  |  |  |

| EffectBindings | struct |  |  |  |  |  |

| PreparedEconomicEffect | struct |  |  |  |  |  |


### src/payments/policy.rs

| check_quorum | function | check_quorum(&self, certificate_threshold: u16) |  |  |  |  |

| check_static | function | check_static(&self, effect: &PaymentEffect) |  |  |  |  |

| default_min_quorum | function | default_min_quorum() |  |  |  |  |

| to_json | function | to_json(&self) |  |  |  |  |

| PrincipalPolicy | struct |  |  |  |  |  |

| SpendPolicy | struct |  |  |  |  |  |


### src/payments/rail.rs

| RailAck | enum |  |  |  |  |  |

| RailError | enum |  |  |  |  |  |

| RailStatus | enum |  |  |  |  |  |

| correlation_id_for | function | correlation_id_for(effect_id: &str) |  |  |  |  |

| payload_digest | function | payload_digest(&self) |  |  |  |  |

| RailInstruction | struct |  |  |  |  |  |

| RailActuator | trait |  |  |  |  |  |


### src/payments/rail_sim.rs

| SimMode | enum |  |  |  |  |  |

| accepted_correlations | function | accepted_correlations(&self) |  |  |  |  |

| open | function | open(root: &Path, mode: SimMode) |  |  |  |  |

| set_mode | function | set_mode(&self, mode: SimMode) |  |  |  |  |

| settlement_count | function | settlement_count(&self, correlation: &str) |  |  |  |  |

| status_history | function | status_history(&self, correlation: &str) |  |  |  |  |

| submissions_seen | function | submissions_seen(&self, correlation: &str) |  |  |  |  |

| SimRail | struct |  |  |  |  |  |


### src/payments/reconcile.rs

| ReconcileResolution | enum |  |  |  |  |  |

| reconcile | function | reconcile(effect_digest: &str, claims: &ClaimStore, ledger: &dyn LedgerPort) |  |  |  |  |

| recover_journal | function | recover_journal(journal_root: &Path) |  |  |  |  |

| JournalRecovery | struct |  |  |  |  |  |


### src/payments/replay.rs

| ReplayVerdict | enum |  |  |  |  |  |

| admit_payment_journaled | function | admit_payment_journaled(
    prepared: PreparedEffect,
    certificate: &ActuationCertificate,
    ctx: &AdmissionContext<'_>,
    journal: &AdmissionJournal,
) |  |  |  |  |

| admit_payment_screened_journaled | function | admit_payment_screened_journaled(
    prepared: PreparedEffect,
    certificate: &ActuationCertificate,
    ctx: &AdmissionContext<'_>,
    screening: &Screening<'_>,
    journal: &AdmissionJournal,
) |  |  |  |  |

| load | function | load(&self, effect_digest: &str) |  |  |  |  |

| open | function | open(root: impl Into<PathBuf>) |  |  |  |  |

| replay_admission | function | replay_admission(journal: &AdmissionJournal, effect_digest: &str) |  |  |  |  |

| replay_admission_anchored | function | replay_admission_anchored(
    journal: &AdmissionJournal,
    effect_digest: &str,
    trusted_registry: &KeyRegistry,
) |  |  |  |  |

| AdmissionJournal | struct |  |  |  |  |  |

| AdmissionRecord | struct |  |  |  |  |  |


### src/payments/settlement.rs

| FinalityKind | enum |  |  |  |  |  |

| FinalityOutcome | enum |  |  |  |  |  |

| apply_finality | function | apply_finality(claims: &ClaimStore, ledger: &dyn LedgerPort, ev: &FinalityEvidence) |  |  |  |  |

| apply_rail_report | function | apply_rail_report(claims: &ClaimStore, ledger: &dyn LedgerPort, report: &RailReport) |  |  |  |  |

| correlation_id | function | correlation_id(&self) |  |  |  |  |

| effect_digest | function | effect_digest(&self) |  |  |  |  |

| evidence_digest | function | evidence_digest(&self) |  |  |  |  |

| kind | function | kind(&self) |  |  |  |  |

| observe_rail | function | observe_rail(rail: &dyn RailActuator, effect_digest: &str) |  |  |  |  |

| payee | function | payee(&self) |  |  |  |  |

| reason | function | reason(&self) |  |  |  |  |

| status | function | status(&self) |  |  |  |  |

| FinalityEvidence | struct |  |  |  |  |  |

| RailReport | struct |  |  |  |  |  |


### src/planner_minimal.rs

| MinimalActionPlanner | struct |  |  |  |  |  |


### src/reconstitution.rs

| FinalDisposition | enum |  |  |  |  |  |

| admission_digest | function | admission_digest(&self) |  |  |  |  |

| admit_empire_reconstitution_for_construct | function | admit_empire_reconstitution_for_construct(
    document: &str,
) |  |  |  |  |

| as_str | function | as_str(self) |  |  |  |  |

| authority_id | function | authority_id(&self) |  |  |  |  |

| capabilities | function | capabilities(&self) |  |  |  |  |

| disposition | function | disposition(&self) |  |  |  |  |

| evidence_ids | function | evidence_ids(&self) |  |  |  |  |

| id | function | id(&self) |  |  |  |  |

| may_actuate | function | may_actuate(&self) |  |  |  |  |

| observable_surfaces | function | observable_surfaces(&self) |  |  |  |  |

| observation_receipt_digest | function | observation_receipt_digest(&self) |  |  |  |  |

| study_id | function | study_id(&self) |  |  |  |  |

| to_o_star_value | function | to_o_star_value(&self) |  |  |  |  |

| AdmissionBrand | struct |  |  |  |  |  |

| EmpireReconstitutionAdmission | struct |  |  |  |  |  |

| ReconstitutedCapability | struct |  |  |  |  |  |

| ReconstitutionRefusal | struct |  |  |  |  |  |


### src/reference_stack/feedback.rs

| project_beam4pm_feedback | function | project_beam4pm_feedback(
    log: &ReceiptedOcelLog,
    subject: &str,
    replay_identity: &str,
) |  |  |  |  |

| Beam4PmFeedback | struct |  |  |  |  |  |


### src/reference_stack/fibo.rs

| bind_verification_receipt | function | bind_verification_receipt(
    projection: &FiboPaymentProjection,
    receipt: &VerificationReceipt,
) |  |  |  |  |

| project_fibo_payment | function | project_fibo_payment(candidate: KnowledgeHookCandidate) |  |  |  |  |

| AuthorityEvidence | struct |  |  |  |  |  |

| FiboPaymentProjection | struct |  |  |  |  |  |

| KnowledgeHookCandidate | struct |  |  |  |  |  |


### src/reference_stack/ownership.rs

| ReferenceStage | enum |  |  |  |  |  |

| canonical_owners | function | canonical_owners() |  |  |  |  |

| ReferenceOwner | struct |  |  |  |  |  |


### src/refusal_conformance.rs

| verify_refusal | function | verify_refusal(observation: RefusalObservation<'_>) |  |  |  |  |

| RefusalObservation | struct |  |  |  |  |  |


### src/sa2a_authority.rs

| Refusal | enum |  |  |  |  |  |

| SignatureAlgorithm | enum |  |  |  |  |  |

| certificate_message | function | certificate_message(c:&ActuationCertificate) |  |  |  |  |

| new | function | new(provider:P,keys:impl IntoIterator<Item=KeyRecord>) |  |  |  |  |

| verify | function | verify(&self,effect:[u8;32],principal:&str,c:&ActuationCertificate,p:&VerificationPolicy) |  |  |  |  |

| ActuationCertificate | struct |  |  |  |  |  |

| AuthorityVerifier | struct |  |  |  |  |  |

| CertificateSignature | struct |  |  |  |  |  |

| KeyRecord | struct |  |  |  |  |  |

| VerificationPolicy | struct |  |  |  |  |  |

| CryptoProvider | trait |  |  |  |  |  |


### src/sa2a_security/algorithm.rs

| SignatureAlgorithm | enum |  |  |  |  |  |

| id | function | id(self) |  |  |  |  |


### src/sa2a_security/certificate.rs

| signing_message | function | signing_message(&self) |  |  |  |  |

| ActuationCertificate | struct |  |  |  |  |  |


### src/sa2a_security/crypto.rs

| verify_signature | function | verify_signature(
    algorithm: SignatureAlgorithm,
    public_key: &[u8],
    message: &[u8],
    signature: &[u8],
) |  |  |  |  |


### src/sa2a_security/encoding.rs

| push_field | function | push_field(out: &mut Vec<u8>, value: &[u8]) |  |  |  |  |

| sha256_tagged | function | sha256_tagged(domain: &[u8], body: &[u8]) |  |  |  |  |

| valid_sha256_tag | function | valid_sha256_tag(value: &str) |  |  |  |  |


### src/sa2a_security/epoch.rs

| admit_epochs | function | admit_epochs(expected: SecurityEpochs, certificate: SecurityEpochs) |  |  |  |  |

| SecurityEpochs | struct |  |  |  |  |  |


### src/sa2a_security/error.rs

| SecurityRefusal | enum |  |  |  |  |  |


### src/sa2a_security/key_registry.rs

| KeyState | enum |  |  |  |  |  |

| from_records | function | from_records(records: impl IntoIterator<Item = KeyRecord>) |  |  |  |  |

| records | function | records(&self) |  |  |  |  |

| resolve | function | resolve(&self, key_id: &str, now_ms: u64, revocation_epoch: u64) |  |  |  |  |

| KeyRecord | struct |  |  |  |  |  |

| KeyRegistry | struct |  |  |  |  |  |


### src/sa2a_security/nonce.rs

| claim | function | claim(&mut self, key_id: &str, nonce: &str) |  |  |  |  |

| NonceFence | struct |  |  |  |  |  |


### src/sa2a_security/prepared_effect.rs

| digest | function | digest(&self) |  |  |  |  |

| PreparedEffect | struct |  |  |  |  |  |


### src/sa2a_security/principal.rs

| preserve_principal | function | preserve_principal(expected: &str, observed: &str) |  |  |  |  |


### src/sa2a_security/quorum.rs

| admit_distinct_quorum | function | admit_distinct_quorum(
    threshold: u16,
    verified: &[(CertificateSignature, KeyRecord) |  |  |  |  |

| VerifiedSigner | struct |  |  |  |  |  |


### src/sa2a_security/receipt.rs

| from_signers | function | from_signers(
        effect_digest: String,
        principal: String,
        policy_epoch: u64,
        revocation_epoch: u64,
        generation: u64,
        audience: String,
        signers: &[VerifiedSigner],
    ) |  |  |  |  |

| VerificationReceipt | struct |  |  |  |  |  |


### src/sa2a_security/resource.rs

| allocate | function | allocate(&mut self, child: ResourceEnvelope) |  |  |  |  |

| committed | function | committed(&self) |  |  |  |  |

| contains | function | contains(self, child: Self) |  |  |  |  |

| new | function | new(root: ResourceEnvelope) |  |  |  |  |

| validate | function | validate(self) |  |  |  |  |

| BudgetLedger | struct |  |  |  |  |  |

| ResourceEnvelope | struct |  |  |  |  |  |


### src/sa2a_security/signature.rs

| CertificateSignature | struct |  |  |  |  |  |


### src/sa2a_security/verifier.rs

| verify | function | verify(
        &self,
        effect: &PreparedEffect,
        certificate: &ActuationCertificate,
    ) |  |  |  |  |

| CertificateVerifier | struct |  |  |  |  |  |


### src/security_universe/admission.inc.rs

| EvidenceSurface | enum |  |  |  |  |  |

| MappingRelation | enum |  |  |  |  |  |

| SecurityStanding | enum |  |  |  |  |  |

| find_source | function | find_source(id: &str) |  |  |  |  |

| find_tool | function | find_tool(id: &str) |  |  |  |  |

| qualify_federated_fortune5 | function | qualify_federated_fortune5(
    base: &crate::fortune5::Fortune5Qualification,
    evidence: &[CoverageEvidence<'_>],
) |  |  |  |  |

| qualify_fortune5_security_universe | function | qualify_fortune5_security_universe(
    evidence: &[CoverageEvidence<'_>],
) |  |  |  |  |

| validate_security_catalog | function | validate_security_catalog() |  |  |  |  |

| validate_tool_catalog | function | validate_tool_catalog() |  |  |  |  |

| AdmittedExternalToolEvidence | struct |  |  |  |  |  |

| AdmittedSecurityMapping | struct |  |  |  |  |  |

| CoverageEvidence | struct |  |  |  |  |  |

| ExternalToolEvidence | struct |  |  |  |  |  |

| FederatedFortune5Qualification | struct |  |  |  |  |  |

| SecurityIntent | struct |  |  |  |  |  |

| SecurityMapping | struct |  |  |  |  |  |

| SecurityQualification | struct |  |  |  |  |  |


### src/security_universe/catalog.inc.rs

| SecurityKind | enum |  |  |  |  |  |

| VersionPolicy | enum |  |  |  |  |  |

| SecuritySource | struct |  |  |  |  |  |


### src/security_universe/tools.inc.rs

| ToolBoundary | enum |  |  |  |  |  |

| SecurityToolIntegration | struct |  |  |  |  |  |


### src/strategic_board.rs

| assess_board_reentry | function | assess_board_reentry(
    constitution: &BoardConstitution,
    strategic_receipt: &BoardStrategicReceipt,
    event: &crate::board::MaterialityEvent,
    policy: &crate::board::MaterialityPolicy,
) |  |  |  |  |

| assess_counterstrategies | function | assess_counterstrategies(
    constitution: &BoardConstitution,
    partition: &StrategyPartition,
    candidate: &CampaignCandidate,
    scenarios: &[CounterstrategyScenario],
) |  |  |  |  |

| build_strategic_board_package | function | build_strategic_board_package(
    base: &crate::board::BoardPackage,
    constitution: &BoardConstitution,
    mandate: &StrategicMandatePacket,
    receipt: &BoardStrategicReceipt,
    portfolio: &CampaignPortfolioAnalysis,
    counterstrategy: &CounterstrategyAssessment,
    twin: &StrategicTwinSnapshot,
    reentry: &BoardReentryDecision,
    generated_at: &str,
) |  |  |  |  |

| build_strategic_twin_snapshot | function | build_strategic_twin_snapshot(
    constitution: &BoardConstitution,
    mandate: &StrategicMandatePacket,
    receipt: &BoardStrategicReceipt,
    portfolio: &CampaignPortfolioAnalysis,
    counterstrategy: &CounterstrategyAssessment,
) |  |  |  |  |

| diff_strategic_twins | function | diff_strategic_twins(
    previous: &StrategicTwinSnapshot,
    current: &StrategicTwinSnapshot,
) |  |  |  |  |

| judge_counterstrategy | function | judge_counterstrategy(
    constitution: &BoardConstitution,
    partition: &StrategyPartition,
    candidate: &CampaignCandidate,
    scenario: &CounterstrategyScenario,
) |  |  |  |  |

| qualify_campaign_portfolio | function | qualify_campaign_portfolio(
    candidates: &[CampaignCandidate],
    verdicts: &[CampaignVerdict],
    policy: &CampaignPortfolioPolicy,
) |  |  |  |  |

| verify_strategic_board_package_offline | function | verify_strategic_board_package_offline(
    package: &StrategicBoardPackage,
) |  |  |  |  |

| BoardReentryDecision | struct |  |  |  |  |  |

| CampaignPortfolioAnalysis | struct |  |  |  |  |  |

| CampaignPortfolioPolicy | struct |  |  |  |  |  |

| CounterstrategyAssessment | struct |  |  |  |  |  |

| CounterstrategyScenario | struct |  |  |  |  |  |

| CounterstrategyVerdict | struct |  |  |  |  |  |

| OfflineBoardPackageVerification | struct |  |  |  |  |  |

| StrategicBoardDelta | struct |  |  |  |  |  |

| StrategicBoardPackage | struct |  |  |  |  |  |

| StrategicTwinSnapshot | struct |  |  |  |  |  |


### src/strategic_command.rs

| BoardAvatar | enum |  |  |  |  |  |

| RecompileScope | enum |  |  |  |  |  |

| ReplanLevel | enum |  |  |  |  |  |

| StrategicAxis | enum |  |  |  |  |  |

| StrategicStanding | enum |  |  |  |  |  |

| admit_board_selection | function | admit_board_selection(
    constitution: &BoardConstitution,
    candidates: &[CampaignCandidate],
    verdicts: &[CampaignVerdict],
    request: BoardSelectionRequest,
) |  |  |  |  |

| as_str | function | as_str(self) |  |  |  |  |

| bind_strategic_mandate_construct_request | function | bind_strategic_mandate_construct_request(
    mut request: crate::castle::ConstructRequest,
    mandate: &StrategicMandatePacket,
) |  |  |  |  |

| candidate_digest | function | candidate_digest(&self) |  |  |  |  |

| compile_board_constitution | function | compile_board_constitution(input: ConstitutionInput) |  |  |  |  |

| compile_board_strategic_receipt | function | compile_board_strategic_receipt(
    constitution: &BoardConstitution,
    candidate: &CampaignCandidate,
    mandate: &StrategicMandatePacket,
    input: BoardReceiptInput,
) |  |  |  |  |

| compile_strategy_doctrine | function | compile_strategy_doctrine(
    constitution: &BoardConstitution,
    premise_digests: BTreeMap<String, String>,
    capability_ids: Vec<String>,
    provenance: Vec<String>,
) |  |  |  |  |

| construct_campaign_candidate | function | construct_campaign_candidate(
    constitution: &BoardConstitution,
    doctrine: &StrategyDoctrine,
    partition: &StrategyPartition,
    candidate_id: impl Into<String>,
    assumptions: Vec<String>,
    falsifier: impl Into<String>,
    objectives: BTreeMap<String, i64>,
    expected_outcomes: Vec<String>,
    capital_committed: u64,
    reversible_capital: u64,
) |  |  |  |  |

| determine_recompile_scope | function | determine_recompile_scope(
    doctrine: &StrategyDoctrine,
    partitions: &[StrategyPartition],
    current_global_premises: &BTreeMap<String, String>,
    current_local_premises: &BTreeMap<String, String>,
) |  |  |  |  |

| judge_campaign_candidate | function | judge_campaign_candidate(
    constitution: &BoardConstitution,
    doctrine: &StrategyDoctrine,
    partition: &StrategyPartition,
    candidate: &CampaignCandidate,
    current_local_premises: &BTreeMap<String, String>,
) |  |  |  |  |

| partition_strategy | function | partition_strategy(
    doctrine: &StrategyDoctrine,
    strategy_id: impl Into<String>,
    strategy: impl Into<String>,
    local_premise_digests: BTreeMap<String, String>,
    local_constraints: Vec<String>,
    operator_ids: Vec<String>,
) |  |  |  |  |

| route_replan | function | route_replan(evidence: DivergenceEvidence) |  |  |  |  |

| to_json | function | to_json(&self) |  |  |  |  |

| BoardConstitution | struct |  |  |  |  |  |

| BoardLens | struct |  |  |  |  |  |

| BoardReceiptInput | struct |  |  |  |  |  |

| BoardSelectionRequest | struct |  |  |  |  |  |

| BoardStrategicReceipt | struct |  |  |  |  |  |

| CampaignCandidate | struct |  |  |  |  |  |

| CampaignVerdict | struct |  |  |  |  |  |

| ConstitutionInput | struct |  |  |  |  |  |

| DivergenceEvidence | struct |  |  |  |  |  |

| StrategicMandatePacket | struct |  |  |  |  |  |

| StrategyDoctrine | struct |  |  |  |  |  |

| StrategyOperator | struct |  |  |  |  |  |

| StrategyPartition | struct |  |  |  |  |  |


### src/v26_8_18/airgap.rs

| admit_airgap_result | function | admit_airgap_result(bundle: &AirgapBundle, result: &AirgapResult) |  |  |  |  |

| manufacture_airgap_bundle | function | manufacture_airgap_bundle(
    bundle_id: String,
    constitution: &GlobalConstitution,
    o_star_snapshot: Value,
    construct_graph: Value,
    prohibited_goals: Value,
) |  |  |  |  |

| AirgapAdmission | struct |  |  |  |  |  |

| AirgapBundle | struct |  |  |  |  |  |

| AirgapResult | struct |  |  |  |  |  |


### src/v26_8_18/brce.rs

| execute_command_process | function | execute_command_process(
    process: &PowlProcess,
    state: &WorldState,
    envelope: &TestEnvelope,
    admission: &ConstructAdmission,
    policy: CommandAdapterPolicy,
    blake3: &dyn Blake3Provider,
    signer: &dyn ReceiptSigner,
    now: impl Fn() |  |  |  |  |

| execute_command_process_durable | function | execute_command_process_durable(
    process: &PowlProcess,
    state: &WorldState,
    envelope: &TestEnvelope,
    admission: &ConstructAdmission,
    policy: CommandAdapterPolicy,
    durable_journal_root: impl AsRef<Path>,
    blake3: &dyn Blake3Provider,
    signer: &dyn ReceiptSigner,
    now: impl Fn() |  |  |  |  |

| fortune5_adapter_catalog | function | fortune5_adapter_catalog() |  |  |  |  |

| journal | function | journal(&self) |  |  |  |  |

| new | function | new(inner: &'a dyn GymActAdapter, blake3: &'a dyn Blake3Provider, signer: &'a dyn ReceiptSigner) |  |  |  |  |

| new_durable | function | new_durable(
        inner: &'a dyn GymActAdapter,
        blake3: &'a dyn Blake3Provider,
        signer: &'a dyn ReceiptSigner,
        durable_journal_root: PathBuf,
    ) |  |  |  |  |

| validate_command_adapter_policy | function | validate_command_adapter_policy(policy: &CommandAdapterPolicy) |  |  |  |  |

| BrceGymActAdapter | struct |  |  |  |  |  |

| BrceTransitionRecord | struct |  |  |  |  |  |

| CommandAdapterPolicy | struct |  |  |  |  |  |

| CommandSpec | struct |  |  |  |  |  |

| DurableBrceOutcomeRecord | struct |  |  |  |  |  |

| DurableBrcePrepareRecord | struct |  |  |  |  |  |

| DurableBrceReceipt | struct |  |  |  |  |  |

| ProviderAdapterDescriptor | struct |  |  |  |  |  |


### src/v26_8_18/chaos.rs

| ChaosScenario | enum |  |  |  |  |  |

| qualify_chaos | function | qualify_chaos(evidence: &[ChaosEvidence]) |  |  |  |  |

| required_chaos_scenarios | function | required_chaos_scenarios() |  |  |  |  |

| ChaosEvidence | struct |  |  |  |  |  |

| ChaosQualification | struct |  |  |  |  |  |


### src/v26_8_18/crypto.rs

| SignatureSuite | enum |  |  |  |  |  |

| as_str | function | as_str(self) |  |  |  |  |

| dual_artifact_identity | function | dual_artifact_identity(bytes: &[u8]) |  |  |  |  |

| implemented_signature_suites | function | implemented_signature_suites() |  |  |  |  |

| qualify_crypto_profile | function | qualify_crypto_profile(profile: &CryptoProfile) |  |  |  |  |

| qualify_pqc_runtime | function | qualify_pqc_runtime() |  |  |  |  |

| sign_pqc_message | function | sign_pqc_message(
    suite: SignatureSuite,
    seed: [u8; 32],
    message: &[u8],
) |  |  |  |  |

| verify_pqc_message | function | verify_pqc_message(proof: &PqcSignatureProof, message: &[u8]) |  |  |  |  |

| ArtifactIdentity | struct |  |  |  |  |  |

| CryptoProfile | struct |  |  |  |  |  |

| CryptoQualification | struct |  |  |  |  |  |

| PqcRuntimeQualification | struct |  |  |  |  |  |

| PqcSignatureProof | struct |  |  |  |  |  |


### src/v26_8_18/dfcm.rs

| ProbePurpose | enum |  |  |  |  |  |

| ReadOnlyProbeKind | enum |  |  |  |  |  |

| args | function | args(self) |  |  |  |  |

| bind_command_policy | function | bind_command_policy(
    binding: &AdapterBinding,
    policy: &CommandAdapterPolicy,
) |  |  |  |  |

| default_program | function | default_program(self) |  |  |  |  |

| live_observation_index | function | live_observation_index(
    observations: &[ProviderProbeObservation],
) |  |  |  |  |

| manufacture_live_probe_plan | function | manufacture_live_probe_plan(manifest: &DeploymentManifest) |  |  |  |  |

| provider_kind | function | provider_kind(self) |  |  |  |  |

| purpose | function | purpose(self) |  |  |  |  |

| qualify_dfcm_closure | function | qualify_dfcm_closure(
    manifest: &DeploymentManifest,
    observations: &[ProviderProbeObservation],
    crypto_profile: &CryptoProfile,
    now_epoch_ms: i64,
    max_live_evidence_age_ms: i64,
) |  |  |  |  |

| qualify_live_deployment | function | qualify_live_deployment(
    manifest: &DeploymentManifest,
    observations: &[ProviderProbeObservation],
    now_epoch_ms: i64,
    max_age_ms: i64,
) |  |  |  |  |

| qualify_protocol_fence | function | qualify_protocol_fence() |  |  |  |  |

| run_read_only_probe | function | run_read_only_probe(
    spec: &ReadOnlyProbeSpec,
    observed_at_epoch_ms: i64,
) |  |  |  |  |

| DfcmClosureQualification | struct |  |  |  |  |  |

| LiveDeploymentQualification | struct |  |  |  |  |  |

| ProtocolDispatchSummary | struct |  |  |  |  |  |

| ProtocolFenceQualification | struct |  |  |  |  |  |

| ProviderProbeObservation | struct |  |  |  |  |  |

| ReadOnlyProbeSpec | struct |  |  |  |  |  |


### src/v26_8_18/evidence.rs

| persist_evidence | function | persist_evidence(root: impl AsRef<Path>, record: &DurableEvidenceRecord) |  |  |  |  |

| verify_evidence_file | function | verify_evidence_file(path: impl AsRef<Path>) |  |  |  |  |

| DurableEvidenceRecord | struct |  |  |  |  |  |

| EvidenceCommit | struct |  |  |  |  |  |

| EvidenceVerification | struct |  |  |  |  |  |


### src/v26_8_18/protocol.rs

| IntentMode | enum |  |  |  |  |  |

| InterfaceOrigin | enum |  |  |  |  |  |

| a2a_agent_card | function | a2a_agent_card() |  |  |  |  |

| admit_interface_intent | function | admit_interface_intent(intent: &InterfaceIntent) |  |  |  |  |

| admit_observation | function | admit_observation(
    observation: &ObservationEnvelope,
    allowed_sources: &BTreeSet<String>,
    now_epoch_ms: i64,
    max_age_ms: i64,
) |  |  |  |  |

| as_str | function | as_str(self) |  |  |  |  |

| dispatch_interface_intent | function | dispatch_interface_intent(intent: &InterfaceIntent) |  |  |  |  |

| mcp_tool_catalog | function | mcp_tool_catalog() |  |  |  |  |

| A2aAgentCard | struct |  |  |  |  |  |

| AdmittedObservation | struct |  |  |  |  |  |

| InterfaceAdmission | struct |  |  |  |  |  |

| InterfaceIntent | struct |  |  |  |  |  |

| McpToolDescriptor | struct |  |  |  |  |  |

| ObservationEnvelope | struct |  |  |  |  |  |

| ProtocolDispatch | struct |  |  |  |  |  |


### src/v26_8_18/replication.rs

| admit_receipt_checkpoint | function | admit_receipt_checkpoint(state: &mut ReplicaState, checkpoint: ReceiptCheckpoint) |  |  |  |  |

| load_durable_replica | function | load_durable_replica(path: impl AsRef<Path>) |  |  |  |  |

| persist_receipt_checkpoint | function | persist_receipt_checkpoint(
    path: impl AsRef<Path>,
    receiver_id: &str,
    checkpoint: ReceiptCheckpoint,
) |  |  |  |  |

| reconcile_transition | function | reconcile_transition(evidence: &ReconciliationEvidence) |  |  |  |  |

| DurableReplicaCommit | struct |  |  |  |  |  |

| ReceiptCheckpoint | struct |  |  |  |  |  |

| ReconciliationDecision | struct |  |  |  |  |  |

| ReconciliationEvidence | struct |  |  |  |  |  |

| ReplicaState | struct |  |  |  |  |  |

| ReplicationAdmission | struct |  |  |  |  |  |


### src/v26_8_18/runtime.rs

| decode_seed_hex | function | decode_seed_hex(value: &str) |  |  |  |  |

| execute_runtime_request | function | execute_runtime_request(
    request: &RuntimeExecutionRequest,
    key_id: String,
    seed: [u8; 32],
    expected_construct_digest: &str,
    now_epoch_ms: i64,
) |  |  |  |  |

| from_seed | function | from_seed(key_id: String, seed: [u8; 32]) |  |  |  |  |

| manufacture_runtime_construct | function | manufacture_runtime_construct(
    request: &RuntimeExecutionRequest,
    key_id: String,
    seed: [u8; 32],
) |  |  |  |  |

| to_envelope | function | to_envelope(&self) |  |  |  |  |

| to_powl | function | to_powl(&self) |  |  |  |  |

| verifier | function | verifier(&self) |  |  |  |  |

| ConstructManufactureSummary | struct |  |  |  |  |  |

| Ed25519RuntimeSigner | struct |  |  |  |  |  |

| Ed25519RuntimeVerifier | struct |  |  |  |  |  |

| NativeBlake3 | struct |  |  |  |  |  |

| PortableActivity | struct |  |  |  |  |  |

| PortableEnvelope | struct |  |  |  |  |  |

| PortableProcess | struct |  |  |  |  |  |

| RuntimeDoSummary | struct |  |  |  |  |  |

| RuntimeExecutionRequest | struct |  |  |  |  |  |


### src/v26_8_18/topology.rs

| CloudProvider | enum |  |  |  |  |  |

| ReleaseStanding | enum |  |  |  |  |  |

| aggregate_global_standing | function | aggregate_global_standing(mut cells: Vec<CellStandingRow>) |  |  |  |  |

| alive | function | alive(&self) |  |  |  |  |

| as_str | function | as_str(self) |  |  |  |  |

| qualify_deployment | function | qualify_deployment(manifest: &DeploymentManifest, now_epoch_ms: i64) |  |  |  |  |

| AdapterBinding | struct |  |  |  |  |  |

| CastleCellManifest | struct |  |  |  |  |  |

| CellStandingRow | struct |  |  |  |  |  |

| DeploymentManifest | struct |  |  |  |  |  |

| DeploymentQualification | struct |  |  |  |  |  |

| GlobalConstitution | struct |  |  |  |  |  |

| GlobalStanding | struct |  |  |  |  |  |

| ProtocolSurface | struct |  |  |  |  |  |


### src/v26_9_28/mod.rs

| EvidenceStanding | enum |  |  |  |  |  |

| WitnessKind | enum |  |  |  |  |  |

| admit_external_witness | function | admit_external_witness(
    manifest: &EcosystemManifest,
    witness: &ExternalWitness,
) |  |  |  |  |

| admit_fond_differential | function | admit_fond_differential(check: &FondDifferentialCheck) |  |  |  |  |

| admit_independent_plan | function | admit_independent_plan(check: &IndependentPlanCheck) |  |  |  |  |

| admit_sa2a_replan_envelope | function | admit_sa2a_replan_envelope(
    expected_subject: &str,
    envelope: &PortableReplanEnvelope,
) |  |  |  |  |

| bind_v26_9_28_construct_request | function | bind_v26_9_28_construct_request(
    mut request: crate::castle::ConstructRequest,
    manifest: &EcosystemManifest,
    witnesses: &[ExternalWitness],
) |  |  |  |  |

| ecosystem_manifest | function | ecosystem_manifest() |  |  |  |  |

| id | function | id(self) |  |  |  |  |

| is_alive | function | is_alive(&self) |  |  |  |  |

| qualify_portable_runtime | function | qualify_portable_runtime(witnesses: &[PortableRuntimeWitness]) |  |  |  |  |

| qualify_v26_9_28_upgrade | function | qualify_v26_9_28_upgrade(manifest: &EcosystemManifest) |  |  |  |  |

| route_edge_local_recovery | function | route_edge_local_recovery(
    subject: &str,
    ordered_providers: &[String],
    failed_providers: &BTreeSet<String>,
) |  |  |  |  |

| EcosystemManifest | struct |  |  |  |  |  |

| ExcludedPr | struct |  |  |  |  |  |

| ExternalWitness | struct |  |  |  |  |  |

| FondDifferentialCheck | struct |  |  |  |  |  |

| IndependentPlanCheck | struct |  |  |  |  |  |

| MarketplacePack | struct |  |  |  |  |  |

| PortableReplanDecision | struct |  |  |  |  |  |

| PortableReplanEnvelope | struct |  |  |  |  |  |

| PortableRuntimeWitness | struct |  |  |  |  |  |

| RecoveryDecision | struct |  |  |  |  |  |

| ReviewWindow | struct |  |  |  |  |  |

| SourceSubject | struct |  |  |  |  |  |

| UpgradeQualification | struct |  |  |  |  |  |

| WitnessLimits | struct |  |  |  |  |  |


### tests/common/payments.rs

| admission_ctx | function | admission_ctx(&self) |  |  |  |  |

| admit | function | admit(&self, effect: PreparedEffect, nonce: &str) |  |  |  |  |

| cert | function | cert(&self, effect: &PreparedEffect, nonce: &str, signers: &[&str]) |  |  |  |  |

| effect | function | effect(&self, amount: &str, obligation: &str) |  |  |  |  |

| exec_ctx | function | exec_ctx(&self) |  |  |  |  |

| new | function | new(tag: &str) |  |  |  |  |

| token_for | function | token_for(&self, admission: &PaymentAdmission) |  |  |  |  |

| unique_dir | function | unique_dir(tag: &str) |  |  |  |  |

| with | function | with(tag: &str, treasury_minor: u64, per_effect_cap: u64, epoch_cap: u64) |  |  |  |  |

| Fixture | struct |  |  |  |  |  |

| RealBlake3 | struct |  |  |  |  |  |

| ReceiptCheck | struct |  |  |  |  |  |

| ReceiptKey | struct |  |  |  |  |  |


### tests/common/scripted_rail.rs

| authoritative | function | authoritative(mut self) |  |  |  |  |

| new | function | new(script: Vec<RailStatus>) |  |  |  |  |

| push | function | push(&self, s: RailStatus) |  |  |  |  |

| rejected | function | rejected() |  |  |  |  |

| returned | function | returned() |  |  |  |  |

| settled | function | settled() |  |  |  |  |

| ScriptedRail | struct |  |  |  |  |  |


### tests/payments_falsifiers.rs

| PaymentAdmission | struct |  |  |  |  |  |


### tests/payments_holds.rs

| admit_submitted | function | admit_submitted(fx: &Fixture, amount: &str, obligation: &str, nonce: &str) |  |  |  |  |



<!-- AGENT-FORBIDDEN-END -->

## Signature/type/default/errors table

<!-- RIGID table: header order is fixed; rows come only from the query. -->

| Item | Type | Signature | Params | Defaults | Errors | Invariants |
|------|------|-----------|--------|----------|--------|------------|

| test | script | node --experimental-strip-types --test test/*.test.ts |  |  |  |  |

| admission_provider | function | admission_provider/0 |  |  |  |  |

| kernel | function | kernel/0 |  |  |  |  |

| receipt_verifier | function | receipt_verifier/0 |  |  |  |  |

| semantic_bundle | function | semantic_bundle/0 |  |  |  |  |

| admit | function | admit/3 |  |  |  |  |

| atom_key | function | atom_key/1 |  |  |  |  |

| atom_key | function | atom_key/1 |  |  |  |  |

| atom_key | function | atom_key/1 |  |  |  |  |

| atom_key | function | atom_key/1 |  |  |  |  |

| atom_key | function | atom_key/1 |  |  |  |  |

| atom_key | function | atom_key/1 |  |  |  |  |

| atom_key | function | atom_key/1 |  |  |  |  |

| atom_key | function | atom_key/1 |  |  |  |  |

| digest? | function | digest?/1 |  |  |  |  |

| digest? | function | digest?/1 |  |  |  |  |

| external_id | function | external_id/1 |  |  |  |  |

| external_id | function | external_id/1 |  |  |  |  |

| external_id | function | external_id/1 |  |  |  |  |

| external_id | function | external_id/1 |  |  |  |  |

| external_id | function | external_id/1 |  |  |  |  |

| external_id | function | external_id/1 |  |  |  |  |

| get | function | get/2 |  |  |  |  |

| stringify | function | stringify/1 |  |  |  |  |

| stringify | function | stringify/1 |  |  |  |  |

| stringify | function | stringify/1 |  |  |  |  |

| verify | function | verify/3 |  |  |  |  |

| verify | function | verify/3 |  |  |  |  |

| start | function | start/2 |  |  |  |  |

| encode | function | encode/1 |  |  |  |  |

| encode | function | encode/1 |  |  |  |  |

| encode | function | encode/1 |  |  |  |  |

| encode | function | encode/1 |  |  |  |  |

| encode | function | encode/1 |  |  |  |  |

| sha256 | function | sha256/1 |  |  |  |  |

| CastlePaaS.Generated.Resource | ash_resource |  |  |  |  |  |

| build_request | function | build_request/1 |  |  |  |  |

| digest? | function | digest?/1 |  |  |  |  |

| digest? | function | digest?/1 |  |  |  |  |

| execute | function | execute/3 |  |  |  |  |

| manufacture | function | manufacture/1 |  |  |  |  |

| release_info | function | release_info/0 |  |  |  |  |

| require_alive_construct | function | require_alive_construct/1 |  |  |  |  |

| require_alive_construct | function | require_alive_construct/1 |  |  |  |  |

| require_castle_identity | function | require_castle_identity/1 |  |  |  |  |

| require_castle_identity | function | require_castle_identity/1 |  |  |  |  |

| require_receipted_do | function | require_receipted_do/0 |  |  |  |  |

| require_receipted_do | function | require_receipted_do/1 |  |  |  |  |

| required_map | function | required_map/2 |  |  |  |  |

| required_string | function | required_string/2 |  |  |  |  |

| run | function | run/2 |  |  |  |  |

| runtime | function | runtime/0 |  |  |  |  |

| stringify | function | stringify/1 |  |  |  |  |

| stringify | function | stringify/1 |  |  |  |  |

| stringify | function | stringify/1 |  |  |  |  |

| write_request | function | write_request/1 |  |  |  |  |

| record | function | record/3 |  |  |  |  |

| verify | function | verify/1 |  |  |  |  |

| installed_extensions | function | installed_extensions/0 |  |  |  |  |

| min_pg_version | function | min_pg_version/0 |  |  |  |  |

| parse | function | parse/1 |  |  |  |  |

| parse | function | parse/1 |  |  |  |  |

| parse | function | parse/1 |  |  |  |  |

| FiboMapping | struct |  |  |  |  |  |

| crypto_capabilities_handler | function | crypto_capabilities_handler() |  |  |  |  |

| deployment_bind_policy_handler | function | deployment_bind_policy_handler(
    binding_path: String,
    policy_path: String,
) |  |  |  |  |

| dfcm_qualify_handler | function | dfcm_qualify_handler(
    manifest_path: String,
    evidence_path: String,
    now_epoch_ms: i64,
    max_evidence_age_ms: i64,
) |  |  |  |  |

| dfcm_verify_handler | function | dfcm_verify_handler() |  |  |  |  |

| live_check_plan_handler | function | live_check_plan_handler(manifest_path: String) |  |  |  |  |

| live_check_qualify_handler | function | live_check_qualify_handler(
    manifest_path: String,
    evidence_path: String,
    now_epoch_ms: i64,
    max_evidence_age_ms: i64,
) |  |  |  |  |

| live_check_run_handler | function | live_check_run_handler(spec_path: String, observed_at_epoch_ms: i64) |  |  |  |  |

| protocol_dispatch_handler | function | protocol_dispatch_handler(intent_path: String) |  |  |  |  |

| replication_admit_handler | function | replication_admit_handler(
    state_path: String,
    checkpoint_path: String,
    receiver_id: String,
) |  |  |  |  |

| evidence_verify_handler | function | evidence_verify_handler(evidence_path: String) |  |  |  |  |

| chaos_qualify_handler | function | chaos_qualify_handler(evidence_path: String) |  |  |  |  |

| construct_manufacture_handler | function | construct_manufacture_handler(request_path: String, signing_key_path: String, key_id: String) |  |  |  |  |

| crypto_capabilities_handler | function | crypto_capabilities_handler() |  |  |  |  |

| deployment_adapters_handler | function | deployment_adapters_handler() |  |  |  |  |

| deployment_qualify_handler | function | deployment_qualify_handler(manifest_path: String, now_epoch_ms: i64) |  |  |  |  |

| do_execute_handler | function | do_execute_handler(request_path: String, signing_key_path: String, key_id: String, expected_construct_digest: String, now_epoch_ms: i64) |  |  |  |  |

| fortune5_qualify_handler | function | fortune5_qualify_handler(subject: String, evidence_path: String, now_epoch_ms: Option<i64>, max_evidence_age_ms: Option<i64>) |  |  |  |  |

| fortune5_requirements_handler | function | fortune5_requirements_handler() |  |  |  |  |

| impact_coverage_handler | function | impact_coverage_handler(classes_path: String, target_coverage_bps: Option<i64>) |  |  |  |  |

| inventory_components_handler | function | inventory_components_handler() |  |  |  |  |

| inventory_goals_handler | function | inventory_goals_handler() |  |  |  |  |

| protocol_a2a_handler | function | protocol_a2a_handler() |  |  |  |  |

| protocol_mcp_handler | function | protocol_mcp_handler() |  |  |  |  |

| release_info_handler | function | release_info_handler() |  |  |  |  |

| replay_admit_handler | function | replay_admit_handler(replay_class_id: String, structural_signature: String, ontology_version: String, provider_semantics_version: String, invariant_set_digest: String, process_digest: String, invariants_hold: bool) |  |  |  |  |

| execute_handler | function | execute_handler(a: ExecuteArgs) |  |  |  |  |

| explain_handler | function | explain_handler(effect_digest: String, state_dir: String, ledger_dir: String, reference_ledger: bool) |  |  |  |  |

| finalize_handler | function | finalize_handler(
    effect_digest: String,
    state_dir: String,
    ledger_dir: String,
    rail_dir: String,
    rail_mode: Option<String>,
    reference_ledger: bool,
    reference_rail: bool,
) |  |  |  |  |

| prepare_handler | function | prepare_handler(
    principal: String,
    payer: String,
    payee: String,
    amount_minor: String,
    currency: String,
    obligation_id: String,
    purpose: String,
    reverses: Option<String>,
) |  |  |  |  |

| rail_submit_handler | function | rail_submit_handler(a: RailSubmitArgs) |  |  |  |  |

| receipt_handler | function | receipt_handler(
    effect_digest: String,
    state_dir: String,
    ledger_dir: String,
    rail_dir: String,
    journal_dir: String,
    reference_ledger: bool,
    reference_rail: bool,
) |  |  |  |  |

| reconcile_handler | function | reconcile_handler(effect_digest: String, state_dir: String, ledger_dir: String, reference_ledger: bool) |  |  |  |  |

| recover_handler | function | recover_handler(state_dir: String) |  |  |  |  |

| replay_handler | function | replay_handler(journal_dir: String, effect_digest: String) |  |  |  |  |

| ExecuteArgs | struct |  |  |  |  |  |

| RailSubmitArgs | struct |  |  |  |  |  |

| assert_blake3_self_test | function | assert_blake3_self_test() |  |  |  |  |

| blake3_hex_utf8 | function | blake3_hex_utf8(input: &str) |  |  |  |  |

| FailureMode | enum |  |  |  |  |  |

| MaterialityDimension | enum |  |  |  |  |  |

| admit_evidence | function | admit_evidence(bundle: &EvidenceBundle, trust: &TrustStore, store: &HashMap<String, ReceiptV2>) |  |  |  |  |

| admit_failure_semantics | function | admit_failure_semantics(input: &FailureSemanticsInput) |  |  |  |  |

| assess_materiality | function | assess_materiality(event: &MaterialityEvent, policy: &MaterialityPolicy) |  |  |  |  |

| board_requirements | function | board_requirements() |  |  |  |  |

| build_board_package | function | build_board_package(admission: &BoardAdmission, generated_at: &str, material_refused_subjects: u32, risk_appetite_breaches: u32) |  |  |  |  |

| classify_icfr_subject | function | classify_icfr_subject(input: &IcfrSubject) |  |  |  |  |

| decode | function | decode(input: &str) |  |  |  |  |

| detect_segregation_of_duty_violations | function | detect_segregation_of_duty_violations(assignments: &[RoleAssignment], incompatible_role_pairs: &[(String, String) |  |  |  |  |

| encode | function | encode(input: &[u8]) |  |  |  |  |

| issue_evidence_receipt | function | issue_evidence_receipt(input: &EvidenceInput, context: &ReceiptIssueContext, signing_key: &SigningKey) |  |  |  |  |

| qualify_fortune5_board | function | qualify_fortune5_board(input: BoardAdmissionInput) |  |  |  |  |

| qualify_verified_fortune5 | function | qualify_verified_fortune5(
    bundles: &[EvidenceBundle],
    context: &QualificationContext,
    trust: &TrustStore,
    receipt_store: &HashMap<String, ReceiptV2>,
    requirements: &[Fortune5Requirement],
) |  |  |  |  |

| verify_receipt_dag | function | verify_receipt_dag(root: &ReceiptV2, trust: &TrustStore, store: &HashMap<String, ReceiptV2>) |  |  |  |  |

| BoardAdmission | struct |  |  |  |  |  |

| BoardAdmissionInput | struct |  |  |  |  |  |

| BoardPackage | struct |  |  |  |  |  |

| EvidenceAdmission | struct |  |  |  |  |  |

| EvidenceBundle | struct |  |  |  |  |  |

| EvidenceInput | struct |  |  |  |  |  |

| FailureSemanticsDecision | struct |  |  |  |  |  |

| FailureSemanticsInput | struct |  |  |  |  |  |

| IcfrClassification | struct |  |  |  |  |  |

| IcfrSubject | struct |  |  |  |  |  |

| MaterialityAssessment | struct |  |  |  |  |  |

| MaterialityEvent | struct |  |  |  |  |  |

| MaterialityPolicy | struct |  |  |  |  |  |

| ReceiptCoreV2 | struct |  |  |  |  |  |

| ReceiptIssueContext | struct |  |  |  |  |  |

| ReceiptV2 | struct |  |  |  |  |  |

| ReceiptVerification | struct |  |  |  |  |  |

| RoleAssignment | struct |  |  |  |  |  |

| SodViolation | struct |  |  |  |  |  |

| TrustKey | struct |  |  |  |  |  |

| TrustStore | struct |  |  |  |  |  |

| VerifiedQualification | struct |  |  |  |  |  |

| do_authority | function | do_authority(_repository: &str) |  |  |  |  |

| donor | function | donor(repository: &str) |  |  |  |  |

| CapabilityDonor | struct |  |  |  |  |  |

| EpistemicClass | enum |  |  |  |  |  |

| GymActStatus | enum |  |  |  |  |  |

| admit_construct_for_do | function | admit_construct_for_do(
    capability: &ConstructCapability,
    process: &PowlProcess,
    envelope: &TestEnvelope,
    blake3: &dyn Blake3Provider,
    verifier: &dyn ReceiptVerifier,
    policy: &ConstructTrustPolicy,
    now: impl Fn() |  |  |  |  |

| apply_zero_day_observation | function | apply_zero_day_observation(graph: &DependencyGraph, observation: ZeroDayObservation) |  |  |  |  |

| as_str | function | as_str(&self) |  |  |  |  |

| compile_adversarial_classes | function | compile_adversarial_classes(goals: &[AdversarialGoal], rules: &[TransitionRule], planners: &[Box<dyn Planner>]) |  |  |  |  |

| compile_witness_to_powl | function | compile_witness_to_powl(id: &str, vulnerability: &VulnerabilityCondition, rules: &[TransitionRule]) |  |  |  |  |

| construct_compromise | function | construct_compromise(&self, dependency_id: &str, capability: &str) |  |  |  |  |

| create_receipt | function | create_receipt(
    artifact: &Value,
    epistemic_class: EpistemicClass,
    subject: &str,
    parent_digests: &[String],
    blake3: &dyn Blake3Provider,
    signer: &dyn ReceiptSigner,
) |  |  |  |  |

| derive_vulnerabilities | function | derive_vulnerabilities(goal: &AdversarialGoal, rules: &[TransitionRule], max_depth: u32) |  |  |  |  |

| enabled_activities | function | enabled_activities(process: &PowlProcess, completed: &BTreeSet<String>) |  |  |  |  |

| execute_powl_with_gym_act | function | execute_powl_with_gym_act(
    process: &PowlProcess,
    state: &WorldState,
    envelope: &TestEnvelope,
    gymact: &dyn GymActAdapter,
    authorization: DoAuthorizationContext<'_>,
) |  |  |  |  |

| local_docker_default | function | local_docker_default(docker_bin: impl Into<String>) |  |  |  |  |

| manufacture_construct_capability | function | manufacture_construct_capability(request: ConstructRequest, blake3: &dyn Blake3Provider, signer: &dyn ReceiptSigner) |  |  |  |  |

| new | function | new(nodes: Vec<DependencyNode>, edges: Vec<DependencyEdge>) |  |  |  |  |

| platform_eng_colima_default | function | platform_eng_colima_default() |  |  |  |  |

| run_planner_ensemble | function | run_planner_ensemble(problem: &PlanningProblem<'_>, planners: &[Box<dyn Planner>]) |  |  |  |  |

| verify_receipt | function | verify_receipt(artifact: &Value, receipt: &Receipt, blake3: &dyn Blake3Provider, verifier: &dyn ReceiptVerifier, trusted_origin_key_ids: &BTreeSet<String>) |  |  |  |  |

| ActuationPermit | struct |  |  |  |  |  |

| AdmissionBrand | struct |  |  |  |  |  |

| AdversarialGoal | struct |  |  |  |  |  |

| AutofdeLabPlanner | struct |  |  |  |  |  |

| CompiledAdversarialClass | struct |  |  |  |  |  |

| ConstructAdmission | struct |  |  |  |  |  |

| ConstructArtifact | struct |  |  |  |  |  |

| ConstructCapability | struct |  |  |  |  |  |

| ConstructRequest | struct |  |  |  |  |  |

| ConstructSourceReceipts | struct |  |  |  |  |  |

| ConstructSources | struct |  |  |  |  |  |

| ConstructTrustPolicy | struct |  |  |  |  |  |

| ConstructedCompromise | struct |  |  |  |  |  |

| ContainerGymActAdapter | struct |  |  |  |  |  |

| CostMinimizingPlanner | struct |  |  |  |  |  |

| DependencyEdge | struct |  |  |  |  |  |

| DependencyGraph | struct |  |  |  |  |  |

| DependencyNode | struct |  |  |  |  |  |

| DoAuthorizationContext | struct |  |  |  |  |  |

| GymActResult | struct |  |  |  |  |  |

| KindClusterReadOnlyGymAct | struct |  |  |  |  |  |

| OcelEvent | struct |  |  |  |  |  |

| OcelLog | struct |  |  |  |  |  |

| OcelObject | struct |  |  |  |  |  |

| PlanCandidate | struct |  |  |  |  |  |

| PlanningProblem | struct |  |  |  |  |  |

| PowlActivity | struct |  |  |  |  |  |

| PowlProcess | struct |  |  |  |  |  |

| ProcessGymActAdapter | struct |  |  |  |  |  |

| Receipt | struct |  |  |  |  |  |

| ReceiptedOcelLog | struct |  |  |  |  |  |

| TestEnvelope | struct |  |  |  |  |  |

| TransitionRule | struct |  |  |  |  |  |

| VulnerabilityCondition | struct |  |  |  |  |  |

| WitnessPlanner | struct |  |  |  |  |  |

| WorldState | struct |  |  |  |  |  |

| ZeroDayImpact | struct |  |  |  |  |  |

| ZeroDayObservation | struct |  |  |  |  |  |

| Blake3Provider | trait |  |  |  |  |  |

| GymActAdapter | trait |  |  |  |  |  |

| Planner | trait |  |  |  |  |  |

| ReceiptSigner | trait |  |  |  |  |  |

| ReceiptVerifier | trait |  |  |  |  |  |

| DdUiRefusal | enum |  |  |  |  |  |

| admit | function | admit(&self) |  |  |  |  |

| PresentationAuthority | struct |  |  |  |  |  |

| EvidenceEpistemicClass | enum |  |  |  |  |  |

| MetricValue | enum |  |  |  |  |  |

| Standing | enum |  |  |  |  |  |

| admit_replay | function | admit_replay(manifest: &ReplayManifest, subject: &ReplaySubject) |  |  |  |  |

| as_str | function | as_str(&self) |  |  |  |  |

| minimum_impact_coverage | function | minimum_impact_coverage(classes: &[AdversarialImpactClass], target_coverage_bps: i64) |  |  |  |  |

| qualify_fortune5 | function | qualify_fortune5(
    observations: &[MetricObservation],
    context: &QualificationContext,
    requirements: &[Fortune5Requirement],
) |  |  |  |  |

| qualify_fortune5_default | function | qualify_fortune5_default(observations: &[MetricObservation], context: &QualificationContext) |  |  |  |  |

| AdversarialImpactClass | struct |  |  |  |  |  |

| ControlEvaluation | struct |  |  |  |  |  |

| Fortune5Qualification | struct |  |  |  |  |  |

| ImpactCoverageSelection | struct |  |  |  |  |  |

| MetricObservation | struct |  |  |  |  |  |

| QualificationContext | struct |  |  |  |  |  |

| ReplayAdmission | struct |  |  |  |  |  |

| ReplayManifest | struct |  |  |  |  |  |

| ReplaySubject | struct |  |  |  |  |  |

| Fortune5Requirement | struct |  |  |  |  |  |

| default_adversarial_goals | function | default_adversarial_goals() |  |  |  |  |

| generated_components | function | generated_components() |  |  |  |  |

| DefaultAdversarialGoal | struct |  |  |  |  |  |

| GeneratedBinding | struct |  |  |  |  |  |

| execute_default_container_observation | function | execute_default_container_observation(
    process: &PowlProcess,
    state: &WorldState,
    envelope: &TestEnvelope,
    authorization: DoAuthorizationContext<'_>,
    docker_bin: impl Into<String>,
) |  |  |  |  |

| EffectClass | enum |  |  |  |  |  |

| as_str | function | as_str(self) |  |  |  |  |

| digest | function | digest(&self) |  |  |  |  |

| to_json | function | to_json(&self) |  |  |  |  |

| OperationEnvelope | struct |  |  |  |  |  |

| StepOutcome | enum |  |  |  |  |  |

| outcomes | function | outcomes(&self) |  |  |  |  |

| PaymentGymActAdapter | struct |  |  |  |  |  |

| admit_payment | function | admit_payment(
    prepared: PreparedEffect,
    certificate: &ActuationCertificate,
    ctx: &AdmissionContext<'_>,
) |  |  |  |  |

| admit_payment_screened | function | admit_payment_screened(
    prepared: PreparedEffect,
    certificate: &ActuationCertificate,
    ctx: &AdmissionContext<'_>,
    screening: &Screening<'_>,
) |  |  |  |  |

| effect | function | effect(&self) |  |  |  |  |

| generation | function | generation(&self) |  |  |  |  |

| nonce | function | nonce(&self) |  |  |  |  |

| screening | function | screening(&self) |  |  |  |  |

| verification | function | verification(&self) |  |  |  |  |

| AdmissionContext | struct |  |  |  |  |  |

| PaymentAdmission | struct |  |  |  |  |  |

| Screening | struct |  |  |  |  |  |

| ScreeningEvidence | struct |  |  |  |  |  |

| ClaimState | enum |  |  |  |  |  |

| get | function | get(&self, digest: &str) |  |  |  |  |

| list | function | list(&self) |  |  |  |  |

| open | function | open(root: impl Into<PathBuf>) |  |  |  |  |

| reserve | function | reserve(&self, claim: &Claim, epoch_cap: Option<u64>) |  |  |  |  |

| reversed_total | function | reversed_total(&self, original: &str) |  |  |  |  |

| transition | function | transition(
        &self,
        digest: &str,
        from: &[ClaimState],
        to: ClaimState,
        construct_digest: Option<&str>,
        detail: &str,
    ) |  |  |  |  |

| Claim | struct |  |  |  |  |  |

| ClaimStore | struct |  |  |  |  |  |

| from_json | function | from_json(bytes: &[u8]) |  |  |  |  |

| normalize | function | normalize(text: &str) |  |  |  |  |

| run_controls | function | run_controls(controls: &[&dyn ComplianceControl], effect: &PaymentEffect) |  |  |  |  |

| with_aliases | function | with_aliases(mut self, aliases: &[(&str, &str) |  |  |  |  |

| ComplianceBundle | struct |  |  |  |  |  |

| ControlEvidence | struct |  |  |  |  |  |

| SanctionsList | struct |  |  |  |  |  |

| ComplianceControl | trait |  |  |  |  |  |

| as_str | function | as_str(&self) |  |  |  |  |

| evidence_digest | function | evidence_digest(&self, payer: &Counterparty, payee: &Counterparty) |  |  |  |  |

| fibo_party_json | function | fibo_party_json(cp: &Counterparty) |  |  |  |  |

| from_json | function | from_json(bytes: &[u8]) |  |  |  |  |

| parse | function | parse(text: &str) |  |  |  |  |

| resolve | function | resolve(&self, account: &str) |  |  |  |  |

| with_check_digits | function | with_check_digits(prefix18: &str) |  |  |  |  |

| with_require_lei | function | with_require_lei(mut self, require: bool) |  |  |  |  |

| Counterparty | struct |  |  |  |  |  |

| CounterpartyRegistry | struct |  |  |  |  |  |

| Lei | struct |  |  |  |  |  |

| acquire | function | acquire(root: &Path) |  |  |  |  |

| publish_new | function | publish_new(root: &Path, final_path: &Path, bytes: &[u8]) |  |  |  |  |

| publish_replace | function | publish_replace(root: &Path, final_path: &Path, bytes: &[u8]) |  |  |  |  |

| sync_dir | function | sync_dir(root: &Path) |  |  |  |  |

| DirLock | struct |  |  |  |  |  |

| digest | function | digest(&self) |  |  |  |  |

| from_prepared | function | from_prepared(prepared: PreparedEffect) |  |  |  |  |

| invoice_ref | function | invoice_ref(&self) |  |  |  |  |

| money | function | money(&self) |  |  |  |  |

| obligation_id | function | obligation_id(&self) |  |  |  |  |

| payee | function | payee(&self) |  |  |  |  |

| payer | function | payer(&self) |  |  |  |  |

| prepare | function | prepare(
        principal: &str,
        payer: &str,
        payee: &str,
        amount_minor: &str,
        currency: Currency,
        obligation_id: &str,
        purpose: &str,
        reverses: Option<&str>,
    ) |  |  |  |  |

| prepare_with_invoice | function | prepare_with_invoice(
        principal: &str,
        payer: &str,
        payee: &str,
        amount_minor: &str,
        currency: Currency,
        obligation_id: &str,
        purpose: &str,
        invoice_ref: Option<&str>,
        reverses: Option<&str>,
    ) |  |  |  |  |

| prepared | function | prepared(&self) |  |  |  |  |

| principal | function | principal(&self) |  |  |  |  |

| purpose | function | purpose(&self) |  |  |  |  |

| reverses | function | reverses(&self) |  |  |  |  |

| PaymentEffect | struct |  |  |  |  |  |

| explain | function | explain(receipt: &EventReceipt) |  |  |  |  |

| ledger_entry_digest | function | ledger_entry_digest(entry: &LedgerEntry) |  |  |  |  |

| seal_event_receipt | function | seal_event_receipt(inputs: &EventInputs) |  |  |  |  |

| verify_event_receipt | function | verify_event_receipt(receipt: &EventReceipt) |  |  |  |  |

| EventInputs | struct |  |  |  |  |  |

| EventReceipt | struct |  |  |  |  |  |

| PaymentStanding | enum |  |  |  |  |  |

| build_construct | function | build_construct(
    admission: &PaymentAdmission,
    ctx: &ExecutionContext<'_>,
) |  |  |  |  |

| build_construct_with | function | build_construct_with(
    admission: &PaymentAdmission,
    ctx: &ExecutionContext<'_>,
    process: PowlProcess,
    allowed_transitions: BTreeSet<String>,
) |  |  |  |  |

| execute_payment | function | execute_payment(admission: PaymentAdmission, ctx: &ExecutionContext<'_>) |  |  |  |  |

| ExecutionContext | struct |  |  |  |  |  |

| PaymentExecution | struct |  |  |  |  |  |

| FinalizeResult | enum |  |  |  |  |  |

| RailStandingAfterSubmit | enum |  |  |  |  |  |

| abandon_unsubmitted | function | abandon_unsubmitted(
    claims: &ClaimStore,
    ledger: &dyn LedgerPort,
    rail: &dyn RailActuator,
    effect_digest: &str,
) |  |  |  |  |

| finalize_via_rail | function | finalize_via_rail(
    effect_digest: &str,
    claims: &ClaimStore,
    ledger: &dyn LedgerPort,
    rail: &dyn RailActuator,
) |  |  |  |  |

| submit_via_rail | function | submit_via_rail(
    admission: PaymentAdmission,
    params: &RailExecutionParams,
    ctx: &ExecutionContext<'_>,
    rail: &dyn RailActuator,
) |  |  |  |  |

| RailExecutionParams | struct |  |  |  |  |  |

| RailSubmission | struct |  |  |  |  |  |

| Knowledge | enum |  |  |  |  |  |

| classify | function | classify(&self, effect: &PaymentEffect) |  |  |  |  |

| key | function | key(&self) |  |  |  |  |

| of | function | of(effect: &PaymentEffect) |  |  |  |  |

| open | function | open(root: impl Into<PathBuf>) |  |  |  |  |

| record_settled | function | record_settled(&self, effect: &PaymentEffect, exec: &PaymentExecution) |  |  |  |  |

| requires_intelligence | function | requires_intelligence(k: &Knowledge) |  |  |  |  |

| ClassRule | struct |  |  |  |  |  |

| ExperienceStore | struct |  |  |  |  |  |

| PaymentClass | struct |  |  |  |  |  |

| claim_jsonld | function | claim_jsonld(claim: &Claim, entry: Option<&LedgerEntry>) |  |  |  |  |

| claim_type_iri | function | claim_type_iri(claim: &Claim) |  |  |  |  |

| currency_iri | function | currency_iri(currency: Currency) |  |  |  |  |

| mapping_for | function | mapping_for(id: &str) |  |  |  |  |

| monetary_amount_json | function | monetary_amount_json(money: &Money) |  |  |  |  |

| unverified_terms | function | unverified_terms() |  |  |  |  |

| message_profile_version | function | message_profile_version() |  |  |  |  |

| pacs008_fi_credit_transfer | function | pacs008_fi_credit_transfer(
    admission: &PaymentAdmission,
    created_at_iso: &str,
    instructing_agent_bic: &str,
    instructed_agent_bic: &str,
    debtor_agent_bic: &str,
    creditor_agent_bic: &str,
) |  |  |  |  |

| pain001_customer_credit_transfer | function | pain001_customer_credit_transfer(
    admission: &PaymentAdmission,
    created_at_iso: &str,
    debtor_name: &str,
    creditor_name: &str,
    debtor_agent_bic: &str,
    creditor_agent_bic: &str,
) |  |  |  |  |

| project_effect_digest_from_pain001 | function | project_effect_digest_from_pain001(xml: &str) |  |  |  |  |

| project_obligation_id_from_pain001 | function | project_obligation_id_from_pain001(xml: &str) |  |  |  |  |

| LedgerError | enum |  |  |  |  |  |

| conserves | function | conserves(&self, currency: Currency) |  |  |  |  |

| construct_digest | function | construct_digest(&self) |  |  |  |  |

| entries | function | entries(&self) |  |  |  |  |

| from_construct | function | from_construct(admission: &ConstructAdmission) |  |  |  |  |

| open | function | open(root: impl Into<PathBuf>, opening: &[(&str, Currency, u64) |  |  |  |  |

| returns | function | returns(&self) |  |  |  |  |

| ActuationToken | struct |  |  |  |  |  |

| FileJournalLedger | struct |  |  |  |  |  |

| LedgerEntry | struct |  |  |  |  |  |

| LedgerHold | struct |  |  |  |  |  |

| LedgerPort | trait |  |  |  |  |  |

| Currency | enum |  |  |  |  |  |

| code | function | code(self) |  |  |  |  |

| exponent | function | exponent(self) |  |  |  |  |

| parse_minor | function | parse_minor(text: &str, currency: Currency) |  |  |  |  |

| to_decimal_string | function | to_decimal_string(self) |  |  |  |  |

| Money | struct |  |  |  |  |  |

| claim | function | claim(&self, principal: &str, nonce: &str) |  |  |  |  |

| open | function | open(root: impl Into<PathBuf>) |  |  |  |  |

| DurableNonceFence | struct |  |  |  |  |  |

| derive_obligation_id | function | derive_obligation_id(payer: &str, payee: &str, purpose: &str, invoice_ref: &str) |  |  |  |  |

| prepare_for_invoice | function | prepare_for_invoice(
    principal: &str,
    payer: &str,
    payee: &str,
    amount_minor: &str,
    currency: Currency,
    purpose: &str,
    invoice_ref: &str,
    reverses: Option<&str>,
) |  |  |  |  |

| check_fresh | function | check_fresh(&self, now_ms: u64) |  |  |  |  |

| reseal_identity | function | reseal_identity(&mut self) |  |  |  |  |

| seal | function | seal(admission: &PaymentAdmission, b: &EffectBindings) |  |  |  |  |

| verify_against | function | verify_against(&self, admission: &PaymentAdmission) |  |  |  |  |

| verify_identity | function | verify_identity(&self) |  |  |  |  |

| EffectBindings | struct |  |  |  |  |  |

| PreparedEconomicEffect | struct |  |  |  |  |  |

| check_quorum | function | check_quorum(&self, certificate_threshold: u16) |  |  |  |  |

| check_static | function | check_static(&self, effect: &PaymentEffect) |  |  |  |  |

| default_min_quorum | function | default_min_quorum() |  |  |  |  |

| to_json | function | to_json(&self) |  |  |  |  |

| PrincipalPolicy | struct |  |  |  |  |  |

| SpendPolicy | struct |  |  |  |  |  |

| RailAck | enum |  |  |  |  |  |

| RailError | enum |  |  |  |  |  |

| RailStatus | enum |  |  |  |  |  |

| correlation_id_for | function | correlation_id_for(effect_id: &str) |  |  |  |  |

| payload_digest | function | payload_digest(&self) |  |  |  |  |

| RailInstruction | struct |  |  |  |  |  |

| RailActuator | trait |  |  |  |  |  |

| SimMode | enum |  |  |  |  |  |

| accepted_correlations | function | accepted_correlations(&self) |  |  |  |  |

| open | function | open(root: &Path, mode: SimMode) |  |  |  |  |

| set_mode | function | set_mode(&self, mode: SimMode) |  |  |  |  |

| settlement_count | function | settlement_count(&self, correlation: &str) |  |  |  |  |

| status_history | function | status_history(&self, correlation: &str) |  |  |  |  |

| submissions_seen | function | submissions_seen(&self, correlation: &str) |  |  |  |  |

| SimRail | struct |  |  |  |  |  |

| ReconcileResolution | enum |  |  |  |  |  |

| reconcile | function | reconcile(effect_digest: &str, claims: &ClaimStore, ledger: &dyn LedgerPort) |  |  |  |  |

| recover_journal | function | recover_journal(journal_root: &Path) |  |  |  |  |

| JournalRecovery | struct |  |  |  |  |  |

| ReplayVerdict | enum |  |  |  |  |  |

| admit_payment_journaled | function | admit_payment_journaled(
    prepared: PreparedEffect,
    certificate: &ActuationCertificate,
    ctx: &AdmissionContext<'_>,
    journal: &AdmissionJournal,
) |  |  |  |  |

| admit_payment_screened_journaled | function | admit_payment_screened_journaled(
    prepared: PreparedEffect,
    certificate: &ActuationCertificate,
    ctx: &AdmissionContext<'_>,
    screening: &Screening<'_>,
    journal: &AdmissionJournal,
) |  |  |  |  |

| load | function | load(&self, effect_digest: &str) |  |  |  |  |

| open | function | open(root: impl Into<PathBuf>) |  |  |  |  |

| replay_admission | function | replay_admission(journal: &AdmissionJournal, effect_digest: &str) |  |  |  |  |

| replay_admission_anchored | function | replay_admission_anchored(
    journal: &AdmissionJournal,
    effect_digest: &str,
    trusted_registry: &KeyRegistry,
) |  |  |  |  |

| AdmissionJournal | struct |  |  |  |  |  |

| AdmissionRecord | struct |  |  |  |  |  |

| FinalityKind | enum |  |  |  |  |  |

| FinalityOutcome | enum |  |  |  |  |  |

| apply_finality | function | apply_finality(claims: &ClaimStore, ledger: &dyn LedgerPort, ev: &FinalityEvidence) |  |  |  |  |

| apply_rail_report | function | apply_rail_report(claims: &ClaimStore, ledger: &dyn LedgerPort, report: &RailReport) |  |  |  |  |

| correlation_id | function | correlation_id(&self) |  |  |  |  |

| effect_digest | function | effect_digest(&self) |  |  |  |  |

| evidence_digest | function | evidence_digest(&self) |  |  |  |  |

| kind | function | kind(&self) |  |  |  |  |

| observe_rail | function | observe_rail(rail: &dyn RailActuator, effect_digest: &str) |  |  |  |  |

| payee | function | payee(&self) |  |  |  |  |

| reason | function | reason(&self) |  |  |  |  |

| status | function | status(&self) |  |  |  |  |

| FinalityEvidence | struct |  |  |  |  |  |

| RailReport | struct |  |  |  |  |  |

| MinimalActionPlanner | struct |  |  |  |  |  |

| FinalDisposition | enum |  |  |  |  |  |

| admission_digest | function | admission_digest(&self) |  |  |  |  |

| admit_empire_reconstitution_for_construct | function | admit_empire_reconstitution_for_construct(
    document: &str,
) |  |  |  |  |

| as_str | function | as_str(self) |  |  |  |  |

| authority_id | function | authority_id(&self) |  |  |  |  |

| capabilities | function | capabilities(&self) |  |  |  |  |

| disposition | function | disposition(&self) |  |  |  |  |

| evidence_ids | function | evidence_ids(&self) |  |  |  |  |

| id | function | id(&self) |  |  |  |  |

| may_actuate | function | may_actuate(&self) |  |  |  |  |

| observable_surfaces | function | observable_surfaces(&self) |  |  |  |  |

| observation_receipt_digest | function | observation_receipt_digest(&self) |  |  |  |  |

| study_id | function | study_id(&self) |  |  |  |  |

| to_o_star_value | function | to_o_star_value(&self) |  |  |  |  |

| AdmissionBrand | struct |  |  |  |  |  |

| EmpireReconstitutionAdmission | struct |  |  |  |  |  |

| ReconstitutedCapability | struct |  |  |  |  |  |

| ReconstitutionRefusal | struct |  |  |  |  |  |

| project_beam4pm_feedback | function | project_beam4pm_feedback(
    log: &ReceiptedOcelLog,
    subject: &str,
    replay_identity: &str,
) |  |  |  |  |

| Beam4PmFeedback | struct |  |  |  |  |  |

| bind_verification_receipt | function | bind_verification_receipt(
    projection: &FiboPaymentProjection,
    receipt: &VerificationReceipt,
) |  |  |  |  |

| project_fibo_payment | function | project_fibo_payment(candidate: KnowledgeHookCandidate) |  |  |  |  |

| AuthorityEvidence | struct |  |  |  |  |  |

| FiboPaymentProjection | struct |  |  |  |  |  |

| KnowledgeHookCandidate | struct |  |  |  |  |  |

| ReferenceStage | enum |  |  |  |  |  |

| canonical_owners | function | canonical_owners() |  |  |  |  |

| ReferenceOwner | struct |  |  |  |  |  |

| verify_refusal | function | verify_refusal(observation: RefusalObservation<'_>) |  |  |  |  |

| RefusalObservation | struct |  |  |  |  |  |

| Refusal | enum |  |  |  |  |  |

| SignatureAlgorithm | enum |  |  |  |  |  |

| certificate_message | function | certificate_message(c:&ActuationCertificate) |  |  |  |  |

| new | function | new(provider:P,keys:impl IntoIterator<Item=KeyRecord>) |  |  |  |  |

| verify | function | verify(&self,effect:[u8;32],principal:&str,c:&ActuationCertificate,p:&VerificationPolicy) |  |  |  |  |

| ActuationCertificate | struct |  |  |  |  |  |

| AuthorityVerifier | struct |  |  |  |  |  |

| CertificateSignature | struct |  |  |  |  |  |

| KeyRecord | struct |  |  |  |  |  |

| VerificationPolicy | struct |  |  |  |  |  |

| CryptoProvider | trait |  |  |  |  |  |

| SignatureAlgorithm | enum |  |  |  |  |  |

| id | function | id(self) |  |  |  |  |

| signing_message | function | signing_message(&self) |  |  |  |  |

| ActuationCertificate | struct |  |  |  |  |  |

| verify_signature | function | verify_signature(
    algorithm: SignatureAlgorithm,
    public_key: &[u8],
    message: &[u8],
    signature: &[u8],
) |  |  |  |  |

| push_field | function | push_field(out: &mut Vec<u8>, value: &[u8]) |  |  |  |  |

| sha256_tagged | function | sha256_tagged(domain: &[u8], body: &[u8]) |  |  |  |  |

| valid_sha256_tag | function | valid_sha256_tag(value: &str) |  |  |  |  |

| admit_epochs | function | admit_epochs(expected: SecurityEpochs, certificate: SecurityEpochs) |  |  |  |  |

| SecurityEpochs | struct |  |  |  |  |  |

| SecurityRefusal | enum |  |  |  |  |  |

| KeyState | enum |  |  |  |  |  |

| from_records | function | from_records(records: impl IntoIterator<Item = KeyRecord>) |  |  |  |  |

| records | function | records(&self) |  |  |  |  |

| resolve | function | resolve(&self, key_id: &str, now_ms: u64, revocation_epoch: u64) |  |  |  |  |

| KeyRecord | struct |  |  |  |  |  |

| KeyRegistry | struct |  |  |  |  |  |

| claim | function | claim(&mut self, key_id: &str, nonce: &str) |  |  |  |  |

| NonceFence | struct |  |  |  |  |  |

| digest | function | digest(&self) |  |  |  |  |

| PreparedEffect | struct |  |  |  |  |  |

| preserve_principal | function | preserve_principal(expected: &str, observed: &str) |  |  |  |  |

| admit_distinct_quorum | function | admit_distinct_quorum(
    threshold: u16,
    verified: &[(CertificateSignature, KeyRecord) |  |  |  |  |

| VerifiedSigner | struct |  |  |  |  |  |

| from_signers | function | from_signers(
        effect_digest: String,
        principal: String,
        policy_epoch: u64,
        revocation_epoch: u64,
        generation: u64,
        audience: String,
        signers: &[VerifiedSigner],
    ) |  |  |  |  |

| VerificationReceipt | struct |  |  |  |  |  |

| allocate | function | allocate(&mut self, child: ResourceEnvelope) |  |  |  |  |

| committed | function | committed(&self) |  |  |  |  |

| contains | function | contains(self, child: Self) |  |  |  |  |

| new | function | new(root: ResourceEnvelope) |  |  |  |  |

| validate | function | validate(self) |  |  |  |  |

| BudgetLedger | struct |  |  |  |  |  |

| ResourceEnvelope | struct |  |  |  |  |  |

| CertificateSignature | struct |  |  |  |  |  |

| verify | function | verify(
        &self,
        effect: &PreparedEffect,
        certificate: &ActuationCertificate,
    ) |  |  |  |  |

| CertificateVerifier | struct |  |  |  |  |  |

| EvidenceSurface | enum |  |  |  |  |  |

| MappingRelation | enum |  |  |  |  |  |

| SecurityStanding | enum |  |  |  |  |  |

| find_source | function | find_source(id: &str) |  |  |  |  |

| find_tool | function | find_tool(id: &str) |  |  |  |  |

| qualify_federated_fortune5 | function | qualify_federated_fortune5(
    base: &crate::fortune5::Fortune5Qualification,
    evidence: &[CoverageEvidence<'_>],
) |  |  |  |  |

| qualify_fortune5_security_universe | function | qualify_fortune5_security_universe(
    evidence: &[CoverageEvidence<'_>],
) |  |  |  |  |

| validate_security_catalog | function | validate_security_catalog() |  |  |  |  |

| validate_tool_catalog | function | validate_tool_catalog() |  |  |  |  |

| AdmittedExternalToolEvidence | struct |  |  |  |  |  |

| AdmittedSecurityMapping | struct |  |  |  |  |  |

| CoverageEvidence | struct |  |  |  |  |  |

| ExternalToolEvidence | struct |  |  |  |  |  |

| FederatedFortune5Qualification | struct |  |  |  |  |  |

| SecurityIntent | struct |  |  |  |  |  |

| SecurityMapping | struct |  |  |  |  |  |

| SecurityQualification | struct |  |  |  |  |  |

| SecurityKind | enum |  |  |  |  |  |

| VersionPolicy | enum |  |  |  |  |  |

| SecuritySource | struct |  |  |  |  |  |

| ToolBoundary | enum |  |  |  |  |  |

| SecurityToolIntegration | struct |  |  |  |  |  |

| assess_board_reentry | function | assess_board_reentry(
    constitution: &BoardConstitution,
    strategic_receipt: &BoardStrategicReceipt,
    event: &crate::board::MaterialityEvent,
    policy: &crate::board::MaterialityPolicy,
) |  |  |  |  |

| assess_counterstrategies | function | assess_counterstrategies(
    constitution: &BoardConstitution,
    partition: &StrategyPartition,
    candidate: &CampaignCandidate,
    scenarios: &[CounterstrategyScenario],
) |  |  |  |  |

| build_strategic_board_package | function | build_strategic_board_package(
    base: &crate::board::BoardPackage,
    constitution: &BoardConstitution,
    mandate: &StrategicMandatePacket,
    receipt: &BoardStrategicReceipt,
    portfolio: &CampaignPortfolioAnalysis,
    counterstrategy: &CounterstrategyAssessment,
    twin: &StrategicTwinSnapshot,
    reentry: &BoardReentryDecision,
    generated_at: &str,
) |  |  |  |  |

| build_strategic_twin_snapshot | function | build_strategic_twin_snapshot(
    constitution: &BoardConstitution,
    mandate: &StrategicMandatePacket,
    receipt: &BoardStrategicReceipt,
    portfolio: &CampaignPortfolioAnalysis,
    counterstrategy: &CounterstrategyAssessment,
) |  |  |  |  |

| diff_strategic_twins | function | diff_strategic_twins(
    previous: &StrategicTwinSnapshot,
    current: &StrategicTwinSnapshot,
) |  |  |  |  |

| judge_counterstrategy | function | judge_counterstrategy(
    constitution: &BoardConstitution,
    partition: &StrategyPartition,
    candidate: &CampaignCandidate,
    scenario: &CounterstrategyScenario,
) |  |  |  |  |

| qualify_campaign_portfolio | function | qualify_campaign_portfolio(
    candidates: &[CampaignCandidate],
    verdicts: &[CampaignVerdict],
    policy: &CampaignPortfolioPolicy,
) |  |  |  |  |

| verify_strategic_board_package_offline | function | verify_strategic_board_package_offline(
    package: &StrategicBoardPackage,
) |  |  |  |  |

| BoardReentryDecision | struct |  |  |  |  |  |

| CampaignPortfolioAnalysis | struct |  |  |  |  |  |

| CampaignPortfolioPolicy | struct |  |  |  |  |  |

| CounterstrategyAssessment | struct |  |  |  |  |  |

| CounterstrategyScenario | struct |  |  |  |  |  |

| CounterstrategyVerdict | struct |  |  |  |  |  |

| OfflineBoardPackageVerification | struct |  |  |  |  |  |

| StrategicBoardDelta | struct |  |  |  |  |  |

| StrategicBoardPackage | struct |  |  |  |  |  |

| StrategicTwinSnapshot | struct |  |  |  |  |  |

| BoardAvatar | enum |  |  |  |  |  |

| RecompileScope | enum |  |  |  |  |  |

| ReplanLevel | enum |  |  |  |  |  |

| StrategicAxis | enum |  |  |  |  |  |

| StrategicStanding | enum |  |  |  |  |  |

| admit_board_selection | function | admit_board_selection(
    constitution: &BoardConstitution,
    candidates: &[CampaignCandidate],
    verdicts: &[CampaignVerdict],
    request: BoardSelectionRequest,
) |  |  |  |  |

| as_str | function | as_str(self) |  |  |  |  |

| bind_strategic_mandate_construct_request | function | bind_strategic_mandate_construct_request(
    mut request: crate::castle::ConstructRequest,
    mandate: &StrategicMandatePacket,
) |  |  |  |  |

| candidate_digest | function | candidate_digest(&self) |  |  |  |  |

| compile_board_constitution | function | compile_board_constitution(input: ConstitutionInput) |  |  |  |  |

| compile_board_strategic_receipt | function | compile_board_strategic_receipt(
    constitution: &BoardConstitution,
    candidate: &CampaignCandidate,
    mandate: &StrategicMandatePacket,
    input: BoardReceiptInput,
) |  |  |  |  |

| compile_strategy_doctrine | function | compile_strategy_doctrine(
    constitution: &BoardConstitution,
    premise_digests: BTreeMap<String, String>,
    capability_ids: Vec<String>,
    provenance: Vec<String>,
) |  |  |  |  |

| construct_campaign_candidate | function | construct_campaign_candidate(
    constitution: &BoardConstitution,
    doctrine: &StrategyDoctrine,
    partition: &StrategyPartition,
    candidate_id: impl Into<String>,
    assumptions: Vec<String>,
    falsifier: impl Into<String>,
    objectives: BTreeMap<String, i64>,
    expected_outcomes: Vec<String>,
    capital_committed: u64,
    reversible_capital: u64,
) |  |  |  |  |

| determine_recompile_scope | function | determine_recompile_scope(
    doctrine: &StrategyDoctrine,
    partitions: &[StrategyPartition],
    current_global_premises: &BTreeMap<String, String>,
    current_local_premises: &BTreeMap<String, String>,
) |  |  |  |  |

| judge_campaign_candidate | function | judge_campaign_candidate(
    constitution: &BoardConstitution,
    doctrine: &StrategyDoctrine,
    partition: &StrategyPartition,
    candidate: &CampaignCandidate,
    current_local_premises: &BTreeMap<String, String>,
) |  |  |  |  |

| partition_strategy | function | partition_strategy(
    doctrine: &StrategyDoctrine,
    strategy_id: impl Into<String>,
    strategy: impl Into<String>,
    local_premise_digests: BTreeMap<String, String>,
    local_constraints: Vec<String>,
    operator_ids: Vec<String>,
) |  |  |  |  |

| route_replan | function | route_replan(evidence: DivergenceEvidence) |  |  |  |  |

| to_json | function | to_json(&self) |  |  |  |  |

| BoardConstitution | struct |  |  |  |  |  |

| BoardLens | struct |  |  |  |  |  |

| BoardReceiptInput | struct |  |  |  |  |  |

| BoardSelectionRequest | struct |  |  |  |  |  |

| BoardStrategicReceipt | struct |  |  |  |  |  |

| CampaignCandidate | struct |  |  |  |  |  |

| CampaignVerdict | struct |  |  |  |  |  |

| ConstitutionInput | struct |  |  |  |  |  |

| DivergenceEvidence | struct |  |  |  |  |  |

| StrategicMandatePacket | struct |  |  |  |  |  |

| StrategyDoctrine | struct |  |  |  |  |  |

| StrategyOperator | struct |  |  |  |  |  |

| StrategyPartition | struct |  |  |  |  |  |

| admit_airgap_result | function | admit_airgap_result(bundle: &AirgapBundle, result: &AirgapResult) |  |  |  |  |

| manufacture_airgap_bundle | function | manufacture_airgap_bundle(
    bundle_id: String,
    constitution: &GlobalConstitution,
    o_star_snapshot: Value,
    construct_graph: Value,
    prohibited_goals: Value,
) |  |  |  |  |

| AirgapAdmission | struct |  |  |  |  |  |

| AirgapBundle | struct |  |  |  |  |  |

| AirgapResult | struct |  |  |  |  |  |

| execute_command_process | function | execute_command_process(
    process: &PowlProcess,
    state: &WorldState,
    envelope: &TestEnvelope,
    admission: &ConstructAdmission,
    policy: CommandAdapterPolicy,
    blake3: &dyn Blake3Provider,
    signer: &dyn ReceiptSigner,
    now: impl Fn() |  |  |  |  |

| execute_command_process_durable | function | execute_command_process_durable(
    process: &PowlProcess,
    state: &WorldState,
    envelope: &TestEnvelope,
    admission: &ConstructAdmission,
    policy: CommandAdapterPolicy,
    durable_journal_root: impl AsRef<Path>,
    blake3: &dyn Blake3Provider,
    signer: &dyn ReceiptSigner,
    now: impl Fn() |  |  |  |  |

| fortune5_adapter_catalog | function | fortune5_adapter_catalog() |  |  |  |  |

| journal | function | journal(&self) |  |  |  |  |

| new | function | new(inner: &'a dyn GymActAdapter, blake3: &'a dyn Blake3Provider, signer: &'a dyn ReceiptSigner) |  |  |  |  |

| new_durable | function | new_durable(
        inner: &'a dyn GymActAdapter,
        blake3: &'a dyn Blake3Provider,
        signer: &'a dyn ReceiptSigner,
        durable_journal_root: PathBuf,
    ) |  |  |  |  |

| validate_command_adapter_policy | function | validate_command_adapter_policy(policy: &CommandAdapterPolicy) |  |  |  |  |

| BrceGymActAdapter | struct |  |  |  |  |  |

| BrceTransitionRecord | struct |  |  |  |  |  |

| CommandAdapterPolicy | struct |  |  |  |  |  |

| CommandSpec | struct |  |  |  |  |  |

| DurableBrceOutcomeRecord | struct |  |  |  |  |  |

| DurableBrcePrepareRecord | struct |  |  |  |  |  |

| DurableBrceReceipt | struct |  |  |  |  |  |

| ProviderAdapterDescriptor | struct |  |  |  |  |  |

| ChaosScenario | enum |  |  |  |  |  |

| qualify_chaos | function | qualify_chaos(evidence: &[ChaosEvidence]) |  |  |  |  |

| required_chaos_scenarios | function | required_chaos_scenarios() |  |  |  |  |

| ChaosEvidence | struct |  |  |  |  |  |

| ChaosQualification | struct |  |  |  |  |  |

| SignatureSuite | enum |  |  |  |  |  |

| as_str | function | as_str(self) |  |  |  |  |

| dual_artifact_identity | function | dual_artifact_identity(bytes: &[u8]) |  |  |  |  |

| implemented_signature_suites | function | implemented_signature_suites() |  |  |  |  |

| qualify_crypto_profile | function | qualify_crypto_profile(profile: &CryptoProfile) |  |  |  |  |

| qualify_pqc_runtime | function | qualify_pqc_runtime() |  |  |  |  |

| sign_pqc_message | function | sign_pqc_message(
    suite: SignatureSuite,
    seed: [u8; 32],
    message: &[u8],
) |  |  |  |  |

| verify_pqc_message | function | verify_pqc_message(proof: &PqcSignatureProof, message: &[u8]) |  |  |  |  |

| ArtifactIdentity | struct |  |  |  |  |  |

| CryptoProfile | struct |  |  |  |  |  |

| CryptoQualification | struct |  |  |  |  |  |

| PqcRuntimeQualification | struct |  |  |  |  |  |

| PqcSignatureProof | struct |  |  |  |  |  |

| ProbePurpose | enum |  |  |  |  |  |

| ReadOnlyProbeKind | enum |  |  |  |  |  |

| args | function | args(self) |  |  |  |  |

| bind_command_policy | function | bind_command_policy(
    binding: &AdapterBinding,
    policy: &CommandAdapterPolicy,
) |  |  |  |  |

| default_program | function | default_program(self) |  |  |  |  |

| live_observation_index | function | live_observation_index(
    observations: &[ProviderProbeObservation],
) |  |  |  |  |

| manufacture_live_probe_plan | function | manufacture_live_probe_plan(manifest: &DeploymentManifest) |  |  |  |  |

| provider_kind | function | provider_kind(self) |  |  |  |  |

| purpose | function | purpose(self) |  |  |  |  |

| qualify_dfcm_closure | function | qualify_dfcm_closure(
    manifest: &DeploymentManifest,
    observations: &[ProviderProbeObservation],
    crypto_profile: &CryptoProfile,
    now_epoch_ms: i64,
    max_live_evidence_age_ms: i64,
) |  |  |  |  |

| qualify_live_deployment | function | qualify_live_deployment(
    manifest: &DeploymentManifest,
    observations: &[ProviderProbeObservation],
    now_epoch_ms: i64,
    max_age_ms: i64,
) |  |  |  |  |

| qualify_protocol_fence | function | qualify_protocol_fence() |  |  |  |  |

| run_read_only_probe | function | run_read_only_probe(
    spec: &ReadOnlyProbeSpec,
    observed_at_epoch_ms: i64,
) |  |  |  |  |

| DfcmClosureQualification | struct |  |  |  |  |  |

| LiveDeploymentQualification | struct |  |  |  |  |  |

| ProtocolDispatchSummary | struct |  |  |  |  |  |

| ProtocolFenceQualification | struct |  |  |  |  |  |

| ProviderProbeObservation | struct |  |  |  |  |  |

| ReadOnlyProbeSpec | struct |  |  |  |  |  |

| persist_evidence | function | persist_evidence(root: impl AsRef<Path>, record: &DurableEvidenceRecord) |  |  |  |  |

| verify_evidence_file | function | verify_evidence_file(path: impl AsRef<Path>) |  |  |  |  |

| DurableEvidenceRecord | struct |  |  |  |  |  |

| EvidenceCommit | struct |  |  |  |  |  |

| EvidenceVerification | struct |  |  |  |  |  |

| IntentMode | enum |  |  |  |  |  |

| InterfaceOrigin | enum |  |  |  |  |  |

| a2a_agent_card | function | a2a_agent_card() |  |  |  |  |

| admit_interface_intent | function | admit_interface_intent(intent: &InterfaceIntent) |  |  |  |  |

| admit_observation | function | admit_observation(
    observation: &ObservationEnvelope,
    allowed_sources: &BTreeSet<String>,
    now_epoch_ms: i64,
    max_age_ms: i64,
) |  |  |  |  |

| as_str | function | as_str(self) |  |  |  |  |

| dispatch_interface_intent | function | dispatch_interface_intent(intent: &InterfaceIntent) |  |  |  |  |

| mcp_tool_catalog | function | mcp_tool_catalog() |  |  |  |  |

| A2aAgentCard | struct |  |  |  |  |  |

| AdmittedObservation | struct |  |  |  |  |  |

| InterfaceAdmission | struct |  |  |  |  |  |

| InterfaceIntent | struct |  |  |  |  |  |

| McpToolDescriptor | struct |  |  |  |  |  |

| ObservationEnvelope | struct |  |  |  |  |  |

| ProtocolDispatch | struct |  |  |  |  |  |

| admit_receipt_checkpoint | function | admit_receipt_checkpoint(state: &mut ReplicaState, checkpoint: ReceiptCheckpoint) |  |  |  |  |

| load_durable_replica | function | load_durable_replica(path: impl AsRef<Path>) |  |  |  |  |

| persist_receipt_checkpoint | function | persist_receipt_checkpoint(
    path: impl AsRef<Path>,
    receiver_id: &str,
    checkpoint: ReceiptCheckpoint,
) |  |  |  |  |

| reconcile_transition | function | reconcile_transition(evidence: &ReconciliationEvidence) |  |  |  |  |

| DurableReplicaCommit | struct |  |  |  |  |  |

| ReceiptCheckpoint | struct |  |  |  |  |  |

| ReconciliationDecision | struct |  |  |  |  |  |

| ReconciliationEvidence | struct |  |  |  |  |  |

| ReplicaState | struct |  |  |  |  |  |

| ReplicationAdmission | struct |  |  |  |  |  |

| decode_seed_hex | function | decode_seed_hex(value: &str) |  |  |  |  |

| execute_runtime_request | function | execute_runtime_request(
    request: &RuntimeExecutionRequest,
    key_id: String,
    seed: [u8; 32],
    expected_construct_digest: &str,
    now_epoch_ms: i64,
) |  |  |  |  |

| from_seed | function | from_seed(key_id: String, seed: [u8; 32]) |  |  |  |  |

| manufacture_runtime_construct | function | manufacture_runtime_construct(
    request: &RuntimeExecutionRequest,
    key_id: String,
    seed: [u8; 32],
) |  |  |  |  |

| to_envelope | function | to_envelope(&self) |  |  |  |  |

| to_powl | function | to_powl(&self) |  |  |  |  |

| verifier | function | verifier(&self) |  |  |  |  |

| ConstructManufactureSummary | struct |  |  |  |  |  |

| Ed25519RuntimeSigner | struct |  |  |  |  |  |

| Ed25519RuntimeVerifier | struct |  |  |  |  |  |

| NativeBlake3 | struct |  |  |  |  |  |

| PortableActivity | struct |  |  |  |  |  |

| PortableEnvelope | struct |  |  |  |  |  |

| PortableProcess | struct |  |  |  |  |  |

| RuntimeDoSummary | struct |  |  |  |  |  |

| RuntimeExecutionRequest | struct |  |  |  |  |  |

| CloudProvider | enum |  |  |  |  |  |

| ReleaseStanding | enum |  |  |  |  |  |

| aggregate_global_standing | function | aggregate_global_standing(mut cells: Vec<CellStandingRow>) |  |  |  |  |

| alive | function | alive(&self) |  |  |  |  |

| as_str | function | as_str(self) |  |  |  |  |

| qualify_deployment | function | qualify_deployment(manifest: &DeploymentManifest, now_epoch_ms: i64) |  |  |  |  |

| AdapterBinding | struct |  |  |  |  |  |

| CastleCellManifest | struct |  |  |  |  |  |

| CellStandingRow | struct |  |  |  |  |  |

| DeploymentManifest | struct |  |  |  |  |  |

| DeploymentQualification | struct |  |  |  |  |  |

| GlobalConstitution | struct |  |  |  |  |  |

| GlobalStanding | struct |  |  |  |  |  |

| ProtocolSurface | struct |  |  |  |  |  |

| EvidenceStanding | enum |  |  |  |  |  |

| WitnessKind | enum |  |  |  |  |  |

| admit_external_witness | function | admit_external_witness(
    manifest: &EcosystemManifest,
    witness: &ExternalWitness,
) |  |  |  |  |

| admit_fond_differential | function | admit_fond_differential(check: &FondDifferentialCheck) |  |  |  |  |

| admit_independent_plan | function | admit_independent_plan(check: &IndependentPlanCheck) |  |  |  |  |

| admit_sa2a_replan_envelope | function | admit_sa2a_replan_envelope(
    expected_subject: &str,
    envelope: &PortableReplanEnvelope,
) |  |  |  |  |

| bind_v26_9_28_construct_request | function | bind_v26_9_28_construct_request(
    mut request: crate::castle::ConstructRequest,
    manifest: &EcosystemManifest,
    witnesses: &[ExternalWitness],
) |  |  |  |  |

| ecosystem_manifest | function | ecosystem_manifest() |  |  |  |  |

| id | function | id(self) |  |  |  |  |

| is_alive | function | is_alive(&self) |  |  |  |  |

| qualify_portable_runtime | function | qualify_portable_runtime(witnesses: &[PortableRuntimeWitness]) |  |  |  |  |

| qualify_v26_9_28_upgrade | function | qualify_v26_9_28_upgrade(manifest: &EcosystemManifest) |  |  |  |  |

| route_edge_local_recovery | function | route_edge_local_recovery(
    subject: &str,
    ordered_providers: &[String],
    failed_providers: &BTreeSet<String>,
) |  |  |  |  |

| EcosystemManifest | struct |  |  |  |  |  |

| ExcludedPr | struct |  |  |  |  |  |

| ExternalWitness | struct |  |  |  |  |  |

| FondDifferentialCheck | struct |  |  |  |  |  |

| IndependentPlanCheck | struct |  |  |  |  |  |

| MarketplacePack | struct |  |  |  |  |  |

| PortableReplanDecision | struct |  |  |  |  |  |

| PortableReplanEnvelope | struct |  |  |  |  |  |

| PortableRuntimeWitness | struct |  |  |  |  |  |

| RecoveryDecision | struct |  |  |  |  |  |

| ReviewWindow | struct |  |  |  |  |  |

| SourceSubject | struct |  |  |  |  |  |

| UpgradeQualification | struct |  |  |  |  |  |

| WitnessLimits | struct |  |  |  |  |  |

| admission_ctx | function | admission_ctx(&self) |  |  |  |  |

| admit | function | admit(&self, effect: PreparedEffect, nonce: &str) |  |  |  |  |

| cert | function | cert(&self, effect: &PreparedEffect, nonce: &str, signers: &[&str]) |  |  |  |  |

| effect | function | effect(&self, amount: &str, obligation: &str) |  |  |  |  |

| exec_ctx | function | exec_ctx(&self) |  |  |  |  |

| new | function | new(tag: &str) |  |  |  |  |

| token_for | function | token_for(&self, admission: &PaymentAdmission) |  |  |  |  |

| unique_dir | function | unique_dir(tag: &str) |  |  |  |  |

| with | function | with(tag: &str, treasury_minor: u64, per_effect_cap: u64, epoch_cap: u64) |  |  |  |  |

| Fixture | struct |  |  |  |  |  |

| RealBlake3 | struct |  |  |  |  |  |

| ReceiptCheck | struct |  |  |  |  |  |

| ReceiptKey | struct |  |  |  |  |  |

| authoritative | function | authoritative(mut self) |  |  |  |  |

| new | function | new(script: Vec<RailStatus>) |  |  |  |  |

| push | function | push(&self, s: RailStatus) |  |  |  |  |

| rejected | function | rejected() |  |  |  |  |

| returned | function | returned() |  |  |  |  |

| settled | function | settled() |  |  |  |  |

| ScriptedRail | struct |  |  |  |  |  |

| PaymentAdmission | struct |  |  |  |  |  |

| admit_submitted | function | admit_submitted(fx: &Fixture, amount: &str, obligation: &str, nonce: &str) |  |  |  |  |


<!-- ============================================================= -->
<!-- AGENT-FORBIDDEN-END: nothing below this line may describe     -->
<!-- code behavior.                                                -->
<!-- ============================================================= -->
