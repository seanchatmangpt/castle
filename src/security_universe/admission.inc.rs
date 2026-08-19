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

pub fn admit_security_mapping<'a>(mapping: SecurityMapping<'a>) -> Result<AdmittedSecurityMapping<'a>, String> {
    if find_source(mapping.left_source_id).is_none() || find_source(mapping.right_source_id).is_none() {
        return Err("REFUSED:UNKNOWN_SECURITY_SOURCE".to_string());
    }
    if mapping.left_object_id.trim().is_empty() || mapping.right_object_id.trim().is_empty() {
        return Err("REFUSED:EMPTY_SECURITY_OBJECT_ID".to_string());
    }
    if mapping.relation == MappingRelation::Equivalent {
        let Some(proof) = mapping.equivalence_proof_receipt else {
            return Err("REFUSED:UNPROVEN_EQUIVALENCE".to_string());
        };
        if !is_digest(proof) {
            return Err("REFUSED:INVALID_EQUIVALENCE_PROOF_RECEIPT".to_string());
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityQualification {
    pub standing: SecurityStanding,
    pub required_sources: usize,
    pub observed_sources: usize,
    pub fully_verified_sources: usize,
    pub missing_sources: Vec<&'static str>,
    pub partial_sources: Vec<&'static str>,
}

/// Qualifies exact, receipted coverage of the admitted core. This is a
/// machine-standing statement only; it does not claim external certification.
pub fn qualify_fortune5_security_universe(
    evidence: &[CoverageEvidence<'_>],
) -> Result<SecurityQualification, String> {
    validate_security_catalog()?;
    validate_tool_catalog()?;

    for (idx, left) in evidence.iter().enumerate() {
        if find_source(left.source_id).is_none() {
            return Err("REFUSED:UNKNOWN_SECURITY_SOURCE".to_string());
        }
        if !is_digest(left.source_digest) {
            return Err("REFUSED:INVALID_SOURCE_DIGEST".to_string());
        }
        if left.mapped_objects > left.imported_objects || left.verified_objects > left.mapped_objects {
            return Err("REFUSED:IMPOSSIBLE_COVERAGE_COUNTS".to_string());
        }
        if left.imported_objects > 0 && left.receipt_digest.is_none() {
            return Err("REFUSED:UNRECEIPTED_SECURITY_EVIDENCE".to_string());
        }
        if let Some(receipt) = left.receipt_digest {
            if !is_digest(receipt) {
                return Err("REFUSED:INVALID_SECURITY_RECEIPT".to_string());
            }
        }
        if evidence[idx + 1..].iter().any(|right| right.source_id == left.source_id) {
            return Err("REFUSED:DUPLICATE_SECURITY_SOURCE_EVIDENCE".to_string());
        }
    }

    let mut missing = Vec::new();
    let mut partial = Vec::new();
    let mut fully_verified = 0usize;

    for required in FORTUNE5_SECURITY_CORE {
        match evidence.iter().find(|item| item.source_id == *required) {
            None => missing.push(*required),
            Some(item) if item.imported_objects == 0 => partial.push(*required),
            Some(item) if item.verified_objects != item.imported_objects => partial.push(*required),
            Some(_) => fully_verified += 1,
        }
    }

    let observed = FORTUNE5_SECURITY_CORE.len() - missing.len();
    let standing = if !missing.is_empty() {
        SecurityStanding::Unknown
    } else if !partial.is_empty() {
        SecurityStanding::PartialAlive
    } else {
        SecurityStanding::Alive
    };

    Ok(SecurityQualification {
        standing,
        required_sources: FORTUNE5_SECURITY_CORE.len(),
        observed_sources: observed,
        fully_verified_sources: fully_verified,
        missing_sources: missing,
        partial_sources: partial,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FederatedFortune5Qualification {
    pub standing: SecurityStanding,
    pub base_standing: crate::fortune5::Standing,
    pub security: SecurityQualification,
}

/// Compose CASTLE's generated Fortune-5 readiness gate with the federated
/// security-universe evidence gate. Neither side can crown the other.
pub fn qualify_federated_fortune5(
    base: &crate::fortune5::Fortune5Qualification,
    evidence: &[CoverageEvidence<'_>],
) -> Result<FederatedFortune5Qualification, String> {
    let security = qualify_fortune5_security_universe(evidence)?;
    let standing = match (base.standing, security.standing) {
        (crate::fortune5::Standing::Refused, _) | (_, SecurityStanding::Refused) => {
            SecurityStanding::Refused
        }
        (crate::fortune5::Standing::Unknown, _) | (_, SecurityStanding::Unknown) => {
            SecurityStanding::Unknown
        }
        (crate::fortune5::Standing::Alive, SecurityStanding::PartialAlive) => {
            SecurityStanding::PartialAlive
        }
        (crate::fortune5::Standing::Alive, SecurityStanding::Alive) => SecurityStanding::Alive,
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

/// Tool integrations manufacture inert intents only. Consequential execution
/// must still cross CASTLE's existing opaque ConstructAdmission -> BRCE ->
/// GymAct path; this module intentionally has no execution function.
pub fn manufacture_security_intent<'a>(
    tool_id: &'a str,
    target_subject: &'a str,
    operation: &'a str,
    parent_evidence_digest: &'a str,
) -> Result<SecurityIntent<'a>, String> {
    if SECURITY_TOOLS.iter().all(|tool| tool.id != tool_id) {
        return Err("REFUSED:UNKNOWN_SECURITY_TOOL".to_string());
    }
    if target_subject.trim().is_empty() || operation.trim().is_empty() {
        return Err("REFUSED:EMPTY_SECURITY_INTENT".to_string());
    }
    if !is_digest(parent_evidence_digest) {
        return Err("REFUSED:INVALID_SECURITY_INTENT_PARENT".to_string());
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

/// Receipt-bound extension rail for security products not yet named in
/// SECURITY_TOOLS. This admits evidence identity only; it deliberately does
/// not register or execute a tool and confers no CASTLE actuation authority.
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
) -> Result<AdmittedExternalToolEvidence<'a>, String> {
    if evidence.tool_id.trim().is_empty()
        || evidence.authority.trim().is_empty()
        || evidence.native_version.trim().is_empty()
        || evidence.native_object_id.trim().is_empty()
    {
        return Err("REFUSED:EMPTY_EXTERNAL_TOOL_IDENTITY".to_string());
    }
    if !is_digest(evidence.adapter_identity_digest) {
        return Err("REFUSED:INVALID_EXTERNAL_ADAPTER_IDENTITY".to_string());
    }
    if !is_digest(evidence.payload_digest) {
        return Err("REFUSED:INVALID_EXTERNAL_TOOL_PAYLOAD".to_string());
    }
    if !is_digest(evidence.receipt_digest) {
        return Err("REFUSED:INVALID_EXTERNAL_TOOL_RECEIPT".to_string());
    }
    Ok(AdmittedExternalToolEvidence {
        evidence,
        direct_actuation_authority: false,
    })
}

pub fn validate_security_catalog() -> Result<(), String> {
    for (idx, source) in SECURITY_SOURCES.iter().enumerate() {
        if source.id.trim().is_empty() || source.authority.trim().is_empty() || source.source_uri.trim().is_empty() {
            return Err("REFUSED:INVALID_SECURITY_SOURCE".to_string());
        }
        if SECURITY_SOURCES[idx + 1..].iter().any(|other| other.id == source.id) {
            return Err("REFUSED:DUPLICATE_SECURITY_SOURCE".to_string());
        }
    }
    for id in FORTUNE5_SECURITY_CORE {
        if find_source(id).is_none() {
            return Err("REFUSED:MISSING_FORTUNE5_SECURITY_CORE_SOURCE".to_string());
        }
    }
    Ok(())
}

pub fn validate_tool_catalog() -> Result<(), String> {
    for (idx, tool) in SECURITY_TOOLS.iter().enumerate() {
        if tool.direct_actuation_authority {
            return Err("REFUSED:TOOL_AMBIENT_DO".to_string());
        }
        if SECURITY_TOOLS[idx + 1..].iter().any(|other| other.id == tool.id) {
            return Err("REFUSED:DUPLICATE_SECURITY_TOOL".to_string());
        }
    }
    Ok(())
}

pub fn find_source(id: &str) -> Option<&'static SecuritySource> {
    SECURITY_SOURCES.iter().find(|source| source.id == id)
}

pub fn find_tool(id: &str) -> Option<&'static SecurityToolIntegration> {
    SECURITY_TOOLS.iter().find(|tool| tool.id == id)
}

fn is_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
