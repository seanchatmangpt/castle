# CASTLE — Mission

## Statement

CASTLE exists because the defender's side of infrastructure security has been running on a
throughput assumption that no longer holds: that a human reviewer, checking work at human
speed, constitutes a real verification gate against an adversary who is not rate-limited the
same way. It doesn't, once the adversary automates. CASTLE's mission is to make the defensive
side automatable and receipted at the same rate an automated adversary can attack — without
relaxing the one invariant that makes automation safe to build: `CONSTRUCT != DO`.

Everything else in this repository is downstream of that sentence.

## Why this is a mission and not a feature list

An attacker who has automated their generation loop — variant exploits, malicious commits,
dependency compromise, supply-chain injection — is not bound by attention-hours. A defense
built on "a human looked at it" is. If defense throughput is human-gated and attack throughput
is machine-gated, the defender loses that race structurally, before any skill differential
matters, purely on rate.

The correct posture is not "remove verification." It's the opposite: verification has to be
*at least as automatable as the attack*, or it isn't a real gate against a software-manufacturing
adversary — it's a gate against a slow, polite one, which is not the threat model
(`README.md:65-74`, `CLAUDE.md:13-19`). CASTLE's answer to "how do you automate verification
without automating recklessness" is `CONSTRUCT != DO`: candidates (from a planner, from an
LLM, from anything) never carry actuation authority on their own. Authority is a separate,
cryptographically receipted admission step (`admit_construct_for_do`), enforced at compile time
in the current Rust implementation, not by convention or by a human's attention that day
(`CLAUDE.md:13-19`).

That's the actual answer to "can this be trusted without a human checking every step": not
"a human checks every step" (which doesn't scale and doesn't out-race an automated attacker
anyway), but "no step can actuate without a signed, receipted, structurally-unforgeable
admission" — a check that runs every time, doesn't fatigue, doesn't skim, and is fast enough to
match what it's defending against.

## What CASTLE is, honestly, as of this writing

A **verified reasoning and receipt-admission kernel** (`VISION.md:33-36`) — DfCM inverse
construction from prohibited goals, POWL-style causal partial orders, a bounded GymAct test
envelope, OCEL v2 evidence, BLAKE3-256 receipts. It is not yet a system that observes or tests
real infrastructure on its own; every "OBSERVED" event in the current test suite comes from a
test double (`VISION.md:33-36`). `VISION.md`'s 2030 horizon names the concrete, falsifiable
conditions that close that gap — a real `GymActAdapter`, a real second planner, CI-enforced
`cargo test`. This document does not restate those; see `VISION.md` for the target state and
`VISION.md:38-54` for the honestly-admitted gaps between kernel and system today.

## What does not change on the way there

`CONSTRUCT != DO` is not a current limitation to be relaxed as CASTLE matures — it's the
property that makes maturing safe. A 2030 CASTLE with real adapters and a real planner ensemble
is only worth building if every one of those adapters inherits the fail-closed admission chain
that exists today, rather than finding a side door around it (`VISION.md:103-107`). The mission
is not "automate defense." It's "automate defense without ever making authority a side effect
of automation" — the two failure modes (too slow to matter, too permissive to trust) are both
losses, and CASTLE's bet is that the receipted-admission design is how you avoid both at once.

## See Also

- [`VISION.md`](VISION.md) — the falsifiable 2030 horizon: what would have to be true, not an
  adjective.
- [`README.md`](README.md) — the DfCM CONSTRUCT origin law and the current executable slice.
- [`CLAUDE.md`](CLAUDE.md) — the implementation-level statement of `CONSTRUCT != DO` and why it
  is enforced by the Rust compiler rather than a runtime check.
- [`PROVENANCE.md`](PROVENANCE.md) — "CASTLE separates semantic authority from consumer
  consequences," the same doctrine applied to who gets to claim CASTLE's standing.
