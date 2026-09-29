#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolBoundary {
    ObserveOnly,
    AssessOnly,
    DetectOnly,
    ConstructIntentOnly,
    ExternalEnforcerBehindCastleAdmission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecurityToolIntegration {
    pub id: &'static str,
    pub ecosystem: &'static str,
    pub native_surface: &'static str,
    pub normalized_output: &'static str,
    pub boundary: ToolBoundary,
    pub direct_actuation_authority: bool,
}

const fn tool(
    id: &'static str,
    ecosystem: &'static str,
    native_surface: &'static str,
    normalized_output: &'static str,
    boundary: ToolBoundary,
) -> SecurityToolIntegration {
    SecurityToolIntegration {
        id,
        ecosystem,
        native_surface,
        normalized_output,
        boundary,
        direct_actuation_authority: false,
    }
}

/// Integration inventory. Vendor-native artifacts are retained by digest;
/// normalization never erases the original evidence identity.
pub const SECURITY_TOOLS: &[SecurityToolIntegration] = &[
    // Cloud-native Fortune-5 ecosystem bridges.
    tool("aws-security-hub", "AWS", "Security Hub findings / ASFF", "OCSF/STIX/evidence", ToolBoundary::ObserveOnly),
    tool("aws-guardduty", "AWS", "GuardDuty findings", "OCSF/STIX/evidence", ToolBoundary::DetectOnly),
    tool("aws-inspector", "AWS", "Inspector findings", "vulnerability/evidence", ToolBoundary::AssessOnly),
    tool("aws-config", "AWS", "Config / conformance packs", "control observations", ToolBoundary::AssessOnly),
    tool("aws-macie", "AWS", "Macie findings", "data-security observations", ToolBoundary::DetectOnly),
    tool("aws-iam-access-analyzer", "AWS", "Access Analyzer findings", "authority observations", ToolBoundary::AssessOnly),
    tool("azure-defender-for-cloud", "Microsoft Azure", "Defender for Cloud alerts/recommendations", "OCSF/STIX/evidence", ToolBoundary::DetectOnly),
    tool("microsoft-sentinel", "Microsoft Azure", "Sentinel incidents/ASIM", "OCSF/STIX/evidence", ToolBoundary::DetectOnly),
    tool("azure-policy", "Microsoft Azure", "Policy state", "control observations", ToolBoundary::AssessOnly),
    tool("entra-id-protection", "Microsoft", "identity risk", "authority observations", ToolBoundary::DetectOnly),
    tool("gcp-security-command-center", "Google Cloud", "SCC findings", "OCSF/STIX/evidence", ToolBoundary::DetectOnly),
    tool("gcp-cloud-asset-inventory", "Google Cloud", "asset inventory", "asset observations", ToolBoundary::ObserveOnly),
    tool("gcp-org-policy", "Google Cloud", "Organization Policy", "control observations", ToolBoundary::AssessOnly),
    tool("google-secops", "Google Cloud", "Google SecOps detections", "OCSF/STIX/evidence", ToolBoundary::DetectOnly),
    tool("oracle-cloud-guard", "Oracle Cloud", "Cloud Guard problems", "OCSF/STIX/evidence", ToolBoundary::DetectOnly),
    tool("oracle-security-zones", "Oracle Cloud", "Security Zones policies", "control observations", ToolBoundary::AssessOnly),
    tool("ibm-qradar", "IBM", "QRadar offenses/events", "OCSF/STIX/evidence", ToolBoundary::DetectOnly),
    tool("ibm-guardium", "IBM", "Guardium findings", "data-security observations", ToolBoundary::DetectOnly),
    tool("sap-enterprise-threat-detection", "SAP", "security events/alerts", "OCSF/STIX/evidence", ToolBoundary::DetectOnly),
    tool("sap-cloud-identity-access-governance", "SAP", "access governance findings", "authority observations", ToolBoundary::AssessOnly),
    tool("salesforce-shield-event-monitoring", "Salesforce", "event monitoring", "OCSF/evidence", ToolBoundary::ObserveOnly),
    tool("salesforce-security-center", "Salesforce", "security posture findings", "control observations", ToolBoundary::AssessOnly),

    // Vulnerability / SBOM / supply chain.
    tool("trivy", "Aqua", "JSON/SARIF/CycloneDX/SPDX", "vulnerability/sbom evidence", ToolBoundary::AssessOnly),
    tool("grype", "Anchore", "JSON/SARIF", "vulnerability evidence", ToolBoundary::AssessOnly),
    tool("syft", "Anchore", "SPDX/CycloneDX", "sbom evidence", ToolBoundary::ObserveOnly),
    tool("osv-scanner", "OpenSSF / Google", "OSV JSON", "vulnerability evidence", ToolBoundary::AssessOnly),
    tool("dependency-track", "OWASP", "CycloneDX / findings API", "supply-chain evidence", ToolBoundary::AssessOnly),
    tool("sigstore-cosign", "Sigstore", "signatures/bundles/attestations", "provenance evidence", ToolBoundary::AssessOnly),
    tool("rekor", "Sigstore", "transparency log", "provenance evidence", ToolBoundary::ObserveOnly),
    tool("openssf-scorecard", "OpenSSF", "Scorecard JSON/SARIF", "repository posture evidence", ToolBoundary::AssessOnly),
    tool("github-dependency-review", "GitHub", "dependency review", "supply-chain evidence", ToolBoundary::AssessOnly),
    tool("github-secret-scanning", "GitHub", "secret alerts", "secret exposure evidence", ToolBoundary::DetectOnly),
    tool("github-codeql", "GitHub", "SARIF", "code-security evidence", ToolBoundary::AssessOnly),

    // SAST / DAST / IaC / containers / Kubernetes.
    tool("semgrep", "Semgrep", "SARIF/JSON", "code-security evidence", ToolBoundary::AssessOnly),
    tool("sonarqube", "Sonar", "findings API", "code-security evidence", ToolBoundary::AssessOnly),
    tool("owasp-zap", "OWASP", "JSON/XML", "DAST evidence", ToolBoundary::AssessOnly),
    tool("greenbone-openvas", "Greenbone", "vulnerability scan results", "vulnerability evidence", ToolBoundary::AssessOnly),
    tool("checkov", "Bridgecrew", "SARIF/JSON", "IaC policy evidence", ToolBoundary::AssessOnly),
    tool("kics", "Checkmarx", "SARIF/JSON", "IaC policy evidence", ToolBoundary::AssessOnly),
    tool("kube-bench", "Aqua", "CIS benchmark results", "control observations", ToolBoundary::AssessOnly),
    tool("kube-hunter", "Aqua", "Kubernetes findings", "vulnerability evidence", ToolBoundary::AssessOnly),

    // Runtime / network / endpoint / detection.
    tool("falco", "CNCF", "Falco events", "OCSF/evidence", ToolBoundary::DetectOnly),
    tool("tetragon", "Cilium", "eBPF events", "OCSF/evidence", ToolBoundary::DetectOnly),
    tool("tracee", "Aqua", "eBPF events", "OCSF/evidence", ToolBoundary::DetectOnly),
    tool("zeek", "Zeek", "Zeek logs", "OCSF/evidence", ToolBoundary::ObserveOnly),
    tool("suricata", "OISF", "EVE JSON", "OCSF/evidence", ToolBoundary::DetectOnly),
    tool("snort", "Cisco", "alerts", "OCSF/evidence", ToolBoundary::DetectOnly),
    tool("wazuh", "Wazuh", "alerts/events", "OCSF/evidence", ToolBoundary::DetectOnly),
    tool("sigma", "SigmaHQ", "Sigma rules", "detection intent", ToolBoundary::ConstructIntentOnly),
    tool("yara", "VirusTotal", "YARA rules/matches", "detection intent/evidence", ToolBoundary::DetectOnly),

    // Policy-as-code. These can enforce in their native systems, but CASTLE
    // integrates them only behind its admission boundary.
    tool("opa-rego", "Open Policy Agent", "Rego decisions", "policy decision evidence", ToolBoundary::ExternalEnforcerBehindCastleAdmission),
    tool("gatekeeper", "CNCF", "ConstraintTemplates/Constraints", "policy decision evidence", ToolBoundary::ExternalEnforcerBehindCastleAdmission),
    tool("kyverno", "CNCF", "Policy/ClusterPolicy", "policy decision evidence", ToolBoundary::ExternalEnforcerBehindCastleAdmission),
    tool("cedar", "Cedar", "Cedar policy decisions", "authority decision evidence", ToolBoundary::ExternalEnforcerBehindCastleAdmission),
    tool("hashicorp-sentinel", "HashiCorp", "Sentinel policy decisions", "policy decision evidence", ToolBoundary::ExternalEnforcerBehindCastleAdmission),

    // Threat intel / case management / evidence aggregation.
    tool("misp", "MISP", "MISP JSON/STIX", "STIX/evidence", ToolBoundary::ObserveOnly),
    tool("opencti", "OpenCTI", "GraphQL/STIX", "STIX/evidence", ToolBoundary::ObserveOnly),
    tool("defectdojo", "OWASP", "findings API", "finding/evidence", ToolBoundary::AssessOnly),
    tool("velociraptor", "Velocidex", "artifacts/results", "endpoint evidence", ToolBoundary::ObserveOnly),
    tool("osquery", "Linux Foundation", "SQL result sets", "endpoint evidence", ToolBoundary::ObserveOnly),
    tool("nmap", "Nmap", "XML", "network inventory evidence", ToolBoundary::ObserveOnly),
];

pub const FORTUNE5_SECURITY_CORE: &[&str] = &[
    "nist-csf",
    "nist-sp-800-53",
    "nist-ssdf",
    "nist-oscal",
    "cis-controls",
    "iso-27001",
    "pci-dss",
    "csa-ccm",
    "owasp-asvs",
    "mitre-attack",
    "cisa-kev",
    "stix",
    "taxii",
    "ocsf",
    "slsa",
    "spdx",
    "cyclonedx",
    "uco",
    "case",
    "w3c-prov-o",
    "w3c-shacl",
    "ocel",
];
