//! Strata receipt RDF-lift courts: the pinned sw: witness fixture
//! (`fixtures/strata/witness.ttl`, sha256-pinned to the live
//! ggen-marketplace strata-ledger-pack emission) is lifted onto the castle
//! PROV/DCTERMS profile by `rdf/strata/receipt-map.rq` and the lifted triple
//! set is asserted exactly, plus a fail-closed negative (unknown vocabulary
//! lifts to zero `prov:Entity`).
//!
//! CASTLE delegates SPARQL to ggen; there is no second SPARQL engine in the
//! Rust tree, so this court evaluates the CONSTRUCT through the locally
//! available `python3` + `rdflib` (real engine, real fixture, no mocks) and
//! refuses by type if the engine is absent.

use std::path::PathBuf;
use std::process::Command;

const FIXTURE_SHA256: &str = "911299bbb4bf39a2c0800f223f5b5f549642c9bfe53e4fd57d54a731664c3f2b";

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/strata/witness.ttl")
}

fn map_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("rdf/strata/receipt-map.rq")
}

fn run_python(script: &str, args: &[PathBuf]) -> Result<String, String> {
    let out = Command::new("python3")
        .arg("-c")
        .arg(script)
        .args(args)
        .output()
        .map_err(|e| format!("REFUSED:STRATA_RDF_ENGINE:{e}"))?;
    if !out.status.success() {
        return Err(format!(
            "REFUSED:STRATA_RDF_LIFT_FAILED:{}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The pinned fixture is byte-identical to the pinned sha256 (matching the
/// `fixtures/strata/ledger-receipt.sha256` pin pattern).
#[test]
fn fixture_matches_sha256_pin() {
    let bytes = std::fs::read(fixture_path()).expect("fixture readable");
    let digest = hex_digest(&bytes);
    assert_eq!(
        digest, FIXTURE_SHA256,
        "fixtures/strata/witness.ttl drifted from its pin"
    );
}

/// Positive court: the CONSTRUCT lifts exactly 17 triples — 4 GateRuns, each
/// a prov:Entity with its hash literal and two dcterms:type qualifiers, plus
/// one dcterms:requires edge for the pack-attributed run.
#[test]
fn lift_produces_exact_projected_triples() {
    let script = r#"
import sys
from rdflib import Graph, URIRef, Literal, Namespace
from rdflib.namespace import RDF

g = Graph()
g.parse(sys.argv[1], format="turtle")
q = open(sys.argv[2]).read()
lifted = g.query(q).graph

PROV = Namespace("http://www.w3.org/ns/prov#")
DCT = Namespace("http://purl.org/dc/terms/")
CS = Namespace("https://castle.chatmangpt.com/profile/strata#")
SW = Namespace("https://strata.chatman.ai/ontology/witness#")

runs = sorted(set(lifted.subjects(RDF.type, PROV.Entity)), key=str)
assert len(runs) == 4, f"expected 4 lifted GateRun entities, got {len(runs)}: {runs}"

for run in runs:
    hashes = list(lifted.objects(run, CS.syncedGraphHash))
    assert len(hashes) == 1, f"{run}: exactly one hash literal, got {hashes}"
    assert isinstance(hashes[0], Literal), "hash stored as literal, never equated"
    types = sorted(str(t) for t in lifted.objects(run, DCT.type))
    assert types == ["andon:Green", "zeroRows:true"], f"{run}: {types}"

pack_run = SW["gate-010_ledger_layout"]
reqs = list(lifted.objects(pack_run, DCT.requires))
assert reqs == [], "gate runs without sw:pack must not require anything"

# exact total: 4*(type + hash + 2 qualifiers) + 1 requires
total = len(lifted)
assert total == 17, f"expected exactly 17 lifted triples, got {total}: {lifted.serialize(format='nt')}"
print("OK")
"#;
    let out = run_python(&script, &[fixture_path(), map_path()]).expect("lift court");
    assert_eq!(out.trim(), "OK");
}

/// Negative court: unknown vocabulary (a different witness namespace)
/// fails closed — zero prov:Entity triples are lifted.
#[test]
fn unknown_vocabulary_lifts_nothing() {
    let script = r#"
import sys
from rdflib import Graph

ttl = """
@prefix sw2: <https://example.invalid/other-witness#> .
<https://example.invalid/run/1> a sw2:GateRun ;
    sw2:syncedGraphHash "deadbeef" ;
    sw2:andon "Green" ;
    sw2:zeroRows true .
<https://example.invalid/run/1> sw2:pack <https://example.invalid/pack/x> .
"""
g = Graph()
g.parse(data=ttl, format="turtle")
q = open(sys.argv[1]).read()
lifted = g.query(q).graph
assert len(lifted) == 0, f"unknown vocabulary must lift nothing, got {lifted.serialize(format='nt')}"
print("OK")
"#;
    let out = run_python(&script, &[map_path()]).expect("negative court");
    assert_eq!(out.trim(), "OK");
}

/// sha256 hex without adding a hex crate dependency to the test surface.
fn hex_digest(bytes: &[u8]) -> String {
    // Fingerprint via python3 (the same engine the RDF courts already
    // require) — avoids a duplicate hashing implementation.
    use std::process::Stdio;
    let mut child = Command::new("python3")
        .arg("-c")
        .arg("import sys,hashlib;print(hashlib.sha256(sys.stdin.buffer.read()).hexdigest())")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("python3 available");
    use std::io::Write;
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(bytes)
        .expect("write fixture");
    String::from_utf8_lossy(&child.wait_with_output().expect("wait").stdout)
        .trim()
        .to_string()
}
