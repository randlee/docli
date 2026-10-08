---
id: b.6
title: crates.io publish readiness
status: planned
branch: feature/phase-b-b6-publish
worktree: ../docli-worktrees/feature/phase-b-b6-publish
target: integrate/phase-b
---

# Sprint b.6 — crates.io publish readiness

## Goal

- Close the **registry install** slice of `REQ-DOCLI-PRODUCT-001` and switch
  maintainer scripts from path deps to **`cargo install docli`** where appropriate.

## Hard Dependencies

- b.1–b.5 merged on `integrate/phase-b`
- Phase B phase-end QA PASS on integration branch

## Deliverables

- [`Cargo.toml`](../../../Cargo.toml) — crates.io metadata review (description, license, repository, readme, keywords)
- [`CHANGELOG.md`](../../../CHANGELOG.md) — first release notes (Phase A + B highlights)
- Version bump policy documented in CHANGELOG (semver 0.x until stable API declared)
- [`scripts/verify-candidate-repos.sh`](../../../scripts/verify-candidate-repos.sh) — document `DOCLI_BIN` or PATH to installed `docli` for post-publish; default remains `cargo build --release` until publish completes
- [`README.md`](../../../README.md) — install from crates.io after publish; link to error contract tests as quality signal
- Release checklist (in CHANGELOG or `docs/release-first-crates-io.md`):
  - `cargo publish --dry-run`
  - `cargo install docli --version …` smoke: `docli generate --help`, one `error_contract`-equivalent manual check
- **Publish execution** is explicit human/team-lead step after merge to `develop`; sprint closes when repo is publish-ready, not necessarily when crates.io has the crate

## Out of Scope

- GitHub Release assets beyond source tag (unless team-lead adds later)
- Homebrew / npm distribution

## Acceptance Criteria

- `cargo publish --dry-run` succeeds from repo root
- README documents `cargo install docli` as primary install after first publish
- Verify script documents both pre-publish (built binary) and post-publish (`docli` on PATH) modes
- All Phase B CI gate commands pass on integration branch

## Required Validation

- Phase B CI gate in [README.md](README.md)
- `cargo publish --dry-run`
- `cargo package --list` includes `README.md`, `LICENSE`, `fixtures/` needed for tests in published crate (tests may stay dev-only; document what ships)
