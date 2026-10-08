#!/usr/bin/env python3
"""Generate .well-known/agent-card.json from the castle CLI's #[verb] registrations.

Skill existence is derived from src/bin/castle/verbs/routes.rs: every published
skill id must correspond to a real `#[verb("verb", "noun")]` registration; the
generator fails closed if one goes missing. Descriptions and tags for the
published A2A surface are carried in SKILL_METADATA below (the card is a curated
subset of the internal CLI surface, so not every registration is published).

Deterministic: two runs over the same inputs are byte-identical.

CONSTRUCT != DO: this card is descriptive only. It grants no authority;
consequential DO is reachable only through the receipted ConstructAdmission ->
BRCE boundary.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
ROUTES = REPO / "src" / "bin" / "castle" / "verbs" / "routes.rs"
CARGO = REPO / "Cargo.toml"
OUT = REPO / ".well-known" / "agent-card.json"

# CONSTRUCT != DO boundary statement (single source, emitted into the card).
CONSTRUCT_NE_DO = (
    "CONSTRUCT and planner results are candidates only; consequential DO is "
    "reachable only through the receipted ConstructAdmission -> BRCE boundary. "
    "This card is descriptive: it grants no authority."
)

AGENT_NAME = "castle"
SUPPORTED_INTERFACES = [
    {"url": "stdio://castle", "protocolBinding": "JSONRPC", "protocolVersion": "1.0"}
]
DEFAULT_INPUT_MODES = ["text/plain"]
DEFAULT_OUTPUT_MODES = ["text/plain"]

# Published A2A skill surface, in card order. Each id must match a #[verb]
# registration in routes.rs (noun, verb) and the handler's doc comment (if any)
# is checked below against DESCRIPTIONS.
SKILL_METADATA = {
    "castle.fortune5.requirements": {
        "description": "Emit the 40-control Fortune-5 readiness requirements profile.",
        "tags": ["qualification", "fortune5"],
    },
    "castle.fortune5.qualify": {
        "description": "Qualify a subject against receipted metric observations; missing evidence is UNKNOWN, failing evidence is REFUSED.",
        "tags": ["qualification", "fortune5"],
    },
    "castle.deployment.qualify": {
        "description": "Qualify deployment readiness from receipted observations.",
        "tags": ["qualification", "deployment"],
    },
    "castle.chaos.qualify": {
        "description": "Fail-closed failure-semantics qualification for degraded/local actuation modes.",
        "tags": ["qualification", "chaos"],
    },
    "castle.construct.manufacture": {
        "description": "Manufacture a public, reversible CONSTRUCT capability bound to BLAKE3 parent receipts (O*, config, ontology, process). Candidates only; no DO authority.",
        "tags": ["construct", "candidate"],
    },
    "castle.do.execute": {
        "description": "The exclusive exported DO path: requires a genuine ConstructAdmission, recomputes process digest and bounds, returns a child receipt. Refuses without admitted CONSTRUCT origin.",
        "tags": ["actuation", "brce", "do"],
    },
    "castle.replay.admit": {
        "description": "Admit a replay against exact-subject receipts.",
        "tags": ["qualification", "replay"],
    },
    "castle.impact.coverage": {
        "description": "Report replay/impact coverage inventory.",
        "tags": ["inventory", "replay"],
    },
    "castle.inventory.components": {
        "description": "Inventory capability components.",
        "tags": ["inventory"],
    },
    "castle.inventory.goals": {
        "description": "Inventory sJira goal-graph goals (authority-free).",
        "tags": ["inventory", "sjira"],
    },
    "castle.release.info": {
        "description": "Report release identity and standing.",
        "tags": ["metadata"],
    },
    "castle.protocol.a2a": {
        "description": "Report A2A protocol surface.",
        "tags": ["protocol", "a2a"],
    },
}

# Doc comments on #[verb] routes, used only to verify the curated descriptions
# stay consistent with the routes file (soft check, reported not fatal).
ROUTE_DOC_COMMENTS = {
    "castle.construct.manufacture": "Manufacture an inert, deterministic CONSTRUCT checkpoint from a runtime request.",
    "castle.do.execute": "Recompute, admit, and execute the exact previously manufactured CONSTRUCT.",
}


def parse_registrations() -> list[tuple[str, str, str | None, str]]:
    """Return (noun, verb, doc_comment_or_None, handler_fn) for every #[verb] in routes.rs."""
    text = ROUTES.read_text()
    out = []
    # Optional preceding /// doc comment, then #[verb("verb", "noun")]
    pattern = re.compile(
        r"(?:///\s*(?P<doc>[^\n]*)\n\s*)?#\[verb\(\"(?P<verb>[^\"]+)\",\s*\"(?P<noun>[^\"]+)\"\)\]\s*\nfn\s+(?P<handler>\w+)"
    )
    for m in pattern.finditer(text):
        out.append(
            (
                m.group("noun"),
                m.group("verb"),
                (m.group("doc") or "").strip() or None,
                m.group("handler"),
            )
        )
    if not out:
        sys.exit(f"FAIL: no #[verb] registrations found in {ROUTES}")
    return out


def parse_version() -> str:
    text = CARGO.read_text()
    m = re.search(r'^version\s*=\s*"([^"]+)"', text, re.MULTILINE)
    if not m:
        sys.exit(f"FAIL: no package version in {CARGO}")
    return m.group(1)


def main() -> None:
    registrations = parse_registrations()
    registered = {f"castle.{noun}.{verb}" for noun, verb, _, _ in registrations}
    handlers = {f"castle.{noun}.{verb}": handler for noun, verb, _, handler in registrations}

    # Fail closed: every published skill must exist as a real registration.
    missing = sorted(set(SKILL_METADATA) - registered)
    if missing:
        sys.exit(f"FAIL: published skills without #[verb] registration: {missing}")

    unpublished = sorted(registered - set(SKILL_METADATA))
    if unpublished:
        print(f"note: {len(unpublished)} registered verbs not published to the card: {unpublished}")

    docs = {f"castle.{noun}.{verb}": doc for noun, verb, doc, _ in registrations if doc}
    for sid, doc in ROUTE_DOC_COMMENTS.items():
        if sid in docs and docs[sid] != doc:
            print(f"note: doc-comment drift for {sid}: routes.rs says {docs[sid]!r}")

    version = parse_version()
    skills = []
    for sid, meta in SKILL_METADATA.items():
        skills.append(
            {
                "id": sid,
                "name": handlers[sid].removeprefix("fn_").replace("_", " "),
                "description": meta["description"],
                "tags": list(meta["tags"]),
            }
        )

    card = {
        "name": AGENT_NAME,
        "description": f"CASTLE board-governed consequence and strategic-command system (CLI). {CONSTRUCT_NE_DO}",
        "version": version,
        "supportedInterfaces": SUPPORTED_INTERFACES,
        "capabilities": {},
        "defaultInputModes": DEFAULT_INPUT_MODES,
        "defaultOutputModes": DEFAULT_OUTPUT_MODES,
        "skills": skills,
    }

    text = json.dumps(card, indent=2, ensure_ascii=False) + "\n"
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(text)
    print(f"wrote {OUT} ({len(skills)} skills, version {version})")


if __name__ == "__main__":
    main()
