# Project plan

Product requirements: [docs/requirements.md](../requirements.md).  
Architecture (ADRs, `ARCH-RULE-*`): [docs/architecture.md](../architecture.md).

## Standard phase Git workflow (all repos)

This repository uses the **same process on every phase** (and the same pattern is used
across sibling repos):

1. **Planning branch** off `develop` — draft and **`/plan-hardening`** on that branch only.
2. **Publish** — merge planning branch → `develop` when the plan is **correct** (again after any fix).
3. **Sync** — `develop` → `integrate/phase-<N>` (bootstrap sprint x.0).
4. **Execute** — operator **go**; **`/sc-gh-stack`** with trunk `integrate/phase-<N>`.
5. **Close** — phase-end QA; `integrate/phase-<N>` → `develop`.

Plan authority is always `docs/plans/phase-<N>/`, not “whatever was merged last.”
Operator guide: [`.plan-hardening/README.md`](../../.plan-hardening/README.md).

| Phase | Integration branch | Plan authority | Status |
|-------|-------------------|----------------|--------|
| A | `integrate/phase-a` | [phase-a/README.md](phase-a/README.md) | Complete (merged to `develop`) |
| B | `integrate/phase-b` | [phase-b/README.md](phase-b/README.md) | Planning branch + harden; execution on integrate after **go** |

## Branch model (every phase)

Two Git tracks, **one phase plan** (`docs/plans/phase-<N>/`):

| Track | Branch | Merge target | Contents |
|-------|--------|--------------|----------|
| **Planning** | `feature/phase-<N>-plan*` (or `plan/phase-<N>`) cut from **`develop`** | **`develop`** | Sprint markdown, [`project-plan.md`](project-plan.md), normative **skill** trees (e.g. creating-ai-clis), template **author/agent** docs. **No** product code, tests, or scripts for the phase. |
| **Execution** | Stacked layers via **`/sc-gh-stack`** | **`integrate/phase-<N>`** (trunk) | Implementation sprints: Rust, tests, scripts, requirements/architecture **as built**. Land with `gh stack merge <stack#> --yes --merge`. |

**Order:**

1. Draft on the **planning branch** (worktree on that branch).
2. **`/plan-hardening` on the planning branch** — vars `branch` / `worktree_path` point at the planning branch; `pr_target` is **`develop`**. Plan edits from hardening land on the planning branch until step 6 PASS.
3. Merge the planning PR to **`develop`** (approved plan only).
4. **b.0:** merge **`develop` → `integrate/phase-<N>`**.
5. Operator **go** → **`/sc-gh-stack`** on the integration trunk (no stack before **go**).

**Phase end:** QA on the integration trunk → one PR **`integrate/phase-<N>` → `develop`** (merge commit), same pattern as Phase A.

Planning and execution **do not run as two phases**; they are two **merge targets** so docs stay readable on `develop` while code lands atomically on the stack.

Phase B is the **only** active plan after Phase A. HTML templating is **b.7–b.10** inside Phase B, not a separate phase ([phase-c/README.md](phase-c/README.md) is a redirect only).

Phase B follows [codex-orchestration](../../.claude/skills/codex-orchestration/SKILL.md):
sprint docs are authoritative for `req-qa` deliverable enumeration.

**Plan authority:** [`docs/plans/phase-b/`](phase-b/README.md) sprint docs must be **correct**.
Merging to **`develop`** is how the team shares the plan; it does **not** freeze it. If the
plan is wrong, change it on the **planning branch** (re-run **`/plan-hardening`** when the
edit is material), merge to **`develop`** again, and **`develop` → `integrate/phase-<N>`** if
execution already started.

**`/plan-hardening`** runs on the **planning branch**, not on `develop` or `integrate/phase-b`.
**Go** means execute against the **current correct plan**, not “plan was merged once.”
