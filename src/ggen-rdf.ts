import { spawn } from "node:child_process";
import { DependencyGraph, type ConstructedCompromise, type DependencyEdge, type DependencyNode } from "./castle.ts";

export const GGEN_PIN = Object.freeze({
  repository: "https://github.com/seanchatmangpt/ggen",
  package: "ggen-engine",
  version: "26.8.15",
  commit: "162e466d8f07d0a75a468b4441b4bc8b1aad369b",
  bridgeManifest: "crates/castle-ggen-rdf/Cargo.toml",
});

const PROV_ENTITY = "http://www.w3.org/ns/prov#Entity";
const DCTERMS_TYPE = "http://purl.org/dc/terms/type";
const DCTERMS_REQUIRES = "http://purl.org/dc/terms/requires";

export interface GgenCommandResult {
  exitCode: number;
  stdout: string;
  stderr: string;
}

export interface GgenCommandRunner {
  run(args: readonly string[]): Promise<GgenCommandResult>;
}

export interface NodeGgenCommandRunnerOptions {
  binary?: string;
  cwd?: string;
  timeoutMs?: number;
  maxOutputBytes?: number;
  env?: Readonly<Record<string, string>>;
}

/**
 * Executes CASTLE's tiny Rust bridge to the exact-revision ggen-engine crate.
 * The bridge contains no RDF implementation; all Turtle/SPARQL semantics stay
 * inside ggen-engine's GraphEngine/GraphLawStore.
 */
export class NodeGgenCommandRunner implements GgenCommandRunner {
  readonly binary: string;
  readonly cwd?: string;
  readonly timeoutMs: number;
  readonly maxOutputBytes: number;
  readonly env: Readonly<Record<string, string>>;

  constructor(options: NodeGgenCommandRunnerOptions = {}) {
    this.binary = options.binary ?? process.env.CASTLE_GGEN_RDF_BIN ?? "castle-ggen-rdf";
    this.cwd = options.cwd;
    this.timeoutMs = options.timeoutMs ?? 30_000;
    this.maxOutputBytes = options.maxOutputBytes ?? 8 * 1024 * 1024;
    this.env = options.env ?? {};
  }

  run(args: readonly string[]): Promise<GgenCommandResult> {
    return new Promise((resolve, reject) => {
      const child = spawn(this.binary, [...args], {
        cwd: this.cwd,
        env: { ...process.env, ...this.env },
        shell: false,
        stdio: ["ignore", "pipe", "pipe"],
      });

      let stdout = "";
      let stderr = "";
      let outputBytes = 0;
      let timedOut = false;

      const timer = setTimeout(() => {
        timedOut = true;
        child.kill("SIGKILL");
      }, this.timeoutMs);

      const append = (target: "stdout" | "stderr", chunk: Buffer): void => {
        outputBytes += chunk.length;
        if (outputBytes > this.maxOutputBytes) {
          child.kill("SIGKILL");
          return;
        }
        if (target === "stdout") stdout += chunk.toString("utf8");
        else stderr += chunk.toString("utf8");
      };

      child.stdout.on("data", (chunk: Buffer) => append("stdout", chunk));
      child.stderr.on("data", (chunk: Buffer) => append("stderr", chunk));
      child.on("error", (error) => {
        clearTimeout(timer);
        reject(new Error(`REFUSED:GGEN_EXECUTION_ERROR:${error.message}`));
      });
      child.on("close", (code) => {
        clearTimeout(timer);
        if (timedOut) {
          reject(new Error(`REFUSED:GGEN_TIMEOUT:${this.timeoutMs}`));
          return;
        }
        if (outputBytes > this.maxOutputBytes) {
          reject(new Error(`REFUSED:GGEN_OUTPUT_LIMIT:${this.maxOutputBytes}`));
          return;
        }
        resolve({ exitCode: code ?? -1, stdout, stderr });
      });
    });
  }
}

export type GgenRdfScalar = string | number | boolean;

export interface GgenQueryResult {
  variables: readonly string[];
  bindings: readonly Readonly<Record<string, GgenRdfScalar>>[];
  resultCount: number;
}

interface GgenVersionEnvelope {
  engine: string;
  version: string;
  commit: string;
}

function parseJsonObject(stdout: string): Record<string, unknown> {
  const trimmed = stdout.trim();
  if (!trimmed) throw new Error("REFUSED:GGEN_EMPTY_OUTPUT");
  let parsed: unknown;
  try {
    parsed = JSON.parse(trimmed);
  } catch {
    throw new Error("REFUSED:GGEN_NON_JSON_OUTPUT");
  }
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error("REFUSED:GGEN_INVALID_JSON_ENVELOPE");
  }
  return parsed as Record<string, unknown>;
}

export function parseGgenQueryJson(stdout: string): GgenQueryResult {
  const record = parseJsonObject(stdout);
  if (record.kind !== "solutions") throw new Error("REFUSED:GGEN_QUERY_NOT_SOLUTIONS");
  const variables = record.variables;
  const bindings = record.bindings;
  const resultCount = record.result_count ?? record.resultCount;
  if (!Array.isArray(variables) || !variables.every((value) => typeof value === "string")) {
    throw new Error("REFUSED:GGEN_INVALID_QUERY_VARIABLES");
  }
  if (!Array.isArray(bindings)) throw new Error("REFUSED:GGEN_INVALID_QUERY_BINDINGS");

  const normalizedBindings = bindings.map((binding) => {
    if (!binding || typeof binding !== "object" || Array.isArray(binding)) {
      throw new Error("REFUSED:GGEN_INVALID_QUERY_BINDING");
    }
    const normalized: Record<string, GgenRdfScalar> = {};
    for (const [key, value] of Object.entries(binding as Record<string, unknown>)) {
      if (typeof value !== "string" && typeof value !== "number" && typeof value !== "boolean") {
        throw new Error("REFUSED:GGEN_INVALID_RDF_SCALAR");
      }
      normalized[key] = value;
    }
    return normalized;
  });

  const count = typeof resultCount === "number" ? resultCount : normalizedBindings.length;
  if (!Number.isInteger(count) || count < 0 || count !== normalizedBindings.length) {
    throw new Error("REFUSED:GGEN_QUERY_COUNT_MISMATCH");
  }
  return { variables, bindings: normalizedBindings, resultCount: count };
}

function safeSparqlIri(iri: string): string {
  if (!iri || /[\s<>"'{}\\]/.test(iri)) throw new Error("REFUSED:UNSAFE_RDF_IRI");
  let parsed: URL;
  try {
    parsed = new URL(iri);
  } catch {
    throw new Error("REFUSED:INVALID_RDF_IRI");
  }
  if (!new Set(["http:", "https:", "urn:"]).has(parsed.protocol)) {
    throw new Error("REFUSED:UNSUPPORTED_RDF_IRI_SCHEME");
  }
  return `<${iri}>`;
}

function requireBinding(
  binding: Readonly<Record<string, GgenRdfScalar>>,
  key: string,
): string {
  const value = binding[key];
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`REFUSED:GGEN_MISSING_STRING_BINDING:${key}`);
  }
  return value;
}

export class GgenRdfEngine {
  constructor(readonly runner: GgenCommandRunner = new NodeGgenCommandRunner()) {}

  private async execute(args: readonly string[]): Promise<GgenCommandResult> {
    const result = await this.runner.run(args);
    if (result.exitCode !== 0) {
      const detail = result.stderr.trim() || result.stdout.trim() || `exit=${result.exitCode}`;
      throw new Error(`REFUSED:GGEN_COMMAND_FAILED:${detail}`);
    }
    return result;
  }

  async assertPinnedVersion(): Promise<GgenVersionEnvelope> {
    const result = await this.execute(["version"]);
    const envelope = parseJsonObject(result.stdout);
    const observed = {
      engine: envelope.engine,
      version: envelope.version,
      commit: envelope.commit,
    };
    if (
      observed.engine !== GGEN_PIN.package ||
      observed.version !== GGEN_PIN.version ||
      observed.commit !== GGEN_PIN.commit
    ) {
      throw new Error(
        `REFUSED:GGEN_VERSION_MISMATCH:expected=${GGEN_PIN.package}@${GGEN_PIN.version}#${GGEN_PIN.commit}`,
      );
    }
    return observed as GgenVersionEnvelope;
  }

  async validate(graphFile: string): Promise<Record<string, unknown>> {
    const result = await this.execute(["validate", "--graph-file", graphFile]);
    return parseJsonObject(result.stdout);
  }

  async query(graphFile: string, sparql: string): Promise<GgenQueryResult> {
    if (!sparql.trim()) throw new Error("REFUSED:EMPTY_SPARQL_QUERY");
    const result = await this.execute(["query", "--graph-file", graphFile, "--sparql", sparql]);
    return parseGgenQueryJson(result.stdout);
  }

  async queryRaw(graphFile: string, sparql: string): Promise<Record<string, unknown>> {
    if (!sparql.trim()) throw new Error("REFUSED:EMPTY_SPARQL_QUERY");
    const result = await this.execute(["query", "--graph-file", graphFile, "--sparql", sparql]);
    return parseJsonObject(result.stdout);
  }

  /** Execute a SPARQL CONSTRUCT through ggen-engine and return its graph envelope. */
  async construct(graphFile: string, sparql: string): Promise<Record<string, unknown>> {
    const result = await this.queryRaw(graphFile, sparql);
    if (result.kind !== "graph") throw new Error("REFUSED:GGEN_CONSTRUCT_NOT_GRAPH");
    return result;
  }

  async loadDependencyGraph(graphFile: string): Promise<DependencyGraph> {
    const nodesResult = await this.query(
      graphFile,
      `PREFIX prov: <http://www.w3.org/ns/prov#>\nPREFIX dcterms: <http://purl.org/dc/terms/>\nSELECT DISTINCT ?node ?kind WHERE {\n  ?node a prov:Entity .\n  OPTIONAL { ?node dcterms:type ?kind . }\n}\nORDER BY ?node\nLIMIT 100000`,
    );
    const edgesResult = await this.query(
      graphFile,
      `PREFIX prov: <http://www.w3.org/ns/prov#>\nPREFIX dcterms: <http://purl.org/dc/terms/>\nSELECT DISTINCT ?from ?to WHERE {\n  ?to dcterms:requires ?from .\n  ?from a prov:Entity .\n  ?to a prov:Entity .\n}\nORDER BY ?from ?to\nLIMIT 100000`,
    );

    const nodes: DependencyNode[] = nodesResult.bindings.map((binding) => ({
      id: requireBinding(binding, "node"),
      kind: typeof binding.kind === "string" ? binding.kind : "entity",
    }));
    const edges: DependencyEdge[] = edgesResult.bindings.map((binding) => ({
      from: requireBinding(binding, "from"),
      to: requireBinding(binding, "to"),
      relation: DCTERMS_REQUIRES,
    }));
    return new DependencyGraph(nodes, edges);
  }

  async impactedClosure(graphFile: string, dependencyId: string): Promise<string[]> {
    const dependency = safeSparqlIri(dependencyId);
    const result = await this.query(
      graphFile,
      `PREFIX prov: <http://www.w3.org/ns/prov#>\nPREFIX dcterms: <http://purl.org/dc/terms/>\nSELECT DISTINCT ?impacted WHERE {\n  VALUES ?dependency { ${dependency} }\n  ?dependency a prov:Entity .\n  { BIND(?dependency AS ?impacted) }\n  UNION { ?impacted dcterms:requires+ ?dependency . }\n}\nORDER BY ?impacted\nLIMIT 100000`,
    );
    if (result.resultCount === 0) throw new Error(`REFUSED:UNKNOWN_RDF_DEPENDENCY:${dependencyId}`);
    return [...new Set(result.bindings.map((binding) => requireBinding(binding, "impacted")))].sort();
  }

  async constructCompromise(
    graphFile: string,
    dependencyId: string,
    capability: string,
  ): Promise<ConstructedCompromise> {
    if (!capability || capability.includes("\u0000")) {
      throw new Error("REFUSED:INVALID_COMPROMISE_CAPABILITY");
    }
    const impacted = await this.impactedClosure(graphFile, dependencyId);
    return {
      dependencyId,
      capability,
      facts: [`compromised:${dependencyId}`, `capability:${dependencyId}:${capability}`],
      impacted,
      epistemicClass: "COUNTERFACTUAL",
    };
  }
}

export const CASTLE_RDF_VOCABULARY = Object.freeze({
  provEntity: PROV_ENTITY,
  dctermsType: DCTERMS_TYPE,
  dctermsRequires: DCTERMS_REQUIRES,
});
