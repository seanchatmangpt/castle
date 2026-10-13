# CASTLE Fortune-5 Readiness Controls

This file is generated from `ggen-marketplace/packs/castle-pack/ontology.ttl`.

The profile is an internal executable readiness gate, not a claim of external certification. Every control is evaluated from receipted evidence; missing evidence remains `UNKNOWN`, failed evidence is `REFUSED`, and only complete passing evidence is `ALIVE`.

| Order | Control | Category | Metric | Gate | Authority | Requirement |
|---:|---|---|---|---|---|---|
| 200 | `F5-AUTH-001` | authority | `zero_unreceipted_actuations` | `EQ 0` | `REQUIRED` | Consequential actuation has zero unreceipted transitions. |
| 210 | `F5-AUTH-002` | authority | `construct_actuation_authority` | `EQ false` | `REQUIRED` | CONSTRUCT has no consequential actuation authority. |
| 220 | `F5-IAM-001` | isolation | `cross_tenant_authority_edges` | `EQ 0` | `REQUIRED` | No unauthorized cross-tenant authority edges are admitted. |
| 230 | `F5-IAM-002` | isolation | `unreceipted_breakglass_events` | `EQ 0` | `REQUIRED` | Emergency authority transitions remain fully receipted. |
| 240 | `F5-SC-001` | supply-chain | `sbom_coverage_bps` | `GTE 10000` | `REQUIRED` | Deployable artifacts have complete software-bill-of-material coverage. |
| 250 | `F5-SC-002` | supply-chain | `signed_provenance_coverage_bps` | `GTE 10000` | `REQUIRED` | Deployable artifacts have signed provenance coverage. |
| 260 | `F5-SC-003` | supply-chain | `content_addressed_dependency_coverage_bps` | `GTE 10000` | `REQUIRED` | Admitted dependencies are content addressed. |
| 270 | `F5-REPLAY-001` | replay | `replay_version_compatible` | `EQ true` | `REQUIRED` | Replay classes match the current ontology and provider-semantics versions. |
| 280 | `F5-REPLAY-002` | replay | `replay_invariant_pass_bps` | `GTE 10000` | `REQUIRED` | Replay preconditions and invariants pass for every admitted reuse. |
| 290 | `F5-OBS-001` | observability | `ocel_actuation_coverage_bps` | `GTE 10000` | `REQUIRED` | Consequential transitions are represented in OCEL evidence. |
| 300 | `F5-OBS-002` | observability | `receipt_actuation_coverage_bps` | `GTE 10000` | `REQUIRED` | Consequential transitions are covered by cryptographic receipts. |
| 310 | `F5-DR-001` | resilience | `restore_drill_passed` | `EQ true` | `REQUIRED` | A restore drill has produced passing observed evidence. |
| 320 | `F5-DR-002` | resilience | `observed_rpo_seconds` | `LTE 300` | `REQUIRED` | Observed recovery point objective is at most five minutes. |
| 330 | `F5-DR-003` | resilience | `observed_rto_seconds` | `LTE 900` | `REQUIRED` | Observed recovery time objective is at most fifteen minutes. |
| 340 | `F5-HA-001` | availability | `independent_failure_domains` | `GTE 3` | `REQUIRED` | The admitted deployment spans at least three independent failure domains. |
| 350 | `F5-SLO-001` | slo | `availability_bps` | `GTE 9990` | `REQUIRED` | Observed service availability meets the default 99.90 percent readiness gate. |
| 360 | `F5-SLO-002` | slo | `p99_control_plane_latency_ms` | `LTE 500` | `REQUIRED` | Observed p99 control-plane latency is at most 500 milliseconds. |
| 370 | `F5-DATA-001` | data | `encryption_coverage_bps` | `GTE 10000` | `REQUIRED` | Protected data has complete encryption coverage across admitted storage and transport. |
| 380 | `F5-DATA-002` | data | `residency_policy_conformant` | `EQ true` | `REQUIRED` | Observed placement conforms to the admitted data residency policy. |
| 390 | `F5-POL-001` | policy | `deny_by_default` | `EQ true` | `REQUIRED` | Unspecified authority is denied by default. |
| 400 | `F5-POL-002` | policy | `policy_exception_receipt_coverage_bps` | `GTE 10000` | `REQUIRED` | Every admitted policy exception is receipt bound. |
| 410 | `F5-SCALE-001` | scale | `tested_concurrent_subjects` | `GTE 10000` | `REQUIRED` | The bounded runtime has observed qualification at ten thousand concurrent subjects. |
| 420 | `F5-DET-001` | determinism | `deterministic_replay_success_bps` | `GTE 9999` | `REQUIRED` | Deterministic replay succeeds for at least 99.99 percent of admitted replay attempts. |
| 430 | `F5-EVID-001` | evidence | `evidence_retention_days` | `GTE 365` | `REQUIRED` | Evidence retention is at least 365 days unless a stricter admitted profile applies. |
| 440 | `F5-EVID-002` | evidence | `tamper_evident_evidence` | `EQ true` | `REQUIRED` | Persisted evidence is tamper evident. |
| 450 | `F5-REFUSE-001` | refusal | `typed_refusal_coverage_bps` | `GTE 10000` | `REQUIRED` | All denied consequential actions return typed refusal standing. |
| 460 | `F5-CHAOS-001` | resilience | `failure_injection_suite_passed` | `EQ true` | `REQUIRED` | The admitted failure-injection suite passes with observed evidence. |
| 470 | `F5-ROLLBACK-001` | change | `rollback_path_coverage_bps` | `GTE 10000` | `REQUIRED` | Every consequential change class has an admitted rollback path or proven irreversible exception. |
| 480 | `F5-AIRGAP-001` | air-gap | `construct_network_dependency_count` | `EQ 0` | `REQUIRED` | The CONSTRUCT capsule has zero network dependencies. |
| 490 | `F5-AIRGAP-002` | air-gap | `construct_secret_dependency_count` | `EQ 0` | `REQUIRED` | The CONSTRUCT capsule has zero secret dependencies. |
| 500 | `F5-ADV-001` | adversarial-coverage | `prohibited_goal_coverage_bps` | `GTE 10000` | `REQUIRED` | Every admitted prohibited-goal class is covered by a winning-region proof, replay class, or preventive invariant. |
| 510 | `F5-ADV-002` | adversarial-coverage | `losing_region_regression_passed` | `EQ true` | `REQUIRED` | Losing-region regression checks pass against the admitted transition relation. |
| 520 | `F5-DATA-003` | data | `backup_encryption_coverage_bps` | `GTE 10000` | `REQUIRED` | Backup artifacts have complete encryption coverage. |
| 530 | `F5-IAM-003` | isolation | `privileged_identity_mfa_coverage_bps` | `GTE 10000` | `REQUIRED` | Privileged identities satisfy the admitted strong-authentication policy. |
| 540 | `F5-CLOCK-001` | evidence | `evidence_clock_skew_ms` | `LTE 1000` | `REQUIRED` | Observed evidence clock skew is at most one second. |
| 550 | `F5-CHANGE-001` | change | `immutable_change_receipt_coverage_bps` | `GTE 10000` | `REQUIRED` | Consequential change records are immutable and receipt bound. |
| 560 | `F5-CHANGE-002` | change | `canary_or_equivalent_validation_coverage_bps` | `GTE 10000` | `REQUIRED` | Consequential changes pass an admitted bounded validation stage before full rollout. |
| 570 | `F5-REGION-001` | resilience | `region_failover_drill_passed` | `EQ true` | `REQUIRED` | A region-failover drill has produced passing observed evidence when the deployment is multi-region. |
| 580 | `F5-QUOTA-001` | scale | `quota_headroom_bps` | `GTE 2000` | `REQUIRED` | Critical quota headroom is at least twenty percent at the qualified load. |
| 590 | `F5-OWN-001` | authority | `owned_or_authorized_subject_coverage_bps` | `GTE 10000` | `REQUIRED` | Every GymAct execution subject is owned or explicitly authorized. |


## Standing law

`ALIVE` requires every control to have valid receipted evidence and satisfy its comparator. Missing observations do not pass by default. Consequential actuation remains BRCE-bound even when the readiness profile is `ALIVE`.
