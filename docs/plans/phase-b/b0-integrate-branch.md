---
id: b.0
title: integrate/phase-b branch bootstrap
status: planned
branch: integrate/phase-b
worktree: ../docli-worktrees/integrate/phase-b
target: develop
---

# Sprint b.0 — integrate/phase-b branch bootstrap

## Goal

`integrate/phase-b` exists from current `develop`, with a worktree later sprints merge into.

## Closes

No requirement id. This sprint does not change product behavior.

## Hard Dependencies

- Phase A is merged to `develop`.

## Deliverables

- Git branch `integrate/phase-b` created from `develop` at the start of Phase B. Later sprints merge feature branches into this branch. The branch is an integration branch: sprint cleanup removes worktrees and does not delete this branch.
- Worktree `../docli-worktrees/integrate/phase-b` checked out on `integrate/phase-b` (worktree directory name matches the branch name).
- No commits on this branch except the branch creation itself. Feature work lands from b.1 onward.

## Out of Scope

- Requirement, architecture, skill, test, script, template, or publish edits (b.1–b.5, b.7–b.11)
- Feature branches `feature/phase-b-b1-baselines` through `feature/phase-b-b6-publish` (each later sprint creates its own branch from `integrate/phase-b`)
- Deleting `develop` or rewriting its history

## Acceptance Criteria

- `integrate/phase-b` exists and `develop` is an ancestor of it
- `git worktree list` shows `../docli-worktrees/integrate/phase-b` on `integrate/phase-b`
- The branch tip matches `develop` when this sprint finishes
- The host gate passes on that worktree

## Required Validation

Run from the docli repo, then from the new worktree for the host gate:

```text
git fetch origin develop
git rev-parse --verify develop
git branch integrate/phase-b develop
git worktree add ../docli-worktrees/integrate/phase-b integrate/phase-b
git merge-base --is-ancestor develop integrate/phase-b
git worktree list
```

Host gate, from `../docli-worktrees/integrate/phase-b`:

```text
cargo test
cargo test --test error_contract
cargo test --test cli_contract
cargo test --test cargo_docli
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
git diff --check
```

If `integrate/phase-b` or the worktree already exists, do not recreate or reset it. `git merge-base --is-ancestor develop integrate/phase-b` and `git worktree list` are the checks.
