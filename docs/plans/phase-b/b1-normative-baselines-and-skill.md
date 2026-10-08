---
id: b.1
title: normative baselines and QA agents
status: planned
branch: feature/phase-b-b1-baselines
worktree: ../docli-worktrees/feature/phase-b-b1-baselines
target: integrate/phase-b
---

# Sprint b.1 — normative baselines and QA agents

## Goal

`docs/requirements.md`, `docs/architecture.md`, and QA agent prompts are the baselines later sprints and reviewers cite.

## Planning vs this sprint

| Piece | Closes on |
|-------|-----------|
| Skill tree + `PROVENANCE.md` (`REQ-DOCLI-NORM-001` bytes) | Planning branch → **`develop`** |
| Req/arch text, QA agents, cross-links | **b.1** on **`integrate/phase-b`** |

b.1 does not re-vendor the skill. After b.0 sync, integrate must contain the skill directory from `develop` (`test -d .claude/skills/creating-ai-clis`).

**Requirement index:** b.1 adds section 11 rows for every `REQ-DOCLI-*` id **named in Phase B sprint Closes sections**, with a “text lands in sprint X” note. Each later sprint (b.7–b.11) updates requirement **body** text and confirms its index row in the **same PR**.

## Closes

- `REQ-DOCLI-NORM-001` (references vendored skill on `develop`; no skill bytes in this PR)
- `ADR-001`, `ADR-002`, `ADR-003` (decision record text in `docs/architecture.md`)
- `ARCH-RULE-001` through `ARCH-RULE-008`
- QA alignment for `req-qa` and `arch-qa` (no new requirement id)

`REQ-DOCLI-CLI-008`–`010` text stays in `requirements.md`; test closure is **b.2**.

## Hard Dependencies

- b.0 — `integrate/phase-b` synced from `develop` (plan + skill already on `develop`).

## Deliverables

- [`docs/requirements.md`](../../requirements.md) — header, `REQ-DOCLI-NORM-001`, section 9 error inventory, `## 11. Requirement index` listing all `REQ-DOCLI-*` ids.
- [`docs/architecture.md`](../../architecture.md) — full decision text for **ADR-001**–**ADR-003** (ADR-003 states machine contract version `"1"`, `ok`, `data`/`error`, kind↔exit per b.2). **ARCH-RULE-001**–**008**: one normative sentence each (e.g. 004: envelope types live in `src/contract.rs`; 007: error-shape changes need REQ row + skill). Error-contract section names `tests/error_contract.rs`.
- [`.claude/agents/req-qa.md`](../../../.claude/agents/req-qa.md) — read `docs/requirements.md` first; `REQ-DOCLI-*` in `source_refs`; missing `error_contract` coverage is **Blocking** for CLI-008–010.
- [`.claude/agents/arch-qa.md`](../../../.claude/agents/arch-qa.md) — read `docs/architecture.md` first; `ARCH-RULE-*` before generic `RULE-*`; emit `rule`, `adr`, `evidence_refs`.
- [`.claude/skills/codex-orchestration/SKILL.md`](../../../.claude/skills/codex-orchestration/SKILL.md) — preconditions cite `docs/plans/project-plan.md` and `docs/plans/phase-b/README.md`.
- Phase A ADR stubs banner → canonical `docs/architecture.md`; [`project-plan.md`](../project-plan.md) and [`README.md`](../../../README.md) link baselines and state default `--html` → `site/cli` per **REQ-DOCLI-HTML-006** / **REQ-DOCLI-CLI-005** (not stdout).

## Out of Scope

- Vendoring or diffing `creating-ai-clis` (planning branch only)
- `tests/error_contract.rs` (b.2)
- New CLI behavior, renderer bytes, crates.io metadata (b.11)

## Acceptance Criteria

- Section 11 index and ADR/ARCH-RULE tables match deliverables above
- `rg` finds a unique normative phrase per **ARCH-RULE-00N** row (not id-only stubs)
- ADR-003 body includes the machine-contract version and envelope field names
- `req-qa.md` / `arch-qa.md` / codex-orchestration preconditions match deliverables
- `develop` already contains the skill tree; this sprint does not modify `.claude/skills/creating-ai-clis/` except doc links

## Required Validation

- Phase B host gate — [README.md](README.md)
- `test -f docs/requirements.md docs/architecture.md .claude/agents/req-qa.md .claude/agents/arch-qa.md`
- `rg -n "## 11\\. Requirement index" docs/requirements.md`
- `rg -n "ARCH-RULE-008" docs/architecture.md`
- `rg -n "docs/plans/phase-b/README.md" .claude/skills/codex-orchestration/SKILL.md`
