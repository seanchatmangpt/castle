import test from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  DependencyGraph,
  WitnessPlanner,
  admitConstructForDo,
  applyZeroDayObservation,
  compileAdversarialClasses,
  createReceipt,
  deriveVulnerabilities,
  executePowlWithGymAct,
  manufactureConstructCapability,
  matchCompiledClasses,
  type AdversarialGoal,
  type Blake3Provider,
  type GymActAdapter,
  type PowlProcess,
  type ReceiptSigner,
  type ReceiptVerifier,
  type TestEnvelope,
  type TransitionRule,
} from "../src/castle.ts";

const goal: AdversarialGoal = {
  id: "unauthorized-authority",
  predicate: "goal:unauthorized-authority",
  consequence: 100,
};

const rules: TransitionRule[] = [
  {
    id: "assume-control-plane",
    preconditions: ["dep:auth-service:execute", "trust:service-account"],
    effects: ["goal:unauthorized-authority"],
  },
  {
    id: "execute-auth-service",
    preconditions: ["capability:auth-lib:execute"],
    effects: ["dep:auth-service:execute"],
  },
];

const testBlake3: Blake3Provider = {
  // Deterministic shape-compatible test double. Production injects a real BLAKE3-256 provider.
  digestUtf8(input) {
    return createHash("sha256").update(`test-blake3:${input}`).digest("hex");
  },
};
const testSigner: ReceiptSigner = {
  keyId: "construct-root",
  signDigest: (digest) => `sig:${digest}`,
};
const testVerifier: ReceiptVerifier = {
  verifyDigest: (keyId, digest, signature) => keyId === "construct-root" && signature === `sig:${digest}`,
};

function constructFor(process: PowlProcess, envelope: TestEnvelope) {
  const capability = manufactureConstructCapability({
    subject: envelope.systemId,
    authority: "defensive-test",
    oStar: { admittedSubject: envelope.systemId },
    configGraph: { root: "configs/CONSTRUCT", zeroUnreceiptedActuation: true },
    ontology: { version: "castle-pack-v1" },
    process,
    envelope,
  }, testBlake3, testSigner);
  const admission = admitConstructForDo(capability, process, envelope, testBlake3, testVerifier, {
    trustedOriginKeyIds: new Set(["construct-root"]),
    allowedAuthorities: new Set(["defensive-test"]),
  }, () => 1);
  return { capability, admission };
}

test("DfCM derives minimal vulnerability conditions backward from the goal", () => {
  const vulnerabilities = deriveVulnerabilities(goal, rules);
  assert.deepEqual(vulnerabilities, [
    {
      goalId: "unauthorized-authority",
      predicates: ["capability:auth-lib:execute", "trust:service-account"],
      witnessTransitions: ["execute-auth-service", "assume-control-plane"],
    },
  ]);
});

test("dependency CONSTRUCT computes downstream impact without claiming observation", () => {
  const graph = new DependencyGraph(
    [
      { id: "auth-lib", kind: "package" },
      { id: "auth-service", kind: "service" },
      { id: "control-plane", kind: "service" },
    ],
    [
      { from: "auth-lib", to: "auth-service", relation: "dependsOn" },
      { from: "auth-service", to: "control-plane", relation: "calls" },
    ],
  );
  const constructed = graph.constructCompromise("auth-lib", "execute");
  assert.equal(constructed.epistemicClass, "COUNTERFACTUAL");
  assert.deepEqual(constructed.impacted, ["auth-lib", "auth-service", "control-plane"]);
});

test("planner ensemble compiles the witness into a POWL causal partial order", async () => {
  const classes = await compileAdversarialClasses([goal], rules, [new WitnessPlanner()]);
  assert.equal(classes.length, 1);
  const process = classes[0]!.process;
  const first = process.activities.find((a) => a.transitionId === "execute-auth-service")!;
  const second = process.activities.find((a) => a.transitionId === "assume-control-plane")!;
  assert.deepEqual(first.predecessors, []);
  assert.deepEqual(second.predecessors, [first.id]);
});

test("known structural vulnerability selects a precompiled adversarial class", async () => {
  const classes = await compileAdversarialClasses([goal], rules);
  const matches = matchCompiledClasses(
    classes,
    new Set(["capability:auth-lib:execute", "trust:service-account"]),
  );
  assert.equal(matches.length, 1);
  assert.equal(matches[0]!.goal.id, goal.id);
});

test("GymAct executes only through admitted CONSTRUCT and returns receipted OCEL v2 evidence", async () => {
  const classes = await compileAdversarialClasses([goal], rules);
  const process = classes[0]!.process;
  const gymact: GymActAdapter = {
    async execute(activity, _state, permit) {
      assert.equal(permit.transitionId, activity.transitionId);
      return {
        transitionId: activity.transitionId,
        status: "OBSERVED",
        objects: [{ id: `object:${activity.transitionId}`, type: "TestObservation" }],
      };
    },
  };
  const envelope: TestEnvelope = {
    systemId: "system:self",
    allowedTransitionIds: new Set(["execute-auth-service", "assume-control-plane"]),
    maxSteps: 2,
    expiresAtEpochMs: 10_000,
  };
  const { admission } = constructFor(process, envelope);
  let tick = 0;
  const log = await executePowlWithGymAct(
    process,
    { systemId: "system:self", facts: new Set() },
    envelope,
    gymact,
    { admission, blake3: testBlake3, receiptSigner: testSigner, now: () => ++tick },
  );
  assert.equal(log.version, "2.0");
  assert.equal(log.events.length, 2);
  assert.deepEqual(log.events.map((e) => e.type), ["execute-auth-service", "assume-control-plane"]);
  assert.equal(log.receipt.epistemicClass, "OBSERVED");
  assert.deepEqual(log.receipt.parentDigests, [admission.constructDigest]);
  assert.ok(log.events.every((event) => event.attributes.constructDigest === admission.constructDigest));
});

test("DO refuses fabricated admission and envelope mutation after CONSTRUCT", async () => {
  const classes = await compileAdversarialClasses([goal], rules);
  const process = classes[0]!.process;
  const envelope: TestEnvelope = {
    systemId: "system:self",
    allowedTransitionIds: new Set(["execute-auth-service", "assume-control-plane"]),
    maxSteps: 2,
    expiresAtEpochMs: 10_000,
  };
  const gymact: GymActAdapter = {
    async execute(activity) { return { transitionId: activity.transitionId, status: "OBSERVED" }; },
  };
  await assert.rejects(
    executePowlWithGymAct(
      process,
      { systemId: "system:self", facts: new Set() },
      envelope,
      gymact,
      { admission: {} as never, blake3: testBlake3, receiptSigner: testSigner, now: () => 1 },
    ),
    /REFUSED:UNRECEIPTED_CONSTRUCT/,
  );

  const { admission } = constructFor(process, envelope);
  const mutatedEnvelope: TestEnvelope = { ...envelope, allowedTransitionIds: new Set(["execute-auth-service"]) };
  await assert.rejects(
    executePowlWithGymAct(
      process,
      { systemId: "system:self", facts: new Set() },
      mutatedEnvelope,
      gymact,
      { admission, blake3: testBlake3, receiptSigner: testSigner, now: () => 1 },
    ),
    /REFUSED:CONSTRUCT_BOUND_MISMATCH/,
  );
});

test("CONSTRUCT admission refuses config mutation, process substitution, and untrusted origin", async () => {
  const classes = await compileAdversarialClasses([goal], rules);
  const process = classes[0]!.process;
  const envelope: TestEnvelope = {
    systemId: "system:self",
    allowedTransitionIds: new Set(["execute-auth-service", "assume-control-plane"]),
    maxSteps: 2,
    expiresAtEpochMs: 10_000,
  };
  const { capability } = constructFor(process, envelope);

  capability.sources.configGraph = { root: "human-edited" };
  assert.throws(
    () => admitConstructForDo(capability, process, envelope, testBlake3, testVerifier, {
      trustedOriginKeyIds: new Set(["construct-root"]),
      allowedAuthorities: new Set(["defensive-test"]),
    }, () => 1),
    /REFUSED:UNVERIFIED_CONSTRUCT_PARENT/,
  );

  const fresh = constructFor(process, envelope).capability;
  const substituted: PowlProcess = { ...process, id: `${process.id}:human-substitution` };
  assert.throws(
    () => admitConstructForDo(fresh, substituted, envelope, testBlake3, testVerifier, {
      trustedOriginKeyIds: new Set(["construct-root"]),
      allowedAuthorities: new Set(["defensive-test"]),
    }, () => 1),
    /REFUSED:/,
  );
  assert.throws(
    () => admitConstructForDo(fresh, process, envelope, testBlake3, testVerifier, {
      trustedOriginKeyIds: new Set(["other-root"]),
      allowedAuthorities: new Set(["defensive-test"]),
    }, () => 1),
    /REFUSED:UNVERIFIED_CONSTRUCT_PARENT/,
  );
});

test("receipt contract binds artifact, subject, parents, origin and signature", () => {
  assert.throws(
    () => createReceipt({ x: 1 }, "CONSTRUCTED", "subject", [], { digestUtf8: () => "bad" }, { keyId: "test", signDigest: () => "sig" }),
    /invalid BLAKE3-256/,
  );
  const receipt = createReceipt(
    { x: 1 },
    "CONSTRUCTED",
    "subject",
    ["b".repeat(64), "a".repeat(64)],
    { digestUtf8: () => "c".repeat(64) },
    { keyId: "test-key", signDigest: (digest) => `sig:${digest}` },
  );
  assert.equal(receipt.algorithm, "BLAKE3-256");
  assert.deepEqual(receipt.parentDigests, ["a".repeat(64), "b".repeat(64)]);
  assert.equal(receipt.artifactDigest, "c".repeat(64));
  assert.equal(receipt.receiptDigest, "c".repeat(64));
  assert.equal(receipt.originKeyId, "test-key");
  assert.equal(receipt.originSignature, `sig:${"c".repeat(64)}`);
});

test("zero-day listener turns a new capability fact into an impacted dependency closure", () => {
  const graph = new DependencyGraph(
    [
      { id: "auth-lib", kind: "package" },
      { id: "auth-service", kind: "service" },
      { id: "control-plane", kind: "service" },
    ],
    [
      { from: "auth-lib", to: "auth-service", relation: "dependsOn" },
      { from: "auth-service", to: "control-plane", relation: "calls" },
    ],
  );
  const impact = applyZeroDayObservation(graph, { dependencyId: "auth-lib", capability: "execute" });
  assert.equal(impact.newlyAdmittedFact, "capability:auth-lib:execute");
  assert.deepEqual(impact.impactedDependencies, ["auth-lib", "auth-service", "control-plane"]);
});

test("marketplace-generated bindings provide architecture components and default goal priorities", async () => {
  const generated = await import("../src/generated.ts");
  assert.equal(generated.GENERATED_COMPONENTS.length, 10);
  assert.equal(generated.DEFAULT_ADVERSARIAL_GOALS.length, 5);
  assert.equal(generated.DEFAULT_ADVERSARIAL_GOALS[0]!.id, "unauthorized-authority");
});
