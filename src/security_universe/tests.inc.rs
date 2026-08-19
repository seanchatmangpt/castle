#[cfg(test)]
mod tests {
    use super::*;

    const D: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn catalog_is_large_cross_domain_and_self_consistent() {
        validate_security_catalog().unwrap();
        validate_tool_catalog().unwrap();
        assert!(SECURITY_SOURCES.len() >= 80, "expected broad security-source universe");
        assert!(SECURITY_TOOLS.len() >= 55, "expected broad tool-integration universe");
        for kind in [
            SecurityKind::GovernanceFramework,
            SecurityKind::ControlCatalog,
            SecurityKind::Regulation,
            SecurityKind::CloudBaseline,
            SecurityKind::ApplicationSecurity,
            SecurityKind::AiSecurity,
            SecurityKind::IndustrialControl,
            SecurityKind::ThreatKnowledge,
            SecurityKind::VulnerabilityKnowledge,
            SecurityKind::DetectionLanguage,
            SecurityKind::TelemetrySchema,
            SecurityKind::SupplyChain,
            SecurityKind::EvidenceExchange,
            SecurityKind::SecurityOntology,
        ] {
            assert!(SECURITY_SOURCES.iter().any(|source| source.kind == kind));
        }
    }

    #[test]
    fn fortune5_cloud_ecosystems_are_represented_without_ambient_do() {
        for ecosystem in ["AWS", "Microsoft Azure", "Google Cloud", "Oracle Cloud", "IBM", "SAP", "Salesforce"] {
            let tools: Vec<_> = SECURITY_TOOLS.iter().filter(|tool| tool.ecosystem == ecosystem).collect();
            assert!(!tools.is_empty(), "missing ecosystem {ecosystem}");
            assert!(tools.iter().all(|tool| !tool.direct_actuation_authority));
        }
    }

    #[test]
    fn equivalence_requires_receipted_proof() {
        let refused = admit_security_mapping(SecurityMapping {
            left_source_id: "nist-sp-800-53",
            left_object_id: "AC-family-object",
            right_source_id: "iso-27001",
            right_object_id: "reference-object",
            relation: MappingRelation::Equivalent,
            equivalence_proof_receipt: None,
        });
        assert_eq!(refused.unwrap_err(), "REFUSED:UNPROVEN_EQUIVALENCE");

        let admitted = admit_security_mapping(SecurityMapping {
            left_source_id: "nist-sp-800-53",
            left_object_id: "AC-family-object",
            right_source_id: "iso-27001",
            right_object_id: "reference-object",
            relation: MappingRelation::Equivalent,
            equivalence_proof_receipt: Some(D),
        });
        assert!(admitted.is_ok());
    }

    #[test]
    fn partial_evidence_cannot_be_crowned_alive() {
        let evidence: Vec<_> = FORTUNE5_SECURITY_CORE
            .iter()
            .map(|id| CoverageEvidence {
                source_id: id,
                source_digest: D,
                imported_objects: 10,
                mapped_objects: 10,
                verified_objects: if *id == "mitre-attack" { 9 } else { 10 },
                receipt_digest: Some(D),
            })
            .collect();
        let result = qualify_fortune5_security_universe(&evidence).unwrap();
        assert_eq!(result.standing, SecurityStanding::PartialAlive);
        assert_eq!(result.partial_sources, vec!["mitre-attack"]);
    }

    #[test]
    fn missing_evidence_is_unknown_not_refused() {
        let evidence = [CoverageEvidence {
            source_id: "nist-csf",
            source_digest: D,
            imported_objects: 1,
            mapped_objects: 1,
            verified_objects: 1,
            receipt_digest: Some(D),
        }];
        let result = qualify_fortune5_security_universe(&evidence).unwrap();
        assert_eq!(result.standing, SecurityStanding::Unknown);
        assert!(!result.missing_sources.is_empty());
    }

    #[test]
    fn full_core_evidence_can_be_alive_without_claiming_certification() {
        let evidence: Vec<_> = FORTUNE5_SECURITY_CORE
            .iter()
            .map(|id| CoverageEvidence {
                source_id: id,
                source_digest: D,
                imported_objects: 1,
                mapped_objects: 1,
                verified_objects: 1,
                receipt_digest: Some(D),
            })
            .collect();
        let result = qualify_fortune5_security_universe(&evidence).unwrap();
        assert_eq!(result.standing, SecurityStanding::Alive);
        assert_eq!(result.fully_verified_sources, FORTUNE5_SECURITY_CORE.len());
    }

    #[test]
    fn impossible_counts_and_unreceipted_evidence_are_refused() {
        let impossible = [CoverageEvidence {
            source_id: "nist-csf",
            source_digest: D,
            imported_objects: 1,
            mapped_objects: 2,
            verified_objects: 1,
            receipt_digest: Some(D),
        }];
        assert_eq!(
            qualify_fortune5_security_universe(&impossible).unwrap_err(),
            "REFUSED:IMPOSSIBLE_COVERAGE_COUNTS"
        );

        let unreceipted = [CoverageEvidence {
            source_id: "nist-csf",
            source_digest: D,
            imported_objects: 1,
            mapped_objects: 1,
            verified_objects: 1,
            receipt_digest: None,
        }];
        assert_eq!(
            qualify_fortune5_security_universe(&unreceipted).unwrap_err(),
            "REFUSED:UNRECEIPTED_SECURITY_EVIDENCE"
        );
    }

    #[test]
    fn future_or_proprietary_tools_can_contribute_receipted_evidence_without_do() {
        let admitted = admit_external_tool_evidence(ExternalToolEvidence {
            tool_id: "future-sec-platform",
            authority: "Example Enterprise",
            native_version: "2030.1",
            native_object_id: "finding-42",
            surface: EvidenceSurface::NativeOpaque,
            adapter_identity_digest: D,
            payload_digest: D,
            receipt_digest: D,
        })
        .unwrap();
        assert_eq!(admitted.evidence.tool_id, "future-sec-platform");
        assert!(!admitted.direct_actuation_authority);
    }

    #[test]
    fn tool_output_manufactures_intent_never_do_authority() {
        let intent = manufacture_security_intent("opa-rego", "prod-cell-7", "quarantine-workload", D).unwrap();
        assert!(!intent.direct_actuation_authority);
        assert_eq!(intent.tool_id, "opa-rego");
    }
}
