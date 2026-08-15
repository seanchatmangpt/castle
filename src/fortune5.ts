import { FORTUNE5_REQUIREMENTS } from "./fortune5.generated.ts";

export type Standing = "ALIVE" | "REFUSED" | "UNKNOWN";
export type MetricValue = number | boolean | string;
export type Comparator = "EQ" | "GTE" | "LTE";

export interface Fortune5Requirement {
  order: number;
  controlId: string;
  category: string;
  description: string;
  metric: string;
  comparator: Comparator | string;
  target: string;
  authority: string;
}

export interface MetricObservation {
  metric: string;
  value: MetricValue;
  receiptDigest: string;
  subject: string;
  observedAt: string;
  epistemicClass: "OBSERVED" | "REPLAYED" | "INFERRED";
}

export interface QualificationContext {
  subject: string;
  nowEpochMs?: number;
  maxEvidenceAgeMs?: number;
}

export interface ControlEvaluation {
  controlId: string;
  category: string;
  metric: string;
  standing: Standing;
  observed?: MetricValue;
  expected: string;
  receiptDigest?: string;
  reason: string;
}

export interface Fortune5Qualification {
  standing: Standing;
  subject: string;
  profile: "CASTLE_FORTUNE5_READINESS_V1";
  controls: readonly ControlEvaluation[];
  alive: number;
  refused: number;
  unknown: number;
  categories: Readonly<Record<string, Standing>>;
}

const DIGEST_RE = /^[0-9a-f]{64}$/;

function parseTarget(target: string): MetricValue {
  if (target === "true") return true;
  if (target === "false") return false;
  if (/^-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?$/.test(target)) return Number(target);
  return target;
}

function compareValue(comparator: string, observed: MetricValue, target: string): boolean {
  const expected = parseTarget(target);
  if (comparator === "EQ") return typeof observed === typeof expected && observed === expected;
  if (comparator === "GTE") {
    return typeof observed === "number" && typeof expected === "number" && Number.isFinite(observed) && observed >= expected;
  }
  if (comparator === "LTE") {
    return typeof observed === "number" && typeof expected === "number" && Number.isFinite(observed) && observed <= expected;
  }
  return false;
}

function evidenceProblem(observation: MetricObservation, context: QualificationContext): string | undefined {
  if (observation.subject !== context.subject) return "REFUSED:EVIDENCE_SUBJECT_MISMATCH";
  if (!DIGEST_RE.test(observation.receiptDigest)) return "REFUSED:INVALID_RECEIPT_DIGEST";
  const observedAt = Date.parse(observation.observedAt);
  if (!Number.isFinite(observedAt)) return "REFUSED:INVALID_EVIDENCE_TIME";
  if (context.maxEvidenceAgeMs !== undefined) {
    const now = context.nowEpochMs ?? Date.now();
    if (context.maxEvidenceAgeMs < 0) return "REFUSED:INVALID_EVIDENCE_AGE_POLICY";
    if (observedAt > now) return "REFUSED:EVIDENCE_FROM_FUTURE";
    if (now - observedAt > context.maxEvidenceAgeMs) return "REFUSED:STALE_EVIDENCE";
  }
  return undefined;
}

export function qualifyFortune5(
  observations: readonly MetricObservation[],
  context: QualificationContext,
  requirements: readonly Fortune5Requirement[] = FORTUNE5_REQUIREMENTS,
): Fortune5Qualification {
  const byMetric = new Map<string, MetricObservation[]>();
  for (const observation of observations) {
    const bucket = byMetric.get(observation.metric) ?? [];
    bucket.push(observation);
    byMetric.set(observation.metric, bucket);
  }

  const controls: ControlEvaluation[] = [...requirements]
    .sort((a, b) => a.order - b.order || a.controlId.localeCompare(b.controlId))
    .map((requirement) => {
      const evidence = byMetric.get(requirement.metric) ?? [];
      const expected = `${requirement.comparator} ${requirement.target}`;
      if (evidence.length === 0) {
        return {
          controlId: requirement.controlId,
          category: requirement.category,
          metric: requirement.metric,
          standing: "UNKNOWN" as const,
          expected,
          reason: "UNKNOWN:MISSING_RECEIPTED_EVIDENCE",
        };
      }
      if (evidence.length !== 1) {
        return {
          controlId: requirement.controlId,
          category: requirement.category,
          metric: requirement.metric,
          standing: "REFUSED" as const,
          expected,
          reason: "REFUSED:AMBIGUOUS_EVIDENCE",
        };
      }

      const observation = evidence[0]!;
      const problem = evidenceProblem(observation, context);
      if (problem) {
        return {
          controlId: requirement.controlId,
          category: requirement.category,
          metric: requirement.metric,
          standing: "REFUSED" as const,
          observed: observation.value,
          expected,
          receiptDigest: observation.receiptDigest,
          reason: problem,
        };
      }

      if (!(["EQ", "GTE", "LTE"] as const).includes(requirement.comparator as Comparator)) {
        return {
          controlId: requirement.controlId,
          category: requirement.category,
          metric: requirement.metric,
          standing: "REFUSED" as const,
          observed: observation.value,
          expected,
          receiptDigest: observation.receiptDigest,
          reason: "REFUSED:UNSUPPORTED_COMPARATOR",
        };
      }

      const passes = compareValue(requirement.comparator, observation.value, requirement.target);
      return {
        controlId: requirement.controlId,
        category: requirement.category,
        metric: requirement.metric,
        standing: passes ? "ALIVE" as const : "REFUSED" as const,
        observed: observation.value,
        expected,
        receiptDigest: observation.receiptDigest,
        reason: passes ? "ALIVE:RECEIPTED_CONTROL_SATISFIED" : "REFUSED:CONTROL_THRESHOLD_NOT_SATISFIED",
      };
    });

  const alive = controls.filter((control) => control.standing === "ALIVE").length;
  const refused = controls.filter((control) => control.standing === "REFUSED").length;
  const unknown = controls.filter((control) => control.standing === "UNKNOWN").length;
  const standing: Standing = refused > 0 ? "REFUSED" : unknown > 0 ? "UNKNOWN" : "ALIVE";

  const categoryMembers = new Map<string, Standing[]>();
  for (const control of controls) {
    const bucket = categoryMembers.get(control.category) ?? [];
    bucket.push(control.standing);
    categoryMembers.set(control.category, bucket);
  }
  const categories: Record<string, Standing> = {};
  for (const [category, members] of [...categoryMembers].sort(([a], [b]) => a.localeCompare(b))) {
    categories[category] = members.includes("REFUSED") ? "REFUSED" : members.includes("UNKNOWN") ? "UNKNOWN" : "ALIVE";
  }

  return {
    standing,
    subject: context.subject,
    profile: "CASTLE_FORTUNE5_READINESS_V1",
    controls,
    alive,
    refused,
    unknown,
    categories,
  };
}

export interface ReplayManifest {
  replayClassId: string;
  structuralSignature: string;
  ontologyVersion: string;
  providerSemanticsVersion: string;
  invariantSetDigest: string;
  processDigest: string;
}

export interface ReplaySubject {
  structuralSignature: string;
  ontologyVersion: string;
  providerSemanticsVersion: string;
  invariantSetDigest: string;
  invariantsHold: boolean;
}

export interface ReplayAdmission {
  standing: "ALIVE" | "REFUSED";
  replayClassId: string;
  reasons: readonly string[];
}

/**
 * Replay is reuse of a proved, parameterized state transformer, not literal API-call playback.
 * A class loses standing when its structural signature, ontology, provider semantics, or invariant
 * set no longer matches the current admitted subject.
 */
export function admitReplay(manifest: ReplayManifest, subject: ReplaySubject): ReplayAdmission {
  const reasons: string[] = [];
  if (!DIGEST_RE.test(manifest.structuralSignature) || !DIGEST_RE.test(subject.structuralSignature)) {
    reasons.push("REFUSED:INVALID_STRUCTURAL_SIGNATURE");
  } else if (manifest.structuralSignature !== subject.structuralSignature) {
    reasons.push("REFUSED:STRUCTURAL_SIGNATURE_MISMATCH");
  }
  if (!manifest.ontologyVersion || manifest.ontologyVersion !== subject.ontologyVersion) {
    reasons.push("REFUSED:ONTOLOGY_VERSION_MISMATCH");
  }
  if (!manifest.providerSemanticsVersion || manifest.providerSemanticsVersion !== subject.providerSemanticsVersion) {
    reasons.push("REFUSED:PROVIDER_SEMANTICS_VERSION_MISMATCH");
  }
  if (!DIGEST_RE.test(manifest.invariantSetDigest) || !DIGEST_RE.test(subject.invariantSetDigest)) {
    reasons.push("REFUSED:INVALID_INVARIANT_SET_DIGEST");
  } else if (manifest.invariantSetDigest !== subject.invariantSetDigest) {
    reasons.push("REFUSED:INVARIANT_SET_MISMATCH");
  }
  if (!DIGEST_RE.test(manifest.processDigest)) reasons.push("REFUSED:INVALID_PROCESS_DIGEST");
  if (!subject.invariantsHold) reasons.push("REFUSED:INVARIANTS_NOT_SATISFIED");

  return {
    standing: reasons.length === 0 ? "ALIVE" : "REFUSED",
    replayClassId: manifest.replayClassId,
    reasons: reasons.length === 0 ? ["ALIVE:REPLAY_CLASS_ADMITTED"] : reasons,
  };
}

export interface AdversarialImpactClass {
  key: string;
  impact: number;
}

export interface ImpactCoverageSelection {
  selected: readonly AdversarialImpactClass[];
  coverageBps: number;
  totalImpact: number;
  selectedImpact: number;
}

/**
 * Deterministically select the smallest prefix of highest-impact classes that reaches a requested
 * consequence-coverage target. The Pareto skew is measured from evidence rather than assumed.
 */
export function minimumImpactCoverage(
  classes: readonly AdversarialImpactClass[],
  targetCoverageBps = 8000,
): ImpactCoverageSelection {
  if (!Number.isInteger(targetCoverageBps) || targetCoverageBps < 1 || targetCoverageBps > 10000) {
    throw new Error("REFUSED:INVALID_COVERAGE_TARGET");
  }
  if (classes.some((item) => !Number.isFinite(item.impact) || item.impact < 0)) {
    throw new Error("REFUSED:INVALID_IMPACT");
  }
  const totalImpact = classes.reduce((sum, item) => sum + item.impact, 0);
  if (totalImpact <= 0) throw new Error("REFUSED:NO_POSITIVE_IMPACT_EVIDENCE");

  const ordered = [...classes].sort((a, b) => b.impact - a.impact || a.key.localeCompare(b.key));
  const selected: AdversarialImpactClass[] = [];
  let selectedImpact = 0;
  for (const item of ordered) {
    selected.push(item);
    selectedImpact += item.impact;
    const coverageBps = Math.floor((selectedImpact * 10000) / totalImpact);
    if (coverageBps >= targetCoverageBps) break;
  }

  return {
    selected,
    coverageBps: Math.floor((selectedImpact * 10000) / totalImpact),
    totalImpact,
    selectedImpact,
  };
}
