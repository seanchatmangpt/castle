export type Predicate = string;

export interface TransitionRule {
  id: string;
  preconditions: readonly Predicate[];
  effects: readonly Predicate[];
  cost?: number;
  plannerHint?: string;
}

export interface AdversarialGoal {
  id: string;
  predicate: Predicate;
  consequence: number;
}

export interface VulnerabilityCondition {
  goalId: string;
  predicates: readonly Predicate[];
  witnessTransitions: readonly string[];
}

export interface DependencyNode {
  id: string;
  kind: string;
}

export interface DependencyEdge {
  from: string;
  to: string;
  relation: string;
}

export interface ConstructedCompromise {
  dependencyId: string;
  capability: string;
  facts: readonly Predicate[];
  impacted: readonly string[];
  epistemicClass: "COUNTERFACTUAL";
}

export interface PowlActivity {
  id: string;
  transitionId: string;
  predecessors: readonly string[];
}

export interface PowlProcess {
  id: string;
  goalId: string;
  activities: readonly PowlActivity[];
}

export interface PlanningProblem {
  goal: AdversarialGoal;
  vulnerability: VulnerabilityCondition;
  rules: readonly TransitionRule[];
}

export interface PlanCandidate {
  plannerId: string;
  process: PowlProcess;
  score: number;
}

export interface Planner {
  id: string;
  applicable(problem: PlanningProblem): boolean;
  plan(problem: PlanningProblem): Promise<readonly PlanCandidate[]>;
}

export interface WorldState {
  systemId: string;
  facts: ReadonlySet<Predicate>;
}

export interface TestEnvelope {
  systemId: string;
  allowedTransitionIds: ReadonlySet<string>;
  maxSteps: number;
  expiresAtEpochMs: number;
}

export interface GymActResult {
  transitionId: string;
  status: "OBSERVED" | "REFUSED";
  objects?: readonly OcelObject[];
  attributes?: Readonly<Record<string, string | number | boolean>>;
}

export interface GymActAdapter {
  execute(activity: PowlActivity, state: WorldState): Promise<GymActResult>;
}

export interface OcelObject {
  id: string;
  type: string;
  attributes?: Readonly<Record<string, string | number | boolean>>;
}

export interface OcelEvent {
  id: string;
  type: string;
  time: string;
  attributes: Readonly<Record<string, string | number | boolean>>;
  objectIds: readonly string[];
}

export interface OcelLog {
  version: "2.0";
  objects: readonly OcelObject[];
  events: readonly OcelEvent[];
}

export type EpistemicClass =
  | "CONSTRUCTED"
  | "COUNTERFACTUAL"
  | "REPLAYED"
  | "OBSERVED"
  | "INFERRED";

export interface Blake3Provider {
  /** Return a lowercase 64-hex-character BLAKE3-256 digest. */
  digestUtf8(input: string): string;
}

export interface ReceiptSigner {
  keyId: string;
  signDigest(digestHex: string): string;
}

export interface Receipt {
  algorithm: "BLAKE3-256";
  artifactDigest: string;
  epistemicClass: EpistemicClass;
  subject: string;
  parentDigests: readonly string[];
  originKeyId: string;
  originSignature: string;
}

export interface CompiledAdversarialClass {
  key: string;
  goal: AdversarialGoal;
  vulnerability: VulnerabilityCondition;
  process: PowlProcess;
}

function setKey(values: Iterable<string>): string {
  return [...new Set(values)].sort().join("\u0000");
}

function isSubset(a: readonly string[], b: readonly string[]): boolean {
  const bs = new Set(b);
  return a.every((x) => bs.has(x));
}

/**
 * DfCM inverse construction: derive minimal base conditions from which a goal
 * is constructible under the admitted transition calculus.
 */
export function deriveVulnerabilities(
  goal: AdversarialGoal,
  rules: readonly TransitionRule[],
  maxDepth = 32,
): VulnerabilityCondition[] {
  const producers = new Map<Predicate, TransitionRule[]>();
  for (const rule of rules) {
    for (const effect of rule.effects) {
      const bucket = producers.get(effect) ?? [];
      bucket.push(rule);
      producers.set(effect, bucket);
    }
  }
  for (const bucket of producers.values()) bucket.sort((a, b) => a.id.localeCompare(b.id));

  type Candidate = { required: Set<Predicate>; witness: string[]; expanded: Set<string> };
  const terminals: Candidate[] = [];
  const stack: Array<{ candidate: Candidate; depth: number }> = [
    {
      candidate: {
        required: new Set([goal.predicate]),
        witness: [],
        expanded: new Set(),
      },
      depth: 0,
    },
  ];
  const visited = new Set<string>();

  while (stack.length > 0) {
    const item = stack.pop()!;
    const { candidate, depth } = item;
    const visitKey = `${setKey(candidate.required)}|${setKey(candidate.witness)}`;
    if (visited.has(visitKey)) continue;
    visited.add(visitKey);

    const expandable = [...candidate.required]
      .sort()
      .find((predicate) => {
        const options = producers.get(predicate);
        return options && options.some((r) => !candidate.expanded.has(`${predicate}|${r.id}`));
      });

    if (!expandable || depth >= maxDepth) {
      terminals.push(candidate);
      continue;
    }

    const options = producers.get(expandable) ?? [];
    let emitted = false;
    for (const rule of options) {
      const expansionKey = `${expandable}|${rule.id}`;
      if (candidate.expanded.has(expansionKey)) continue;
      emitted = true;
      const required = new Set(candidate.required);
      required.delete(expandable);
      for (const p of rule.preconditions) required.add(p);
      const expanded = new Set(candidate.expanded);
      expanded.add(expansionKey);
      stack.push({
        candidate: {
          required,
          witness: [...candidate.witness, rule.id],
          expanded,
        },
        depth: depth + 1,
      });
    }
    if (!emitted) terminals.push(candidate);
  }

  const normalized: VulnerabilityCondition[] = terminals.map((c) => ({
    goalId: goal.id,
    predicates: [...c.required].sort(),
    witnessTransitions: [...new Set(c.witness)].reverse(),
  }));

  normalized.sort((a, b) =>
    a.predicates.length - b.predicates.length ||
    setKey(a.predicates).localeCompare(setKey(b.predicates)) ||
    setKey(a.witnessTransitions).localeCompare(setKey(b.witnessTransitions)),
  );

  const minimal: VulnerabilityCondition[] = [];
  for (const candidate of normalized) {
    if (minimal.some((m) => isSubset(m.predicates, candidate.predicates))) continue;
    minimal.push(candidate);
  }
  return minimal;
}

export class DependencyGraph {
  readonly nodes: ReadonlyMap<string, DependencyNode>;
  readonly edges: readonly DependencyEdge[];
  private readonly dependents = new Map<string, Set<string>>();

  constructor(nodes: readonly DependencyNode[], edges: readonly DependencyEdge[]) {
    this.nodes = new Map(nodes.map((n) => [n.id, n]));
    this.edges = [...edges].sort((a, b) =>
      `${a.from}|${a.to}|${a.relation}`.localeCompare(`${b.from}|${b.to}|${b.relation}`),
    );
    for (const edge of this.edges) {
      if (!this.nodes.has(edge.from) || !this.nodes.has(edge.to)) {
        throw new Error(`dependency edge references unknown node: ${edge.from} -> ${edge.to}`);
      }
      const bucket = this.dependents.get(edge.from) ?? new Set<string>();
      bucket.add(edge.to);
      this.dependents.set(edge.from, bucket);
    }
  }

  impactedClosure(changedIds: Iterable<string>): string[] {
    const seen = new Set<string>();
    const queue = [...changedIds].sort();
    while (queue.length > 0) {
      const id = queue.shift()!;
      if (seen.has(id)) continue;
      seen.add(id);
      for (const dependent of [...(this.dependents.get(id) ?? [])].sort()) {
        if (!seen.has(dependent)) queue.push(dependent);
      }
    }
    return [...seen].sort();
  }

  constructCompromise(dependencyId: string, capability: string): ConstructedCompromise {
    if (!this.nodes.has(dependencyId)) throw new Error(`unknown dependency: ${dependencyId}`);
    return {
      dependencyId,
      capability,
      facts: [
        `compromised:${dependencyId}`,
        `capability:${dependencyId}:${capability}`,
      ],
      impacted: this.impactedClosure([dependencyId]),
      epistemicClass: "COUNTERFACTUAL",
    };
  }
}

/** Compile a witness into a partial order based on data dependencies between transitions. */
export function compileWitnessToPowl(
  id: string,
  vulnerability: VulnerabilityCondition,
  rules: readonly TransitionRule[],
): PowlProcess {
  const byId = new Map(rules.map((r) => [r.id, r]));
  const witness = vulnerability.witnessTransitions.filter((id) => byId.has(id));
  const activities: PowlActivity[] = witness.map((transitionId) => {
    const rule = byId.get(transitionId)!;
    const predecessors = witness
      .filter((otherId) => otherId !== transitionId)
      .filter((otherId) => {
        const other = byId.get(otherId)!;
        return other.effects.some((effect) => rule.preconditions.includes(effect));
      })
      .sort();
    return { id: `activity:${transitionId}`, transitionId, predecessors: predecessors.map((p) => `activity:${p}`) };
  });
  activities.sort((a, b) => a.id.localeCompare(b.id));
  return { id, goalId: vulnerability.goalId, activities };
}

export function enabledActivities(process: PowlProcess, completed: ReadonlySet<string>): PowlActivity[] {
  return process.activities
    .filter((a) => !completed.has(a.id))
    .filter((a) => a.predecessors.every((p) => completed.has(p)))
    .sort((a, b) => a.id.localeCompare(b.id));
}

export class WitnessPlanner implements Planner {
  readonly id: string;
  constructor(id = "witness-partial-order") { this.id = id; }
  applicable(problem: PlanningProblem): boolean {
    return problem.vulnerability.witnessTransitions.length > 0;
  }
  async plan(problem: PlanningProblem): Promise<readonly PlanCandidate[]> {
    if (!this.applicable(problem)) return [];
    const process = compileWitnessToPowl(
      `powl:${problem.goal.id}:${setKey(problem.vulnerability.predicates)}`,
      problem.vulnerability,
      problem.rules,
    );
    const score = process.activities.length + problem.vulnerability.predicates.length;
    return [{ plannerId: this.id, process, score }];
  }
}

export async function runPlannerEnsemble(
  problem: PlanningProblem,
  planners: readonly Planner[],
): Promise<PlanCandidate[]> {
  const applicable = [...planners].filter((p) => p.applicable(problem)).sort((a, b) => a.id.localeCompare(b.id));
  const batches = await Promise.all(applicable.map((p) => p.plan(problem)));
  return batches.flat().sort((a, b) =>
    a.score - b.score || a.plannerId.localeCompare(b.plannerId) || a.process.id.localeCompare(b.process.id),
  );
}

export async function executePowlWithGymAct(
  process: PowlProcess,
  state: WorldState,
  envelope: TestEnvelope,
  gymact: GymActAdapter,
  now: () => number = Date.now,
): Promise<OcelLog> {
  if (envelope.systemId !== state.systemId) throw new Error("REFUSED: envelope subject mismatch");
  if (now() > envelope.expiresAtEpochMs) throw new Error("REFUSED: envelope expired");

  const completed = new Set<string>();
  const objects = new Map<string, OcelObject>();
  objects.set(state.systemId, { id: state.systemId, type: "System" });
  const events: OcelEvent[] = [];
  let steps = 0;

  while (completed.size < process.activities.length) {
    const enabled = enabledActivities(process, completed);
    if (enabled.length === 0) throw new Error("REFUSED: POWL deadlock or cyclic precedence");
    if (steps + enabled.length > envelope.maxSteps) throw new Error("REFUSED: max step budget exceeded");

    for (const activity of enabled) {
      if (!envelope.allowedTransitionIds.has(activity.transitionId)) {
        throw new Error(`REFUSED: transition not admitted: ${activity.transitionId}`);
      }
    }

    const results = await Promise.all(enabled.map((activity) => gymact.execute(activity, state)));
    for (let i = 0; i < enabled.length; i += 1) {
      const activity = enabled[i]!;
      const result = results[i]!;
      if (result.status !== "OBSERVED") throw new Error(`REFUSED: GymAct refused ${activity.transitionId}`);
      for (const object of result.objects ?? []) objects.set(object.id, object);
      events.push({
        id: `event:${events.length + 1}:${activity.transitionId}`,
        type: activity.transitionId,
        time: new Date(now()).toISOString(),
        attributes: { epistemicClass: "OBSERVED", ...(result.attributes ?? {}) },
        objectIds: [state.systemId, ...(result.objects ?? []).map((o) => o.id)].sort(),
      });
      completed.add(activity.id);
      steps += 1;
    }
  }

  return {
    version: "2.0",
    objects: [...objects.values()].sort((a, b) => a.id.localeCompare(b.id)),
    events,
  };
}

function canonicalJson(value: unknown): string {
  if (value === null || typeof value !== "object") return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  const record = value as Record<string, unknown>;
  return `{${Object.keys(record).sort().map((key) => `${JSON.stringify(key)}:${canonicalJson(record[key])}`).join(",")}}`;
}

export function createReceipt(
  artifact: unknown,
  epistemicClass: EpistemicClass,
  subject: string,
  parentDigests: readonly string[],
  blake3: Blake3Provider,
  signer: ReceiptSigner,
): Receipt {
  const digest = blake3.digestUtf8(canonicalJson(artifact));
  if (!/^[0-9a-f]{64}$/.test(digest)) throw new Error("invalid BLAKE3-256 digest provider output");
  const signature = signer.signDigest(digest);
  if (!signature) throw new Error("receipt signer returned an empty origin signature");
  return {
    algorithm: "BLAKE3-256",
    artifactDigest: digest,
    epistemicClass,
    subject,
    parentDigests: [...parentDigests].sort(),
    originKeyId: signer.keyId,
    originSignature: signature,
  };
}

export async function compileAdversarialClasses(
  goals: readonly AdversarialGoal[],
  rules: readonly TransitionRule[],
  planners: readonly Planner[] = [new WitnessPlanner()],
): Promise<CompiledAdversarialClass[]> {
  const compiled: CompiledAdversarialClass[] = [];
  for (const goal of [...goals].sort((a, b) => b.consequence - a.consequence || a.id.localeCompare(b.id))) {
    const vulnerabilities = deriveVulnerabilities(goal, rules);
    for (const vulnerability of vulnerabilities) {
      const candidates = await runPlannerEnsemble({ goal, vulnerability, rules }, planners);
      const selected = candidates[0];
      if (!selected) continue;
      compiled.push({
        key: `${goal.id}|${setKey(vulnerability.predicates)}`,
        goal,
        vulnerability,
        process: selected.process,
      });
    }
  }
  return compiled.sort((a, b) => a.key.localeCompare(b.key));
}

export function matchCompiledClasses(
  classes: readonly CompiledAdversarialClass[],
  facts: ReadonlySet<Predicate>,
): CompiledAdversarialClass[] {
  return classes
    .filter((c) => c.vulnerability.predicates.every((p) => facts.has(p)))
    .sort((a, b) => b.goal.consequence - a.goal.consequence || a.key.localeCompare(b.key));
}

export interface ZeroDayObservation {
  dependencyId: string;
  capability: string;
}

export interface ZeroDayImpact {
  observation: ZeroDayObservation;
  impactedDependencies: readonly string[];
  newlyAdmittedFact: Predicate;
}

export function applyZeroDayObservation(
  graph: DependencyGraph,
  observation: ZeroDayObservation,
): ZeroDayImpact {
  if (!graph.nodes.has(observation.dependencyId)) throw new Error(`unknown dependency: ${observation.dependencyId}`);
  return {
    observation,
    impactedDependencies: graph.impactedClosure([observation.dependencyId]),
    newlyAdmittedFact: `capability:${observation.dependencyId}:${observation.capability}`,
  };
}
