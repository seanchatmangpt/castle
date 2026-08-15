import test from "node:test";
import assert from "node:assert/strict";
import { generateKeyPairSync } from "node:crypto";
import { BLAKE3_EMPTY_HEX, blake3HexUtf8 } from "../src/blake3.ts";
import {
  BOARD_ADMISSION_REQUIREMENTS,
  admitEvidence,
  admitFailureSemantics,
  assessMateriality,
  boardAdversarialGoals,
  buildBoardPackage,
  classifyIcfrSubject,
  detectSegregationOfDutyViolations,
  issueEvidenceReceipt,
  qualifyFortune5Board,
  type EvidenceBundle,
  type TrustKey,
  type TrustStore,
} from "../src/board.ts";

const POLICY = "1".repeat(64);
const AUTHORITY = "2".repeat(64);
const OBSERVED_AT = "2026-08-15T20:00:00.000Z";

const internalPair = generateKeyPairSync("ed25519");
const independentPair = generateKeyPairSync("ed25519");
const internalPrivate = internalPair.privateKey.export({ type: "pkcs8", format: "pem" }).toString();
const internalPublic = internalPair.publicKey.export({ type: "spki", format: "pem" }).toString();
const independentPrivate = independentPair.privateKey.export({ type: "pkcs8", format: "pem" }).toString();
const independentPublic = independentPair.publicKey.export({ type: "spki", format: "pem" }).toString();

const internalKey: TrustKey = {
  keyId: "castle-internal-key",
  publicKeyPem: internalPublic,
  validFromEpoch: 1,
  assuranceDomain: "castle-internal",
};
const independentKey: TrustKey = {
  keyId: "independent-assurance-key",
  publicKeyPem: independentPublic,
  validFromEpoch: 1,
  assuranceDomain: "independent-assurance",
};
const trust: TrustStore = {
  currentEpoch: 1,
  keys: new Map([
    [internalKey.keyId, internalKey],
    [independentKey.keyId, independentKey],
  ]),
};

function targetValue(target: string): number | boolean | string {
  if (target === "true") return true;
  if (target === "false") return false;
  if (/^-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?$/.test(target)) return Number(target);
  return target;
}

function evidenceForSubject(subject: string, overrides: Readonly<Record<string, number | boolean | string>> = {}): EvidenceBundle[] {
  return BOARD_ADMISSION_REQUIREMENTS.map((requirement) => {
    const independent = ["independent_verifier_agreement_bps", "board_package_independent_assurance_passed"].includes(requirement.metric);
    return issueEvidenceReceipt(
      {
        metric: requirement.metric,
        value: overrides[requirement.metric] ?? targetValue(requirement.target),
        subject,
        observedAt: OBSERVED_AT,
        epistemicClass: "OBSERVED",
      },
      {
        policyDigest: POLICY,
        authorityDigest: AUTHORITY,
        keyId: independent ? independentKey.keyId : internalKey.keyId,
        trustEpoch: 1,
        assuranceDomain: independent ? independentKey.assuranceDomain : internalKey.assuranceDomain,
      },
      independent ? independentPrivate : internalPrivate,
    );
  });
}

test("DfCM board pack composes 40 base controls with 49 board controls and 10 prohibited goals", () => {
  assert.equal(BOARD_ADMISSION_REQUIREMENTS.length, 89);
  assert.equal(new Set(BOARD_ADMISSION_REQUIREMENTS.map((requirement) => requirement.controlId)).size, 89);
  assert.equal(boardAdversarialGoals().length, 10);
  assert.ok(boardAdversarialGoals().every((goal) => goal.authority === "PROHIBITED"));
});

test("in-repo BLAKE3 implementation matches the standard empty-input vector", () => {
  assert.equal(blake3HexUtf8(""), BLAKE3_EMPTY_HEX);
});

test("evidence admission verifies payload, signature, trust root, revocation, and receipt parents", () => {
  const parent = issueEvidenceReceipt(
    { metric: "parent", value: true, subject: "subject:a", observedAt: OBSERVED_AT, epistemicClass: "OBSERVED" },
    { policyDigest: POLICY, authorityDigest: AUTHORITY, keyId: internalKey.keyId, trustEpoch: 1, assuranceDomain: internalKey.assuranceDomain },
    internalPrivate,
  );
  const child = issueEvidenceReceipt(
    { metric: "child", value: 1, subject: "subject:a", observedAt: OBSERVED_AT, epistemicClass: "OBSERVED" },
    {
      policyDigest: POLICY,
      authorityDigest: AUTHORITY,
      parentDigests: [parent.receipt.receiptDigest],
      keyId: internalKey.keyId,
      trustEpoch: 1,
      assuranceDomain: internalKey.assuranceDomain,
    },
    internalPrivate,
  );

  assert.equal(admitEvidence(child, trust, new Map([[parent.receipt.receiptDigest, parent.receipt]])).standing, "ALIVE");
  assert.ok(admitEvidence(child, trust).reasons.includes("REFUSED:ORPHAN_RECEIPT_PARENT"));

  const tampered = { ...child, observation: { ...child.observation, value: 2 } };
  assert.ok(admitEvidence(tampered, trust, new Map([[parent.receipt.receiptDigest, parent.receipt]])).reasons.includes("REFUSED:EVIDENCE_PAYLOAD_MISMATCH"));

  const revokedTrust: TrustStore = {
    currentEpoch: 1,
    keys: new Map([[internalKey.keyId, { ...internalKey, revokedAtEpoch: 1 }]]),
  };
  assert.ok(admitEvidence(parent, revokedTrust).reasons.includes("REFUSED:REVOKED_RECEIPT_KEY"));
});

test("Fortune-5 board admission requires both enterprise and CASTLE ALIVE plus independent assurance", () => {
  const enterpriseSubject = "enterprise:fortune5:test";
  const castleSubject = "castle:control-plane";
  const result = qualifyFortune5Board({
    enterprise: { context: { subject: enterpriseSubject }, evidence: evidenceForSubject(enterpriseSubject) },
    castle: { context: { subject: castleSubject }, evidence: evidenceForSubject(castleSubject) },
    trust,
    castleAssuranceDomain: "castle-internal",
    independentAssuranceDomain: "independent-assurance",
  });
  assert.equal(result.standing, "ALIVE");
  assert.equal(result.enterprise.qualification.alive, 89);
  assert.equal(result.castle.qualification.alive, 89);

  const packageResult = buildBoardPackage(result, "2026-08-15T20:01:00.000Z", 0, 0);
  assert.equal(packageResult.controlCount, 89);
  assert.match(packageResult.evidenceDigest, /^[0-9a-f]{64}$/);
});

test("CASTLE cannot crown an enterprise when CASTLE self-governance is refused", () => {
  const enterpriseSubject = "enterprise:fortune5:test";
  const castleSubject = "castle:control-plane";
  const result = qualifyFortune5Board({
    enterprise: { context: { subject: enterpriseSubject }, evidence: evidenceForSubject(enterpriseSubject) },
    castle: {
      context: { subject: castleSubject },
      evidence: evidenceForSubject(castleSubject, { castle_self_exemption_path_count: 1 }),
    },
    trust,
    castleAssuranceDomain: "castle-internal",
    independentAssuranceDomain: "independent-assurance",
  });
  assert.equal(result.standing, "REFUSED");
  assert.ok(result.reasons.some((reason) => reason.startsWith("REFUSED:CASTLE_NOT_ALIVE")));
});

test("failure semantics never manufacture unreceipted degraded-mode authority", () => {
  assert.deepEqual(
    admitFailureSemantics({ mode: "FAIL_CLOSED", castleAvailable: false, localCapabilityVerified: false, receiptChannelAvailable: false }),
    { standing: "ALIVE", mayActuate: false, reason: "ALIVE:FAIL_CLOSED" },
  );
  assert.equal(
    admitFailureSemantics({ mode: "LOCAL_CAPABILITY", castleAvailable: false, localCapabilityVerified: true, receiptChannelAvailable: false }).standing,
    "REFUSED",
  );
  assert.equal(
    admitFailureSemantics({ mode: "LOCAL_CAPABILITY", castleAvailable: false, localCapabilityVerified: true, receiptChannelAvailable: true }).mayActuate,
    true,
  );
});

test("materiality is deterministic and produces an escalation deadline", () => {
  const assessment = assessMateriality(
    {
      id: "event:1",
      subject: "enterprise:fortune5:test",
      occurredAtEpochMs: 1_000,
      impactBps: {
        financial: 8000,
        operational: 1000,
        customer: 1000,
        legal: 0,
        regulatory: 0,
        reputational: 0,
        systemic: 0,
      },
    },
    {
      policyDigest: POLICY,
      authorityDigest: AUTHORITY,
      perDimensionThresholdBps: {
        financial: 5000,
        operational: 5000,
        customer: 5000,
        legal: 5000,
        regulatory: 5000,
        reputational: 5000,
        systemic: 5000,
      },
      aggregateThresholdBps: 5000,
      escalationWithinMs: 60_000,
    },
  );
  assert.equal(assessment.material, true);
  assert.deepEqual(assessment.triggeringDimensions, ["financial"]);
  assert.equal(assessment.escalateByEpochMs, 61_000);
});

test("ICFR scope and segregation of duties are explicit machine predicates", () => {
  const classification = classifyIcfrSubject({
    subject: "service:ledger-writer",
    processes: ["general-ledger"],
    materialAccounts: ["cash"],
    affectsFinancialReporting: true,
  });
  assert.equal(classification.inScope, true);

  const violations = detectSegregationOfDutyViolations(
    [
      { principal: "alice", role: "prepare-payment" },
      { principal: "alice", role: "approve-payment" },
      { principal: "bob", role: "prepare-payment" },
    ],
    [["prepare-payment", "approve-payment"]],
  );
  assert.deepEqual(violations, [{ principal: "alice", roles: ["prepare-payment", "approve-payment"] }]);
});