# castle: land or delete remote branch `feat/empire-reconstitution-admission`

- Standing: OPEN
- Created: 2026-09-19 (v26.9.19 gh survey wave)
- Source: remote branch `feat/empire-reconstitution-admission` — not merged into `main`, no open PR
- Evidence: `git branch -r --no-merged origin/main` lists it; absent from `gh pr list` heads

## Work to complete
- Decide: open a PR (`gh pr create -R seanchatmangpt/castle --head feat/empire-reconstitution-admission`) or delete (`git push origin --delete feat/empire-reconstitution-admission`).
- If superseded, delete; otherwise land through review.

## Acceptance
- After `git fetch --prune`, `git branch -r --no-merged origin/main` no longer lists `feat/empire-reconstitution-admission`.

## History
- 2026-09-19 | OPEN | survey found PR-less unmerged branch | feat/empire-reconstitution-admission | decision pending
