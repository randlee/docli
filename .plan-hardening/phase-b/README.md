# Phase B plan hardening

**Run `/plan-hardening` on the planning branch**, not on `develop` or `integrate/phase-b`.

## Worktree

Check out the planning branch (e.g. `feature/phase-b-plans-and-skill`) in a dedicated
worktree. All arch-ctm plan edits from steps 1–5 commit to **that branch**.

## Vars

Copy [`vars.example.json`](vars.example.json) to `vars.json` (local; do not commit secrets).
Set:

- `worktree_path` — absolute path to the planning worktree
- `branch` — planning branch name
- `pr_target` — `develop` (merge target after hardening PASS, not where hardening runs)
- `source_of_truth` / `references` — paths under `docs/plans/phase-b/`

## Round table

Record rounds in `rounds.md` in this directory (see
`.claude/skills/plan-hardening/examples/plan-hardening-rounds.example.md`).

## After step 6 PASS

1. Open or update PR: planning branch → `develop`.
2. Merge when the plan is **correct** (not because hardening is a one-time lock).

If the plan was already on `develop` but is **wrong**, edit on the planning branch again;
re-harden when needed, merge to `develop`, and re-run **b.0** if integrate is active.

3. Run sprint **b.0** (`develop` → `integrate/phase-b`) when execution should see the latest plan.
4. Operator **go** starts or continues `/sc-gh-stack` against the **current** sprint docs.
