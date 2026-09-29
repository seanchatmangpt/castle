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

export interface ActuationPermit {
  constructDigest: string;
  processDigest: string;
  subject: string;
  authority: string;
  transitionId: string;
  expiresAtEpochMs: number;
}

export interface GymActResult {
  transitionId: string;
  status: "OBSERVED" | "REFUSED";
  objects?: readonly OcelObject[];
  attributes?: Readonly<Record<string, string | number | boolean>>;
}

export interface GymActAdapter {
  execute(activity: PowlActivity, state: WorldState, permit: ActuationPermit): Promise<GymActResult>;
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

export interface ReceiptVerifier {
  verifyDigest(keyId: string, digestHex: string, signature: string): boolean;
}

export interface Receipt {
  algorithm: "BLAKE3-256";
  artifactDigest: string;
  receiptDigest: string;
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

export interface ConstructSources {
  oStar: unknown;
  configGraph: unknown;
  ontology: unknown;
}

export interface ConstructArtifact {
  kind: "CASTLE_CONSTRUCT_V1";
  algorithm: "BLAKE3-256";
  subject: string;
  authority: string;
  oStarDigest: string;
  configGraphDigest: string;
  ontologyDigest: string;
  processDigest: string;
  replayIdentityDigest: string;
  allowedTransitionIds: readonly string[];
  maxSteps: number;
  expiresAtEpochMs: number;
}

export interface ConstructSourceReceipts {
  oStar: Receipt;
  configGraph: Receipt;
  ontology: Receipt;
  process: Receipt;
}

export interface ConstructCapability {
  sources: ConstructSources;
  artifact: ConstructArtifact;
  sourceReceipts: ConstructSourceReceipts;
  receipt: Receipt;
}

export interface ConstructRequest extends ConstructSources {
  subject: string;
  authority: string;
  process: PowlProcess;
  envelope: TestEnvelope;
}

export interface ConstructTrustPolicy {
  trustedOriginKeyIds: ReadonlySet<string>;
  allowedAuthorities: ReadonlySet<string>;
}

const CONSTRUCT_ADMISSION_BRAND = Symbol("CASTLE_CONSTRUCT_ADMISSION");

export interface ConstructAdmission {
  readonly standing: "ALIVE";
  readonly constructDigest: string;
  readonly processDigest: string;
  readonly oStarDigest: string;
  readonly configGraphDigest: string;
  readonly ontologyDigest: string;
  readonly replayIdentityDigest: string;
  readonly subject: string;
  readonly authority: string;
  readonly allowedTransitionIds: readonly string[];
  readonly maxSteps: number;
  readonly expiresAtEpochMs: number;
  readonly [CONSTRUCT_ADMISSION_BRAND]: true;
}

export interface DoAuthorizationContext {
  admission: ConstructAdmission;
  blake3: Blake3Provider;
  receiptSigner: ReceiptSigner;
  now?: () => number;
}

export interface ReceiptedOcelLog extends OcelLog {
  constructDigest: string;
  receipt: Receipt;
}

function setKey(values: Iterable<string>): string {
  return [...new Set(values)].sort().join("\u0000");
}

function isSubset(a: readonly string[], b: readonly string[]): boolean {
  const bs = new Set(b);
  return a.every((x) => bs.has(x));
}

function sameStrings(a: readonly string[], b: readonly string[]): boolean {
  return a.length === b.length && a.every((value, index) => value === b[index]);
}

function canonicalJson(value: unknown): string {
  if (value === null || typeof value !== "object") return JSON.stringify(value);
  if (value instanceof Set) return canonicalJson([...value].sort());
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  const record = value as Record<string, unknown>;
  return `{${Object.keys(record).sort().map((key) => `${JSON.stringify(key)}:${canonicalJson(record[key])}`).join(",")}}`;
}

function digestCanonical(value: unknown, blake3: Blake3Provider): string {
  const digest = blake3.digestUtf8(canonicalJson(value));
  if (!/^[0-9a-f]{64}$/.test(digest)) throw new Error("invalid BLAKE3-256 digest provider output");
  return digest;
}

function unsignedReceiptPayload(receipt: Omit<Receipt, "receiptDigest" | "originSignature">): unknown {
  return {
    algorithm: receipt.algorithm,
    artifactDigest: receipt.artifactDigest,
    epistemicClass: receipt.epistemicClass,
    subject: receipt.subject,
    parentDigests: [...receipt.parentDigests].sort(),
    originKeyId: receipt.originKeyId,
  };
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
      facts: [`compromised:${dependencyId}`, `capability:${dependencyId}:${capability}`],
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

export function createReceipt(
  artifact: unknown,
  epistemicClass: EpistemicClass,
  subject: string,
  parentDigests: readonly string[],
  blake3: Blake3Provider,
  signer: ReceiptSigner,
): Receipt {
  const artifactDigest = digestCanonical(artifact, blake3);
  const normalizedParents = [...parentDigests].sort();
  if (normalizedParents.some((digest) => !/^[0-9a-f]{64}$/.test(digest))) {
    throw new Error("invalid parent BLAKE3-256 digest");
  }
  if (!signer.keyId) throw new Error("receipt signer keyId is required");
  const unsigned = {
    algorithm: "BLAKE3-256" as const,
    artifactDigest,
    epistemicClass,
    subject,
    parentDigests: normalizedParents,
    originKeyId: signer.keyId,
  };
  const receiptDigest = digestCanonical(unsignedReceiptPayload(unsigned), blake3);
  const originSignature = signer.signDigest(receiptDigest);
  if (!originSignature) throw new Error("receipt signer returned an empty origin signature");
  return { ...unsigned, receiptDigest, originSignature };
}

export function verifyReceipt(
  artifact: unknown,
  receipt: Receipt,
  blake3: Blake3Provider,
  verifier: ReceiptVerifier,
  trustedOriginKeyIds: ReadonlySet<string>,
): boolean {
  if (receipt.algorithm !== "BLAKE3-256") return false;
  if (!trustedOriginKeyIds.has(receipt.originKeyId)) return false;
  if (!/^[0-9a-f]{64}$/.test(receipt.artifactDigest) || !/^[0-9a-f]{64}$/.test(receipt.receiptDigest)) return false;
  if (receipt.parentDigests.some((digest) => !/^[0-9a-f]{64}$/.test(digest))) return false;
  if (digestCanonical(artifact, blake3) !== receipt.artifactDigest) return false;
  const expectedReceiptDigest = digestCanonical(
    unsignedReceiptPayload({
      algorithm: receipt.algorithm,
      artifactDigest: receipt.artifactDigest,
      epistemicClass: receipt.epistemicClass,
      subject: receipt.subject,
      parentDigests: receipt.parentDigests,
      originKeyId: receipt.originKeyId,
    }),
    blake3,
  );
  if (expectedReceiptDigest !== receipt.receiptDigest) return false;
  return verifier.verifyDigest(receipt.originKeyId, receipt.receiptDigest, receipt.originSignature);
}

function buildConstructArtifact(request: ConstructRequest, sourceReceipts: ConstructSourceReceipts, blake3: Blake3Provider): ConstructArtifact {
  const allowedTransitionIds = [...request.envelope.allowedTransitionIds].sort();
  const replayIdentityDigest = digestCanonical(
    {
      subject: request.subject,
      authority: request.authority,
      oStarDigest: sourceReceipts.oStar.artifactDigest,
      configGraphDigest: sourceReceipts.configGraph.artifactDigest,
      ontologyDigest: sourceReceipts.ontology.artifactDigest,
      processDigest: sourceReceipts.process.artifactDigest,
      allowedTransitionIds,
      maxSteps: request.envelope.maxSteps,
      expiresAtEpochMs: request.envelope.expiresAtEpochMs,
    },
    blake3,
  );
  return {
    kind: "CASTLE_CONSTRUCT_V1",
    algorithm: "BLAKE3-256",
    subject: request.subject,
    authority: request.authority,
    oStarDigest: sourceReceipts.oStar.artifactDigest,
    configGraphDigest: sourceReceipts.configGraph.artifactDigest,
    ontologyDigest: sourceReceipts.ontology.artifactDigest,
    processDigest: sourceReceipts.process.artifactDigest,
    replayIdentityDigest,
    allowedTransitionIds,
    maxSteps: request.envelope.maxSteps,
    expiresAtEpochMs: request.envelope.expiresAtEpochMs,
  };
}

/**
 * DfCM CONSTRUCT manufacture. This creates no execution authority by itself: only
 * admitConstructForDo can turn a cryptographically valid construction into an opaque DO capability.
 */
export function manufactureConstructCapability(
  request: ConstructRequest,
  blake3: Blake3Provider,
  signer: ReceiptSigner,
): ConstructCapability {
  if (!request.subject || request.envelope.systemId !== request.subject) {
    throw new Error("REFUSED:CONSTRUCT_SUBJECT_MISMATCH");
  }
  if (!request.authority) throw new Error("REFUSED:MISSING_CONSTRUCT_AUTHORITY");
  if (!Number.isInteger(request.envelope.maxSteps) || request.envelope.maxSteps < 0) {
    throw new Error("REFUSED:INVALID_CONSTRUCT_STEP_BOUND");
  }
  const processTransitions = [...new Set(request.process.activities.map((activity) => activity.transitionId))].sort();
  const allowed = [...request.envelope.allowedTransitionIds].sort();
  if (processTransitions.some((transitionId) => !request.envelope.allowedTransitionIds.has(transitionId))) {
    throw new Error("REFUSED:CONSTRUCT_PROCESS_EXCEEDS_BOUNDS");
  }
  if (request.process.activities.length > request.envelope.maxSteps) {
    throw new Error("REFUSED:CONSTRUCT_PROCESS_EXCEEDS_STEP_BOUND");
  }

  const sourceReceipts: ConstructSourceReceipts = {
    oStar: createReceipt(request.oStar, "CONSTRUCTED", request.subject, [], blake3, signer),
    configGraph: createReceipt(request.configGraph, "CONSTRUCTED", request.subject, [], blake3, signer),
    ontology: createReceipt(request.ontology, "CONSTRUCTED", request.subject, [], blake3, signer),
    process: createReceipt(request.process, "CONSTRUCTED", request.subject, [], blake3, signer),
  };
  const artifact = buildConstructArtifact(request, sourceReceipts, blake3);
  if (!sameStrings(artifact.allowedTransitionIds, allowed)) throw new Error("REFUSED:NONDETERMINISTIC_CONSTRUCT_BOUNDS");
  const parentDigests = Object.values(sourceReceipts).map((receipt) => receipt.artifactDigest).sort();
  const receipt = createReceipt(artifact, "CONSTRUCTED", request.subject, parentDigests, blake3, signer);
  return {
    sources: { oStar: request.oStar, configGraph: request.configGraph, ontology: request.ontology },
    artifact,
    sourceReceipts,
    receipt,
  };
}

export function admitConstructForDo(
  capability: ConstructCapability,
  process: PowlProcess,
  envelope: TestEnvelope,
  blake3: Blake3Provider,
  verifier: ReceiptVerifier,
  policy: ConstructTrustPolicy,
  now: () => number = Date.now,
): ConstructAdmission {
  const refuse = (reason: string): never => { throw new Error(`REFUSED:${reason}`); };
  const artifact = capability.artifact;
  if (artifact.kind !== "CASTLE_CONSTRUCT_V1" || artifact.algorithm !== "BLAKE3-256") refuse("INVALID_CONSTRUCT_KIND");
  if (!policy.allowedAuthorities.has(artifact.authority)) refuse("CONSTRUCT_AUTHORITY_NOT_ADMITTED");
  if (artifact.subject !== envelope.systemId || capability.receipt.subject !== envelope.systemId) refuse("CONSTRUCT_SUBJECT_MISMATCH");
  if (now() > artifact.expiresAtEpochMs || now() > envelope.expiresAtEpochMs) refuse("CONSTRUCT_EXPIRED");

  const receipts: Array<[unknown, Receipt]> = [
    [capability.sources.oStar, capability.sourceReceipts.oStar],
    [capability.sources.configGraph, capability.sourceReceipts.configGraph],
    [capability.sources.ontology, capability.sourceReceipts.ontology],
    [process, capability.sourceReceipts.process],
  ];
  for (const [source, receipt] of receipts) {
    if (receipt.subject !== artifact.subject || receipt.epistemicClass !== "CONSTRUCTED") refuse("INVALID_CONSTRUCT_PARENT");
    if (!verifyReceipt(source, receipt, blake3, verifier, policy.trustedOriginKeyIds)) refuse("UNVERIFIED_CONSTRUCT_PARENT");
  }

  const expectedRequest: ConstructRequest = {
    subject: artifact.subject,
    authority: artifact.authority,
    oStar: capability.sources.oStar,
    configGraph: capability.sources.configGraph,
    ontology: capability.sources.ontology,
    process,
    envelope,
  };
  const expectedArtifact = buildConstructArtifact(expectedRequest, capability.sourceReceipts, blake3);
  if (canonicalJson(expectedArtifact) !== canonicalJson(artifact)) refuse("CONSTRUCT_BINDING_MISMATCH");

  const parentDigests = Object.values(capability.sourceReceipts).map((receipt) => receipt.artifactDigest).sort();
  if (!sameStrings([...capability.receipt.parentDigests].sort(), parentDigests)) refuse("CONSTRUCT_PARENT_CHAIN_MISMATCH");
  if (capability.receipt.epistemicClass !== "CONSTRUCTED") refuse("INVALID_CONSTRUCT_RECEIPT_CLASS");
  if (!verifyReceipt(artifact, capability.receipt, blake3, verifier, policy.trustedOriginKeyIds)) refuse("UNVERIFIED_CONSTRUCT_RECEIPT");

  const processDigest = digestCanonical(process, blake3);
  if (processDigest !== artifact.processDigest) refuse("PROCESS_DIGEST_MISMATCH");
  if (!sameStrings([...envelope.allowedTransitionIds].sort(), artifact.allowedTransitionIds)) refuse("CONSTRUCT_BOUND_MISMATCH");
  if (envelope.maxSteps !== artifact.maxSteps || envelope.expiresAtEpochMs !== artifact.expiresAtEpochMs) refuse("CONSTRUCT_BOUND_MISMATCH");
  if (process.activities.some((activity) => !envelope.allowedTransitionIds.has(activity.transitionId))) refuse("PROCESS_OUTSIDE_CONSTRUCT_BOUNDS");

  return Object.freeze({
    standing: "ALIVE",
    constructDigest: capability.receipt.artifactDigest,
    processDigest: artifact.processDigest,
    oStarDigest: artifact.oStarDigest,
    configGraphDigest: artifact.configGraphDigest,
    ontologyDigest: artifact.ontologyDigest,
    replayIdentityDigest: artifact.replayIdentityDigest,
    subject: artifact.subject,
    authority: artifact.authority,
    allowedTransitionIds: Object.freeze([...artifact.allowedTransitionIds]),
    maxSteps: artifact.maxSteps,
    expiresAtEpochMs: artifact.expiresAtEpochMs,
    [CONSTRUCT_ADMISSION_BRAND]: true,
  });
}

/**
 * Exclusive DO path. There is intentionally no unreceipted GymAct execution path.
 * The opaque admission must have been manufactured by admitConstructForDo.
 */
export async function executePowlWithGymAct(
  process: PowlProcess,
  state: WorldState,
  envelope: TestEnvelope,
  gymact: GymActAdapter,
  authorization: DoAuthorizationContext,
): Promise<ReceiptedOcelLog> {
  const { admission, blake3, receiptSigner } = authorization;
  const now = authorization.now ?? Date.now;
  if (!admission || admission[CONSTRUCT_ADMISSION_BRAND] !== true) throw new Error("REFUSED:UNRECEIPTED_CONSTRUCT");
  if (admission.standing !== "ALIVE") throw new Error("REFUSED:CONSTRUCT_NOT_ALIVE");
  if (envelope.systemId !== state.systemId || admission.subject !== state.systemId) throw new Error("REFUSED: envelope subject mismatch");
  if (now() > envelope.expiresAtEpochMs || now() > admission.expiresAtEpochMs) throw new Error("REFUSED: envelope expired");
  if (digestCanonical(process, blake3) !== admission.processDigest) throw new Error("REFUSED:PROCESS_DIGEST_MISMATCH");
  if (!sameStrings([...envelope.allowedTransitionIds].sort(), admission.allowedTransitionIds)) throw new Error("REFUSED:CONSTRUCT_BOUND_MISMATCH");
  if (envelope.maxSteps !== admission.maxSteps || envelope.expiresAtEpochMs !== admission.expiresAtEpochMs) {
    throw new Error("REFUSED:CONSTRUCT_BOUND_MISMATCH");
  }

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
      if (!envelope.allowedTransitionIds.has(activity.transitionId) || !admission.allowedTransitionIds.includes(activity.transitionId)) {
        throw new Error(`REFUSED: transition not admitted: ${activity.transitionId}`);
      }
    }

    const results = await Promise.all(enabled.map((activity) => gymact.execute(activity, state, {
      constructDigest: admission.constructDigest,
      processDigest: admission.processDigest,
      subject: admission.subject,
      authority: admission.authority,
      transitionId: activity.transitionId,
      expiresAtEpochMs: admission.expiresAtEpochMs,
    })));
    for (let i = 0; i < enabled.length; i += 1) {
      const activity = enabled[i]!;
      const result = results[i]!;
      if (result.transitionId !== activity.transitionId) throw new Error(`REFUSED: GymAct transition receipt mismatch ${activity.transitionId}`);
      if (result.status !== "OBSERVED") throw new Error(`REFUSED: GymAct refused ${activity.transitionId}`);
      for (const object of result.objects ?? []) objects.set(object.id, object);
      events.push({
        id: `event:${events.length + 1}:${activity.transitionId}`,
        type: activity.transitionId,
        time: new Date(now()).toISOString(),
        attributes: {
          epistemicClass: "OBSERVED",
          constructDigest: admission.constructDigest,
          processDigest: admission.processDigest,
          configGraphDigest: admission.configGraphDigest,
          oStarDigest: admission.oStarDigest,
          ontologyDigest: admission.ontologyDigest,
          replayIdentityDigest: admission.replayIdentityDigest,
          authority: admission.authority,
          ...(result.attributes ?? {}),
        },
        objectIds: [state.systemId, ...(result.objects ?? []).map((o) => o.id)].sort(),
      });
      completed.add(activity.id);
      steps += 1;
    }
  }

  const log: OcelLog = {
    version: "2.0",
    objects: [...objects.values()].sort((a, b) => a.id.localeCompare(b.id)),
    events,
  };
  const receipt = createReceipt(log, "OBSERVED", state.systemId, [admission.constructDigest], blake3, receiptSigner);
  return { ...log, constructDigest: admission.constructDigest, receipt };
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
