---
id: a.5
title: test-repo command models
status: complete
branch: feature/phase-a-a5-fixtures
worktree: ../docli-worktrees/feature/phase-a-a5-fixtures
target: develop
---

# Sprint a.5 — test-repo command models

## Goal

- Checked-in command models from the clap CLIs atm-core and sc-compose render through `generate --html site/cli`.

## CI Hard Dependencies

- a.1 `generate` and `show`

`tests/repo_fixtures.rs` parses committed JSON and runs `generate` / `show`. It does not call `from_clap` and does not compile atm-core or sc-compose.

## Fixture Capture Prerequisites

Not a CI dependency. Regeneration is local:

```text
cargo run --bin gen-fixtures -- --atm-core <atm-core-checkout> --sc-compose <sc-compose-checkout>
```

`<atm-core-checkout>` and `<sc-compose-checkout>` are arguments. The binary does not default them to a home-directory path.

`gen-fixtures` is behind Cargo feature `gen-fixtures` (`required-features` on that bin). `cargo test` does not enable the feature and does not build the bin.

The bin does not depend on atm-core or sc-compose in `Cargo.toml`. For each checkout it temporarily patches that crate's `Cargo.toml` with a path dependency on this docli tree, appends a `docli_gen_fixtures` cfg-gated test module that calls `docli::from_clap`, runs `cargo test` for that package, reads the JSON from `DOCLI_GEN_OUT`, then restores the patched files. The injected test calls `docli::from_clap` on:

- atm-core: `Cli::command()` in `crates/atm` (`clap::CommandFactory`)
- sc-compose: `cli::Cli::command()` in `crates/sc-compose`

sc-observability has no clap `Command` (`crates/sc-observe` is a library). This sprint does not invent a CLI there and does not commit `fixtures/repos/sc-observability.json`.

## Deliverables

- `Cargo.toml` — feature `gen-fixtures` and `[[bin]]` `gen-fixtures` with `required-features = ["gen-fixtures"]`
- `src/bin/gen-fixtures.rs`
- `fixtures/repos/atm-core.json`
- `fixtures/repos/sc-compose.json`
- `tests/repo_fixtures.rs` — each JSON file generates `site/cli/index.html` under a temp directory and `show` reports the same sha256

## Out of Scope

- Pull requests or `site/cli/` commits in atm-core, sc-compose, or sc-observability
- A sc-observability command model
- Worktrees of those repos as a Phase A deliverable
- Running `gen-fixtures` in CI

## Acceptance Criteria

- `fixtures/repos/atm-core.json` and `fixtures/repos/sc-compose.json` each parse as `CliModel` and have a non-empty `name`
- `docli generate --input fixtures/repos/atm-core.json --html <tmp>/site/cli --json` exits 0 and writes `<tmp>/site/cli/index.html`
- The same command for `fixtures/repos/sc-compose.json` exits 0 and writes that page
- `docli show --html <tmp>/site/cli --json` exits 0 and the artifact sha256 equals the matching `generate` output entry
- Both `generate` invocations pass `--html` explicitly
- `fixtures/repos/sc-observability.json` is absent

## Required Validation

- Phase A CI gate in [README.md](README.md)
- `cargo test --test repo_fixtures`
