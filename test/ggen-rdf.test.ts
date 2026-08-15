import assert from "node:assert/strict";
import test from "node:test";
import {
  GGEN_PIN,
  GgenRdfEngine,
  parseGgenQueryJson,
  type GgenCommandResult,
  type GgenCommandRunner,
} from "../src/ggen-rdf.ts";

class ScriptedRunner implements GgenCommandRunner {
  readonly calls: string[][] = [];
  constructor(private readonly results: readonly GgenCommandResult[]) {}

  async run(args: readonly string[]): Promise<GgenCommandResult> {
    this.calls.push([...args]);
    const result = this.results[this.calls.length - 1];
    if (!result) throw new Error("unexpected ggen call");
    return result;
  }
}

function ok(stdout: string): GgenCommandResult {
  return { exitCode: 0, stdout, stderr: "" };
}

function solutions(variables: string[], bindings: Record<string, string | number | boolean>[]) {
  return ok(JSON.stringify({
    kind: "solutions",
    variables,
    result_count: bindings.length,
    bindings,
  }));
}

test("parses ggen-engine SELECT JSON deterministically", () => {
  const result = parseGgenQueryJson(JSON.stringify({
    kind: "solutions",
    variables: ["node", "kind"],
    result_count: 1,
    bindings: [{ node: "https://example.test/a", kind: "service" }],
  }));
  assert.equal(result.resultCount, 1);
  assert.deepEqual(result.variables, ["node", "kind"]);
});

test("preserves datatype-aware RDF scalar bindings", () => {
  const result = parseGgenQueryJson(JSON.stringify({
    kind: "solutions",
    variables: ["enabled", "count"],
    result_count: 1,
    bindings: [{ enabled: true, count: 3 }],
  }));
  assert.deepEqual(result.bindings[0], { enabled: true, count: 3 });
});

test("rejects malformed ggen query envelopes", () => {
  assert.throws(
    () => parseGgenQueryJson('{"kind":"solutions","variables":[],"result_count":2,"bindings":[]}'),
    /REFUSED:GGEN_QUERY_COUNT_MISMATCH/,
  );
});

test("requires the admitted ggen-engine version and commit", async () => {
  const runner = new ScriptedRunner([ok(JSON.stringify({
    engine: GGEN_PIN.package,
    version: GGEN_PIN.version,
    commit: GGEN_PIN.commit,
  }))]);
  const observed = await new GgenRdfEngine(runner).assertPinnedVersion();
  assert.deepEqual(observed, {
    engine: GGEN_PIN.package,
    version: GGEN_PIN.version,
    commit: GGEN_PIN.commit,
  });
  assert.deepEqual(runner.calls, [["version"]]);
});

test("refuses semantic engine version drift", async () => {
  const runner = new ScriptedRunner([ok(JSON.stringify({
    engine: "ggen-engine",
    version: "99.0.0",
    commit: GGEN_PIN.commit,
  }))]);
  await assert.rejects(new GgenRdfEngine(runner).assertPinnedVersion(), /REFUSED:GGEN_VERSION_MISMATCH/);
});

test("maps public RDF dependency semantics into CASTLE graph types", async () => {
  const runner = new ScriptedRunner([
    solutions(["node", "kind"], [
      { node: "https://example.test/crypto", kind: "package" },
      { node: "https://example.test/identity", kind: "service" },
      { node: "https://example.test/control", kind: "service" },
    ]),
    solutions(["from", "to"], [
      { from: "https://example.test/crypto", to: "https://example.test/identity" },
      { from: "https://example.test/identity", to: "https://example.test/control" },
    ]),
  ]);

  const graph = await new GgenRdfEngine(runner).loadDependencyGraph("system.ttl");
  assert.equal(graph.nodes.size, 3);
  assert.deepEqual(graph.impactedClosure(["https://example.test/crypto"]), [
    "https://example.test/control",
    "https://example.test/crypto",
    "https://example.test/identity",
  ]);
  assert.equal(runner.calls.length, 2);
  assert.deepEqual(runner.calls[0].slice(0, 3), ["query", "--graph-file", "system.ttl"]);
  assert.match(runner.calls[0][4], /prov:Entity/);
});

test("ggen computes dependency impact closure for CONSTRUCT", async () => {
  const runner = new ScriptedRunner([
    solutions(["impacted"], [
      { impacted: "https://example.test/control" },
      { impacted: "https://example.test/crypto" },
      { impacted: "https://example.test/identity" },
    ]),
  ]);
  const engine = new GgenRdfEngine(runner);
  assert.deepEqual(await engine.impactedClosure("system.ttl", "https://example.test/crypto"), [
    "https://example.test/control",
    "https://example.test/crypto",
    "https://example.test/identity",
  ]);
  assert.match(runner.calls[0][4], /dcterms:requires\+/);
});

test("unsafe dependency IRI is refused before invoking ggen", async () => {
  const runner = new ScriptedRunner([]);
  await assert.rejects(
    new GgenRdfEngine(runner).impactedClosure("system.ttl", "https://example.test/x> } DROP ALL {"),
    /REFUSED:UNSAFE_RDF_IRI/,
  );
  assert.equal(runner.calls.length, 0);
});

test("CONSTRUCT compromise retains explicit counterfactual standing", async () => {
  const runner = new ScriptedRunner([
    solutions(["impacted"], [{ impacted: "https://example.test/crypto" }]),
  ]);
  const result = await new GgenRdfEngine(runner).constructCompromise(
    "system.ttl",
    "https://example.test/crypto",
    "execute",
  );
  assert.equal(result.epistemicClass, "COUNTERFACTUAL");
  assert.deepEqual(result.facts, [
    "compromised:https://example.test/crypto",
    "capability:https://example.test/crypto:execute",
  ]);
});

test("SPARQL CONSTRUCT is delegated to ggen-engine", async () => {
  const runner = new ScriptedRunner([ok(JSON.stringify({
    kind: "graph",
    variables: [],
    bindings: [],
    result_count: 1,
    triples: [{
      subject: "<https://example.test/a>",
      predicate: "<https://example.test/p>",
      object: "v",
      ntriples: '<https://example.test/a> <https://example.test/p> "v"',
    }],
  }))]);
  const engine = new GgenRdfEngine(runner);
  const result = await engine.construct(
    "system.ttl",
    "CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }",
  );
  assert.equal(result.kind, "graph");
  assert.deepEqual(runner.calls[0].slice(0, 3), ["query", "--graph-file", "system.ttl"]);
});

test("non-zero ggen execution is a typed refusal", async () => {
  const runner = new ScriptedRunner([{ exitCode: 7, stdout: "", stderr: "bad graph" }]);
  await assert.rejects(new GgenRdfEngine(runner).validate("bad.ttl"), /REFUSED:GGEN_COMMAND_FAILED:bad graph/);
});
