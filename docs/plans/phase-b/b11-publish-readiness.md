---
id: b.11
title: crates.io publish readiness
status: planned
branch: feature/phase-b-b11-publish
worktree: ../docli-worktrees/feature/phase-b-b11-publish
target: integrate/phase-b
---

# Sprint b.11 — crates.io publish readiness

## Goal

- Close the **registry install** slice of `REQ-DOCLI-PRODUCT-001` and switch
  maintainer scripts from path deps to **`cargo install docli`** where appropriate.
- First publish includes **bundled HTML template packs** (b.7–b.10).

## Hard Dependencies

- b.1–b.10 merged on `integrate/phase-b`

Phase-end **quality-mgr** runs **after** this sprint merges, then **`integrate/phase-b` → `develop`** (see phase-b README).

## Deliverables

- [`Cargo.toml`](../../../Cargo.toml) — crates.io metadata review (description, license, repository, readme, keywords); ship template assets in published crate
- [`CHANGELOG.md`](../../../CHANGELOG.md) — first release notes (Phase A + B: errors, verify tooling, templates, preview)
- Version bump policy documented in CHANGELOG (semver 0.x until stable API declared)
- [`scripts/verify-candidate-repos.sh`](../../../scripts/verify-candidate-repos.sh) — document `DOCLI_BIN` or PATH to installed `docli` for post-publish; default remains `cargo build --release` until publish completes
- [`README.md`](../../../README.md) — install from crates.io after publish; link to `docli templates`, `AGENT-PREVIEW.md`, error contract tests
- Release checklist (in CHANGELOG or `docs/release-first-crates-io.md`):
  - `cargo publish --dry-run`
  - `cargo install docli --version …` smoke: `docli templates list --json`, one preview generate, `cargo test --test error_contract`
- **Publish execution** is explicit human/team-lead step after merge to `develop`; sprint closes when repo is publish-ready, not necessarily when crates.io has the crate

## Out of Scope

- GitHub Release assets beyond source tag (unless team-lead adds later)
- Homebrew / npm distribution

## Acceptance Criteria

- `cargo publish --dry-run` succeeds from repo root; packaged crate includes bundled templates
- README documents `cargo install docli` as primary install after first publish
- Verify script documents both pre-publish (built binary) and post-publish (`docli` on PATH) modes
- All Phase B CI gate commands pass on integration branch

## Required Validation

- Phase B host gate in [README.md](README.md)
- `cargo publish --dry-run`
- `cargo package --list` includes `README.md`, `LICENSE`, template packs, and fixtures needed for tests (document what ships vs dev-only)
