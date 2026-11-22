# Victory Audit Protocol — CASTLE-SPEC-26.11.20

Zero-context replay protocol: a runtime holding **only this file plus the canonical
checkout `/Users/sac/castle`** executes every gate below, in order, and seals or refuses
the campaign's ALIVE receipt purely from observed command output. No conversation
memory, no agent claims, no CI conclusions are inputs. `inspection ≠ execution` — every
gate is an execution.

| Field | Value |
|---|---|
| Protocol lane | zai-20-victory-auditor |
| Written | 2026-11-20 |
| Repo | `/Users/sac/castle` (package `castle`, edition 2021, bin `castle` @ `src/bin/castle/main.rs`) |
| Pre-campaign base observed at write time | `3ffa1bbd66f6d1d871da046a94a32daf10ebf6b1` (`preserve(v26.11.16): migration snapshot`) |
| Authoritative base | `identity.base_sha` inside `docs/sjira/v26.11.20/victory.receipt.json` (the receipt wins if it differs; this file's value is an observation, not law) |
| Target receipt | `docs/sjira/v26.11.20/victory.receipt.json` |
| Validator (operator harness, outside repo) | `~/.zcode/dfcm/validate_receipt.py` against `~/.zcode/dfcm/receipt.schema.json` |

Environment preconditions verified 2026-11-20: cargo ≥ 1.85 observed at 1.97.0;
path dependency `../chicago-tdd-tools` present; fixture
`tests/fixtures/fortune5_evidence_alive.json` present; validator + schema present.
Python is used **only** as the operator-side receipt validator named above — zero Python
in repo runtime/dev code.

Run every gate from the repo root. Stop at the first non-conforming gate; the sealing
standing is then `BLOCKED` or `BUILD_BROKEN` (never papered over to ALIVE).

---

## Gate 0 — identity pin (R.identity)

```sh
cd /Users/sac/castle
git rev-parse HEAD                      # → SUBJECT_SHA, exit 0
BASE_SHA=$(python3 -c "import json;print(json.load(open('docs/sjira/v26.11.20/victory.receipt.json'))['identity']['subject_sha'])")
test "$SUBJECT_SHA" = "$BASE_SHA"       # exit 0 = identity holds
git cat-file -e "$SUBJECT_SHA"          # exit 0 = subject resolves as a real commit
git merge-base --is-ancestor "$(python3 -c "import json;print(json.load(open('docs/sjira/v26.11.20/victory.receipt.json'))['identity']['base_sha'])")" HEAD  # exit 0 = base is an ancestor, base never silently moved
```

Pass: all four exit 0. Fail: `R_missing_identity` — refuse the seal, standing `BLOCKED`.

## Gate 1 — git diff check (R.consequence)

```sh
cd /Users/sac/castle
git diff --check HEAD --                # exit 0, empty stdout: no conflict markers, no whitespace errors
git status --porcelain                  # nothing under src/, tests/, fixtures/, Cargo.toml, Cargo.lock — tree fully committed; untracked allowed only under docs/sjira/ and output/
git diff --stat <BASE_SHA>..HEAD        # enumerable file list; every path attributable to a declared lane target
git diff --exit-code <BASE_SHA>..HEAD -- src/lib.rs Cargo.toml Cargo.lock   # exit 0
```

The protected-file zero-diff is the anti-collision check for the 20-lane partition:
any nonzero diff against `src/lib.rs`, `Cargo.toml`, or `Cargo.lock` must be named in the
receipt's consequence field with its owning authority; otherwise `mu_unlawful` — refuse.
`<BASE_SHA>` is the receipt's `identity.base_sha` (Gate 0 confirmed ancestry).

## Gate 2 — full cargo suite (R.replay, primary)

```sh
cd /Users/sac/castle
cargo test --no-fail-fast 2>&1 | tee /tmp/castle-v26.11.20-cargo.log
echo "exit=$?"
```

Pass: exit 0 and `0 failed` across all targets (lib unit tests + all 75 integration
targets, including the subprocess CLI courts `cli_fortune5`, `cli_construct_do_v26_8_18`,
`cli_evidence_verify_v26_8_18`, `cli_negative_evidence`). Record the `test result:` lines
from the log — they are the receipt's replay evidence. Exit ≠ 0: `BUILD_BROKEN`, preserve
command/exit/diagnostic from the log, no re-run without a new hypothesis.

## Gate 3 — subprocess CLI replay (R.replay, independent re-witness)

Cargo's `CARGO_BIN_EXE_castle` mechanism already ran these as real child processes inside
Gate 2; this gate re-witnesses them from the shell so the receipt's replay commands are
externally executable. Build once, then run each command **twice** and require identical
output (determinism twin — all inputs below are pinned, no wall-clock enters).

```sh
cd /Users/sac/castle
cargo build                              # exit 0
BIN=./target/debug/castle

# 3a — static requirement surface: exit 0, valid JSON, count 40, first control F5-AUTH-001
$BIN fortune5 requirements --format json | tee /tmp/r1.json >/dev/null
$BIN fortune5 requirements --format json | tee /tmp/r2.json >/dev/null
cmp /tmp/r1.json /tmp/r2.json && python3 - <<'EOF'
import json; d = json.load(open('/tmp/r1.json'))
assert d["count"] == 40 and len(d["requirements"]) == 40
assert d["requirements"][0]["controlId"] == "F5-AUTH-001"
EOF

# 3b — qualify against the pinned-epoch ALIVE fixture: exit 0, standing ALIVE, alive 40
$BIN fortune5 qualify --subject castle:cli-fixture \
  --evidence-path tests/fixtures/fortune5_evidence_alive.json \
  --now-epoch-ms 1786824000000 --max-evidence-age-ms 300000 --format json \
  | tee /tmp/q1.json >/dev/null
$BIN fortune5 qualify --subject castle:cli-fixture \
  --evidence-path tests/fixtures/fortune5_evidence_alive.json \
  --now-epoch-ms 1786824000000 --max-evidence-age-ms 300000 --format json \
  | tee /tmp/q2.json >/dev/null
cmp /tmp/q1.json /tmp/q2.json && python3 - <<'EOF'
import json; d = json.load(open('/tmp/q1.json'))
assert d["standing"] == "ALIVE" and d["alive"] == 40
assert d["refused"] == 0 and d["unknown"] == 0
assert d["subject"] == "castle:cli-fixture"
assert len(d["controls"]) == 40
assert all(c["standing"] == "ALIVE" for c in d["controls"])
EOF

# 3c — surface verbs answer with exit 0 + valid JSON (shape asserted, no fabricated fields)
for args in "release info" "protocol mcp" "protocol a2a" "inventory components" "inventory goals"; do
  out=$($BIN $args 2>/dev/null); code=$?
  test "$code" -eq 0 && echo "$out" | python3 -c "import json,sys; json.load(sys.stdin)" || exit 1
done
```

Typed-refusal contract (documented in `src/bin/castle/main.rs`, witnessed in Gate 2 by
`cli_negative_evidence`): a refused/block invocation exits **2** with
`{"ok":false,"class":"refused"|"blocked","code":...}` on stdout; non-settled standing
exits **3**. The receipt may cite this contract only if Gate 2 contains the witnessing
test run.

Pass: every block exits 0 and every assertion holds. Any mismatch (including a
determinism-twin `cmp` failure): standing `PARTIAL_ALIVE` at best — non-deterministic
output cannot seal ALIVE.

## Gate 4 — receipt validation (R.standing, fail-closed admission)

Minted only after Gates 0–3 pass on the final integrated HEAD:

```sh
cd /Users/sac/castle
python3 ~/.zcode/dfcm/validate_receipt.py docs/sjira/v26.11.20/victory.receipt.json
echo "exit=$?"                            # 0 = all receipts admitted; 1 = REFUSED
```

The validator independently enforces, and this protocol relies on it refusing:
`ALIVE` with a non-zero replay exit (`admission_vacuous`), `identity.subject_sha`
resolvable as a real commit via `git cat-file` (`R_missing_identity`), and
top-level extension keys namespaced as `provider_ext.<provider>` only. Gate 0/2/3 outputs
are the replay facts the receipt's fields must match — the validator's schema pass plus
the witnessed gates together are the admission; neither alone suffices.

---

## Seal

All gates exit-conformant → standing **ALIVE**, sealed by appending the gate table
(command, exit, key observable) to the campaign receipt document in
`docs/sjira/v26.11.20/` (coordinator-owned), citing SUBJECT_SHA. Any gate refusing →
standing `BLOCKED` / `BUILD_BROKEN` with the `broken_term` from the taxonomy
(`R_missing_identity`, `R_missing_consequence`, `admission_vacuous`, `mu_unlawful`,
`R_missing_replay`) and the preserved command/exit/diagnostic. Refusal is a valid
outcome; a papered-over ALIVE is not.

## Falsifiers (anti-vacuity — run before trusting a green seal)

| # | Mutation | Required observation |
|---|---|---|
| F1 | Revert any single lane's src/tests diff from `<BASE_SHA>..HEAD` | That lane's named `cargo test --test <target>` fails — acceptance is non-vacuous |
| F2 | Flip one target value in `tests/fixtures/fortune5_evidence_alive.json` | Gate 3b exits non-zero / `standing ≠ ALIVE` |
| F3 | Set receipt `identity.subject_sha` to any non-HEAD SHA | Validator exits 1 (`R_missing_identity`) |
| F4 | Add an un-namespaced top-level key to the receipt | Validator exits 1 |

A seal under which any falsifier passes the mutated subject is `admission_vacuous` —
the gate never refused, so it admits nothing.
