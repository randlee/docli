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

Run after **`/plan-hardening`** PASS on `plan/phase-d` and the plan is merged to **`develop`**. Repeat **`develop` → `integrate/phase-d`** whenever the plan changes during execution.

## Closes

No requirement id.

## Hard Dependencies

- Planning branch `plan/phase-d` merged to `develop`. No earlier Phase D sprint.

## Deliverables

- Branch `integrate/phase-d` exists; tip is **`develop`** after merge (or fast-forward).
- Worktree `../docli-worktrees/integrate/phase-d` on `integrate/phase-d` when the team uses worktrees.

## Out of Scope

- Feature commits on `integrate/phase-d` (stack lands only)
- Host gate, `dotnet test`, `cargo test` (no code change)
- Any `.cs`, `.csproj`, or `.sln` file
- Phase D non-closures in [README.md](README.md): a .NET HTML or Markdown renderer, a template-engine port, Go or Python adapters, an MCP wrapper, `REQ-DOCLI-PRODUCT-004`, the renderer sentence of `REQ-DOCLI-NET-001`, `REQ-DOCLI-GEN-003`

## Acceptance Criteria

- `git merge-base --is-ancestor develop integrate/phase-d`
- After sync, `integrate/phase-d` includes `docs/plans/phase-d/` from `develop`

## Required Validation

```text
git fetch origin develop integrate/phase-d
git checkout integrate/phase-d
git merge origin/develop
git push origin integrate/phase-d
git merge-base --is-ancestor origin/develop origin/integrate/phase-d
git worktree list
```

If the branch or worktree already exists, merge `develop`; do not recreate or reset without operator approval.
