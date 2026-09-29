# CASTLE Security Universe

## Status

This document defines CASTLE's federated cybersecurity integration boundary. It is an authored runtime contract, not a generated projection and not an assertion of external certification.

CASTLE does not copy every external control catalog into a private schema. It preserves the native authority, version policy, object identity, evidence identity, and provenance of each framework or tool, then projects only the relationships needed for bounded admission, coverage analysis, and receipted execution.

## Constitutional fence

```text
native authority / native tool output
    -> source identity + version policy
    -> observed native objects / evidence digest
    -> normalized security graph
    -> mapping admission
         equivalence requires an explicit proof receipt
    -> receipted coverage evidence
    -> SecurityStanding
    -> inert SecurityIntent
    -> existing ConstructAdmission
    -> BRCE PREPARE
    -> GymAct / admitted provider adapter
    -> BRCE OUTCOME
    -> OCEL + cryptographic receipt
```

`SecurityIntent != ConstructAdmission != DO`.

A catalog entry, scanner finding, SIEM event, detection rule, policy-engine decision, LLM output, threat-intelligence object, compliance crosswalk, or external enforcement result cannot manufacture CASTLE actuation authority. Consequential execution remains owned by the existing sealed CONSTRUCT admission and BRCE/GymAct path.

## Federated authority model

`src/security_universe.rs` registers security authorities as metadata descriptors rather than vendoring their normative text. Each source records:

- stable CASTLE identifier;
- upstream authority;
- security-domain kind;
- pinned or rolling version policy;
- canonical upstream source URI;
- whether the source is machine-readable;
- whether native object identity must be preserved.

Pinned sources name a specific version when CASTLE intentionally binds to one. Rolling sources require runtime observation of the upstream version/digest before evidence can obtain current standing. A registry descriptor is therefore discovery topology, not proof that CASTLE has imported or verified the corresponding controls.

## Integrated authority families

The registry spans the major families needed to evaluate a global enterprise security posture:

- governance and controls: NIST CSF, NIST SP 800-53, NIST SSDF, NIST SP 800-171, Zero Trust, CIS Controls, CIS Benchmarks, ISO/IEC 27000 family, SOC 2 TSC, CSA CCM, COBIT, Open FAIR;
- regulated assurance: PCI DSS, FedRAMP, CMMC, GDPR, DORA, NIS2, EU CRA, HIPAA, GLBA, NYDFS, NERC CIP, IEC 62443, NIST SP 800-82;
- application and AI security: OWASP ASVS, MASVS, SAMM, Top 10, API Security, LLM/GenAI security, NIST AI RMF, MITRE ATLAS;
- adversary and vulnerability knowledge: MITRE ATT&CK, D3FEND, CWE, CAPEC, CVE, NVD, CPE, CVSS, EPSS, CISA KEV, OSV;
- exchange and telemetry: STIX, TAXII, OpenC2, CACAO, SARIF, CSAF, OCSF, ECS, Splunk CIM, Microsoft ASIM, OpenTelemetry, Sigma, YARA, Suricata EVE, syslog;
- software supply chain: SLSA, SPDX, CycloneDX, in-toto, Sigstore, TUF, OpenSSF Scorecard and OSPS;
- semantic/evidence vocabularies: UCO, CASE, PROV-O, SKOS, SHACL, ODRL, DCTERMS, QUDT, SOSA/SSN and OCEL.

The code registry is intentionally extensible. Inclusion means CASTLE knows the external authority identity and can admit evidence about its objects; it does not mean every normative object has already been observed or satisfied.

## Fortune-5 ecosystem bridges

The tool registry includes cloud-native evidence and enforcement surfaces across AWS, Azure, Google Cloud, Oracle Cloud, IBM, SAP and Salesforce, plus common enterprise security platforms and open security tooling.

Examples include Security Hub, GuardDuty, Inspector, AWS Config, Macie, IAM Access Analyzer, Defender for Cloud, Sentinel, Azure Policy, Entra, Security Command Center, Cloud Asset Inventory, Organization Policy, Google SecOps, Oracle Cloud Guard and Security Zones, QRadar, Guardium, SAP Enterprise Threat Detection, SAP Cloud Identity Access Governance, Salesforce Shield and Salesforce Security Center.

It also covers scanner/SBOM/SAST/DAST/IaC/container/runtime/detection/policy/intelligence/IR surfaces such as Trivy, Grype, Syft, OSV-Scanner, Dependency-Track, Cosign/Rekor, CodeQL, Semgrep, SonarQube, ZAP, OpenVAS, Checkov, KICS, kube-bench, kube-hunter, Falco, Tetragon, Tracee, Zeek, Suricata, Snort, Wazuh, Sigma, YARA, OPA/Gatekeeper, Kyverno, Cedar, HashiCorp Sentinel, MISP, OpenCTI, DefectDojo, Velociraptor, osquery and nmap.

Every tool descriptor has `direct_actuation_authority = false`. Tools that can enforce in their native environment are classified as external enforcers behind CASTLE admission; their existence does not open a second CASTLE DO path.

The named registry is not a closed world. `ExternalToolEvidence` provides a receipt-bound extension rail for future, proprietary, or organization-specific products across OSCAL, STIX/TAXII, OCSF, SARIF, CycloneDX, SPDX, CSAF, OpenTelemetry, OCEL, JSON/JSON-LD, RDF, CSV, XML, syslog, or a digest-bound native opaque representation. Admission binds tool authority/version/native object identity, adapter identity, payload identity, and receipt identity. It still returns `direct_actuation_authority = false`.

## Fortune-5 core qualification

`FORTUNE5_SECURITY_CORE` identifies the cross-domain minimum that the security-universe gate requires before it can return `ALIVE`:

- NIST CSF, NIST SP 800-53, NIST SSDF and OSCAL;
- CIS Controls;
- ISO/IEC 27001;
- PCI DSS and CSA CCM;
- OWASP ASVS;
- MITRE ATT&CK and CISA KEV;
- STIX and TAXII;
- OCSF;
- SLSA, SPDX and CycloneDX;
- UCO and CASE;
- PROV-O and SHACL;
- OCEL.

For every required source, `ALIVE` requires a nonzero observed object set, complete verification of that set, and a receipt binding the evidence. Missing evidence is `UNKNOWN`; incomplete but valid evidence is `PARTIAL_ALIVE`; impossible counts, malformed identities, duplicates, or unreceipted evidence are typed `REFUSED` errors.

This standing is a CASTLE machine-standing statement about the admitted evidence set. It is never converted into an assertion that an auditor, regulator, standards body, cloud provider, or certification authority has certified the subject.

## Crosswalk law

Mappings are typed relationships. `Related`, `Narrows`, `Broadens`, `Implements`, `Detects`, `Mitigates`, `Evidences`, and `Contradicts` can be recorded as explicit bounded assertions. `Equivalent` is stronger and is refused unless an explicit 64-hex proof-receipt identity is supplied.

Therefore:

```text
adjacency != equivalence
mapping != compliance
coverage != certification
scanner finding != truth
policy decision != CASTLE authority
threat intelligence != observed compromise
```

Cross-framework equivalence must be proven for the same admitted objects and boundaries. A convenient crosswalk cannot crown itself.

## DfCM / combinatorial maximalism

The security universe is deliberately broad before selection. CASTLE can preserve multiple lawful interpretations and mappings across frameworks, techniques, controls, evidence schemas, clouds, and tools while deferring irreversible selection to a later admitted stage.

A missing adapter or failed source edge is topology, not graph failure. The qualification function reports the actual standing of the admitted evidence set instead of treating the registry size as completion.

## Exclusions

The security universe does **not**:

- vendor copyrighted normative control text merely to claim framework coverage;
- infer certification, audit opinion, regulatory compliance, or risk acceptance;
- convert tool output directly into `Observed` without an admitted evidence receipt;
- let a cloud policy engine, SIEM, scanner, threat feed, planner, model or rule file bypass CONSTRUCT/BRCE;
- treat a rolling version descriptor as current evidence without observing the current native version/digest;
- treat a framework crosswalk as semantic equivalence without proof;
- create an exploit runner or a new arbitrary-command execution path.

## Falsifiers

Revoke the security-universe standing if any of the following becomes true:

1. A registered tool can directly manufacture CASTLE DO authority.
2. A security intent can execute without the sealed `ConstructAdmission` and BRCE boundary.
3. `Equivalent` can be admitted without a proof receipt.
4. Missing or unreceipted required-source evidence can produce `ALIVE`.
5. A native source/object identity is silently rewritten so replay can no longer bind to the original authority.
6. A rolling authority is treated as pinned/current without observed version identity.
7. Registry presence is reported as external certification or control satisfaction.
8. Generated CASTLE projections are hand-edited to manufacture semantic authority.

## Extension protocol

To add a framework or tool:

1. preserve its canonical external authority and native identity;
2. classify it without giving it ambient actuation authority;
3. add only metadata required for discovery and admission;
4. import/observe native objects through a bounded adapter or canonical graph source;
5. bind evidence to source/object/version/digest/subject;
6. emit a receipt before the evidence affects standing;
7. add crosswalks with the weakest correct relation type;
8. require proof before declaring equivalence;
9. route consequential remediation through the existing CONSTRUCT -> BRCE -> GymAct path;
10. add an executable verifier before promoting the new edge to `ALIVE`.

That keeps the graph extensible without weakening the constitutional authority boundary.
