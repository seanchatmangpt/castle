import assert from "node:assert/strict";
import test from "node:test";
import { GGEN_PIN, GgenRdfEngine, NodeGgenCommandRunner } from "../src/ggen-rdf.ts";

const integration = process.env.CASTLE_GGEN_INTEGRATION === "1" ? test : test.skip;
const fixture = "rdf/examples/dependency-system.ttl";
const crypto = "https://castle.chatmangpt.com/system/crypto";

integration("real ggen executes CASTLE RDF dependency semantics", async () => {
  const engine = new GgenRdfEngine(new NodeGgenCommandRunner({
    binary: process.env.GGEN_BIN,
    timeoutMs: 60_000,
  }));

  const version = await engine.assertPinnedVersion();
  assert.match(version, new RegExp(GGEN_PIN.version.replaceAll(".", "\\.")));

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

  const compromise = await engine.constructCompromise(fixture, crypto, "execute");
  assert.equal(compromise.epistemicClass, "COUNTERFACTUAL");
  assert.deepEqual(compromise.impacted, impact);
});
