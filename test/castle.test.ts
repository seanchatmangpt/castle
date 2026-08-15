import test from "node:test";
import assert from "node:assert/strict";
import {
  DependencyGraph,
  WitnessPlanner,
  applyZeroDayObservation,
  compileAdversarialClasses,
  createReceipt,
  deriveVulnerabilities,
  executePowlWithGymAct,
  matchCompiledClasses,
  type AdversarialGoal,
  type GymActAdapter,
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

test("GymAct executes only admitted POWL transitions and returns OCEL v2 evidence", async () => {
  const classes = await compileAdversarialClasses([goal], rules);
  const process = classes[0]!.process;
  const gymact: GymActAdapter = {
    async execute(activity) {
      return {
        transitionId: activity.transitionId,
        status: "OBSERVED",
        objects: [{ id: `object:${activity.transitionId}`, type: "TestObservation" }],
      };
    },
  };
  let tick = 0;
  const log = await executePowlWithGymAct(
    process,
    { systemId: "system:self", facts: new Set() },
    {
      systemId: "system:self",
      allowedTransitionIds: new Set(["execute-auth-service", "assume-control-plane"]),
      maxSteps: 2,
      expiresAtEpochMs: 10_000,
    },
    gymact,
    () => ++tick,
  );
  assert.equal(log.version, "2.0");
  assert.equal(log.events.length, 2);
  assert.deepEqual(log.events.map((e) => e.type), ["execute-auth-service", "assume-control-plane"]);
});

test("GymAct refuses a transition outside the admitted test envelope", async () => {
  const classes = await compileAdversarialClasses([goal], rules);
  await assert.rejects(
    executePowlWithGymAct(
      classes[0]!.process,
      { systemId: "system:self", facts: new Set() },
      {
        systemId: "system:self",
        allowedTransitionIds: new Set(["execute-auth-service"]),
        maxSteps: 2,
        expiresAtEpochMs: 10_000,
      },
      { async execute(activity) { return { transitionId: activity.transitionId, status: "OBSERVED" }; } },
      () => 1,
    ),
    /REFUSED: transition not admitted/,
  );
});

test("receipt contract refuses non-BLAKE3-shaped provider output", () => {
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
