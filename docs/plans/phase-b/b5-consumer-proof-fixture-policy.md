---
id: b.5
title: consumer proof and fixture policy
status: planned
branch: feature/phase-b-b5-consumers
worktree: ../docli-worktrees/feature/phase-b-b5-consumers
target: integrate/phase-b
---

# Sprint b.5 — consumer proof and fixture policy

## Goal

- Prove docli works on **real** atm-core and sc-compose trees before first
  publish, with an explicit **fixture policy** so CI stability and live capture
  drift are not confused.

## Hard Dependencies

- b.3 — verify script and envelope helpers
- Phase A — `fixtures/repos/*.json`, `gen-fixtures`, `tests/repo_fixtures.rs`

## Deliverables

- [`docs/requirements.md`](../../requirements.md) — short **Fixture policy** subsection:
  - CI uses **committed** `fixtures/repos/*.json` only
  - Live `gen-fixtures` may differ from committed (e.g. empty `help` when upstream lacks doc comments); refreshing committed fixtures is a ** deliberate PR** with review, not a silent verify side effect
  - No `fixtures/repos/sc-observability.json`
- [`docs/plans/phase-b/README.md`](README.md) — pointer to policy
- Evidence recorded in sprint closeout (PR body or `.cursor/phase-b-orchestration.json` if used):
  - `./scripts/verify-candidate-repos.sh` log (or `DOCLI_SKIP_GEN_FIXTURES=1` when regen out of scope)
  - `site/cli/index.html` generated in docli repo (dogfood)
- Confirm **no** open PRs to atm-core, sc-compose, sc-observability for docli output

## Out of Scope

- Committing `site/cli/` in consumer repos
- sc-observability clap model
- Pinning consumer repos to a docli git SHA in their manifests (post–crates.io)

## Acceptance Criteria

- `cargo test --test repo_fixtures` passes on integration branch
- `DOCLI_SKIP_GEN_FIXTURES=1 ./scripts/verify-candidate-repos.sh` passes
- Requirements document states committed-vs-live fixture rule in plain language
- atm-core and sc-compose: `docli generate` + `show` succeed using **committed** JSON (envelope `ok: true`)

## Required Validation

- Phase B CI gate in [README.md](README.md)
- `cargo test --test repo_fixtures`
- `DOCLI_SKIP_GEN_FIXTURES=1 ./scripts/verify-candidate-repos.sh`
