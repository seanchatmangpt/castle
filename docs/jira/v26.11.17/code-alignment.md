# code-alignment — castle v26.11.17

Standing: PASS

## Identity
- Path: `/Users/sac/castle`
- Branch at generation: main
- HEAD: 3ffa1bbd66f6d1d871da046a94a32daf10ebf6b1
- Tracked files: 361
- Tracked docs markdown: 37

## Build manifests (tracked at the commit above)
- `Cargo.toml`
- `package.json`
- `Dockerfile`
- `ggen.toml`
- `CLAUDE.md`
- `AGENTS.md`
- `README.md`

## Top-level layout (tracked file counts)
- `.`: 18
- `.github`: 8
- `.qme`: 1
- `.sa2a`: 1
- `.well-known`: 1
- `configs`: 6
- `crates`: 7
- `dd-ui`: 2
- `docs`: 48
- `fixtures`: 28
- `ontology`: 7
- `output`: 6
- `paas`: 12
- `policy`: 1
- `rdf`: 2
- `release`: 2
- `schemas`: 18
- `scripts`: 2
- `src`: 106
- `test`: 6
- `tests`: 79

## Prior doc waves
- `docs/jira/v26.8.18`
- `docs/jira/v26.9.19`
- `docs/jira/v26.11.16`

## Falsifiers
- Any tracked path above absent at HEAD, or a count differing from `git ls-files`.
- HEAD sha not equal to or an ancestor of the live HEAD.

## Acceptance
```bash
python3 ~/zcode-cli/scripts/docs_v261117.py validate /Users/sac/castle
```
Expected: exit 0 and `PASS`.

## See Also
- `~/zcode-cli/scripts/docs_v261117.py`
