import { sign as ed25519Sign, verify as ed25519Verify } from "node:crypto";
import { assertBlake3SelfTest, blake3HexUtf8 } from "./blake3.ts";
import {
  qualifyFortune5,
  type Fortune5Qualification,
  type Fortune5Requirement,
  type MetricObservation,
  type MetricValue,
  type QualificationContext,
  type Standing,
} from "./fortune5.ts";
import { FORTUNE5_REQUIREMENTS } from "./fortune5.generated.ts";

const DIGEST_RE = /^[0-9a-f]{64}$/;

function canonicalJson(value: unknown): string {
  if (value === null || typeof value !== "object") {
    if (typeof value === "number" && !Number.isFinite(value)) throw new Error("REFUSED:NON_FINITE_CANONICAL_VALUE");
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  const record = value as Record<string, unknown>;
  return `{${Object.keys(record).sort().map((key) => `${JSON.stringify(key)}:${canonicalJson(record[key])}`).join(",")}}`;
}

export type SignatureAlgorithm = "Ed25519";

export interface ReceiptCoreV2 {
  version: "CASTLE-RECEIPT-V2";
  algorithm: "BLAKE3-256";
  signatureAlgorithm: SignatureAlgorithm;
  payloadDigest: string;
  subject: string;
  metric: string;
  policyDigest: string;
  authorityDigest: string;
  parentDigests: readonly string[];
  keyId: string;
  trustEpoch: number;
  issuedAt: string;
  assuranceDomain: string;
}

export interface ReceiptV2 extends ReceiptCoreV2 {
  receiptDigest: string;
  signature: string;
}

export interface TrustKey {
  keyId: string;
  publicKeyPem: string;
  validFromEpoch: number;
  revokedAtEpoch?: number;
  assuranceDomain: string;
}

export interface TrustStore {
  currentEpoch: number;
  keys: ReadonlyMap<string, TrustKey>;
}

export interface EvidenceInput {
  metric: string;
  value: MetricValue;
  subject: string;
  observedAt: string;
  epistemicClass: "OBSERVED" | "REPLAYED" | "INFERRED";
}

export interface EvidenceBundle {
  observation: MetricObservation;
  receipt: ReceiptV2;
}

export interface ReceiptIssueContext {
  policyDigest: string;
  authorityDigest: string;
  parentDigests?: readonly string[];
  keyId: string;
  trustEpoch: number;
  assuranceDomain: string;
}

function observationPayload(input: EvidenceInput | MetricObservation): EvidenceInput {
  return {
    metric: input.metric,
    value: input.value,
    subject: input.subject,
    observedAt: input.observedAt,
    epistemicClass: input.epistemicClass,
  };
}

function validateDigest(label: string, digest: string): void {
  if (!DIGEST_RE.test(digest)) throw new Error(`REFUSED:INVALID_${label}_DIGEST`);
}

function receiptCoreDigest(core: ReceiptCoreV2): string {
  return blake3HexUtf8(canonicalJson(core));
}

export function issueEvidenceReceipt(
  input: EvidenceInput,
  context: ReceiptIssueContext,
  privateKeyPem: string,
): EvidenceBundle {
  assertBlake3SelfTest();
  validateDigest("POLICY", context.policyDigest);
  validateDigest("AUTHORITY", context.authorityDigest);
  if (!context.keyId || !context.assuranceDomain) throw new Error("REFUSED:INCOMPLETE_RECEIPT_AUTHORITY");
  if (!Number.isInteger(context.trustEpoch) || context.trustEpoch < 0) throw new Error("REFUSED:INVALID_TRUST_EPOCH");
  if (!Number.isFinite(Date.parse(input.observedAt))) throw new Error("REFUSED:INVALID_EVIDENCE_TIME");

  const payloadDigest = blake3HexUtf8(canonicalJson(observationPayload(input)));
  const core: ReceiptCoreV2 = {
    version: "CASTLE-RECEIPT-V2",
    algorithm: "BLAKE3-256",
    signatureAlgorithm: "Ed25519",
    payloadDigest,
    subject: input.subject,
    metric: input.metric,
    policyDigest: context.policyDigest,
    authorityDigest: context.authorityDigest,
    parentDigests: [...new Set(context.parentDigests ?? [])].sort(),
    keyId: context.keyId,
    trustEpoch: context.trustEpoch,
    issuedAt: input.observedAt,
    assuranceDomain: context.assuranceDomain,
  };
  for (const parent of core.parentDigests) validateDigest("PARENT", parent);
  const receiptDigest = receiptCoreDigest(core);
  const signature = ed25519Sign(null, Buffer.from(receiptDigest, "hex"), privateKeyPem).toString("base64");
  const receipt: ReceiptV2 = { ...core, receiptDigest, signature };
  return {
    observation: { ...input, receiptDigest },
    receipt,
  };
}

export interface ReceiptVerification {
  standing: "ALIVE" | "REFUSED";
  reasons: readonly string[];
  verifiedDigests: readonly string[];
}

function verifyReceiptNode(
  receipt: ReceiptV2,
  trust: TrustStore,
  store: ReadonlyMap<string, ReceiptV2>,
  visiting: Set<string>,
  verified: Set<string>,
  reasons: string[],
): void {
  if (verified.has(receipt.receiptDigest)) return;
  if (visiting.has(receipt.receiptDigest)) {
    reasons.push("REFUSED:RECEIPT_DAG_CYCLE");
    return;
  }
  visiting.add(receipt.receiptDigest);

  if (receipt.version !== "CASTLE-RECEIPT-V2") reasons.push("REFUSED:UNSUPPORTED_RECEIPT_VERSION");
  if (receipt.algorithm !== "BLAKE3-256") reasons.push("REFUSED:UNSUPPORTED_RECEIPT_DIGEST_ALGORITHM");
  if (receipt.signatureAlgorithm !== "Ed25519") reasons.push("REFUSED:UNSUPPORTED_SIGNATURE_ALGORITHM");
  if (!DIGEST_RE.test(receipt.receiptDigest)) reasons.push("REFUSED:INVALID_RECEIPT_DIGEST");
  if (!DIGEST_RE.test(receipt.payloadDigest)) reasons.push("REFUSED:INVALID_PAYLOAD_DIGEST");
  if (!DIGEST_RE.test(receipt.policyDigest)) reasons.push("REFUSED:INVALID_POLICY_DIGEST");
  if (!DIGEST_RE.test(receipt.authorityDigest)) reasons.push("REFUSED:INVALID_AUTHORITY_DIGEST");
  if (!Number.isInteger(receipt.trustEpoch) || receipt.trustEpoch < 0 || receipt.trustEpoch > trust.currentEpoch) {
    reasons.push("REFUSED:INVALID_TRUST_EPOCH");
  }

  const { receiptDigest: _digest, signature: _signature, ...core } = receipt;
  const expectedDigest = receiptCoreDigest(core);
  if (expectedDigest !== receipt.receiptDigest) reasons.push("REFUSED:RECEIPT_CONTENT_MISMATCH");

  const key = trust.keys.get(receipt.keyId);
  if (!key) {
    reasons.push("REFUSED:UNTRUSTED_RECEIPT_KEY");
  } else {
    if (receipt.assuranceDomain !== key.assuranceDomain) reasons.push("REFUSED:ASSURANCE_DOMAIN_MISMATCH");
    if (receipt.trustEpoch < key.validFromEpoch) reasons.push("REFUSED:KEY_NOT_YET_VALID");
    if (key.revokedAtEpoch !== undefined && receipt.trustEpoch >= key.revokedAtEpoch) reasons.push("REFUSED:REVOKED_RECEIPT_KEY");
    try {
      const valid = ed25519Verify(
        null,
        Buffer.from(receipt.receiptDigest, "hex"),
        key.publicKeyPem,
        Buffer.from(receipt.signature, "base64"),
      );
      if (!valid) reasons.push("REFUSED:INVALID_RECEIPT_SIGNATURE");
    } catch {
      reasons.push("REFUSED:INVALID_RECEIPT_SIGNATURE");
    }
  }

  for (const parentDigest of receipt.parentDigests) {
    const parent = store.get(parentDigest);
    if (!parent) {
      reasons.push("REFUSED:ORPHAN_RECEIPT_PARENT");
      continue;
    }
    verifyReceiptNode(parent, trust, store, visiting, verified, reasons);
  }

  visiting.delete(receipt.receiptDigest);
  verified.add(receipt.receiptDigest);
}

export function verifyReceiptDag(
  root: ReceiptV2,
  trust: TrustStore,
  store: ReadonlyMap<string, ReceiptV2> = new Map(),
): ReceiptVerification {
  assertBlake3SelfTest();
  const reasons: string[] = [];
  const verified = new Set<string>();
  verifyReceiptNode(root, trust, store, new Set(), verified, reasons);
  return {
    standing: reasons.length === 0 ? "ALIVE" : "REFUSED",
    reasons: reasons.length === 0 ? ["ALIVE:RECEIPT_DAG_VERIFIED"] : [...new Set(reasons)].sort(),
    verifiedDigests: [...verified].sort(),
  };
}

export interface EvidenceAdmission {
  standing: "ALIVE" | "REFUSED";
  reasons: readonly string[];
  observation?: MetricObservation;
}

export function admitEvidence(
  bundle: EvidenceBundle,
  trust: TrustStore,
  store: ReadonlyMap<string, ReceiptV2> = new Map(),
): EvidenceAdmission {
  const reasons: string[] = [];
  const { observation, receipt } = bundle;
  if (observation.receiptDigest !== receipt.receiptDigest) reasons.push("REFUSED:EVIDENCE_RECEIPT_REFERENCE_MISMATCH");
  if (receipt.subject !== observation.subject) reasons.push("REFUSED:EVIDENCE_SUBJECT_MISMATCH");
  if (receipt.metric !== observation.metric) reasons.push("REFUSED:EVIDENCE_METRIC_MISMATCH");
  if (receipt.issuedAt !== observation.observedAt) reasons.push("REFUSED:EVIDENCE_TIME_BINDING_MISMATCH");
  const payloadDigest = blake3HexUtf8(canonicalJson(observationPayload(observation)));
  if (payloadDigest !== receipt.payloadDigest) reasons.push("REFUSED:EVIDENCE_PAYLOAD_MISMATCH");

  const dag = verifyReceiptDag(receipt, trust, store);
  if (dag.standing !== "ALIVE") reasons.push(...dag.reasons);
  return {
    standing: reasons.length === 0 ? "ALIVE" : "REFUSED",
    reasons: reasons.length === 0 ? ["ALIVE:RECEIPTED_EVIDENCE_ADMITTED"] : [...new Set(reasons)].sort(),
    observation: reasons.length === 0 ? observation : undefined,
  };
}

export interface VerifiedQualification {
  standing: Standing;
  qualification: Fortune5Qualification;
  evidenceRefusals: readonly string[];
}

export function qualifyVerifiedFortune5(
  bundles: readonly EvidenceBundle[],
  context: QualificationContext,
  trust: TrustStore,
  receiptStore: ReadonlyMap<string, ReceiptV2> = new Map(),
  requirements: readonly Fortune5Requirement[] = FORTUNE5_REQUIREMENTS,
): VerifiedQualification {
  const admitted: MetricObservation[] = [];
  const refusals: string[] = [];
  for (const bundle of bundles) {
    const admission = admitEvidence(bundle, trust, receiptStore);
    if (admission.standing === "ALIVE" && admission.observation) admitted.push(admission.observation);
    else refusals.push(...admission.reasons.map((reason) => `${bundle.observation.metric}:${reason}`));
  }
  const qualification = qualifyFortune5(admitted, context, requirements);
  return {
    standing: refusals.length > 0 ? "REFUSED" : qualification.standing,
    qualification,
    evidenceRefusals: [...new Set(refusals)].sort(),
  };
}

export interface BoardAdmissionInput {
  enterprise: { context: QualificationContext; evidence: readonly EvidenceBundle[] };
  castle: { context: QualificationContext; evidence: readonly EvidenceBundle[] };
  trust: TrustStore;
  receiptStore?: ReadonlyMap<string, ReceiptV2>;
  castleAssuranceDomain: string;
  independentAssuranceDomain: string;
}

export interface BoardAdmission {
  standing: Standing;
  enterprise: VerifiedQualification;
  castle: VerifiedQualification;
  reasons: readonly string[];
}

function findEvidenceDomain(evidence: readonly EvidenceBundle[], metric: string): string | undefined {
  return evidence.find((item) => item.observation.metric === metric)?.receipt.assuranceDomain;
}

export function qualifyFortune5Board(input: BoardAdmissionInput): BoardAdmission {
  const store = input.receiptStore ?? new Map<string, ReceiptV2>();
  const enterprise = qualifyVerifiedFortune5(input.enterprise.evidence, input.enterprise.context, input.trust, store);
  const castle = qualifyVerifiedFortune5(input.castle.evidence, input.castle.context, input.trust, store);
  const reasons: string[] = [];
  if (enterprise.standing !== "ALIVE") reasons.push(`REFUSED:ENTERPRISE_NOT_ALIVE:${enterprise.standing}`);
  if (castle.standing !== "ALIVE") reasons.push(`REFUSED:CASTLE_NOT_ALIVE:${castle.standing}`);
  if (!input.castleAssuranceDomain || !input.independentAssuranceDomain || input.castleAssuranceDomain === input.independentAssuranceDomain) {
    reasons.push("REFUSED:ASSURANCE_NOT_INDEPENDENT");
  }
  for (const metric of ["independent_verifier_agreement_bps", "board_package_independent_assurance_passed"]) {
    const enterpriseDomain = findEvidenceDomain(input.enterprise.evidence, metric);
    const castleDomain = findEvidenceDomain(input.castle.evidence, metric);
    if (enterpriseDomain !== input.independentAssuranceDomain || castleDomain !== input.independentAssuranceDomain) {
      reasons.push(`REFUSED:INDEPENDENT_ASSURANCE_EVIDENCE:${metric}`);
    }
  }
  return {
    standing: reasons.length === 0 ? "ALIVE" : "REFUSED",
    enterprise,
    castle,
    reasons: reasons.length === 0 ? ["ALIVE:BOARD_ADMISSION_PROVED"] : reasons,
  };
}

export type FailureMode = "FAIL_CLOSED" | "SAFE_DEGRADE" | "LOCAL_CAPABILITY" | "DEFER";

export interface FailureSemanticsInput {
  mode: FailureMode;
  castleAvailable: boolean;
  localCapabilityVerified: boolean;
  receiptChannelAvailable: boolean;
}

export interface FailureSemanticsDecision {
  standing: "ALIVE" | "REFUSED";
  mayActuate: boolean;
  reason: string;
}

export function admitFailureSemantics(input: FailureSemanticsInput): FailureSemanticsDecision {
  if (input.castleAvailable) {
    return {
      standing: input.receiptChannelAvailable ? "ALIVE" : "REFUSED",
      mayActuate: input.receiptChannelAvailable,
      reason: input.receiptChannelAvailable ? "ALIVE:RECEIPTED_PRIMARY_PATH" : "REFUSED:NO_RECEIPT_CHANNEL",
    };
  }
  if (input.mode === "FAIL_CLOSED") return { standing: "ALIVE", mayActuate: false, reason: "ALIVE:FAIL_CLOSED" };
  if (input.mode === "DEFER") return { standing: "ALIVE", mayActuate: false, reason: "ALIVE:DEFERRED" };
  if (!input.receiptChannelAvailable) return { standing: "REFUSED", mayActuate: false, reason: "REFUSED:UNRECEIPTABLE_DEGRADED_DO" };
  if (input.mode === "LOCAL_CAPABILITY" && !input.localCapabilityVerified) {
    return { standing: "REFUSED", mayActuate: false, reason: "REFUSED:LOCAL_CAPABILITY_NOT_VERIFIED" };
  }
  return {
    standing: "ALIVE",
    mayActuate: true,
    reason: input.mode === "LOCAL_CAPABILITY" ? "ALIVE:RECEIPTED_LOCAL_CAPABILITY" : "ALIVE:RECEIPTED_SAFE_DEGRADE",
  };
}

export type MaterialityDimension = "financial" | "operational" | "customer" | "legal" | "regulatory" | "reputational" | "systemic";

export interface MaterialityPolicy {
  policyDigest: string;
  authorityDigest: string;
  perDimensionThresholdBps: Readonly<Record<MaterialityDimension, number>>;
  aggregateThresholdBps: number;
  escalationWithinMs: number;
}

export interface MaterialityEvent {
  id: string;
  subject: string;
  occurredAtEpochMs: number;
  impactBps: Readonly<Record<MaterialityDimension, number>>;
}

export interface MaterialityAssessment {
  material: boolean;
  scoreBps: number;
  triggeringDimensions: readonly MaterialityDimension[];
  escalateByEpochMs?: number;
  policyDigest: string;
  authorityDigest: string;
}

export function assessMateriality(event: MaterialityEvent, policy: MaterialityPolicy): MaterialityAssessment {
  validateDigest("POLICY", policy.policyDigest);
  validateDigest("AUTHORITY", policy.authorityDigest);
  if (!Number.isInteger(policy.aggregateThresholdBps) || policy.aggregateThresholdBps < 0) throw new Error("REFUSED:INVALID_MATERIALITY_POLICY");
  if (!Number.isInteger(policy.escalationWithinMs) || policy.escalationWithinMs < 0) throw new Error("REFUSED:INVALID_ESCALATION_POLICY");
  const dimensions = Object.keys(event.impactBps) as MaterialityDimension[];
  let scoreBps = 0;
  const triggeringDimensions: MaterialityDimension[] = [];
  for (const dimension of dimensions) {
    const impact = event.impactBps[dimension];
    const threshold = policy.perDimensionThresholdBps[dimension];
    if (!Number.isFinite(impact) || impact < 0 || impact > 10000 || !Number.isFinite(threshold) || threshold < 0 || threshold > 10000) {
      throw new Error("REFUSED:INVALID_MATERIALITY_IMPACT");
    }
    scoreBps += impact;
    if (impact >= threshold) triggeringDimensions.push(dimension);
  }
  scoreBps = Math.min(10000, Math.floor(scoreBps / dimensions.length));
  const material = triggeringDimensions.length > 0 || scoreBps >= policy.aggregateThresholdBps;
  return {
    material,
    scoreBps,
    triggeringDimensions: triggeringDimensions.sort(),
    escalateByEpochMs: material ? event.occurredAtEpochMs + policy.escalationWithinMs : undefined,
    policyDigest: policy.policyDigest,
    authorityDigest: policy.authorityDigest,
  };
}

export interface IcfrSubject {
  subject: string;
  processes: readonly string[];
  materialAccounts: readonly string[];
  affectsFinancialReporting: boolean;
}

export interface IcfrClassification {
  subject: string;
  inScope: boolean;
  reasons: readonly string[];
}

export function classifyIcfrSubject(input: IcfrSubject): IcfrClassification {
  const reasons: string[] = [];
  if (input.affectsFinancialReporting) reasons.push("affects-financial-reporting");
  if (input.materialAccounts.length > 0) reasons.push("material-account-impact");
  if (input.processes.some((process) => ["revenue", "payments", "payroll", "purchasing", "inventory", "general-ledger", "financial-reporting"].includes(process))) {
    reasons.push("key-financial-process");
  }
  return { subject: input.subject, inScope: reasons.length > 0, reasons: reasons.sort() };
}

export interface RoleAssignment {
  principal: string;
  role: string;
}

export interface SodViolation {
  principal: string;
  roles: readonly [string, string];
}

export function detectSegregationOfDutyViolations(
  assignments: readonly RoleAssignment[],
  incompatibleRolePairs: readonly (readonly [string, string])[],
): SodViolation[] {
  const byPrincipal = new Map<string, Set<string>>();
  for (const assignment of assignments) {
    const roles = byPrincipal.get(assignment.principal) ?? new Set<string>();
    roles.add(assignment.role);
    byPrincipal.set(assignment.principal, roles);
  }
  const violations: SodViolation[] = [];
  for (const [principal, roles] of [...byPrincipal].sort(([a], [b]) => a.localeCompare(b))) {
    for (const pair of incompatibleRolePairs) {
      if (roles.has(pair[0]) && roles.has(pair[1])) {
        violations.push({ principal, roles: [pair[0], pair[1]] });
      }
    }
  }
  return violations.sort((a, b) => `${a.principal}|${a.roles.join("|")}`.localeCompare(`${b.principal}|${b.roles.join("|")}`));
}

export interface BoardPackage {
  profile: "CASTLE_FORTUNE5_BOARD_V1";
  enterpriseSubject: string;
  castleSubject: string;
  generatedAt: string;
  enterpriseStanding: Standing;
  castleStanding: Standing;
  materialRefusedSubjects: number;
  riskAppetiteBreaches: number;
  controlCount: number;
  evidenceDigest: string;
}

export function buildBoardPackage(
  admission: BoardAdmission,
  generatedAt: string,
  materialRefusedSubjects: number,
  riskAppetiteBreaches: number,
): BoardPackage {
  if (admission.standing !== "ALIVE") throw new Error("REFUSED:BOARD_PACKAGE_WITHOUT_BOARD_ADMISSION");
  if (!Number.isFinite(Date.parse(generatedAt))) throw new Error("REFUSED:INVALID_BOARD_PACKAGE_TIME");
  if (!Number.isInteger(materialRefusedSubjects) || materialRefusedSubjects < 0) throw new Error("REFUSED:INVALID_MATERIAL_REFUSED_COUNT");
  if (!Number.isInteger(riskAppetiteBreaches) || riskAppetiteBreaches < 0) throw new Error("REFUSED:INVALID_RISK_APPETITE_BREACH_COUNT");
  const evidenceDigest = blake3HexUtf8(canonicalJson({
    enterprise: admission.enterprise.qualification.controls,
    castle: admission.castle.qualification.controls,
  }));
  return {
    profile: "CASTLE_FORTUNE5_BOARD_V1",
    enterpriseSubject: admission.enterprise.qualification.subject,
    castleSubject: admission.castle.qualification.subject,
    generatedAt,
    enterpriseStanding: admission.enterprise.standing,
    castleStanding: admission.castle.standing,
    materialRefusedSubjects,
    riskAppetiteBreaches,
    controlCount: FORTUNE5_REQUIREMENTS.length,
    evidenceDigest,
  };
}

export function boardRequirements(): readonly Fortune5Requirement[] {
  return FORTUNE5_REQUIREMENTS;
}
