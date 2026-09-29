# castle: push or delete local-only branch `chore/commit-pending-changes`

- Standing: OPEN
- Created: 2026-09-19 (v26.9.19 gh survey wave)
- Source: local branch `chore/commit-pending-changes` has 1 commit(s) not on `origin/main`, no upstream
- Evidence: `git rev-list --count origin/main..chore/commit-pending-changes` = 1

## Work to complete
- Push (`git push -u origin chore/commit-pending-changes`) if the work matters; otherwise delete the branch after confirming the commits are obsolete.

## Acceptance
- Branch pushed and visible on GitHub, or deleted locally with commits confirmed recoverable-or-unwanted.

## History
- 2026-09-19 | OPEN | survey found local-only branch | chore/commit-pending-changes (1 commits) | decision pending
