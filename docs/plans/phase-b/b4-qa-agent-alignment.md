---
id: b.4
title: req-qa and arch-qa baseline alignment
status: planned
branch: feature/phase-b-b4-qa-agents
worktree: ../docli-worktrees/feature/phase-b-b4-qa-agents
target: integrate/phase-b
---

# Sprint b.4 — req-qa and arch-qa baseline alignment

## Goal

`req-qa` and `arch-qa` validate docli from `docs/requirements.md` and `docs/architecture.md`.

## Closes

No new requirement id. Agent instructions must enforce:

- `REQ-DOCLI-NORM-001`
- `REQ-DOCLI-CLI-008`, `REQ-DOCLI-CLI-009`, `REQ-DOCLI-CLI-010`
- `ADR-001`, `ADR-002`, `ADR-003`
- `ARCH-RULE-001` through `ARCH-RULE-008`

## Hard Dependencies

- b.1 merged to `integrate/phase-b` (baselines and `ARCH-RULE-*` table)
- b.2 merged to `integrate/phase-b` (error inventory and `tests/error_contract.rs`)

## Deliverables

- [`.claude/agents/req-qa.md`](../../../.claude/agents/req-qa.md)
  - Mandatory read order: `docs/requirements.md` (section 11 index), `docs/architecture.md`, `docs/plans/project-plan.md`.
  - Findings that assert requirement drift cite a `REQ-DOCLI-*` id in `source_refs`.
  - A `DOCLI.*` code in requirements section 9 without a matching test in `tests/error_contract.rs` is **Blocking** under `REQ-DOCLI-CLI-008`–`010`.
- [`.claude/agents/arch-qa.md`](../../../.claude/agents/arch-qa.md)
  - Mandatory read: `docs/architecture.md` first (`ADR-001`–`ADR-003`, then `ARCH-RULE-001`–`008`).
  - Finding fields: `rule` (`ARCH-RULE-00N`), `adr` (`ADR-00N`), `evidence_refs` (includes a `docs/architecture.md` reference).
  - Apply `ARCH-RULE-*` before generic `RULE-001`–`RULE-013`. Those generic rules are `not-applicable` for docli unless the sprint doc names one.
- [`.claude/skills/codex-orchestration/SKILL.md`](../../../.claude/skills/codex-orchestration/SKILL.md) preconditions cite `docs/plans/project-plan.md` and, for Phase B, `docs/plans/phase-b/README.md`.

`review_mode` stays the agent's existing set (`sprint_review`, `round_limit`, `phase_end`, `integration_review`). Phase B sprint review sets `authoritative_sprint_doc` to the sprint file (`docs/plans/phase-b/b0-integrate-branch.md` through `b6-publish-readiness.md`). This sprint does not add a `plan` mode.

## Out of Scope

- `.cursor/agents/` copies
- GitHub Actions that run these agents
- A second error checklist under `.claude/skills/reviewing-ai-clis/`
- Editing `ARCH-RULE-*` text (that text is b.1)

## Acceptance Criteria

- `req-qa.md` tells the agent to read `docs/requirements.md` before other plan docs, to put `REQ-DOCLI-*` ids in `source_refs`, and to treat missing `tests/error_contract.rs` coverage as Blocking for `REQ-DOCLI-CLI-008`, `REQ-DOCLI-CLI-009`, and `REQ-DOCLI-CLI-010`
- `arch-qa.md` tells the agent to apply `ARCH-RULE-001`–`008` from `docs/architecture.md` before `RULE-001`–`013`, to mark those generic rules `not-applicable` unless a sprint doc names one, and to emit `rule`, `adr`, and `evidence_refs`
- `codex-orchestration` preconditions cite `docs/plans/project-plan.md` and `docs/plans/phase-b/README.md`
- `docs/requirements.md` and `docs/architecture.md` are present for those reads
- Sprint QA input uses `review_mode` `sprint_review` and `authoritative_sprint_doc` set to a Phase B sprint doc

## Required Validation

```text
cargo test
cargo test --test error_contract
cargo test --test cli_contract
cargo test --test cargo_docli
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
git diff --check
```

```text
test -f docs/requirements.md
test -f docs/architecture.md
test -f .claude/agents/req-qa.md
test -f .claude/agents/arch-qa.md
rg -n "docs/requirements.md" .claude/agents/req-qa.md
rg -n "docs/plans/project-plan.md" .claude/agents/req-qa.md
rg -n "source_refs" .claude/agents/req-qa.md
rg -n "tests/error_contract.rs" .claude/agents/req-qa.md
rg -n "REQ-DOCLI-CLI-008" .claude/agents/req-qa.md
rg -n "REQ-DOCLI-CLI-009" .claude/agents/req-qa.md
rg -n "REQ-DOCLI-CLI-010" .claude/agents/req-qa.md
rg -n "Blocking" .claude/agents/req-qa.md
rg -n "docs/architecture.md" .claude/agents/arch-qa.md
rg -n "ARCH-RULE-001" .claude/agents/arch-qa.md
rg -n "ARCH-RULE-008" .claude/agents/arch-qa.md
rg -n "evidence_refs" .claude/agents/arch-qa.md
rg -n "not-applicable" .claude/agents/arch-qa.md
rg -n "RULE-001" .claude/agents/arch-qa.md
rg -n "docs/plans/project-plan.md" .claude/skills/codex-orchestration/SKILL.md
rg -n "docs/plans/phase-b/README.md" .claude/skills/codex-orchestration/SKILL.md
if rg -n "docs/project-plan\\.md" .claude/skills/codex-orchestration/SKILL.md; then
  echo "precondition still cites docs/project-plan.md" >&2
  exit 1
fi
rg -n "sprint_review" .claude/agents/arch-qa.md
```
