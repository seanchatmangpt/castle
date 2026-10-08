#!/usr/bin/env python3
"""Generate castle-gates-skills.json from docs/sjira/v26.10.8/skills.ttl.

Projects the crown-gate sj:Action individuals (semantic-jira-pack
gates/060_a2a_skills.rq shape) into a deterministic JSON sidecar. Parsing goes
through rapper (turtle -> ntriples); the script fails closed if any exposed
skill is missing a required a2a: property, if a2a:exposed is not true, or if
the required OBSERVE-surface law sentence is absent from the description.

Deterministic: two runs over the same inputs are byte-identical (keys sorted,
entries sorted by skill name).

CONSTRUCT != DO: these are OBSERVE-surface gate STATUS queries. A SELECT
result never actuates; consequential DO is reachable only through the
receipted ConstructAdmission -> BRCE boundary.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SKILLS_TTL = REPO / "docs/sjira/v26.10.8/skills.ttl"
OUT = REPO / "docs/sjira/v26.10.8/castle-gates-skills.json"

A2A = "https://ggen-igniter.dev/ontology/a2a#"
GI = "https://ggen-igniter.dev/ontology/ggen-igniter#"
RDFS_LABEL = "http://www.w3.org/2000/01/rdf-schema#label"
DCTERMS_DESC = "http://purl.org/dc/terms/description"
LAW = "receipt-driven STOP; SELECT never implies DO"

REQUIRED = ["skillName", "ashAction", "consequence", "requiresCapability", "generatedBy"]
RDF_TYPE = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
SJ_ACTION = "https://ggen-igniter.dev/ontology/semantic-jira#Action"


def ntriples() -> list[str]:
    if not SKILLS_TTL.exists():
        sys.exit(f"REFUSED:MISSING_SKILLS_TTL {SKILLS_TTL}")
    proc = subprocess.run(
        ["rapper", "-i", "turtle", "-o", "ntriples", str(SKILLS_TTL)],
        capture_output=True, text=True,
    )
    if proc.returncode != 0:
        print(proc.stderr, file=sys.stderr)
        sys.exit("REFUSED:INVALID_TURTLE")
    return [ln for ln in proc.stdout.splitlines() if ln.startswith("<")]


def parse() -> dict[str, dict[str, str]]:
    """subject -> {predicate -> object}; last write wins (skills.ttl is flat)."""
    triples: dict[str, dict[str, str]] = {}
    line_re = re.compile(r'^<([^>]+)> <([^>]+)> (?:<([^>]*)>|"((?:[^"\\]|\\.)*)")')
    for line in ntriples():
        m = line_re.match(line)
        if m:
            subj, pred = m.group(1), m.group(2)
            obj = m.group(3) if m.group(3) is not None else m.group(4)
            triples.setdefault(subj, {})[pred] = obj
    return triples


def main() -> None:
    triples = parse()

    actions = {
        s: props
        for s, props in triples.items()
        if props.get(RDF_TYPE) == SJ_ACTION
    }
    if len(actions) != 14:
        sys.exit(f"REFUSED:EXPECTED_14_GATES found={len(actions)}")

    skills = []
    for subj, props in actions.items():
        missing = [f for f in REQUIRED if (A2A + f) not in props]
        if missing:
            sys.exit(f"REFUSED:MISSING_PROPERTY {subj} {missing}")
        if props.get(A2A + "exposed") != "true":
            gate = props.get(A2A + "skillName", subj)
            sys.exit(f"REFUSED:NOT_EXPOSED {gate}")
        description = props.get(DCTERMS_DESC, "")
        if LAW not in description:
            sys.exit(f"REFUSED:LAW_SENTENCE_MISSING {props.get(A2A + 'skillName', subj)}")
        gen_iri = props.get(A2A + "gate-skill-generator", "")
        gen_iri = props.get(A2A + "generatedBy", "")
        if triples.get(gen_iri, {}).get(GI + "standing") != "ALIVE":
            sys.exit(f"REFUSED:GENERATOR_NOT_ALIVE {gen_iri}")
        skills.append({
            "skillName": props[A2A + "skillName"],
            "ashAction": props[A2A + "ashAction"],
            "consequence": props[A2A + "consequence"],
            "exposed": True,
            "requiresCapability": props[A2A + "requiresCapability"],
            "generatedBy": gen_iri,
            "label": props.get(RDFS_LABEL, ""),
            "description": description,
        })

    doc = {
        "surface": "OBSERVE",
        "law": LAW,
        "authorityCeiling": "CONSTRUCT",
        "skills": skills,
    }
    OUT.write_text(json.dumps(doc, indent=2, sort_keys=True) + "\n")
    print(f"ok: {len(skills)} skills -> {OUT}")


if __name__ == "__main__":
    main()
