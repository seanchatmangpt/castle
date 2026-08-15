import test from "node:test";
import assert from "node:assert/strict";
import { generateKeyPairSync, sign as ed25519Sign, verify as ed25519Verify } from "node:crypto";
import { blake3HexUtf8 } from "../src/blake3.ts";
import { FORTUNE5_REQUIREMENTS } from "../src/fortune5.generated.ts";
import type { Fortune5Qualification } from "../src/fortune5.ts";
import {
  issueEvidenceReceipt,
  verifyReceiptDag,
  admitEvidence,
  qualifyVerifiedFortune5,
  qualifyFortune5Board,
  admitFailureSemantics,
  assessMateriality,
  classifyIcfrSubject,
  detectSegregationOfDutyViolations,
  buildBoardPackage,
  boardRequirements,
  type EvidenceInput,
  type EvidenceBundle,
  type ReceiptIssueContext,
  type ReceiptV2,
  type ReceiptCoreV2,
  type TrustStore,
  type VerifiedQualification,
  type BoardAdmission,
} from "../src/board.ts";

function canonicalJson(value: unknown): string {
  if (value === null || typeof value !== "object") {
    if (typeof value === "number" && !Number.isFinite(value)) throw new Error("REFUSED:NON_FINITE_CANONICAL_VALUE");
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  const record = value as Record<string, unknown>;
  return `{${Object.keys(record).sort().map((key) => `${JSON.stringify(key)}:${canonicalJson(record[key])}`).join(",")}}`;
}

function targetValue(target: string): number | boolean | string {
  if (target === "true") return true;
  if (target === "false") return false;
  if (/^-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?$/.test(target)) return Number(target);
  return target;
}

// ---------------------------------------------------------------------------
// CHUNK 1: evidence-receipts
// ---------------------------------------------------------------------------

function makeKeyPair() {
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  return {
    publicKeyPem: publicKey.export({ type: "spki", format: "pem" }).toString(),
    privateKeyPem: privateKey.export({ type: "pkcs8", format: "pem" }).toString(),
  };
}

const RECEIPT_SUBJECT = "castle:enterprise:receipt-test";
const RECEIPT_INPUT: EvidenceInput = {
  metric: "deny_by_default",
  value: true,
  subject: RECEIPT_SUBJECT,
  observedAt: "2026-08-15T19:59:00.000Z",
  epistemicClass: "OBSERVED",
};

function baseContext(overrides: Partial<ReceiptIssueContext> = {}): ReceiptIssueContext {
  return {
    policyDigest: "1".repeat(64),
    authorityDigest: "2".repeat(64),
    keyId: "key-1",
    trustEpoch: 1,
    assuranceDomain: "castle-primary",
    ...overrides,
  };
}

test("valid issuance produces a receipt whose receiptDigest and signature verify", () => {
  const { publicKeyPem, privateKeyPem } = makeKeyPair();
  const bundle = issueEvidenceReceipt(RECEIPT_INPUT, baseContext(), privateKeyPem);

  assert.equal(bundle.receipt.version, "CASTLE-RECEIPT-V2");
  assert.equal(bundle.receipt.algorithm, "BLAKE3-256");
  assert.equal(bundle.receipt.signatureAlgorithm, "Ed25519");
  assert.match(bundle.receipt.receiptDigest, /^[0-9a-f]{64}$/);
  assert.equal(bundle.observation.receiptDigest, bundle.receipt.receiptDigest);

  const { receiptDigest, signature, ...core } = bundle.receipt;
  const recomputedDigest = blake3HexUtf8(
    JSON.stringify(
      Object.keys(core)
        .sort()
        .reduce((acc: Record<string, unknown>, key) => {
          acc[key] = (core as Record<string, unknown>)[key];
          return acc;
        }, {}),
    ),
  );
  // receiptDigest must be internally consistent and the signature must verify against it.
  assert.equal(recomputedDigest, receiptDigest);
  const valid = ed25519Verify(
    null,
    Buffer.from(receiptDigest, "hex"),
    publicKeyPem,
    Buffer.from(signature, "base64"),
  );
  assert.equal(valid, true);

  const tamperedValid = ed25519Verify(
    null,
    Buffer.from("0".repeat(64), "hex"),
    publicKeyPem,
    Buffer.from(signature, "base64"),
  );
  assert.equal(tamperedValid, false);
});

test("issuance rejects malformed policyDigest and authorityDigest", () => {
  const { privateKeyPem } = makeKeyPair();
  assert.throws(
    () => issueEvidenceReceipt(RECEIPT_INPUT, baseContext({ policyDigest: "not-hex" }), privateKeyPem),
    /REFUSED:INVALID_POLICY_DIGEST/,
  );
  assert.throws(
    () => issueEvidenceReceipt(RECEIPT_INPUT, baseContext({ policyDigest: "a".repeat(63) }), privateKeyPem),
    /REFUSED:INVALID_POLICY_DIGEST/,
  );
  assert.throws(
    () => issueEvidenceReceipt(RECEIPT_INPUT, baseContext({ authorityDigest: "ZZZZ" }), privateKeyPem),
    /REFUSED:INVALID_AUTHORITY_DIGEST/,
  );
  assert.throws(
    () => issueEvidenceReceipt(RECEIPT_INPUT, baseContext({ authorityDigest: "a".repeat(65) }), privateKeyPem),
    /REFUSED:INVALID_AUTHORITY_DIGEST/,
  );
});

test("issuance rejects missing keyId or assuranceDomain", () => {
  const { privateKeyPem } = makeKeyPair();
  assert.throws(
    () => issueEvidenceReceipt(RECEIPT_INPUT, baseContext({ keyId: "" }), privateKeyPem),
    /REFUSED:INCOMPLETE_RECEIPT_AUTHORITY/,
  );
  assert.throws(
    () => issueEvidenceReceipt(RECEIPT_INPUT, baseContext({ assuranceDomain: "" }), privateKeyPem),
    /REFUSED:INCOMPLETE_RECEIPT_AUTHORITY/,
  );
});

test("issuance rejects non-integer or negative trustEpoch", () => {
  const { privateKeyPem } = makeKeyPair();
  assert.throws(
    () => issueEvidenceReceipt(RECEIPT_INPUT, baseContext({ trustEpoch: 1.5 }), privateKeyPem),
    /REFUSED:INVALID_TRUST_EPOCH/,
  );
  assert.throws(
    () => issueEvidenceReceipt(RECEIPT_INPUT, baseContext({ trustEpoch: -1 }), privateKeyPem),
    /REFUSED:INVALID_TRUST_EPOCH/,
  );
  assert.throws(
    () => issueEvidenceReceipt(RECEIPT_INPUT, baseContext({ trustEpoch: Number.NaN }), privateKeyPem),
    /REFUSED:INVALID_TRUST_EPOCH/,
  );
});

test("issuance rejects invalid observedAt time", () => {
  const { privateKeyPem } = makeKeyPair();
  assert.throws(
    () =>
      issueEvidenceReceipt(
        { ...RECEIPT_INPUT, observedAt: "not-a-date" },
        baseContext(),
        privateKeyPem,
      ),
    /REFUSED:INVALID_EVIDENCE_TIME/,
  );
  assert.throws(
    () =>
      issueEvidenceReceipt(
        { ...RECEIPT_INPUT, observedAt: "" },
        baseContext(),
        privateKeyPem,
      ),
    /REFUSED:INVALID_EVIDENCE_TIME/,
  );
});

test("issuance dedupes and sorts parentDigests", () => {
  const { privateKeyPem } = makeKeyPair();
  const parentA = "a".repeat(64);
  const parentB = "b".repeat(64);
  const bundle = issueEvidenceReceipt(
    RECEIPT_INPUT,
    baseContext({ parentDigests: [parentB, parentA, parentB, parentA] }),
    privateKeyPem,
  );
  assert.deepEqual(bundle.receipt.parentDigests, [parentA, parentB]);
});

test("issuance rejects malformed parent digests", () => {
  const { privateKeyPem } = makeKeyPair();
  assert.throws(
    () =>
      issueEvidenceReceipt(
        RECEIPT_INPUT,
        baseContext({ parentDigests: ["not-a-digest"] }),
        privateKeyPem,
      ),
    /REFUSED:INVALID_PARENT_DIGEST/,
  );
  assert.throws(
    () =>
      issueEvidenceReceipt(
        RECEIPT_INPUT,
        baseContext({ parentDigests: ["a".repeat(64), "b".repeat(63)] }),
        privateKeyPem,
      ),
    /REFUSED:INVALID_PARENT_DIGEST/,
  );
});

// ---------------------------------------------------------------------------
// CHUNK 2: receipt-dag
// ---------------------------------------------------------------------------

test("verifyReceiptDag: valid single receipt against matching TrustStore verifies ALIVE", () => {
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const publicKeyPem = publicKey.export({ type: "spki", format: "pem" }).toString();
  const privateKeyPem = privateKey.export({ type: "pkcs8", format: "pem" }).toString();
  const keyId = "key-1";
  const assuranceDomain = "domain-a";
  const trust: TrustStore = {
    currentEpoch: 10,
    keys: new Map([[keyId, { keyId, publicKeyPem, validFromEpoch: 0, assuranceDomain }]]),
  };
  const { receipt } = issueEvidenceReceipt(
    { metric: "m1", value: 1, subject: "s1", observedAt: "2026-01-01T00:00:00.000Z", epistemicClass: "OBSERVED" },
    { policyDigest: blake3HexUtf8("policy"), authorityDigest: blake3HexUtf8("authority"), keyId, trustEpoch: 5, assuranceDomain },
    privateKeyPem,
  );
  const result = verifyReceiptDag(receipt, trust);
  assert.equal(result.standing, "ALIVE");
  assert.deepEqual(result.reasons, ["ALIVE:RECEIPT_DAG_VERIFIED"]);
  assert.deepEqual(result.verifiedDigests, [receipt.receiptDigest]);
});

test("verifyReceiptDag: unknown keyId is REFUSED:UNTRUSTED_RECEIPT_KEY", () => {
  const { privateKey } = generateKeyPairSync("ed25519");
  const privateKeyPem = privateKey.export({ type: "pkcs8", format: "pem" }).toString();
  const trust: TrustStore = { currentEpoch: 10, keys: new Map() };
  const { receipt } = issueEvidenceReceipt(
    { metric: "m1", value: 1, subject: "s1", observedAt: "2026-01-01T00:00:00.000Z", epistemicClass: "OBSERVED" },
    { policyDigest: blake3HexUtf8("policy"), authorityDigest: blake3HexUtf8("authority"), keyId: "unknown-key", trustEpoch: 5, assuranceDomain: "domain-a" },
    privateKeyPem,
  );
  const result = verifyReceiptDag(receipt, trust);
  assert.equal(result.standing, "REFUSED");
  assert.ok(result.reasons.includes("REFUSED:UNTRUSTED_RECEIPT_KEY"));
});

test("verifyReceiptDag: revoked key (revokedAtEpoch <= trustEpoch) is REFUSED:REVOKED_RECEIPT_KEY", () => {
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const publicKeyPem = publicKey.export({ type: "spki", format: "pem" }).toString();
  const privateKeyPem = privateKey.export({ type: "pkcs8", format: "pem" }).toString();
  const keyId = "key-revoked";
  const assuranceDomain = "domain-a";
  const trust: TrustStore = {
    currentEpoch: 10,
    keys: new Map([[keyId, { keyId, publicKeyPem, validFromEpoch: 0, revokedAtEpoch: 5, assuranceDomain }]]),
  };
  const { receipt } = issueEvidenceReceipt(
    { metric: "m1", value: 1, subject: "s1", observedAt: "2026-01-01T00:00:00.000Z", epistemicClass: "OBSERVED" },
    { policyDigest: blake3HexUtf8("policy"), authorityDigest: blake3HexUtf8("authority"), keyId, trustEpoch: 5, assuranceDomain },
    privateKeyPem,
  );
  const result = verifyReceiptDag(receipt, trust);
  assert.equal(result.standing, "REFUSED");
  assert.ok(result.reasons.includes("REFUSED:REVOKED_RECEIPT_KEY"));
});

test("verifyReceiptDag: key not yet valid (validFromEpoch > trustEpoch) is REFUSED:KEY_NOT_YET_VALID", () => {
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const publicKeyPem = publicKey.export({ type: "spki", format: "pem" }).toString();
  const privateKeyPem = privateKey.export({ type: "pkcs8", format: "pem" }).toString();
  const keyId = "key-future";
  const assuranceDomain = "domain-a";
  const trust: TrustStore = {
    currentEpoch: 10,
    keys: new Map([[keyId, { keyId, publicKeyPem, validFromEpoch: 8, assuranceDomain }]]),
  };
  const { receipt } = issueEvidenceReceipt(
    { metric: "m1", value: 1, subject: "s1", observedAt: "2026-01-01T00:00:00.000Z", epistemicClass: "OBSERVED" },
    { policyDigest: blake3HexUtf8("policy"), authorityDigest: blake3HexUtf8("authority"), keyId, trustEpoch: 5, assuranceDomain },
    privateKeyPem,
  );
  const result = verifyReceiptDag(receipt, trust);
  assert.equal(result.standing, "REFUSED");
  assert.ok(result.reasons.includes("REFUSED:KEY_NOT_YET_VALID"));
});

test("verifyReceiptDag: wrong assuranceDomain is REFUSED:ASSURANCE_DOMAIN_MISMATCH", () => {
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const publicKeyPem = publicKey.export({ type: "spki", format: "pem" }).toString();
  const privateKeyPem = privateKey.export({ type: "pkcs8", format: "pem" }).toString();
  const keyId = "key-domain";
  const trust: TrustStore = {
    currentEpoch: 10,
    keys: new Map([[keyId, { keyId, publicKeyPem, validFromEpoch: 0, assuranceDomain: "domain-trusted" }]]),
  };
  const { receipt } = issueEvidenceReceipt(
    { metric: "m1", value: 1, subject: "s1", observedAt: "2026-01-01T00:00:00.000Z", epistemicClass: "OBSERVED" },
    { policyDigest: blake3HexUtf8("policy"), authorityDigest: blake3HexUtf8("authority"), keyId, trustEpoch: 5, assuranceDomain: "domain-other" },
    privateKeyPem,
  );
  const result = verifyReceiptDag(receipt, trust);
  assert.equal(result.standing, "REFUSED");
  assert.ok(result.reasons.includes("REFUSED:ASSURANCE_DOMAIN_MISMATCH"));
});

test("verifyReceiptDag: tampered receipt content (mismatched digest) is REFUSED:RECEIPT_CONTENT_MISMATCH", () => {
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const publicKeyPem = publicKey.export({ type: "spki", format: "pem" }).toString();
  const privateKeyPem = privateKey.export({ type: "pkcs8", format: "pem" }).toString();
  const keyId = "key-tamper";
  const assuranceDomain = "domain-a";
  const trust: TrustStore = {
    currentEpoch: 10,
    keys: new Map([[keyId, { keyId, publicKeyPem, validFromEpoch: 0, assuranceDomain }]]),
  };
  const { receipt } = issueEvidenceReceipt(
    { metric: "m1", value: 1, subject: "s1", observedAt: "2026-01-01T00:00:00.000Z", epistemicClass: "OBSERVED" },
    { policyDigest: blake3HexUtf8("policy"), authorityDigest: blake3HexUtf8("authority"), keyId, trustEpoch: 5, assuranceDomain },
    privateKeyPem,
  );
  const tampered: ReceiptV2 = { ...receipt, subject: "s1-tampered" };
  const result = verifyReceiptDag(tampered, trust);
  assert.equal(result.standing, "REFUSED");
  assert.ok(result.reasons.includes("REFUSED:RECEIPT_CONTENT_MISMATCH"));
});

test("verifyReceiptDag: invalid signature is REFUSED:INVALID_RECEIPT_SIGNATURE", () => {
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const { privateKey: otherPrivateKey } = generateKeyPairSync("ed25519");
  const publicKeyPem = publicKey.export({ type: "spki", format: "pem" }).toString();
  const otherPrivateKeyPem = otherPrivateKey.export({ type: "pkcs8", format: "pem" }).toString();
  const keyId = "key-sig";
  const assuranceDomain = "domain-a";
  const trust: TrustStore = {
    currentEpoch: 10,
    keys: new Map([[keyId, { keyId, publicKeyPem, validFromEpoch: 0, assuranceDomain }]]),
  };
  const { receipt } = issueEvidenceReceipt(
    { metric: "m1", value: 1, subject: "s1", observedAt: "2026-01-01T00:00:00.000Z", epistemicClass: "OBSERVED" },
    { policyDigest: blake3HexUtf8("policy"), authorityDigest: blake3HexUtf8("authority"), keyId, trustEpoch: 5, assuranceDomain },
    otherPrivateKeyPem,
  );
  const result = verifyReceiptDag(receipt, trust);
  assert.equal(result.standing, "REFUSED");
  assert.ok(result.reasons.includes("REFUSED:INVALID_RECEIPT_SIGNATURE"));
});

test("verifyReceiptDag: parent chain with an orphan parent digest is REFUSED:ORPHAN_RECEIPT_PARENT", () => {
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const publicKeyPem = publicKey.export({ type: "spki", format: "pem" }).toString();
  const privateKeyPem = privateKey.export({ type: "pkcs8", format: "pem" }).toString();
  const keyId = "key-orphan";
  const assuranceDomain = "domain-a";
  const trust: TrustStore = {
    currentEpoch: 10,
    keys: new Map([[keyId, { keyId, publicKeyPem, validFromEpoch: 0, assuranceDomain }]]),
  };
  const orphanParentDigest = blake3HexUtf8("nonexistent-parent");
  const { receipt } = issueEvidenceReceipt(
    { metric: "m1", value: 1, subject: "s1", observedAt: "2026-01-01T00:00:00.000Z", epistemicClass: "OBSERVED" },
    { policyDigest: blake3HexUtf8("policy"), authorityDigest: blake3HexUtf8("authority"), parentDigests: [orphanParentDigest], keyId, trustEpoch: 5, assuranceDomain },
    privateKeyPem,
  );
  const result = verifyReceiptDag(receipt, trust, new Map());
  assert.equal(result.standing, "REFUSED");
  assert.ok(result.reasons.includes("REFUSED:ORPHAN_RECEIPT_PARENT"));
});

test("verifyReceiptDag: cyclic parent chain is REFUSED:RECEIPT_DAG_CYCLE", () => {
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const publicKeyPem = publicKey.export({ type: "spki", format: "pem" }).toString();
  const privateKeyPem = privateKey.export({ type: "pkcs8", format: "pem" }).toString();
  const keyId = "key-cycle";
  const assuranceDomain = "domain-a";
  const trust: TrustStore = {
    currentEpoch: 10,
    keys: new Map([[keyId, { keyId, publicKeyPem, validFromEpoch: 0, assuranceDomain }]]),
  };

  const { receipt: receiptA } = issueEvidenceReceipt(
    { metric: "m1", value: 1, subject: "s1", observedAt: "2026-01-01T00:00:00.000Z", epistemicClass: "OBSERVED" },
    { policyDigest: blake3HexUtf8("policy"), authorityDigest: blake3HexUtf8("authority"), keyId, trustEpoch: 5, assuranceDomain },
    privateKeyPem,
  );
  const { receipt: receiptB } = issueEvidenceReceipt(
    { metric: "m2", value: 2, subject: "s2", observedAt: "2026-01-01T00:00:01.000Z", epistemicClass: "OBSERVED" },
    { policyDigest: blake3HexUtf8("policy"), authorityDigest: blake3HexUtf8("authority"), parentDigests: [receiptA.receiptDigest], keyId, trustEpoch: 5, assuranceDomain },
    privateKeyPem,
  );

  // Force receiptA to point at receiptB as a parent, creating a cycle A -> B -> A.
  const cyclicCoreA: ReceiptCoreV2 = { ...receiptA, parentDigests: [receiptB.receiptDigest] };
  delete (cyclicCoreA as Partial<ReceiptV2>).receiptDigest;
  delete (cyclicCoreA as Partial<ReceiptV2>).signature;
  const cyclicDigestA = blake3HexUtf8(canonicalJson(cyclicCoreA));
  const cyclicSignatureA = ed25519Sign(null, Buffer.from(cyclicDigestA, "hex"), privateKeyPem).toString("base64");
  const cyclicReceiptA: ReceiptV2 = { ...cyclicCoreA, receiptDigest: cyclicDigestA, signature: cyclicSignatureA };

  const store = new Map<string, ReceiptV2>([
    [receiptA.receiptDigest, cyclicReceiptA],
    [receiptB.receiptDigest, receiptB],
  ]);

  const result = verifyReceiptDag(receiptB, trust, store);
  assert.equal(result.standing, "REFUSED");
  assert.ok(result.reasons.includes("REFUSED:RECEIPT_DAG_CYCLE"));
});

test("admitEvidence: detects mismatched observation/receipt fields and reference mismatch", () => {
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const publicKeyPem = publicKey.export({ type: "spki", format: "pem" }).toString();
  const privateKeyPem = privateKey.export({ type: "pkcs8", format: "pem" }).toString();
  const keyId = "key-admit";
  const assuranceDomain = "domain-a";
  const trust: TrustStore = {
    currentEpoch: 10,
    keys: new Map([[keyId, { keyId, publicKeyPem, validFromEpoch: 0, assuranceDomain }]]),
  };
  const bundle = issueEvidenceReceipt(
    { metric: "m1", value: 1, subject: "s1", observedAt: "2026-01-01T00:00:00.000Z", epistemicClass: "OBSERVED" },
    { policyDigest: blake3HexUtf8("policy"), authorityDigest: blake3HexUtf8("authority"), keyId, trustEpoch: 5, assuranceDomain },
    privateKeyPem,
  );

  const validAdmission = admitEvidence(bundle, trust);
  assert.equal(validAdmission.standing, "ALIVE");

  const badSubject = admitEvidence(
    { observation: { ...bundle.observation, subject: "wrong-subject" }, receipt: bundle.receipt },
    trust,
  );
  assert.equal(badSubject.standing, "REFUSED");
  assert.ok(badSubject.reasons.includes("REFUSED:EVIDENCE_SUBJECT_MISMATCH"));

  const badMetric = admitEvidence(
    { observation: { ...bundle.observation, metric: "wrong-metric" }, receipt: bundle.receipt },
    trust,
  );
  assert.equal(badMetric.standing, "REFUSED");
  assert.ok(badMetric.reasons.includes("REFUSED:EVIDENCE_METRIC_MISMATCH"));

  const badTime = admitEvidence(
    { observation: { ...bundle.observation, observedAt: "2026-02-02T00:00:00.000Z" }, receipt: bundle.receipt },
    trust,
  );
  assert.equal(badTime.standing, "REFUSED");
  assert.ok(badTime.reasons.includes("REFUSED:EVIDENCE_TIME_BINDING_MISMATCH"));

  const badPayload = admitEvidence(
    { observation: { ...bundle.observation, value: 999 }, receipt: bundle.receipt },
    trust,
  );
  assert.equal(badPayload.standing, "REFUSED");
  assert.ok(badPayload.reasons.includes("REFUSED:EVIDENCE_PAYLOAD_MISMATCH"));

  const badReference = admitEvidence(
    { observation: { ...bundle.observation, receiptDigest: blake3HexUtf8("wrong-digest") }, receipt: bundle.receipt },
    trust,
  );
  assert.equal(badReference.standing, "REFUSED");
  assert.ok(badReference.reasons.includes("REFUSED:EVIDENCE_RECEIPT_REFERENCE_MISMATCH"));
});

// ---------------------------------------------------------------------------
// CHUNK 3: fortune5-board-admission
// ---------------------------------------------------------------------------

test("qualifyVerifiedFortune5 admits a fully receipted evidence bundle set as ALIVE with no evidence refusals", () => {
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const publicKeyPem = publicKey.export({ type: "spki", format: "pem" }).toString();
  const privateKeyPem = privateKey.export({ type: "pkcs8", format: "pem" }).toString();

  const subject = "castle:enterprise:verified-test";
  const assuranceDomain = "castle:domain:primary";
  const keyId = "key:primary:v1";
  const observedAt = "2026-08-15T19:59:00.000Z";

  const trust: TrustStore = {
    currentEpoch: 5,
    keys: new Map([[keyId, { keyId, publicKeyPem, validFromEpoch: 0, assuranceDomain }]]),
  };

  const issueContext: ReceiptIssueContext = {
    policyDigest: "1".repeat(64),
    authorityDigest: "2".repeat(64),
    keyId,
    trustEpoch: 5,
    assuranceDomain,
  };

  const bundles: EvidenceBundle[] = FORTUNE5_REQUIREMENTS.map((requirement) =>
    issueEvidenceReceipt(
      {
        metric: requirement.metric,
        value: targetValue(requirement.target),
        subject,
        observedAt,
        epistemicClass: "OBSERVED",
      },
      issueContext,
      privateKeyPem,
    ),
  );

  const result = qualifyVerifiedFortune5(
    bundles,
    { subject, nowEpochMs: Date.parse("2026-08-15T20:00:00.000Z"), maxEvidenceAgeMs: 5 * 60 * 1000 },
    trust,
  );

  assert.equal(result.standing, "ALIVE");
  assert.equal(result.qualification.standing, "ALIVE");
  assert.equal(result.qualification.alive, FORTUNE5_REQUIREMENTS.length);
  assert.deepEqual(result.evidenceRefusals, []);
});

test("qualifyVerifiedFortune5 surfaces evidenceRefusals when a bundle payload is tampered", () => {
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const publicKeyPem = publicKey.export({ type: "spki", format: "pem" }).toString();
  const privateKeyPem = privateKey.export({ type: "pkcs8", format: "pem" }).toString();

  const subject = "castle:enterprise:tampered-test";
  const assuranceDomain = "castle:domain:primary";
  const keyId = "key:primary:v1";
  const observedAt = "2026-08-15T19:59:00.000Z";

  const trust: TrustStore = {
    currentEpoch: 5,
    keys: new Map([[keyId, { keyId, publicKeyPem, validFromEpoch: 0, assuranceDomain }]]),
  };

  const issueContext: ReceiptIssueContext = {
    policyDigest: "1".repeat(64),
    authorityDigest: "2".repeat(64),
    keyId,
    trustEpoch: 5,
    assuranceDomain,
  };

  const bundles: EvidenceBundle[] = FORTUNE5_REQUIREMENTS.map((requirement) =>
    issueEvidenceReceipt(
      {
        metric: requirement.metric,
        value: targetValue(requirement.target),
        subject,
        observedAt,
        epistemicClass: "OBSERVED",
      },
      issueContext,
      privateKeyPem,
    ),
  );

  const tamperIndex = bundles.findIndex((bundle) => bundle.observation.metric === "zero_unreceipted_actuations");
  bundles[tamperIndex] = {
    ...bundles[tamperIndex]!,
    observation: { ...bundles[tamperIndex]!.observation, value: 1 },
  };

  const result = qualifyVerifiedFortune5(
    bundles,
    { subject, nowEpochMs: Date.parse("2026-08-15T20:00:00.000Z"), maxEvidenceAgeMs: 5 * 60 * 1000 },
    trust,
  );

  assert.equal(result.standing, "REFUSED");
  assert.ok(result.evidenceRefusals.length > 0);
  assert.ok(result.evidenceRefusals.some((reason) => reason.includes("REFUSED:EVIDENCE_PAYLOAD_MISMATCH")));
});

test("qualifyFortune5Board admits ALIVE when enterprise and castle both qualify and independent-assurance metrics are receipted under a distinct domain", () => {
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const publicKeyPem = publicKey.export({ type: "spki", format: "pem" }).toString();
  const privateKeyPem = privateKey.export({ type: "pkcs8", format: "pem" }).toString();

  const independentPair = generateKeyPairSync("ed25519");
  const independentPublicKeyPem = independentPair.publicKey.export({ type: "spki", format: "pem" }).toString();
  const independentPrivateKeyPem = independentPair.privateKey.export({ type: "pkcs8", format: "pem" }).toString();

  const castleAssuranceDomain = "castle:domain:primary";
  const independentAssuranceDomain = "castle:domain:independent";
  const primaryKeyId = "key:primary:v1";
  const independentKeyId = "key:independent:v1";
  const observedAt = "2026-08-15T19:59:00.000Z";

  const trust: TrustStore = {
    currentEpoch: 5,
    keys: new Map([
      [primaryKeyId, { keyId: primaryKeyId, publicKeyPem, validFromEpoch: 0, assuranceDomain: castleAssuranceDomain }],
      [independentKeyId, { keyId: independentKeyId, publicKeyPem: independentPublicKeyPem, validFromEpoch: 0, assuranceDomain: independentAssuranceDomain }],
    ]),
  };

  // These two metrics are not part of FORTUNE5_REQUIREMENTS; they are the
  // hardcoded independent-assurance metrics that qualifyFortune5Board checks
  // directly, so they must be supplied as additional evidence bundles.
  const independentMetrics = ["independent_verifier_agreement_bps", "board_package_independent_assurance_passed"];

  function buildEvidence(subject: string): EvidenceBundle[] {
    const requirementBundles = FORTUNE5_REQUIREMENTS.map((requirement) => {
      const issueContext: ReceiptIssueContext = {
        policyDigest: "1".repeat(64),
        authorityDigest: "2".repeat(64),
        keyId: primaryKeyId,
        trustEpoch: 5,
        assuranceDomain: castleAssuranceDomain,
      };
      return issueEvidenceReceipt(
        {
          metric: requirement.metric,
          value: targetValue(requirement.target),
          subject,
          observedAt,
          epistemicClass: "OBSERVED",
        },
        issueContext,
        privateKeyPem,
      );
    });
    const independentBundles = independentMetrics.map((metric) => {
      const issueContext: ReceiptIssueContext = {
        policyDigest: "1".repeat(64),
        authorityDigest: "2".repeat(64),
        keyId: independentKeyId,
        trustEpoch: 5,
        assuranceDomain: independentAssuranceDomain,
      };
      return issueEvidenceReceipt(
        {
          metric,
          value: true,
          subject,
          observedAt,
          epistemicClass: "OBSERVED",
        },
        issueContext,
        independentPrivateKeyPem,
      );
    });
    return [...requirementBundles, ...independentBundles];
  }

  const enterpriseEvidence = buildEvidence("castle:enterprise:board-test");
  const castleEvidence = buildEvidence("castle:castle:board-test");

  const nowEpochMs = Date.parse("2026-08-15T20:00:00.000Z");
  const maxEvidenceAgeMs = 5 * 60 * 1000;

  const admission = qualifyFortune5Board({
    enterprise: { context: { subject: "castle:enterprise:board-test", nowEpochMs, maxEvidenceAgeMs }, evidence: enterpriseEvidence },
    castle: { context: { subject: "castle:castle:board-test", nowEpochMs, maxEvidenceAgeMs }, evidence: castleEvidence },
    trust,
    castleAssuranceDomain,
    independentAssuranceDomain,
  });

  assert.equal(admission.standing, "ALIVE");
  assert.equal(admission.enterprise.standing, "ALIVE");
  assert.equal(admission.castle.standing, "ALIVE");
  assert.deepEqual(admission.reasons, ["ALIVE:BOARD_ADMISSION_PROVED"]);

  const sameDomainAdmission = qualifyFortune5Board({
    enterprise: { context: { subject: "castle:enterprise:board-test", nowEpochMs, maxEvidenceAgeMs }, evidence: enterpriseEvidence },
    castle: { context: { subject: "castle:castle:board-test", nowEpochMs, maxEvidenceAgeMs }, evidence: castleEvidence },
    trust,
    castleAssuranceDomain,
    independentAssuranceDomain: castleAssuranceDomain,
  });

  assert.equal(sameDomainAdmission.standing, "REFUSED");
  assert.ok(sameDomainAdmission.reasons.includes("REFUSED:ASSURANCE_NOT_INDEPENDENT"));
});

test("qualifyFortune5Board refuses when independent-assurance metrics are not receipted under the independent domain", () => {
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const publicKeyPem = publicKey.export({ type: "spki", format: "pem" }).toString();
  const privateKeyPem = privateKey.export({ type: "pkcs8", format: "pem" }).toString();

  const castleAssuranceDomain = "castle:domain:primary";
  const independentAssuranceDomain = "castle:domain:independent";
  const primaryKeyId = "key:primary:v1";
  const observedAt = "2026-08-15T19:59:00.000Z";

  const trust: TrustStore = {
    currentEpoch: 5,
    keys: new Map([
      [primaryKeyId, { keyId: primaryKeyId, publicKeyPem, validFromEpoch: 0, assuranceDomain: castleAssuranceDomain }],
    ]),
  };

  function buildAllPrimaryEvidence(subject: string): EvidenceBundle[] {
    const issueContext: ReceiptIssueContext = {
      policyDigest: "1".repeat(64),
      authorityDigest: "2".repeat(64),
      keyId: primaryKeyId,
      trustEpoch: 5,
      assuranceDomain: castleAssuranceDomain,
    };
    return FORTUNE5_REQUIREMENTS.map((requirement) =>
      issueEvidenceReceipt(
        {
          metric: requirement.metric,
          value: targetValue(requirement.target),
          subject,
          observedAt,
          epistemicClass: "OBSERVED",
        },
        issueContext,
        privateKeyPem,
      ),
    );
  }

  const enterpriseEvidence = buildAllPrimaryEvidence("castle:enterprise:board-refusal-test");
  const castleEvidence = buildAllPrimaryEvidence("castle:castle:board-refusal-test");

  const nowEpochMs = Date.parse("2026-08-15T20:00:00.000Z");
  const maxEvidenceAgeMs = 5 * 60 * 1000;

  const admission = qualifyFortune5Board({
    enterprise: { context: { subject: "castle:enterprise:board-refusal-test", nowEpochMs, maxEvidenceAgeMs }, evidence: enterpriseEvidence },
    castle: { context: { subject: "castle:castle:board-refusal-test", nowEpochMs, maxEvidenceAgeMs }, evidence: castleEvidence },
    trust,
    castleAssuranceDomain,
    independentAssuranceDomain,
  });

  assert.equal(admission.standing, "REFUSED");
  assert.ok(admission.reasons.includes("REFUSED:INDEPENDENT_ASSURANCE_EVIDENCE:independent_verifier_agreement_bps"));
  assert.ok(admission.reasons.includes("REFUSED:INDEPENDENT_ASSURANCE_EVIDENCE:board_package_independent_assurance_passed"));
});

// ---------------------------------------------------------------------------
// CHUNK 4: failure-materiality-icfr-sod-package
// ---------------------------------------------------------------------------

test("admitFailureSemantics: castleAvailable+receiptChannelAvailable => ALIVE mayActuate true", () => {
  const result = admitFailureSemantics({
    mode: "FAIL_CLOSED",
    castleAvailable: true,
    localCapabilityVerified: false,
    receiptChannelAvailable: true,
  });
  assert.strictEqual(result.standing, "ALIVE");
  assert.strictEqual(result.mayActuate, true);
  assert.strictEqual(result.reason, "ALIVE:RECEIPTED_PRIMARY_PATH");
});

test("admitFailureSemantics: castleAvailable+no receipt channel => REFUSED", () => {
  const result = admitFailureSemantics({
    mode: "DEFER",
    castleAvailable: true,
    localCapabilityVerified: false,
    receiptChannelAvailable: false,
  });
  assert.strictEqual(result.standing, "REFUSED");
  assert.strictEqual(result.mayActuate, false);
  assert.strictEqual(result.reason, "REFUSED:NO_RECEIPT_CHANNEL");
});

test("admitFailureSemantics: !castleAvailable+FAIL_CLOSED => ALIVE mayActuate false", () => {
  const result = admitFailureSemantics({
    mode: "FAIL_CLOSED",
    castleAvailable: false,
    localCapabilityVerified: false,
    receiptChannelAvailable: true,
  });
  assert.strictEqual(result.standing, "ALIVE");
  assert.strictEqual(result.mayActuate, false);
  assert.strictEqual(result.reason, "ALIVE:FAIL_CLOSED");
});

test("admitFailureSemantics: !castleAvailable+DEFER => ALIVE mayActuate false", () => {
  const result = admitFailureSemantics({
    mode: "DEFER",
    castleAvailable: false,
    localCapabilityVerified: false,
    receiptChannelAvailable: true,
  });
  assert.strictEqual(result.standing, "ALIVE");
  assert.strictEqual(result.mayActuate, false);
  assert.strictEqual(result.reason, "ALIVE:DEFERRED");
});

test("admitFailureSemantics: !castleAvailable+no receipt channel => REFUSED:UNRECEIPTABLE_DEGRADED_DO", () => {
  const result = admitFailureSemantics({
    mode: "SAFE_DEGRADE",
    castleAvailable: false,
    localCapabilityVerified: false,
    receiptChannelAvailable: false,
  });
  assert.strictEqual(result.standing, "REFUSED");
  assert.strictEqual(result.mayActuate, false);
  assert.strictEqual(result.reason, "REFUSED:UNRECEIPTABLE_DEGRADED_DO");
});

test("admitFailureSemantics: !castleAvailable+LOCAL_CAPABILITY+not verified => REFUSED", () => {
  const result = admitFailureSemantics({
    mode: "LOCAL_CAPABILITY",
    castleAvailable: false,
    localCapabilityVerified: false,
    receiptChannelAvailable: true,
  });
  assert.strictEqual(result.standing, "REFUSED");
  assert.strictEqual(result.mayActuate, false);
  assert.strictEqual(result.reason, "REFUSED:LOCAL_CAPABILITY_NOT_VERIFIED");
});

test("admitFailureSemantics: !castleAvailable+LOCAL_CAPABILITY+verified+receiptChannelAvailable => ALIVE mayActuate true", () => {
  const result = admitFailureSemantics({
    mode: "LOCAL_CAPABILITY",
    castleAvailable: false,
    localCapabilityVerified: true,
    receiptChannelAvailable: true,
  });
  assert.strictEqual(result.standing, "ALIVE");
  assert.strictEqual(result.mayActuate, true);
  assert.strictEqual(result.reason, "ALIVE:RECEIPTED_LOCAL_CAPABILITY");
});

test("admitFailureSemantics: !castleAvailable+SAFE_DEGRADE+receiptChannelAvailable => ALIVE mayActuate true", () => {
  const result = admitFailureSemantics({
    mode: "SAFE_DEGRADE",
    castleAvailable: false,
    localCapabilityVerified: false,
    receiptChannelAvailable: true,
  });
  assert.strictEqual(result.standing, "ALIVE");
  assert.strictEqual(result.mayActuate, true);
  assert.strictEqual(result.reason, "ALIVE:RECEIPTED_SAFE_DEGRADE");
});

test("assessMateriality: triggering dimension over per-dimension threshold marks material with correct escalateByEpochMs", () => {
  const policy = {
    policyDigest: "a".repeat(64),
    authorityDigest: "b".repeat(64),
    perDimensionThresholdBps: {
      financial: 5000,
      operational: 10000,
      customer: 10000,
      legal: 10000,
      regulatory: 10000,
      reputational: 10000,
      systemic: 10000,
    },
    aggregateThresholdBps: 9000,
    escalationWithinMs: 60000,
  };
  const event = {
    id: "evt-1",
    subject: "subject-1",
    occurredAtEpochMs: 1000,
    impactBps: { financial: 6000 },
  };
  const result = assessMateriality(event, policy);
  assert.strictEqual(result.material, true);
  assert.deepStrictEqual(result.triggeringDimensions, ["financial"]);
  assert.strictEqual(result.escalateByEpochMs, 1000 + 60000);
  assert.strictEqual(result.policyDigest, policy.policyDigest);
  assert.strictEqual(result.authorityDigest, policy.authorityDigest);
});

test("assessMateriality: below all thresholds and below aggregate is not material", () => {
  const policy = {
    policyDigest: "a".repeat(64),
    authorityDigest: "b".repeat(64),
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
    escalationWithinMs: 60000,
  };
  const event = {
    id: "evt-2",
    subject: "subject-1",
    occurredAtEpochMs: 1000,
    impactBps: { financial: 1000, operational: 1000 },
  };
  const result = assessMateriality(event, policy);
  assert.strictEqual(result.material, false);
  assert.deepStrictEqual(result.triggeringDimensions, []);
  assert.strictEqual(result.escalateByEpochMs, undefined);
});

test("assessMateriality: invalid digests/policy/impact values throw the documented REFUSED:* errors", () => {
  const validPolicy = {
    policyDigest: "a".repeat(64),
    authorityDigest: "b".repeat(64),
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
    escalationWithinMs: 60000,
  };
  const validEvent = {
    id: "evt-3",
    subject: "subject-1",
    occurredAtEpochMs: 1000,
    impactBps: { financial: 1000 },
  };

  assert.throws(
    () => assessMateriality(validEvent, { ...validPolicy, policyDigest: "not-a-digest" }),
    /REFUSED:INVALID_POLICY_DIGEST/,
  );
  assert.throws(
    () => assessMateriality(validEvent, { ...validPolicy, authorityDigest: "not-a-digest" }),
    /REFUSED:INVALID_AUTHORITY_DIGEST/,
  );
  assert.throws(
    () => assessMateriality(validEvent, { ...validPolicy, aggregateThresholdBps: -1 }),
    /REFUSED:INVALID_MATERIALITY_POLICY/,
  );
  assert.throws(
    () => assessMateriality(validEvent, { ...validPolicy, escalationWithinMs: -1 }),
    /REFUSED:INVALID_ESCALATION_POLICY/,
  );
  assert.throws(
    () => assessMateriality({ ...validEvent, impactBps: { financial: 20000 } }, validPolicy),
    /REFUSED:INVALID_MATERIALITY_IMPACT/,
  );
  assert.throws(
    () => assessMateriality({ ...validEvent, impactBps: { financial: -1 } }, validPolicy),
    /REFUSED:INVALID_MATERIALITY_IMPACT/,
  );
});

test("assessMateriality: scoreBps is floor(avg) capped at 10000", () => {
  const policy = {
    policyDigest: "a".repeat(64),
    authorityDigest: "b".repeat(64),
    perDimensionThresholdBps: {
      financial: 10000,
      operational: 10000,
      customer: 10000,
      legal: 10000,
      regulatory: 10000,
      reputational: 10000,
      systemic: 10000,
    },
    aggregateThresholdBps: 10000,
    escalationWithinMs: 0,
  };
  const event = {
    id: "evt-4",
    subject: "subject-1",
    occurredAtEpochMs: 1000,
    impactBps: { financial: 10000, operational: 9999 },
  };
  const result = assessMateriality(event, policy);
  assert.strictEqual(result.scoreBps, Math.floor((10000 + 9999) / 2));
  assert.ok(result.scoreBps <= 10000);
});

test("classifyIcfrSubject: affectsFinancialReporting triggers inScope with correct reason", () => {
  const result = classifyIcfrSubject({
    subject: "s1",
    processes: [],
    materialAccounts: [],
    affectsFinancialReporting: true,
  });
  assert.strictEqual(result.inScope, true);
  assert.deepStrictEqual(result.reasons, ["affects-financial-reporting"]);
});

test("classifyIcfrSubject: materialAccounts triggers inScope with correct reason", () => {
  const result = classifyIcfrSubject({
    subject: "s2",
    processes: [],
    materialAccounts: ["accounts-receivable"],
    affectsFinancialReporting: false,
  });
  assert.strictEqual(result.inScope, true);
  assert.deepStrictEqual(result.reasons, ["material-account-impact"]);
});

test("classifyIcfrSubject: key financial process (revenue) triggers inScope with correct reason", () => {
  const result = classifyIcfrSubject({
    subject: "s3",
    processes: ["revenue"],
    materialAccounts: [],
    affectsFinancialReporting: false,
  });
  assert.strictEqual(result.inScope, true);
  assert.deepStrictEqual(result.reasons, ["key-financial-process"]);
});

test("classifyIcfrSubject: none of the triggers => inScope false with empty reasons", () => {
  const result = classifyIcfrSubject({
    subject: "s4",
    processes: ["marketing"],
    materialAccounts: [],
    affectsFinancialReporting: false,
  });
  assert.strictEqual(result.inScope, false);
  assert.deepStrictEqual(result.reasons, []);
});

test("detectSegregationOfDutyViolations: a principal holding both roles of an incompatible pair is flagged", () => {
  const assignments = [
    { principal: "alice", role: "payment-initiator" },
    { principal: "alice", role: "payment-approver" },
  ];
  const pairs = [["payment-initiator", "payment-approver"]];
  const violations = detectSegregationOfDutyViolations(assignments, pairs);
  assert.strictEqual(violations.length, 1);
  assert.strictEqual(violations[0].principal, "alice");
  assert.deepStrictEqual(violations[0].roles, ["payment-initiator", "payment-approver"]);
});

test("detectSegregationOfDutyViolations: sorted deterministic output", () => {
  const assignments = [
    { principal: "bob", role: "role-a" },
    { principal: "bob", role: "role-b" },
    { principal: "alice", role: "role-a" },
    { principal: "alice", role: "role-b" },
  ];
  const pairs = [["role-a", "role-b"]];
  const violations = detectSegregationOfDutyViolations(assignments, pairs);
  assert.strictEqual(violations.length, 2);
  assert.strictEqual(violations[0].principal, "alice");
  assert.strictEqual(violations[1].principal, "bob");
});

test("detectSegregationOfDutyViolations: no violation when roles are compatible or held by different principals", () => {
  const compatibleAssignments = [
    { principal: "alice", role: "role-a" },
    { principal: "alice", role: "role-c" },
  ];
  const pairs = [["role-a", "role-b"]];
  assert.deepStrictEqual(detectSegregationOfDutyViolations(compatibleAssignments, pairs), []);

  const differentPrincipals = [
    { principal: "alice", role: "role-a" },
    { principal: "bob", role: "role-b" },
  ];
  assert.deepStrictEqual(detectSegregationOfDutyViolations(differentPrincipals, pairs), []);
});

function makeAliveBoardAdmission(): BoardAdmission {
  const verifiedQualification = (subject: string): VerifiedQualification => ({
    standing: "ALIVE",
    qualification: {
      subject,
      standing: "ALIVE",
      controls: [],
      reasons: ["ALIVE:QUALIFIED"],
    } as unknown as Fortune5Qualification,
    evidenceRefusals: [],
  });
  return {
    standing: "ALIVE",
    enterprise: verifiedQualification("enterprise-subject"),
    castle: verifiedQualification("castle-subject"),
    reasons: ["ALIVE:BOARD_ADMISSION_PROVED"],
  };
}

test("buildBoardPackage: throws REFUSED:BOARD_PACKAGE_WITHOUT_BOARD_ADMISSION when admission.standing !== ALIVE", () => {
  const admission = makeAliveBoardAdmission();
  admission.standing = "REFUSED";
  assert.throws(
    () => buildBoardPackage(admission, new Date().toISOString(), 0, 0),
    /REFUSED:BOARD_PACKAGE_WITHOUT_BOARD_ADMISSION/,
  );
});

test("buildBoardPackage: throws on invalid generatedAt/counts", () => {
  const admission = makeAliveBoardAdmission();
  assert.throws(
    () => buildBoardPackage(admission, "not-a-date", 0, 0),
    /REFUSED:INVALID_BOARD_PACKAGE_TIME/,
  );
  assert.throws(
    () => buildBoardPackage(admission, new Date().toISOString(), -1, 0),
    /REFUSED:INVALID_MATERIAL_REFUSED_COUNT/,
  );
  assert.throws(
    () => buildBoardPackage(admission, new Date().toISOString(), 0, -1),
    /REFUSED:INVALID_RISK_APPETITE_BREACH_COUNT/,
  );
  assert.throws(
    () => buildBoardPackage(admission, new Date().toISOString(), 1.5, 0),
    /REFUSED:INVALID_MATERIAL_REFUSED_COUNT/,
  );
  assert.throws(
    () => buildBoardPackage(admission, new Date().toISOString(), 0, 1.5),
    /REFUSED:INVALID_RISK_APPETITE_BREACH_COUNT/,
  );
});

test("buildBoardPackage: on a valid ALIVE BoardAdmission returns a BoardPackage with profile and controlCount", () => {
  const admission = makeAliveBoardAdmission();
  const generatedAt = new Date().toISOString();
  const pkg = buildBoardPackage(admission, generatedAt, 2, 1);
  assert.strictEqual(pkg.profile, "CASTLE_FORTUNE5_BOARD_V1");
  assert.strictEqual(pkg.controlCount, FORTUNE5_REQUIREMENTS.length);
  assert.strictEqual(pkg.enterpriseSubject, "enterprise-subject");
  assert.strictEqual(pkg.castleSubject, "castle-subject");
  assert.strictEqual(pkg.enterpriseStanding, "ALIVE");
  assert.strictEqual(pkg.castleStanding, "ALIVE");
  assert.strictEqual(pkg.materialRefusedSubjects, 2);
  assert.strictEqual(pkg.riskAppetiteBreaches, 1);
  assert.strictEqual(pkg.generatedAt, generatedAt);
  assert.ok(typeof pkg.evidenceDigest === "string" && pkg.evidenceDigest.length > 0);
});

test("boardRequirements: returns the same array as FORTUNE5_REQUIREMENTS", () => {
  assert.strictEqual(boardRequirements(), FORTUNE5_REQUIREMENTS);
});
