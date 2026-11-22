# ROADMAP.md — Sovereign Ecosystem Execution Plan (2026-11-20 $\to$ 2028-09-01)

**Horizon:** November 20, 2026 $\to$ September 1, 2028 (21.5 Months)

**Operating Doctrine:** TAI $555 \to 1$ Solo-Architect Foundry ($F = 0$)

**Quality Methodology:** Design for Lean Six Sigma (DfLSS — **DMADV**) targeting $C_{pk} \ge 2.0$ and Executable Six Sigma FMEA (unauthorized transitions $= 0$, conformance fitness $= 1.0$)

**Governing Conservation Law:** $\Delta S_{\text{admitted}} + \mathbf{\Phi}_{\text{lease}} \equiv \mathbf{R}_{\text{receipt}} \pmod{\mathbf{\Sigma}_{\text{PQC}}}$

---

## 1. DfLSS (DMADV) Operating Spine

Every engineering milestone and RevOps funnel stage is governed as a zero-defect industrial assembly line ($C_{pk} \ge 2.0$) rather than artisanal software or subjective sales. Across both the **Foundry Engineering Spine** and the **RevOps Lifecycle**, work progresses strictly through five DMADV gates:

| DMADV Phase | Foundry Engineering Application | RevOps Pipeline & Commercial Application | Hard CTQ (Critical-to-Quality) Metric |
| --- | --- | --- | --- |
| **D — Define** | Lock `TASK_CONTRACT.md`, ratified public **Truex** ontologies (FIBO, PROV-O, P-PLAN, SHACL, SHACL UI `shui:` `http://www.w3.org/ns/shacl-ui/`, OCEL v2, ODRL), and prohibited board outcomes. | Define buyer archetypes ($\nu_{\text{ICV}}$), 2027–2028 statutory cliffs (0.003% legacy clearance), and the 4-tier power law $10^k(\$100\text{k} + \$5\text{k}N)$. | Zero private namespaces (`000_forbid_private_namespaces.rq`); zero uncontracted scope. |
| **M — Measure** | Capture baseline `file:line` truth via `castle probe` (`anti-cheat-scorecard` + IaC/SAST/RLS) and real hardware Criterion benchmarks (`cgap`, WAL recovery). | Measure 21,000-node LinkedIn graph via WSJF, track **Mean Time to Disqualification (MTTD)**, and quantify exact operational leakage in `/drop/:lease_id`. | Verified recovery $\ge$ tranche cap; $L_{\text{effective}}$ crash recovery $< 5\text{ ms}$ ($B = 1$). |
| **A — Analyze** | Run FM Limiter Circuit (`Claims Versus Code`), `minimum_impact_coverage()` Pareto triage, and solver deadlock discovery ($\mathcal{A}_{\text{valid}}(s) = \emptyset$). | Analyze buyer mental-accounting blocks ($\Pi_k^{-1}$), counterparty compliance via 45-day CPRA deletion tests, and partner margin physics. | 80/20 empirical Pareto defect coverage; 100% claim-to-code phase lock. |
| **D — Design** | Compile AOT clean-room artifacts via `ggen` / `ggen_igniter`, `ferroplan` HTN-FOND $\to$ POWL v2, Zero-String `.aod`, FFB keyless WASM, and 4-Cloud RAID (`Smalt`). | Design polymorphic **Zero-Lawyer Pull Rails** (virtual cards, MACC, capped ACH, Vault sweeps), Woz & Jobs `--brand` OEM kits, and 3-channel batch ingress. | `rm -rf` + `mix ggen_igniter.sync` $\implies$ `git diff --exit-code == 0`; `agent-card.json` $\le 10\text{ KB}$. |
| **V — Verify** | Execute **Executable Six Sigma FMEA**: adversarial A2A HTN-FOND red-team courts + OCEL v2 $\leftrightarrow$ POWL v2 trace alignment ($\gamma = 0$), Chicago mutation courts, 50-cycle SIGKILL drills, and 5-agent Victory Audits. | Verify live cash clearing on Monday prepaid pull-rails, Seat AS-01 WebAuthn UV seals, `--public-anonymize` redaction courts, and `< 250 KB` air-gapped WASM verifier isolation. | Unauthorized transition occurrence $= 0$; conformance fitness $= 1.0$ ($\gamma = 0$); 0% private data leakage. |

### 1.1 Ratified Public NS — SHACL UI for Crystallized Control Surfaces (CCS)

**Define-gate law:** mobile and Zero-HITL UI surfaces are **Crystallized Control Surfaces (CCS)** — not private pack IRIs, not LLM-tuned CSS vocabularies. Presentation, grouping, and widget binding must reuse the W3C **SHACL UI** public namespace only:

* Prefix: `shui:`
* IRI: `http://www.w3.org/ns/shacl-ui/`
* Companion core: `sh:` (`http://www.w3.org/ns/shacl#`) for `PropertyGroup` / `group` / `order`

**CCS → public SHACL UI mapping (admitted):**

| CCS role | Public binding | Forbidden substitute |
| --- | --- | --- |
| Sight Glass (data viewer) | `shui:viewer` + `shui:propertyRole` | Private `*Viewer` classes under a pack IRI |
| Human Circuit Breaker (sole HITL actuation) | `shui:editor` | Private editor schema / DO-written CSS tokens as ontology |
| Progressive disclosure / cognitive budget | `shui:parentGroup` + `sh:PropertyGroup` / `sh:order` | Ad-hoc nested private groups |

Admission is narrative + allowlist twins (`whitelist.prefixes` / `000_forbid_private_namespaces.rq`); this ROADMAP ratifies the **public-NS choice** for CCS so Define-phase contracts and Truex Public Ontology Cover stay phase-locked. Inventing a castle-private UI namespace fails the Define CTQ (zero private namespaces).

---

## 2. RevOps Pipeline & Full-Lifecycle Tooling Stack

The RevOps engine replaces SDR headcount, traditional CRMs, discovery calls, and screenshare click-demos with Toyota Production System single-piece flow and mathematically enforced scarcity.

```text
[1. SIGNAL & DEMAND]       [2. INTAKE AIRLOCK]         [3. FORENSIC PROOF]
castle publish             revops.fnox.io              castle probe + audit
 • Headline Carousels      • 21k WSJF Triage           • anti-cheat-scorecard (Go)
 • 21k LinkedIn Newsletter • /drop/:lease_id (SHA-256) • Trivy / OpenGrep / RLS
 • Free <250KB WASM        • 48h/24h SlotLease         • Claims Versus Code PDF
   Gatekeeper Verifier     • $25k odrl:Duty NDA Fee    • 45-Day Deletion Receipt
          │                         │                           │
          └─────────────────────────┴───────────────────────────┘
                                    │
                                    ▼
[4. ZERO-LAWYER RAILS]     [5. COMMISSION & OPERATE]   [6. COMPOUND & PROPAGATE]
Π_k^-1 Inverse Compiler    Appomattox Disarmament      Kudzu + Woz & Jobs OEM
 • $5k/wk Virtual Card     • 3-Channel Batch Ingress   • cos.fnox.io 3-4% Float
 • Cloud MACC Private Offer• FFB Zero-Rewrite Shims    • Woz & Jobs --brand Kits
 • Zero-Principal Vault    • 4-Cloud RAID (Smalt)      • CindyCat §107/§127 Guild
 • Monday Stop-Work Gate   • Seat AS-01 WebAuthn Seal  • 72h M&A Bolt-On Engine
```

### Lifecycle Tooling & Control Plane Matrix

| Lifecycle Stage | Primary Tool / Service | Operational Mechanism | Fail-Closed Guardrail |
| --- | --- | --- | --- |
| **1. Demand & Newsjacking** | `castle publish` (`--public-anonymize`, `--brand`) | Compiles live outage/breach headline manifests into `1080×1350` LinkedIn PDF carousels, 21k-graph Newsletters, and action-first posts. | Redaction court blocks any private client name, `/Users/` path, or unredacted `file:line` citation. |
| **2. Gatekeeper Ratchet** | `Affidavit` `< 250 KB` WASM Verifier (`verify_witness`) | Free air-gapped micro-verifier distributed to cyber/mutual insurers, regulators, and EU Notified Bodies to check 352B CTRT / 736B PCCf receipts. | Zero network egress; zero payload mapping; no domain ontology required. |
| **3. Triage & Intake** | `revops.fnox.io` (`/drop/:lease_id`) | Ingests 21,000-row LinkedIn `Connections.csv`, ranks via WSJF, enforces 48h/24h `SlotLease` windows (Edge-Shell Clock Injection), and stages read-only BI/ERP exports. | Verify-Before-Parse SHA-256, 256 MiB cap, symlink jail, formula neutralization, $25,000 $\times 10^k$ `odrl:Duty` custom-NDA fee. |
| **4. Forensic Proof** | `castle probe` + `castle audit` + `castle deletion` | Triggers pinned `anti-cheat-scorecard` (52 Go detectors) + IaC/SAST/RLS probes; compiles `Claims Versus Code` PDF; tracks first-party 45-day CPRA deletion receipts. | Signed `OWNED` / `AUTHORIZED_CLIENT` scope in `TASK_CONTRACT.md`; SHA-256 pinned probe binaries. |
| **5. Contracting & Billing** | $\Pi_k^{-1}$ Inverse Surface Compiler + Pull Rails | Routes prospects by $\nu_{\text{ICV}}$ to Virtual Spend Cards ($5k/wk cell), Cloud MACC, Capped ACH, SPV Draws, or Zero-Principal `RECOVERY-VAULT` sweeps. | Tenant Billing Quarantine; Monday Prepaid Stop-Work Invariant; `SELECT ... FOR UPDATE` live unspent vault locks. |
| **6. Fulfillment & Support** | 3-Channel Ingress + `FFB` + `Smalt` 4-Cloud RAID | 1 weekly 60-min call + Figma/email/transcript ingress; FFB shims preserve legacy UIs while swapping backends onto client-owned AWS+Azure+GCP+OCI RAID. | Zero WhatsApp; zero uncontracted scope; 0% client custody; 0.00% transaction rent. |
| **7. Scale & Distribution** | Woz & Jobs OEM + `cos.fnox.io` (`FinFab`) + `CindyCat` | Supplies wholesale 4-Cloud RAID, validated updates, branded content, and RevOps to independent partners; propagates via 3%–4% vendor float and §107/§127 civic hubs. | Monotonic wholesale Monday leases; keyless zero-symbol WASM; unsoftened `P28-SIL v2.1.0` sovereign wall. |

---

## 3. Master Execution Roadmap (Nov 2026 $\to$ Sep 2028)

### Phase 0: Baseline Seal, Cash Lock & Free Inbound Launch

**Window:** November 20 – December 31, 2026 (6 Weeks)

**Primary Objective:** Stabilize live $9,000/week baseline cash flow, seal active software seams before scaling outbound, and ignite the $0 LinkedIn headline engine.

* **Engineering & DfLSS Deliverables:**
  * Seal the 4 FFB v26.18.26 seams (WebTransport `h3` framing, non-extractable WebCrypto reconnect, Worker 32 MB arena partition, Unicode math) and ship GA on `agents.fnox.io` (Nov 25) with `agent-card.json` $\le 10\text{ KB}$.
  * Close `cos` ledger conservation fixes, eliminate the `--seed 7` nondeterminism race, revert wall-clock/lease allowlists, and verify in-process `:prime256v1` Seat AS-01 capability gates.
  * Complete `servengine.fnox.io` M4 AshPostgres wiring, verify `c1b7cb87` Sentinel Victory Audit (J01–J10 green, live GCP OCEL v2 logs), and harden `arcus` + `smalt` `ServengineSim` local cloud twins.
  * Wire `anti-cheat-scorecard` (Go) as Adapter #1 in `castle probe` and ship `castle publish --target linkedin-carousel|newsletter` with the `--public-anonymize` redaction court.
* **RevOps & Commercial Deliverables:**
  * Lock household/cash alignment with active **Courtroom.ai** ($4,000/wk) and **Vince Pilot** ($5,000/wk, Nov 23 – Dec 31) under the Monday Prepaid Stop-Work Invariant, 3-Channel Batch Ingress, and signed Background IP carve-outs.
  * Stand up direct merchant card/ACH billing (Stripe Billing) under Tenant Billing Quarantine and initiate Cloud Marketplace Seller accounts.
  * Unveil the independently built **Chief of Staff / Financial System (`cos.fnox.io` / `FinFab`)** in early December (Weeks 2–3) to anchor the January 2027 multi-cell upgrade.
  * Launch the $0 LinkedIn campaign (Personal Profile overhaul, 3 posts/week anchored to live outages/breaches, and Personal Newsletter Issue #1 blasting the 21,000-connection graph).
* **Phase 0 Exit Gate (DfLSS Verify):**
  * Two consecutive LOW-only skeptic waves on `cos`; FFB v26.18.26 live with P28-SIL v2.1.0 active; 6 consecutive Monday clearances ($30,000 Vince pilot + Courtroom.ai invoices); first 4 weeks of headline-anchored LinkedIn exhibits live.

---

### Phase 1: Core Compiler Spine, Self-Serve Airlock & Multi-Cell Ramp

**Window:** Q1 2027 (January 1 – March 31, 2027)

**External Trigger:** January 1, 2027 — EU Data Act cloud egress zeroing & California SB 923 expanded data-deletion mandate take effect.

**Primary Objective:** Close the single horizontal compiler spine before vertical expansion, launch `revops.fnox.io` autonomously, and step weekly cash flow from 1 pilot cell to 3–7 active cells.

* **Engineering & DfLSS Deliverables:**
  * **Close the Horizontal Compiler Spine:** Unify `curve25519-dalek` v4 in `affidavit`, close `cgap-aoq` 64B header framing (`KD-192`/`KD-194`), implement `u64` OCEL 2.0 marking in `cgap-bosym` with real Criterion nanosecond receipts, implement `admit_pccf` in `graphlaw/src/pccf.rs`, and double-qualify `packs/htn-fond-powl2-compiler-pack`.
  * **Zero-String `.aod` & Verifier Build:** Resolve `pqc_seal` wire encoding and FFOR fixed-stride contract in `AFFI-SPEC-26.11.19`; ship the 5 `affi aod` CLI verbs and compile the standalone **`< 250 KB` air-gapped WASM verifier** (`verify_witness`).
  * **Complete `CASTLE` Probe & Publisher Suite:** Add `trivy` (IaC), `opengrep` (SAST), `gitleaks` (PII/secrets), `syft`/`osv-scanner`, and native SQL/Prisma RLS probes; ship byte-identical `castle audit claims-vs-code` PDF compilation and `castle deletion` (45-day CPRA/SB 923 tracker).
  * **Automate 4-Cloud RAID:** Ship the `< 20 min` Arcus/Terraform provisioning script standing up client-owned AWS + Azure + GCP + OCI erasure-coded arrays + baseline SRE Watchtower.
  * **Manufacture `revops.fnox.io`:** Run clean-room `ggen_igniter` synthesis of `revops-pack` + 14th GCP Terraform root (`infra/gcp/revops`) passing `git diff --exit-code == 0`.
* **RevOps & Commercial Deliverables:**
  * Convert Vince from 1 pilot cell ($5k/wk) to 3–5 active cells ($15k–$25k/wk) funded out of the $198,420 verified recovery pool ($28,346/entity = 5.7 weeks self-funded per cell).
  * Open `revops.fnox.io` `/drop/:lease_id` to the WSJF-ranked 21,000-connection LinkedIn graph, calibrating `CanonicalizeToFibo` alias coverage on the first 5–10 wild CSV drops.
  * Onboard Node #1 high-$\nu_{\text{ICV}}$ external HoldCo principal via Sovereign Obsolescence Notices and the Zero-Principal Arbitrage Offer.
* **Phase 1 Exit Gate (DfLSS Verify):**
  * Horizontal spine (`HTN-FOND -> POWL2 -> cgap u64 -> 736B PCCf`) verified with real hardware Criterion benchmarks; WASM verifier `< 250 KB` passing standalone air-gap isolation test; `revops.fnox.io` signed `ADMITTED` under the 7-Court turnstile; weekly metered run-rate $\ge \$15\text{k}–\$25\text{k}/\text{wk}$.

---

### Phase 2: Gatekeeper Ratchet, Woz & Jobs OEM, & CindyCat Anchor #1

**Window:** Q2 – Q3 2027 (April 1 – September 30, 2027)

**Primary Objective:** Shift from direct solo outbound to three self-compounding distribution multipliers—**Gatekeepers**, **Woz & Jobs Whitelabel Partners**, and **Kudzu / CindyCat** percolation.

* **Engineering & DfLSS Deliverables:**
  * Ship `castle publish --brand <partner-brand.json> --partner-id <id>` with monotonic wholesale partner lease binding and mandatory client-signed `TASK_CONTRACT.md` authorization gates.
  * Implement `MonotonicLease` + atomic **Provisional Voucher / Cure-Obligation Debt** ($V_{\text{vouch}} + D_{\text{debt}}$) splits in `fnox` + `cos` and deploy the Strict Causal Diode (`BeamPM`/`wasm4pm`) out-of-band Petri-net court.
  * Author and qualify the **CindyCat Church / Civic `ggen` Ontology Pack** (Planning Center + QBO + dual-elder approvals + IRC § 107 housing allowance + permanent IRC § 127 $5,250/yr student-loan TPA).
  * Implement Anti-Honeypot constant-rate 1,350B AoQ QUIC padding + Packet-1 RFC 9266 silent decoy shunting ($V_{\text{decoy}}$) on production gateways.
* **RevOps & Commercial Deliverables:**
  * **Gatekeeper Seeding:** Deliver the free `< 250 KB` air-gapped WASM verifier + `ALIVE` vs. `REFUSED` benchmark exhibits to cyber/crime/E&O insurers (including Church Mutual / Brotherhood Mutual), M&A diligence teams, and trade associations.
  * **Woz & Jobs OEM Launch:** Recruit and onboard the first **10–25 independent whitelabel partners** (displaced AppSec/cloud architects, boutique vCISOs, MSPs, fractional CFOs) on flat Monday prepaid wholesale rails—supplying them with 4-Cloud RAID, validated updates, branded LinkedIn carousels, and `revops.fnox.io` intake links.
  * **Kudzu Wave 1:** Activate `cos.fnox.io` (`FinFab`) 3%–4% instant-clear vendor float across Vince's 7–10 active cells ($35k–$50k/wk run-rate, crossing $610k–$830k cumulative by Week 26) and attach zero-install `ash_a2a` runners onto shared subcontractors and freight carriers.
  * **CindyCat Anchor #1:** Deploy the first Sovereign Church Pack over Planning Center + QBO, capture the zero-phishing + § 107 / § 127 cash-flow-positive receipt ($C_{\text{adopt}} < 0$), and trigger peer-pastor $\to$ insurer percolation.
  * **Entity Scaling Trigger:** Instantiate the two-tier **Wyoming IP HoldCo + Out-of-State Managed Delaware Registered Series SPV** structure once multi-cell and OEM volume warrants insulating California FTB per-series tax drag.
* **Phase 2 Exit Gate (DfLSS Verify):**
  * First insurer/association `verify_witness` pilot active; 10+ Woz & Jobs OEM partners clearing Monday wholesale pull-rails with zero end-client calls touching the Principal; Kudzu vendor float live on Anchor HoldCo #1; Anchor Church #1 live on 4-Cloud RAID.

---

### Phase 3: Pre-Cliff Surge & 5,000-Stratum M&A Cleanroom

**Window:** Q4 2027 – Q1 2028 (October 1, 2027 – March 31, 2028)

**External Trigger:** December 2, 2027 — EU AI Act / CRA strict-liability enforcement wave begins.

**Primary Objective:** Ascend the $10^k$ power-law ladder ($k = 1, 2$: Syndicate $1M+ PE Operating Partners & Enterprise $10M+ Cloud MACC Private Offers) and operationalize the 72-hour M&A assimilation engine.

* **Engineering & DfLSS Deliverables:**
  * Lock the **5,000-Stratum Coupled Datalog $\times$ N3 Compiler** (`duckdb_purrdf` SIMD integer equi-joins + `Eyeron` higher-order N3 quoted graphs + compile-time acyclic stratification verifier $\text{stratum}(\text{head}) > \text{stratum}(\text{body})$).
  * Ship the Stratum-0 Pre-LOI SHACL Quarantine & Min/Max Exposure Engine plus the reproducible **50M-Quad $\times$ 5,000-Stratum Mega-M&A Cleanroom Benchmark** (80-entity US-GAAP/IFRS consolidation, ASC 805/810 elimination, Change-of-Control poison pills).
  * Package **Sovereign HoldCo OS** turnkey 72-hour bolt-on assimilation sagas (Hour 0–12 diligence $\to$ Hour 12–36 `ggen` ERP adapter compile $\to$ Hour 36–72 `FinFab` cash-sweep lock & `AshPPlan` 100-day cutover).
  * Expand `ggen-marketplace` ~50 KB vertical ontology packs (Commercial Trades, Healthcare/Clinics, Freight/Fleet/VIN, Reinsurance) strictly on top of the locked horizontal spine.
* **RevOps & Commercial Deliverables:**
  * Publish verified T1 Operator ($135k Genesis) and 7-entity recovery receipts as LinkedIn Featured Benchmark Artifacts to unlock **T2 Syndicate ($1M+)** and **T3 Enterprise ($10M+ Cloud MACC Private Offers)** via `revops.fnox.io`.
  * Scale the **Woz & Jobs OEM Guild** to 50–100+ white-glove partners across North America and the EU (transatlantic US–EU footprint echoing TAI Alexandria–Heidelberg) to absorb December 2027 CRA/NIS2 remediation demand.
  * Distribute the interactive Mega-M&A simulator to PE operating partners, family offices, and antitrust/Big Four audit teams to establish `ash_affidavit` W3C PROV-O proof bundles as deal-table standard.
* **Phase 3 Exit Gate (DfLSS Verify):**
  * 50M-quad $\times$ 5,000-stratum cleanroom benchmark reproduced end-to-end in sub-minute compile time; first live 72-hour bolt-on assimilation completed with $\gamma = 0$ trace alignment; first $k \ge 1$ Syndicate/MACC contract cleared.

---

### Phase 4: Regulatory Conjunction & $O(N^2)$ Clearing Crossover

**Window:** Q2 – Q3 2028 (April 1 – September 1, 2028)

**External Trigger:** August 2, 2028 — EU AI Act Annex I High-Risk enforcement cliff locks the 0.003% multiplicative statutory AND-gate.

**Primary Objective:** Transition from $O(N)$ software/infrastructure subscriptions into self-reinforcing $O(N^2)$ cross-party Proof-Carrying Conformance (`PCCf`) B2B clearing and T+0 atomic M&A transfers.

* **Engineering & DfLSS Deliverables:**
  * Harden multi-party **HTN-FOND Hierarchical Interface Projection** so mutually blind enterprises chain cross-company physical and financial transitions over **736-byte `PCCf`** envelopes with $O(1)$ bitwise lookup, zero trade-secret leakage, and self-liquidating Provisional Vouchers.
  * Deploy air-gapped $\mathbf{\Sigma}_{\text{PQC}}$ RTGS settlement enclaves for Tier-3/Tier-4 Sovereign Clearing ($k = 3$) and scale centrally maintained 4-Cloud RAID baseline routing with bounded sub-penny per-node Watchtower unit economics.
  * Maintain continuous automated `CASTLE` verification courts and `ggen` regulatory pack updates across all deployed Woz & Jobs partner cells and CindyCat civic nodes.
* **RevOps & Commercial Deliverables:**
  * **Binary Underwriting Split:** Cyber, E&O, and mutual insurers enforce `verify_witness` cryptographic standing (`ALIVE` vs. `REFUSED`) at renewal, routing non-compliant enterprises into the Woz & Jobs OEM network and `revops.fnox.io` airlock.
  * **Kudzu $\to$ $O(N^2)$ Network Crossover:** Subcontractors, carriers, and suppliers onboarded via HoldCo instant factoring carry `Truex` / `FinFab` rails upstream across overlapping $100M+ HoldCos and operating SFOs at $0 CAC.
  * **Civic & Horizontal Guild Scale:** CindyCat horizontal cross-deployment services (4-cloud watchtower, `ggen` packs, § 107 / § 127 TPA, Jubilee distressed debt buybacks at 3–12¢/$1, automated cyber attestation) operate at 90%+ gross margin with 0% custody and 0.00% transaction rent.
* **Phase 4 Exit Gate (Vision 2028 State — September 1, 2028):**
  * Autonomous foundry operating at TAI $555 \to 1$ solo-architect leverage ($F = 0$); Woz & Jobs OEM guild, Gatekeeper insurers, Kudzu HoldCos, and CindyCat civic hubs running continuously under $\Delta S_{\text{admitted}} + \mathbf{\Phi}_{\text{lease}} \equiv \mathbf{R}_{\text{receipt}} \pmod{\mathbf{\Sigma}_{\text{PQC}}}$.

---

## 4. Critical-Path Risk Register & Stop-Work Controls

| Risk ID | Active Risk / Blocker | Mitigation & Fail-Closed Control | Target Closure |
| --- | --- | --- | --- |
| **R-01** | **Unsealed `cos` Ledger Conservation & `--seed 7` Race** | Do not launch outbound across the 21,000-node LinkedIn graph into `revops.fnox.io` until all 6 `cos` ledger fixes and `--seed 7` pass two consecutive LOW-only skeptic waves. | Phase 0 (Dec 2026) |
| **R-02** | **Scope Absorption & Intermediary Drift** | Enforce the 3-Channel Batch Ingress (1 weekly 60-min call + Figma/email/transcript), keep Tony permanently air-gapped on phone/SMS, and fence `cos.fnox.io` outside the ServEngine SOW. | Continuous (Active) |
| **R-03** | **Unverified `cgap` Benchmarks & `dalek` / `KD-192` Wire Blockers** | Block greenfield vertical packs (VIN, PE indentures) until `cgap-bosym` Criterion receipts, `curve25519-dalek` v4 unification, and `KD-192`/`KD-194` close. | Phase 1 (Q1 2027) |
| **R-04** | **Manual `.ex` / Generated-Code Drift ("Quick Fix" Trap)** | Enforce compiler-first hardening (`rm -rf` + `mix ggen_igniter.sync` + `git diff --exit-code == 0`) across all CI pipelines. | Continuous (Phase 0+) |
| **R-05** | **Armstrong / RCA IP Leakage Before Signature** | Zero Wave 1–6 clean-sheet deliverables or foundry source exposure without signed Background IP carve-outs; ship keyless zero-symbol WASM under unsoftened `P28-SIL v2.1.0`. | Phase 0 (Nov 2026) |
| **R-06** | **Partner Unauthorized Scanning or Baseline Watchtower Cost Creep** | Require client-signed `TASK_CONTRACT.md` authorization in `castle probe` and bound per-node 4-Cloud RAID Watchtower marginal cost strictly below flat wholesale/cross-subsidy economics. | Phase 2 (Q2 2027) |
