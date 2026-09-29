#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecurityKind {
    GovernanceFramework,
    ControlCatalog,
    Assurance,
    Regulation,
    CloudBaseline,
    ApplicationSecurity,
    AiSecurity,
    IndustrialControl,
    ThreatKnowledge,
    VulnerabilityKnowledge,
    DetectionLanguage,
    TelemetrySchema,
    SupplyChain,
    EvidenceExchange,
    SecurityOntology,
    PolicyLanguage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionPolicy {
    Pinned,
    Rolling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecuritySource {
    pub id: &'static str,
    pub authority: &'static str,
    pub version: &'static str,
    pub version_policy: VersionPolicy,
    pub kind: SecurityKind,
    pub machine_surface: &'static str,
    pub source_uri: &'static str,
}

/// Federated source registry. A row means "CASTLE knows how to identify and
/// bind evidence to this authority family"; it is not a certification claim.
pub const SECURITY_SOURCES: &[SecuritySource] = &[
    // Governance, control, assurance, risk.
    src("nist-csf", "NIST", "2.0", VersionPolicy::Pinned, SecurityKind::GovernanceFramework, "CSF Core / informative references", "https://www.nist.gov/cyberframework"),
    src("nist-sp-800-53", "NIST", "Rev.5 / 5.1 catalog", VersionPolicy::Pinned, SecurityKind::ControlCatalog, "OSCAL/XML/CSV", "https://csrc.nist.gov/Projects/risk-management/sp800-53-controls"),
    src("nist-ssdf", "NIST", "1.1", VersionPolicy::Pinned, SecurityKind::ControlCatalog, "SSDF practices", "https://csrc.nist.gov/pubs/sp/800/218/final"),
    src("nist-sp-800-171", "NIST", "Rev.3", VersionPolicy::Pinned, SecurityKind::ControlCatalog, "security requirements", "https://csrc.nist.gov/pubs/sp/800/171/r3/final"),
    src("nist-zero-trust", "NIST", "SP 800-207", VersionPolicy::Pinned, SecurityKind::GovernanceFramework, "zero-trust architecture", "https://csrc.nist.gov/pubs/sp/800/207/final"),
    src("nist-privacy-framework", "NIST", "rolling", VersionPolicy::Rolling, SecurityKind::GovernanceFramework, "privacy framework", "https://www.nist.gov/privacy-framework"),
    src("nist-oscal", "NIST", "rolling", VersionPolicy::Rolling, SecurityKind::EvidenceExchange, "JSON/XML/YAML OSCAL models", "https://pages.nist.gov/OSCAL/"),
    src("cis-controls", "CIS", "8.1", VersionPolicy::Pinned, SecurityKind::ControlCatalog, "Safeguards / mappings", "https://www.cisecurity.org/controls/v8-1"),
    src("cis-benchmarks", "CIS", "rolling", VersionPolicy::Rolling, SecurityKind::CloudBaseline, "benchmark profiles", "https://www.cisecurity.org/cis-benchmarks"),
    src("iso-27001", "ISO/IEC", "27001:2022+Amd1:2024", VersionPolicy::Pinned, SecurityKind::Assurance, "ISMS identity/reference only", "https://www.iso.org/standard/27001"),
    src("iso-27002", "ISO/IEC", "27002:2022", VersionPolicy::Pinned, SecurityKind::ControlCatalog, "control guidance identity/reference only", "https://www.iso.org/standard/75652.html"),
    src("iso-27017", "ISO/IEC", "rolling", VersionPolicy::Rolling, SecurityKind::CloudBaseline, "cloud security guidance identity/reference only", "https://www.iso.org/standard/43757.html"),
    src("iso-27018", "ISO/IEC", "rolling", VersionPolicy::Rolling, SecurityKind::Assurance, "public-cloud PII guidance identity/reference only", "https://www.iso.org/standard/76559.html"),
    src("iso-27701", "ISO/IEC", "rolling", VersionPolicy::Rolling, SecurityKind::Assurance, "privacy information management identity/reference only", "https://www.iso.org/standard/85819.html"),
    src("soc2-tsc", "AICPA", "rolling", VersionPolicy::Rolling, SecurityKind::Assurance, "Trust Services Criteria identity/reference", "https://www.aicpa-cima.com/resources/landing/system-and-organization-controls-soc-suite-of-services"),
    src("pci-dss", "PCI SSC", "4.0.1", VersionPolicy::Pinned, SecurityKind::Assurance, "PCI DSS requirements identity/reference", "https://www.pcisecuritystandards.org/standards/pci-dss/"),
    src("csa-ccm", "Cloud Security Alliance", "4.1", VersionPolicy::Pinned, SecurityKind::CloudBaseline, "CCM/CAIQ mappings", "https://cloudsecurityalliance.org/research/cloud-controls-matrix"),
    src("fedramp", "FedRAMP", "Rev.5", VersionPolicy::Pinned, SecurityKind::CloudBaseline, "OSCAL baselines/packages", "https://www.fedramp.gov/"),
    src("cmmc", "US DoD", "2.0", VersionPolicy::Pinned, SecurityKind::Assurance, "assessment/control identity", "https://dodcio.defense.gov/CMMC/"),
    src("cobit", "ISACA", "2019", VersionPolicy::Pinned, SecurityKind::GovernanceFramework, "governance objectives identity/reference", "https://www.isaca.org/resources/cobit"),
    src("open-fair", "The Open Group", "rolling", VersionPolicy::Rolling, SecurityKind::GovernanceFramework, "risk taxonomy/model", "https://www.opengroup.org/forum/security-forum-0/overviews/open-fair"),

    // Regulations / sector baselines.
    src("eu-gdpr", "European Union", "2016/679", VersionPolicy::Pinned, SecurityKind::Regulation, "legal obligations/reference", "https://eur-lex.europa.eu/eli/reg/2016/679/oj"),
    src("eu-dora", "European Union", "2022/2554", VersionPolicy::Pinned, SecurityKind::Regulation, "digital operational resilience/reference", "https://eur-lex.europa.eu/eli/reg/2022/2554/oj"),
    src("eu-nis2", "European Union", "2022/2555", VersionPolicy::Pinned, SecurityKind::Regulation, "cybersecurity obligations/reference", "https://eur-lex.europa.eu/eli/dir/2022/2555/oj"),
    src("eu-cra", "European Union", "2024/2847", VersionPolicy::Pinned, SecurityKind::Regulation, "cyber resilience requirements/reference", "https://eur-lex.europa.eu/eli/reg/2024/2847/oj"),
    src("hipaa-security", "US HHS", "rolling", VersionPolicy::Rolling, SecurityKind::Regulation, "Security Rule/reference", "https://www.hhs.gov/hipaa/for-professionals/security/index.html"),
    src("glba-safeguards", "US FTC", "rolling", VersionPolicy::Rolling, SecurityKind::Regulation, "Safeguards Rule/reference", "https://www.ftc.gov/business-guidance/privacy-security/gramm-leach-bliley-act"),
    src("nydfs-500", "NYDFS", "rolling", VersionPolicy::Rolling, SecurityKind::Regulation, "23 NYCRR 500/reference", "https://www.dfs.ny.gov/industry-guidance/cybersecurity"),
    src("nerc-cip", "NERC", "rolling", VersionPolicy::Rolling, SecurityKind::IndustrialControl, "CIP standards/reference", "https://www.nerc.com/pa/Stand/Pages/CIPStandards.aspx"),
    src("iec-62443", "IEC", "family", VersionPolicy::Rolling, SecurityKind::IndustrialControl, "IACS security standards identity/reference", "https://www.iec.ch/cyber-security"),
    src("nist-sp-800-82", "NIST", "Rev.3", VersionPolicy::Pinned, SecurityKind::IndustrialControl, "OT security guidance", "https://csrc.nist.gov/pubs/sp/800/82/r3/final"),

    // Application and AI security.
    src("owasp-asvs", "OWASP", "5.0.0", VersionPolicy::Pinned, SecurityKind::ApplicationSecurity, "CSV/JSON/requirements", "https://owasp.org/www-project-application-security-verification-standard/"),
    src("owasp-masvs", "OWASP", "rolling", VersionPolicy::Rolling, SecurityKind::ApplicationSecurity, "mobile verification requirements", "https://mas.owasp.org/MASVS/"),
    src("owasp-samm", "OWASP", "rolling", VersionPolicy::Rolling, SecurityKind::ApplicationSecurity, "maturity model", "https://owaspsamm.org/"),
    src("owasp-top10", "OWASP", "2025", VersionPolicy::Pinned, SecurityKind::ApplicationSecurity, "risk taxonomy", "https://owasp.org/Top10/2025/"),
    src("owasp-api-top10", "OWASP", "2023", VersionPolicy::Pinned, SecurityKind::ApplicationSecurity, "API risk taxonomy", "https://owasp.org/API-Security/editions/2023/en/0x11-t10/"),
    src("owasp-llm-top10", "OWASP", "rolling", VersionPolicy::Rolling, SecurityKind::AiSecurity, "LLM risk taxonomy", "https://genai.owasp.org/llm-top-10/"),
    src("owasp-llmsvs", "OWASP", "2.0", VersionPolicy::Pinned, SecurityKind::AiSecurity, "LLM verification requirements", "https://owasp.org/www-project-llm-verification-standard/"),
    src("nist-ai-rmf", "NIST", "1.0", VersionPolicy::Pinned, SecurityKind::AiSecurity, "AI RMF / profiles", "https://www.nist.gov/itl/ai-risk-management-framework"),
    src("mitre-atlas", "MITRE", "rolling", VersionPolicy::Rolling, SecurityKind::AiSecurity, "AI adversary knowledge base", "https://atlas.mitre.org/"),

    // Threat, weakness, vulnerability, prioritization.
    src("mitre-attack", "MITRE", "19.2", VersionPolicy::Pinned, SecurityKind::ThreatKnowledge, "STIX 2.1 / CTI graph", "https://attack.mitre.org/"),
    src("mitre-d3fend", "MITRE", "rolling", VersionPolicy::Rolling, SecurityKind::ThreatKnowledge, "knowledge graph", "https://d3fend.mitre.org/"),
    src("mitre-cwe", "MITRE", "rolling", VersionPolicy::Rolling, SecurityKind::VulnerabilityKnowledge, "weakness taxonomy/XML", "https://cwe.mitre.org/"),
    src("mitre-capec", "MITRE", "rolling", VersionPolicy::Rolling, SecurityKind::ThreatKnowledge, "attack patterns/XML", "https://capec.mitre.org/"),
    src("cve", "CVE Program", "rolling", VersionPolicy::Rolling, SecurityKind::VulnerabilityKnowledge, "CVE records / JSON", "https://www.cve.org/"),
    src("nvd", "NIST", "rolling", VersionPolicy::Rolling, SecurityKind::VulnerabilityKnowledge, "NVD JSON APIs/feeds", "https://nvd.nist.gov/"),
    src("cpe", "NIST", "2.3", VersionPolicy::Pinned, SecurityKind::VulnerabilityKnowledge, "CPE naming", "https://nvd.nist.gov/products/cpe"),
    src("cvss", "FIRST", "4.0", VersionPolicy::Pinned, SecurityKind::VulnerabilityKnowledge, "CVSS vectors", "https://www.first.org/cvss/"),
    src("epss", "FIRST", "rolling", VersionPolicy::Rolling, SecurityKind::VulnerabilityKnowledge, "EPSS probability feed", "https://www.first.org/epss/"),
    src("cisa-kev", "CISA", "rolling", VersionPolicy::Rolling, SecurityKind::VulnerabilityKnowledge, "CSV/JSON/JSON Schema", "https://www.cisa.gov/known-exploited-vulnerabilities-catalog"),
    src("osv", "OpenSSF / Google", "rolling", VersionPolicy::Rolling, SecurityKind::VulnerabilityKnowledge, "OSV schema/API", "https://osv.dev/"),

    // Threat intelligence, automation, detection, telemetry.
    src("stix", "OASIS", "2.1", VersionPolicy::Pinned, SecurityKind::EvidenceExchange, "STIX JSON", "https://www.oasis-open.org/standard/stix-version-2-1/"),
    src("taxii", "OASIS", "2.1", VersionPolicy::Pinned, SecurityKind::EvidenceExchange, "TAXII REST API", "https://www.oasis-open.org/standard/taxii-version-2-1/"),
    src("openc2", "OASIS", "rolling", VersionPolicy::Rolling, SecurityKind::EvidenceExchange, "cyber command language", "https://www.oasis-open.org/committees/openc2/"),
    src("cacao", "OASIS", "rolling", VersionPolicy::Rolling, SecurityKind::EvidenceExchange, "security playbooks", "https://www.oasis-open.org/committees/tc_home.php?wg_abbrev=cacao"),
    src("sarif", "OASIS", "2.1.0", VersionPolicy::Pinned, SecurityKind::EvidenceExchange, "SARIF JSON", "https://www.oasis-open.org/standard/sarifv2-1-0/"),
    src("csaf", "OASIS", "2.0", VersionPolicy::Pinned, SecurityKind::EvidenceExchange, "CSAF JSON", "https://docs.oasis-open.org/csaf/csaf/v2.0/"),
    src("ocsf", "OCSF / Linux Foundation", "rolling", VersionPolicy::Rolling, SecurityKind::TelemetrySchema, "normative JSON schema", "https://ocsf.io/"),
    src("elastic-ecs", "Elastic", "rolling", VersionPolicy::Rolling, SecurityKind::TelemetrySchema, "ECS schema", "https://www.elastic.co/guide/en/ecs/current/index.html"),
    src("splunk-cim", "Splunk", "rolling", VersionPolicy::Rolling, SecurityKind::TelemetrySchema, "CIM data models", "https://help.splunk.com/en/splunk-enterprise/common-information-model"),
    src("microsoft-asim", "Microsoft", "rolling", VersionPolicy::Rolling, SecurityKind::TelemetrySchema, "ASIM schemas/parsers", "https://learn.microsoft.com/azure/sentinel/normalization"),
    src("opentelemetry", "CNCF", "rolling", VersionPolicy::Rolling, SecurityKind::TelemetrySchema, "OTLP / semantic conventions", "https://opentelemetry.io/docs/specs/"),
    src("sigma", "SigmaHQ", "rolling", VersionPolicy::Rolling, SecurityKind::DetectionLanguage, "Sigma rules/specification", "https://sigmahq.io/"),
    src("yara", "VirusTotal", "rolling", VersionPolicy::Rolling, SecurityKind::DetectionLanguage, "YARA rules", "https://yara.readthedocs.io/"),
    src("suricata-eve", "OISF", "rolling", VersionPolicy::Rolling, SecurityKind::TelemetrySchema, "EVE JSON", "https://docs.suricata.io/"),
    src("syslog", "IETF", "RFC 5424", VersionPolicy::Pinned, SecurityKind::TelemetrySchema, "syslog protocol", "https://www.rfc-editor.org/rfc/rfc5424"),

    // Supply chain, provenance, BOM, attestations.
    src("slsa", "SLSA / Linux Foundation", "1.2", VersionPolicy::Pinned, SecurityKind::SupplyChain, "provenance / VSA", "https://slsa.dev/spec/v1.2/"),
    src("spdx", "Linux Foundation", "3.0", VersionPolicy::Pinned, SecurityKind::SupplyChain, "JSON-LD / SHACL", "https://spdx.dev/use/specifications/"),
    src("cyclonedx", "OWASP / Ecma", "1.7", VersionPolicy::Pinned, SecurityKind::SupplyChain, "JSON/XML/Protobuf", "https://cyclonedx.org/specification/overview/"),
    src("in-toto", "in-toto / CNCF", "rolling", VersionPolicy::Rolling, SecurityKind::SupplyChain, "attestations/layouts", "https://in-toto.io/"),
    src("sigstore", "Sigstore / Linux Foundation", "rolling", VersionPolicy::Rolling, SecurityKind::SupplyChain, "bundle/transparency/signature", "https://docs.sigstore.dev/"),
    src("tuf", "The Update Framework / CNCF", "rolling", VersionPolicy::Rolling, SecurityKind::SupplyChain, "signed repository metadata", "https://theupdateframework.io/"),
    src("openssf-scorecard", "OpenSSF", "rolling", VersionPolicy::Rolling, SecurityKind::SupplyChain, "automated repository security checks", "https://openssf.org/projects/scorecard/"),
    src("openssf-osps-baseline", "OpenSSF", "rolling", VersionPolicy::Rolling, SecurityKind::SupplyChain, "baseline criteria", "https://baseline.openssf.org/"),

    // Semantic/ontology substrate. These are public graph vocabularies, not
    // replacements for the external cyber authorities above.
    src("uco", "UCO Project", "rolling", VersionPolicy::Rolling, SecurityKind::SecurityOntology, "RDF/OWL/SHACL", "https://unifiedcyberontology.org/"),
    src("case", "CASE Community", "rolling", VersionPolicy::Rolling, SecurityKind::SecurityOntology, "RDF/OWL/SHACL", "https://caseontology.org/"),
    src("w3c-prov-o", "W3C", "Recommendation", VersionPolicy::Pinned, SecurityKind::SecurityOntology, "RDF/OWL", "https://www.w3.org/TR/prov-o/"),
    src("w3c-skos", "W3C", "Recommendation", VersionPolicy::Pinned, SecurityKind::SecurityOntology, "RDF/OWL", "https://www.w3.org/TR/skos-reference/"),
    src("w3c-shacl", "W3C", "Recommendation", VersionPolicy::Pinned, SecurityKind::SecurityOntology, "RDF validation", "https://www.w3.org/TR/shacl/"),
    src("w3c-odrl", "W3C", "2.2", VersionPolicy::Pinned, SecurityKind::SecurityOntology, "RDF policy model", "https://www.w3.org/TR/odrl-model/"),
    src("dcterms", "DCMI", "rolling", VersionPolicy::Rolling, SecurityKind::SecurityOntology, "RDF vocabulary", "https://www.dublincore.org/specifications/dublin-core/dcmi-terms/"),
    src("qudt", "QUDT", "rolling", VersionPolicy::Rolling, SecurityKind::SecurityOntology, "RDF units/quantities", "https://qudt.org/"),
    src("sosa-ssn", "W3C/OGC", "Recommendation", VersionPolicy::Pinned, SecurityKind::SecurityOntology, "RDF observations/sensors", "https://www.w3.org/TR/vocab-ssn/"),
    src("ocel", "OCEL", "2.0", VersionPolicy::Pinned, SecurityKind::EvidenceExchange, "OCEL JSON/XML/SQLite", "https://www.ocel-standard.org/"),
];

const fn src(
    id: &'static str,
    authority: &'static str,
    version: &'static str,
    version_policy: VersionPolicy,
    kind: SecurityKind,
    machine_surface: &'static str,
    source_uri: &'static str,
) -> SecuritySource {
    SecuritySource { id, authority, version, version_policy, kind, machine_surface, source_uri }
}
