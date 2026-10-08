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

- Phase-end and sprint QA **mechanically** validates against `docs/requirements.md`
  and `docs/architecture.md`, not ad hoc prose or foreign-repo rules.

## Hard Dependencies

- b.1 — baselines and ARCH-RULE table
- b.2 — error inventory and `REQ-DOCLI-CLI-008`–`010`

## Deliverables

- [`.claude/agents/req-qa.md`](../../../.claude/agents/req-qa.md)
  - Mandatory read: requirements (§11 index), architecture (conflicts)
  - Blocking: missing `tests/error_contract.rs` coverage for any inventory code
  - Findings must cite `REQ-DOCLI-*` in `source_refs`
- [`.claude/agents/arch-qa.md`](../../../.claude/agents/arch-qa.md)
  - Mandatory read: `architecture.md` ADRs + `ARCH-RULE-001`–`008` **first**
  - Finding JSON includes `adr`, `rule` (`ARCH-RULE-00N`), `evidence_refs`
  - Generic RULE-001–013 (ATM/sc-observability) default **not-applicable** for docli unless sprint doc scopes them
- [`.claude/skills/codex-orchestration/SKILL.md`](../../../.claude/skills/codex-orchestration/SKILL.md) — Preconditions list `docs/plans/project-plan.md` Phase B when executing Phase B
- Optional cross-ref: [`.claude/skills/reviewing-ai-clis/references/error-review.md`](../../../.claude/skills/reviewing-ai-clis/references/error-review.md) linked from b.2 sprint doc (no duplicate checklist)

## Out of Scope

- Changing Cursor-only agents under `.cursor/agents/` unless repo policy requires parity
- Automating QA via GitHub Actions (human/agent QA only)

## Acceptance Criteria

- `grep ARCH-RULE docs/architecture.md` returns rules 001–008
- `req-qa.md` mentions `tests/error_contract.rs` and `REQ-DOCLI-CLI-008`
- `arch-qa.md` instructs docli-specific rules before Wyvern/ATM rules
- Plan QA (`review_mode: plan`) can run on Phase B sprint docs with `sprint_doc` = any `b*.md`

## Required Validation

- Phase B CI gate in [README.md](README.md)
- Dry-run: read agent files and confirm baseline paths exist on disk
