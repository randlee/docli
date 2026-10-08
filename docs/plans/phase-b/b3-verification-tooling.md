---
id: b.3
title: agent-safe verification tooling
status: planned
branch: feature/phase-b-b3-verify
worktree: ../docli-worktrees/feature/phase-b-b3-verify
target: integrate/phase-b
---

# Sprint b.3 — agent-safe verification tooling

## Goal

- Maintainer and agent workflows that invoke docli **must** surface actionable
  envelope errors (creating-ai-clis), not silent success or bash-only failures.

## Hard Dependencies

- b.2 — `tests/common/envelope.rs`, error inventory
- b.1 — requirements QA section

## Deliverables

- [`scripts/docli-envelope.sh`](../../../scripts/docli-envelope.sh)
  - `docli_require_envelope_ok` — run command with `--json`, assert `ok: true`
  - on failure: print `code`, `message`, `suggested_action`, `details.cause`
- [`scripts/verify-candidate-repos.sh`](../../../scripts/verify-candidate-repos.sh)
  - optional `gen-fixtures` (path dep today)
  - `generate` + `show` in docli + atm-core + sc-compose + sc-observability smoke
  - uses `docli-envelope.sh`; hints on missing checkout or gen-fixtures failure
- [`tests/live_candidate_repos.rs`](../../../tests/live_candidate_repos.rs) — opt-in via `DOCLI_LIVE_CANDIDATE_REPOS=1`; same envelope assertions as b.2 helpers
- [`src/bin/gen-fixtures.rs`](../../../src/bin/gen-fixtures.rs) — `hint:` line on failure (clean checkout, toolchain)
- [`README.md`](../../../README.md) — Candidate repo verification + envelope behavior

## Out of Scope

- Running live candidate repos in default CI
- Consumer repo PRs
- crates.io install in script (b.6 updates scripts to `cargo install docli`)

## Acceptance Criteria

- `DOCLI_SKIP_GEN_FIXTURES=1 ./scripts/verify-candidate-repos.sh` exits 0 with default sibling paths (when checkouts exist)
- Forced failure (`docli generate --input /no/such.json --json`) prints `DOCLI.INPUT_NOT_FOUND` and `suggested_action` when run through `docli-envelope.sh`
- `cargo test --test live_candidate_repos` skips without env; passes with env + paths set
- Script never redirects docli `--json` stdout to `/dev/null` on failure paths

## Required Validation

- Phase B CI gate in [README.md](README.md)
- `DOCLI_SKIP_GEN_FIXTURES=1 ./scripts/verify-candidate-repos.sh`
- `cargo test --test live_candidate_repos`
