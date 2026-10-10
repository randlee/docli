---
id: d.0
title: integrate/phase-d branch bootstrap
status: planned
branch: integrate/phase-d
worktree: ../docli-worktrees/integrate/phase-d
target: develop
---

# Sprint d.0 — integrate/phase-d branch bootstrap

## Goal

`integrate/phase-d` tracks **`develop`** after this plan is merged. No product commits on this step.

Run after **`/plan-hardening`** PASS on `plan/phase-d` and the plan is merged to **`develop`**. Each later re-sync of **`develop` → `integrate/phase-d`** reuses the Required Validation block below. A re-sync does not add a sprint doc.

## Closes

No requirement id.

## Hard Dependencies

- Planning branch `plan/phase-d` merged to `develop`. No earlier Phase D sprint.

## Deliverables

- Branch `integrate/phase-d` exists; tip contains **`origin/develop`** after the create or merge path below.
- Worktree `../docli-worktrees/integrate/phase-d` on `integrate/phase-d` when the team uses worktrees.

## Out of Scope

- Phase D non-closures: see [README.md](README.md)
- Feature commits on `integrate/phase-d` (stack lands only)
- Host gate, `dotnet test`, `cargo test` (no code change)
- Any `.cs`, `.csproj`, or `.sln` file

## Acceptance Criteria

- `git merge-base --is-ancestor origin/develop origin/integrate/phase-d`
- `git ls-tree -r origin/integrate/phase-d --name-only` includes `docs/plans/phase-d/README.md`
- When worktrees are used, `git worktree list` contains a row whose path ends with `docli-worktrees/integrate/phase-d` and whose branch is `integrate/phase-d`

## Required Validation

First time, when `origin/integrate/phase-d` does not exist:

```text
git fetch origin develop
git checkout -b integrate/phase-d origin/develop
git push -u origin integrate/phase-d
git worktree add ../docli-worktrees/integrate/phase-d integrate/phase-d
git merge-base --is-ancestor origin/develop origin/integrate/phase-d
git ls-tree -r origin/integrate/phase-d --name-only | rg '^docs/plans/phase-d/README.md$'
git worktree list
```

When the branch already exists, merge. Do not recreate or reset without operator approval:

```text
git fetch origin develop integrate/phase-d
git checkout integrate/phase-d
git merge origin/develop
git push origin integrate/phase-d
git merge-base --is-ancestor origin/develop origin/integrate/phase-d
git ls-tree -r origin/integrate/phase-d --name-only | rg '^docs/plans/phase-d/README.md$'
git worktree list
```

If the worktree row already exists, skip `git worktree add`.
