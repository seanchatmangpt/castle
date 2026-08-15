import assert from "node:assert/strict";
import test from "node:test";
import { GGEN_PIN, GgenRdfEngine, NodeGgenCommandRunner } from "../src/ggen-rdf.ts";

const integration = process.env.CASTLE_GGEN_INTEGRATION === "1" ? test : test.skip;
const fixture = "rdf/examples/dependency-system.ttl";
const crypto = "https://castle.chatmangpt.com/system/crypto";

integration("real ggen-engine executes CASTLE RDF dependency semantics", async () => {
  const engine = new GgenRdfEngine(new NodeGgenCommandRunner({
    binary: process.env.CASTLE_GGEN_RDF_BIN,
    timeoutMs: 60_000,
  }));

  const version = await engine.assertPinnedVersion();
  assert.deepEqual(version, {
    engine: GGEN_PIN.package,
    version: GGEN_PIN.version,
    commit: GGEN_PIN.commit,
  });

  const validation = await engine.validate(fixture);
  assert.equal(validation.valid, true);
  assert.equal(validation.quad_count, 10);
  assert.match(String(validation.state_hash), /^[0-9a-f]{64}$/);

  const graph = await engine.loadDependencyGraph(fixture);
  assert.equal(graph.nodes.size, 4);
  assert.deepEqual(graph.impactedClosure([crypto]), [
    "https://castle.chatmangpt.com/system/control-plane",
    crypto,
    "https://castle.chatmangpt.com/system/identity-api",
  ]);

  const impact = await engine.impactedClosure(fixture, crypto);
  assert.deepEqual(impact, [
    "https://castle.chatmangpt.com/system/control-plane",
    crypto,
    "https://castle.chatmangpt.com/system/identity-api",
  ]);

  const constructed = await engine.construct(
    fixture,
    `PREFIX dcterms: <http://purl.org/dc/terms/>\nCONSTRUCT { ?service dcterms:requires ?dependency } WHERE { ?service dcterms:requires ?dependency }`,
  );
  assert.equal(constructed.kind, "graph");
  assert.equal(constructed.result_count, 2);

  const compromise = await engine.constructCompromise(fixture, crypto, "execute");
  assert.equal(compromise.epistemicClass, "COUNTERFACTUAL");
  assert.deepEqual(compromise.impacted, impact);
});
