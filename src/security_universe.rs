//! Receipt-bound admission calculus over the ggen-generated security universe.
//!
//! Identity/topology is generated from `ggen-marketplace/castle-pack`. This file
//! contains only runtime admission, qualification, extension, and refusal logic.
//! Registry membership is not certification; relation adjacency is not equivalence;
//! tools manufacture evidence/intents but receive no ambient CASTLE DO authority.

use crate::fortune5;
use crate::security_core_generated::FORTUNE5_SECURITY_CORE;
use crate::security_sources_generated::{SecuritySource, SECURITY_SOURCES};
use crate::security_tools_generated::{SecurityToolIntegration, SECURITY_TOOLS};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingRelation {
    Related,
    InformativeReference,
    Narrows,
    Broadens,
    Implements,
    Assesses,
    Mitigates,
    Detects,
    Observes,
    Evidences,
    Translates,
    Contradicts,
    Equivalent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecurityMapping<'a> {
    pub left_source_id: &'a str,
    pub left_object_id: &'a str,
    pub right_source_id: &'a str,
    pub right_object_id: &'a str,
    pub relation: MappingRelation,
    pub equivalence_proof_receipt: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmittedSecurityMapping<'a> {
    pub mapping: SecurityMapping<'a>,
}

pub fn admit_security_mapping<'a>(
    mapping: SecurityMapping<'a>,
) -> Result<AdmittedSecurityMapping<'a>, &'static str> {
    if find_source(mapping.left_source_id).is_none() || find_source(mapping.right_source_id).is_none() {
        return Err("REFUSED:UNKNOWN_SECURITY_SOURCE");
    }
    if mapping.left_object_id.trim().is_empty() || mapping.right_object_id.trim().is_empty() {
        return Err("REFUSED:EMPTY_SECURITY_OBJECT_ID");
    }
    if mapping.relation == MappingRelation::Equivalent {
        let Some(proof) = mapping.equivalence_proof_receipt else {
            return Err("REFUSED:UNPROVEN_EQUIVALENCE");
        };
        if !is_digest(proof) {
            return Err("REFUSED:INVALID_EQUIVALENCE_PROOF_RECEIPT");
        }
    }
    Ok(AdmittedSecurityMapping { mapping })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoverageEvidence<'a> {
    pub source_id: &'a str,
    pub source_digest: &'a str,
    pub imported_objects: u64,
    pub mapped_objects: u64,
    pub verified_objects: u64,
    pub receipt_digest: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityStanding {
    Unknown,
    PartialAlive,
    Alive,
    Refused,
}

impl SecurityStanding {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            SecurityStanding::Unknown => "UNKNOWN",
            SecurityStanding::PartialAlive => "PARTIAL_ALIVE",
            SecurityStanding::Alive => "ALIVE",
            SecurityStanding::Refused => "REFUSED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityQualification {
    pub standing: SecurityStanding,
    pub required_sources: usize,
    pub observed_sources: usize,
    pub fully_verified_sources: usize,
    pub missing_sources: Vec<&'static str>,
    pub partial_sources: Vec<&'static str>,
}

/// Qualify exact, receipt-bound coverage of the generated security core.
/// This manufactures only a machine standing; it cannot assert external
/// certification, compliance, accreditation, or legal sufficiency.
pub fn qualify_fortune5_security_universe(
    evidence: &[CoverageEvidence<'_>],
) -> Result<SecurityQualification, &'static str> {
    validate_security_catalog()?;
    validate_tool_catalog()?;

    for (index, item) in evidence.iter().enumerate() {
        if find_source(item.source_id).is_none() {
            return Err("REFUSED:UNKNOWN_SECURITY_SOURCE");
        }
        if !is_digest(item.source_digest) {
            return Err("REFUSED:INVALID_SOURCE_DIGEST");
        }
        if item.mapped_objects > item.imported_objects || item.verified_objects > item.mapped_objects {
            return Err("REFUSED:IMPOSSIBLE_COVERAGE_COUNTS");
        }
        if item.imported_objects > 0 && item.receipt_digest.is_none() {
            return Err("REFUSED:UNRECEIPTED_SECURITY_EVIDENCE");
        }
        if let Some(receipt) = item.receipt_digest {
            if !is_digest(receipt) {
                return Err("REFUSED:INVALID_SECURITY_RECEIPT");
            }
        }
        if evidence[index + 1..]
            .iter()
            .any(|other| other.source_id == item.source_id)
        {
            return Err("REFUSED:DUPLICATE_SECURITY_SOURCE_EVIDENCE");
        }
    }

    let mut missing_sources = Vec::new();
    let mut partial_sources = Vec::new();
    let mut fully_verified_sources = 0usize;

    for required in FORTUNE5_SECURITY_CORE {
        match evidence.iter().find(|item| item.source_id == *required) {
            None => missing_sources.push(*required),
            Some(item) if item.imported_objects == 0 => partial_sources.push(*required),
            Some(item) if item.verified_objects != item.imported_objects => partial_sources.push(*required),
            Some(_) => fully_verified_sources += 1,
        }
    }

    let observed_sources = FORTUNE5_SECURITY_CORE.len() - missing_sources.len();
    let standing = if !missing_sources.is_empty() {
        SecurityStanding::Unknown
    } else if !partial_sources.is_empty() {
        SecurityStanding::PartialAlive
    } else {
        SecurityStanding::Alive
    };

    Ok(SecurityQualification {
        standing,
        required_sources: FORTUNE5_SECURITY_CORE.len(),
        observed_sources,
        fully_verified_sources,
        missing_sources,
        partial_sources,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FederatedFortune5Qualification {
    pub standing: SecurityStanding,
    pub base_standing: fortune5::Standing,
    pub security: SecurityQualification,
}

/// Compose CASTLE's generated Fortune-5 runtime gate with the federated
/// security-evidence gate. Neither side can crown the other.
pub fn qualify_federated_fortune5(
    base: &fortune5::Fortune5Qualification,
    evidence: &[CoverageEvidence<'_>],
) -> Result<FederatedFortune5Qualification, &'static str> {
    let security = qualify_fortune5_security_universe(evidence)?;
    let standing = match (base.standing, security.standing) {
        (fortune5::Standing::Refused, _) | (_, SecurityStanding::Refused) => SecurityStanding::Refused,
        (fortune5::Standing::Unknown, _) | (_, SecurityStanding::Unknown) => SecurityStanding::Unknown,
        (fortune5::Standing::Alive, SecurityStanding::PartialAlive) => SecurityStanding::PartialAlive,
        (fortune5::Standing::Alive, SecurityStanding::Alive) => SecurityStanding::Alive,
    };
    Ok(FederatedFortune5Qualification {
        standing,
        base_standing: base.standing,
        security,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityIntent<'a> {
    pub tool_id: &'a str,
    pub target_subject: &'a str,
    pub operation: &'a str,
    pub parent_evidence_digest: &'a str,
    pub direct_actuation_authority: bool,
}

/// Manufacture an inert security intent from a known generated tool identity.
/// No execution function exists on this type; consequential work must re-enter
/// CASTLE's normal admission/BRCE path.
pub fn manufacture_security_intent<'a>(
    tool_id: &'a str,
    target_subject: &'a str,
    operation: &'a str,
    parent_evidence_digest: &'a str,
) -> Result<SecurityIntent<'a>, &'static str> {
    if find_tool(tool_id).is_none() {
        return Err("REFUSED:UNKNOWN_SECURITY_TOOL");
    }
    if target_subject.trim().is_empty() || operation.trim().is_empty() {
        return Err("REFUSED:EMPTY_SECURITY_INTENT");
    }
    if !is_digest(parent_evidence_digest) {
        return Err("REFUSED:INVALID_SECURITY_INTENT_PARENT");
    }
    Ok(SecurityIntent {
        tool_id,
        target_subject,
        operation,
        parent_evidence_digest,
        direct_actuation_authority: false,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceSurface {
    Oscal,
    Stix21,
    Taxii21,
    Ocsf,
    Sarif,
    CycloneDx,
    Spdx,
    Csaf,
    OpenTelemetry,
    Ocel,
    Json,
    JsonLd,
    Rdf,
    Csv,
    Xml,
    Syslog,
    NativeOpaque,
}

/// Receipt-bound extension rail for products not yet named in SECURITY_TOOLS.
/// It admits evidence identity only and deliberately confers no DO authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExternalToolEvidence<'a> {
    pub tool_id: &'a str,
    pub authority: &'a str,
    pub native_version: &'a str,
    pub native_object_id: &'a str,
    pub surface: EvidenceSurface,
    pub adapter_identity_digest: &'a str,
    pub payload_digest: &'a str,
    pub receipt_digest: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmittedExternalToolEvidence<'a> {
    pub evidence: ExternalToolEvidence<'a>,
    pub direct_actuation_authority: bool,
}

pub fn admit_external_tool_evidence<'a>(
    evidence: ExternalToolEvidence<'a>,
) -> Result<AdmittedExternalToolEvidence<'a>, &'static str> {
    if evidence.tool_id.trim().is_empty()
        || evidence.authority.trim().is_empty()
        || evidence.native_version.trim().is_empty()
        || evidence.native_object_id.trim().is_empty()
    {
        return Err("REFUSED:EMPTY_EXTERNAL_TOOL_IDENTITY");
    }
    if !is_digest(evidence.adapter_identity_digest) {
        return Err("REFUSED:INVALID_EXTERNAL_ADAPTER_IDENTITY");
    }
    if !is_digest(evidence.payload_digest) {
        return Err("REFUSED:INVALID_EXTERNAL_TOOL_PAYLOAD");
    }
    if !is_digest(evidence.receipt_digest) {
        return Err("REFUSED:INVALID_EXTERNAL_TOOL_RECEIPT");
    }
    Ok(AdmittedExternalToolEvidence {
        evidence,
        direct_actuation_authority: false,
    })
}

pub fn validate_security_catalog() -> Result<(), &'static str> {
    for (index, source) in SECURITY_SOURCES.iter().enumerate() {
        if source.id.trim().is_empty()
            || source.authority.trim().is_empty()
            || source.source_uri.trim().is_empty()
        {
            return Err("REFUSED:INVALID_SECURITY_SOURCE");
        }
        if SECURITY_SOURCES[index + 1..]
            .iter()
            .any(|other| other.id == source.id)
        {
            return Err("REFUSED:DUPLICATE_SECURITY_SOURCE");
        }
    }
    for id in FORTUNE5_SECURITY_CORE {
        if find_source(id).is_none() {
            return Err("REFUSED:MISSING_FORTUNE5_SECURITY_CORE_SOURCE");
        }
    }
    Ok(())
}

pub fn validate_tool_catalog() -> Result<(), &'static str> {
    for (index, tool) in SECURITY_TOOLS.iter().enumerate() {
        if tool.direct_actuation_authority {
            return Err("REFUSED:TOOL_AMBIENT_DO");
        }
        if SECURITY_TOOLS[index + 1..]
            .iter()
            .any(|other| other.id == tool.id)
        {
            return Err("REFUSED:DUPLICATE_SECURITY_TOOL");
        }
    }
    Ok(())
}

#[must_use]
pub fn find_source(id: &str) -> Option<&'static SecuritySource> {
    SECURITY_SOURCES.iter().find(|source| source.id == id)
}

#[must_use]
pub fn find_tool(id: &str) -> Option<&'static SecurityToolIntegration> {
    SECURITY_TOOLS.iter().find(|tool| tool.id == id)
}

fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";

    fn complete_core_evidence() -> Vec<CoverageEvidence<'static>> {
        FORTUNE5_SECURITY_CORE
            .iter()
            .map(|id| CoverageEvidence {
                source_id: id,
                source_digest: DIGEST,
                imported_objects: 1,
                mapped_objects: 1,
                verified_objects: 1,
                receipt_digest: Some(DIGEST),
            })
            .collect()
    }

    #[test]
    fn generated_catalogs_are_valid_and_tool_bridges_have_no_ambient_do() {
        assert_eq!(FORTUNE5_SECURITY_CORE.len(), 22);
        assert_eq!(validate_security_catalog(), Ok(()));
        assert_eq!(validate_tool_catalog(), Ok(()));
        assert!(SECURITY_TOOLS.iter().all(|tool| !tool.direct_actuation_authority));
    }

    #[test]
    fn equivalence_requires_a_receipted_proof() {
        let mapping = SecurityMapping {
            left_source_id: "nist-csf",
            left_object_id: "GV.OC-01",
            right_source_id: "cis-controls",
            right_object_id: "1.1",
            relation: MappingRelation::Equivalent,
            equivalence_proof_receipt: None,
        };
        assert_eq!(
            admit_security_mapping(mapping),
            Err("REFUSED:UNPROVEN_EQUIVALENCE")
        );
    }

    #[test]
    fn adjacency_can_be_admitted_without_being_promoted_to_equivalence() {
        let mapping = SecurityMapping {
            left_source_id: "nist-csf",
            left_object_id: "GV.OC-01",
            right_source_id: "cis-controls",
            right_object_id: "1.1",
            relation: MappingRelation::Related,
            equivalence_proof_receipt: None,
        };
        assert_eq!(
            admit_security_mapping(mapping)
                .expect("related mapping should admit")
                .mapping
                .relation,
            MappingRelation::Related
        );
    }

    #[test]
    fn missing_core_evidence_is_unknown_not_alive() {
        let qualification = qualify_fortune5_security_universe(&[]).expect("catalog is valid");
        assert_eq!(qualification.standing, SecurityStanding::Unknown);
        assert_eq!(qualification.observed_sources, 0);
        assert_eq!(qualification.required_sources, 22);
    }

    #[test]
    fn complete_receipted_core_evidence_is_alive() {
        let qualification = qualify_fortune5_security_universe(&complete_core_evidence())
            .expect("complete evidence should qualify");
        assert_eq!(qualification.standing, SecurityStanding::Alive);
        assert_eq!(qualification.fully_verified_sources, 22);
        assert!(qualification.missing_sources.is_empty());
        assert!(qualification.partial_sources.is_empty());
    }

    #[test]
    fn incomplete_verified_counts_are_partial_alive() {
        let mut evidence = complete_core_evidence();
        evidence[0].verified_objects = 0;
        let qualification = qualify_fortune5_security_universe(&evidence)
            .expect("partial coverage is a standing, not a transport failure");
        assert_eq!(qualification.standing, SecurityStanding::PartialAlive);
        assert_eq!(qualification.partial_sources.len(), 1);
    }

    #[test]
    fn duplicate_evidence_is_refused() {
        let duplicate = CoverageEvidence {
            source_id: "nist-csf",
            source_digest: DIGEST,
            imported_objects: 1,
            mapped_objects: 1,
            verified_objects: 1,
            receipt_digest: Some(DIGEST),
        };
        assert_eq!(
            qualify_fortune5_security_universe(&[duplicate, duplicate]),
            Err("REFUSED:DUPLICATE_SECURITY_SOURCE_EVIDENCE")
        );
    }

    #[test]
    fn known_tools_manufacture_intents_without_do_authority() {
        let intent = manufacture_security_intent(
            "aws-security-hub",
            "system:owned",
            "normalize-finding",
            DIGEST,
        )
        .expect("generated tool identity is known");
        assert!(!intent.direct_actuation_authority);
    }

    #[test]
    fn external_tool_extension_admits_identity_without_do_authority() {
        let admitted = admit_external_tool_evidence(ExternalToolEvidence {
            tool_id: "future-tool",
            authority: "future-authority",
            native_version: "1",
            native_object_id: "finding:1",
            surface: EvidenceSurface::NativeOpaque,
            adapter_identity_digest: DIGEST,
            payload_digest: DIGEST,
            receipt_digest: DIGEST,
        })
        .expect("valid evidence identity should admit");
        assert!(!admitted.direct_actuation_authority);
    }
}
