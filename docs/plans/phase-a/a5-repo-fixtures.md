---
id: a.5
title: test-repo command models
status: planned
branch: feature/phase-a-a5-fixtures
worktree: ../docli-worktrees/feature/phase-a-a5-fixtures
target: develop
---

# Sprint a.5 — test-repo command models

## Goal

- Checked-in command models from atm-core, sc-compose, and sc-observability render through `generate --html site/cli`.

## Hard Dependencies

- a.1 `generate` and `show`
- a.4 `from_clap` for producing the snapshots
- Read-only checkouts of those three repos on `develop` under `~/Documents/github/`

## Deliverables

- `fixtures/repos/atm-core.json`
- `fixtures/repos/sc-compose.json`
- `fixtures/repos/sc-observability.json`
- `tests/repo_fixtures.rs` — each file generates `site/cli/index.html` in a temp directory and `show` reports the same sha256

Each JSON file is the neutral model from `docli::from_clap` on that repo's command type, captured from `develop` and committed here. docli CI does not compile those repos.

## Out of Scope

- Pull requests or `site/cli/` commits in atm-core, sc-compose, or sc-observability
- Worktrees of those repos as a Phase A deliverable
- Re-rendering when those CLIs change, beyond replacing the three JSON files in a later change

## Acceptance Criteria

- Each of the three JSON files parses as `CliModel` and has a non-empty `name`
- `docli generate --input fixtures/repos/<repo>.json --html <tmp>/site/cli --json` exits 0 and writes `<tmp>/site/cli/index.html`
- `docli show --html <tmp>/site/cli --json` exits 0 and the artifact sha256 equals the `generate` output entry
- The `generate` invocation in the test passes `--html` explicitly

## Required Validation

- Phase A CI gate in [README.md](README.md)
- `cargo test --test repo_fixtures`
