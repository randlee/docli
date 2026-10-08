---
id: b.1
title: normative baselines and full creating-ai-clis skill
status: planned
branch: feature/phase-b-b1-baselines
worktree: ../docli-worktrees/feature/phase-b-b1-baselines
target: integrate/phase-b
---

# Sprint b.1 — normative baselines and full creating-ai-clis skill

## Goal

- `docs/requirements.md` and `docs/architecture.md` are the **only** authoritative
  baselines for `req-qa` and `arch-qa` on docli.
- The mandatory **creating-ai-clis** skill is vendored **in full** (all references
  and templates), with provenance for upstream sync.

## Hard Dependencies

- Phase A merged to `develop`
- Upstream: `synaptic-canvas/packages/sc-ai-cli` `skills/creating-ai-clis/` (package v0.12.0)

## Deliverables

- [`docs/requirements.md`](../../requirements.md)
  - Header: authoritative baseline, hard `REQ-DOCLI-*` semantics, QA validation section
  - `REQ-DOCLI-NORM-001` — CLI must conform to vendored creating-ai-clis
  - Section 11 requirement index with ADR cross-links
- [`docs/architecture.md`](../../architecture.md)
  - Canonical **ADR-001**, **ADR-002**, **ADR-003** (full decision text, not links only)
  - **ARCH-RULE-001**–**008** table for `arch-qa`
  - Error contract verification pointer to `tests/error_contract.rs`
- [`.claude/skills/creating-ai-clis/`](../../../.claude/skills/creating-ai-clis/) — **complete** upstream tree:
  - `SKILL.md`, all `references/*.md`, all `assets/templates/**`
  - `PROVENANCE.md` — upstream path, version, wholesale re-import policy
- [`docs/plans/phase-a/adr-001-ops-boundary.md`](../phase-a/adr-001-ops-boundary.md) and
  [`adr-002-search-index.md`](../phase-a/adr-002-search-index.md) — banner pointing to canonical `architecture.md`
- [`docs/plans/project-plan.md`](../project-plan.md) — Phase B row and link to this README
- [`README.md`](../../../README.md) — links to requirements, architecture, skill path

## Out of Scope

- New CLI commands or error codes
- crates.io publish (b.6)
- Changing renderer HTML bytes without a separate requirement

## Acceptance Criteria

- `req-qa` agent doc lists `docs/requirements.md` first; findings cite `REQ-DOCLI-*` ids
- `arch-qa` agent doc lists `docs/architecture.md` ADRs and `ARCH-RULE-*` first
- Vendored skill file count matches upstream `creating-ai-clis` (25 files at v0.12.0)
- `PROVENANCE.md` states upstream package path and version
- No sprint plan under `docs/plans/phase-a/` contradicts canonical ADRs in `architecture.md`

## Required Validation

- Phase B CI gate in [README.md](README.md) except `error_contract` (lands in b.2)
- `git diff --check`
- Manual: `diff -rq` vendored skill vs upstream (or documented hash in PROVENANCE)
