import test from "node:test";
import assert from "node:assert/strict";
import { FORTUNE5_REQUIREMENTS } from "../src/fortune5.generated.ts";
import {
  admitReplay,
  minimumImpactCoverage,
  qualifyFortune5,
  type MetricObservation,
} from "../src/fortune5.ts";

const SUBJECT = "castle:enterprise:test";
const NOW = Date.parse("2026-08-15T20:00:00.000Z");
const RECEIPT = "a".repeat(64);

function targetValue(target: string): number | boolean | string {
  if (target === "true") return true;
  if (target === "false") return false;
  if (/^-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?$/.test(target)) return Number(target);
  return target;
}

function perfectEvidence(): MetricObservation[] {
  return FORTUNE5_REQUIREMENTS.map((requirement, index) => ({
    metric: requirement.metric,
    value: targetValue(requirement.target),
    receiptDigest: (index % 16).toString(16).repeat(64),
    subject: SUBJECT,
    observedAt: "2026-08-15T19:59:00.000Z",
    epistemicClass: "OBSERVED" as const,
  }));
}

test("marketplace projection exposes the complete Fortune-5 readiness profile", () => {
  assert.equal(FORTUNE5_REQUIREMENTS.length, 40);
  assert.equal(new Set(FORTUNE5_REQUIREMENTS.map((r) => r.controlId)).size, 40);
  assert.equal(new Set(FORTUNE5_REQUIREMENTS.map((r) => r.metric)).size, 40);
  for (const category of [
    "authority",
    "isolation",
    "supply-chain",
    "replay",
    "observability",
    "resilience",
    "availability",
    "slo",
    "data",
    "policy",
    "scale",
    "determinism",
    "evidence",
    "refusal",
    "change",
    "air-gap",
    "adversarial-coverage",
  ]) {
    assert.ok(FORTUNE5_REQUIREMENTS.some((r) => r.category === category), `missing ${category}`);
  }
});

test("missing evidence is UNKNOWN rather than vacuously passing", () => {
  const result = qualifyFortune5([], { subject: SUBJECT, nowEpochMs: NOW });
  assert.equal(result.standing, "UNKNOWN");
  assert.equal(result.alive, 0);
  assert.equal(result.refused, 0);
  assert.equal(result.unknown, 40);
  assert.ok(result.controls.every((control) => control.reason === "UNKNOWN:MISSING_RECEIPTED_EVIDENCE"));
});

test("complete receipted boundary evidence qualifies ALIVE", () => {
  const result = qualifyFortune5(perfectEvidence(), {
    subject: SUBJECT,
    nowEpochMs: NOW,
    maxEvidenceAgeMs: 5 * 60 * 1000,
  });
  assert.equal(result.standing, "ALIVE");
  assert.equal(result.alive, 40);
  assert.equal(result.refused, 0);
  assert.equal(result.unknown, 0);
  assert.ok(Object.values(result.categories).every((standing) => standing === "ALIVE"));
});

test("a failed hard control refuses the aggregate profile", () => {
  const evidence = perfectEvidence();
  const index = evidence.findIndex((observation) => observation.metric === "zero_unreceipted_actuations");
  evidence[index] = { ...evidence[index]!, value: 1 };
  const result = qualifyFortune5(evidence, { subject: SUBJECT, nowEpochMs: NOW });
  assert.equal(result.standing, "REFUSED");
  const control = result.controls.find((item) => item.controlId === "F5-AUTH-001")!;
  assert.equal(control.standing, "REFUSED");
  assert.equal(control.reason, "REFUSED:CONTROL_THRESHOLD_NOT_SATISFIED");
});

test("unreceipted, stale, cross-subject, and ambiguous evidence is refused", () => {
  const base = perfectEvidence();
  const metric = "deny_by_default";
  const index = base.findIndex((item) => item.metric === metric);

  const badDigest = [...base];
  badDigest[index] = { ...badDigest[index]!, receiptDigest: "not-a-digest" };
  assert.equal(qualifyFortune5(badDigest, { subject: SUBJECT }).standing, "REFUSED");

  const stale = [...base];
  stale[index] = { ...stale[index]!, observedAt: "2026-08-15T18:00:00.000Z" };
  assert.equal(
    qualifyFortune5(stale, { subject: SUBJECT, nowEpochMs: NOW, maxEvidenceAgeMs: 60_000 }).standing,
    "REFUSED",
  );

  const wrongSubject = [...base];
  wrongSubject[index] = { ...wrongSubject[index]!, subject: "castle:other" };
  assert.equal(qualifyFortune5(wrongSubject, { subject: SUBJECT }).standing, "REFUSED");

  const ambiguous = [...base, { ...base[index]!, receiptDigest: RECEIPT }];
  const result = qualifyFortune5(ambiguous, { subject: SUBJECT });
  assert.equal(result.standing, "REFUSED");
  assert.equal(result.controls.find((item) => item.metric === metric)!.reason, "REFUSED:AMBIGUOUS_EVIDENCE");
});

test("replay standing requires structural, ontology, provider, and invariant compatibility", () => {
  const manifest = {
    replayClassId: "class:authority-loss:v1",
    structuralSignature: "1".repeat(64),
    ontologyVersion: "castle-ontology:26.8.15+f5.1",
    providerSemanticsVersion: "aws-control-plane:2026-08-15",
    invariantSetDigest: "2".repeat(64),
    processDigest: "3".repeat(64),
  };
  const subject = {
    structuralSignature: manifest.structuralSignature,
    ontologyVersion: manifest.ontologyVersion,
    providerSemanticsVersion: manifest.providerSemanticsVersion,
    invariantSetDigest: manifest.invariantSetDigest,
    invariantsHold: true,
  };
  assert.equal(admitReplay(manifest, subject).standing, "ALIVE");

  const providerDrift = admitReplay(manifest, {
    ...subject,
    providerSemanticsVersion: "aws-control-plane:2026-08-16",
  });
  assert.equal(providerDrift.standing, "REFUSED");
  assert.ok(providerDrift.reasons.includes("REFUSED:PROVIDER_SEMANTICS_VERSION_MISMATCH"));

  const invariantFailure = admitReplay(manifest, { ...subject, invariantsHold: false });
  assert.equal(invariantFailure.standing, "REFUSED");
  assert.ok(invariantFailure.reasons.includes("REFUSED:INVARIANTS_NOT_SATISFIED"));
});

test("Pareto coverage is measured from consequence impact rather than assumed", () => {
  const result = minimumImpactCoverage([
    { key: "authority", impact: 60 },
    { key: "confidentiality", impact: 20 },
    { key: "integrity", impact: 10 },
    { key: "availability", impact: 10 },
  ], 8000);
  assert.deepEqual(result.selected.map((item) => item.key), ["authority", "confidentiality"]);
  assert.equal(result.coverageBps, 8000);
  assert.equal(result.selectedImpact, 80);
  assert.equal(result.totalImpact, 100);
});

test("invalid impact evidence and impossible coverage targets are typed refusals", () => {
  assert.throws(() => minimumImpactCoverage([{ key: "x", impact: -1 }]), /REFUSED:INVALID_IMPACT/);
  assert.throws(() => minimumImpactCoverage([{ key: "x", impact: 1 }], 0), /REFUSED:INVALID_COVERAGE_TARGET/);
  assert.throws(() => minimumImpactCoverage([{ key: "x", impact: 0 }]), /REFUSED:NO_POSITIVE_IMPACT_EVIDENCE/);
});
