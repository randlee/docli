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

`integrate/phase-b` tracks **`develop`** (approved plan). No product commits on this step.

Run after **`/plan-hardening`** PASS on the **planning branch** and the plan is merged to **`develop`**. Repeat **`develop` → `integrate/phase-b`** whenever the plan changes during execution.

## Closes

No requirement id.

## Deliverables

- Branch `integrate/phase-b` exists; tip is **`develop`** after merge (or fast-forward).
- Worktree `../docli-worktrees/integrate/phase-b` on `integrate/phase-b` when the team uses worktrees.

## Out of Scope

- Feature commits on `integrate/phase-b` (stack lands only)
- Host gate / `cargo test` (no code change)

## Acceptance Criteria

- `git merge-base --is-ancestor develop integrate/phase-b`
- After sync, `integrate/phase-b` includes sprint docs and skill from `develop`

## Required Validation

```text
git fetch origin develop integrate/phase-b
git checkout integrate/phase-b
git merge origin/develop
git push origin integrate/phase-b
git merge-base --is-ancestor origin/develop origin/integrate/phase-b
git worktree list
```

If the branch or worktree already exists, merge `develop`; do not recreate or reset without operator approval.
