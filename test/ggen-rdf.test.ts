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

test("parses ggen graph query JSON deterministically", () => {
  const result = parseGgenQueryJson(JSON.stringify({
    variables: ["node", "kind"],
    result_count: 1,
    bindings: [{ node: "<https://example.test/a>", kind: '"service"' }],
  }));
  assert.equal(result.resultCount, 1);
  assert.deepEqual(result.variables, ["node", "kind"]);
});

test("rejects malformed ggen query envelopes", () => {
  assert.throws(
    () => parseGgenQueryJson('{"variables":[],"result_count":2,"bindings":[]}'),
    /REFUSED:GGEN_QUERY_COUNT_MISMATCH/,
  );
});

test("requires the admitted ggen release version", async () => {
  const runner = new ScriptedRunner([ok(`ggen ${GGEN_PIN.version}\n`)]);
  const engine = new GgenRdfEngine(runner);
  assert.equal(await engine.assertPinnedVersion(), `ggen ${GGEN_PIN.version}`);
  assert.deepEqual(runner.calls, [["--version"]]);
});

test("refuses semantic engine version drift", async () => {
  const runner = new ScriptedRunner([ok("ggen 99.0.0\n")]);
  await assert.rejects(new GgenRdfEngine(runner).assertPinnedVersion(), /REFUSED:GGEN_VERSION_MISMATCH/);
});

test("maps public RDF dependency semantics into CASTLE graph types", async () => {
  const runner = new ScriptedRunner([
    ok(JSON.stringify({
      variables: ["node", "kind"],
      result_count: 3,
      bindings: [
        { node: "<https://example.test/crypto>", kind: '"package"' },
        { node: "<https://example.test/identity>", kind: '"service"' },
        { node: "<https://example.test/control>", kind: '"service"' },
      ],
    })),
    ok(JSON.stringify({
      variables: ["from", "to"],
      result_count: 2,
      bindings: [
        { from: "<https://example.test/crypto>", to: "<https://example.test/identity>" },
        { from: "<https://example.test/identity>", to: "<https://example.test/control>" },
      ],
    })),
  ]);

  const graph = await new GgenRdfEngine(runner).loadDependencyGraph("system.ttl");
  assert.equal(graph.nodes.size, 3);
  assert.deepEqual(graph.impactedClosure(["https://example.test/crypto"]), [
    "https://example.test/control",
    "https://example.test/crypto",
    "https://example.test/identity",
  ]);
  assert.equal(runner.calls.length, 2);
  assert.equal(runner.calls[0][0], "graph");
  assert.equal(runner.calls[0][1], "query");
  assert.ok(runner.calls[0].includes("system.ttl"));
});

test("ggen computes dependency impact closure for CONSTRUCT", async () => {
  const runner = new ScriptedRunner([
    ok(JSON.stringify({
      variables: ["impacted"],
      result_count: 3,
      bindings: [
        { impacted: "<https://example.test/control>" },
        { impacted: "<https://example.test/crypto>" },
        { impacted: "<https://example.test/identity>" },
      ],
    })),
  ]);
  const engine = new GgenRdfEngine(runner);
  assert.deepEqual(await engine.impactedClosure("system.ttl", "https://example.test/crypto"), [
    "https://example.test/control",
    "https://example.test/crypto",
    "https://example.test/identity",
  ]);
  assert.match(runner.calls[0][2], /dcterms:requires\+/);
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
    ok(JSON.stringify({
      variables: ["impacted"],
      result_count: 1,
      bindings: [{ impacted: "<https://example.test/crypto>" }],
    })),
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

test("mu1 ontology construction is delegated to ggen", async () => {
  const runner = new ScriptedRunner([ok('{"standing":"constructed"}\n')]);
  await new GgenRdfEngine(runner).constructOntology("ontology.ttl", { manifest: "ggen.toml" });
  assert.deepEqual(runner.calls[0], [
    "sync",
    "--manifest",
    "ggen.toml",
    "--stage",
    "mu1",
    "--ontology",
    "ontology.ttl",
    "--format",
    "json",
    "--dry-run",
    "true",
  ]);
});

test("non-zero ggen execution is a typed refusal", async () => {
  const runner = new ScriptedRunner([{ exitCode: 7, stdout: "", stderr: "bad graph" }]);
  await assert.rejects(new GgenRdfEngine(runner).validate("bad.ttl"), /REFUSED:GGEN_COMMAND_FAILED:bad graph/);
});
