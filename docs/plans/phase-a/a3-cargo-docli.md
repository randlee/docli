---
id: a.3
title: cargo docli entry
status: planned
branch: feature/phase-a-a3-cargo
worktree: ../docli-worktrees/feature/phase-a-a3-cargo
target: develop
---

# Sprint a.3 — cargo docli entry

## Goal

- `cargo docli` runs the same operations as `docli`, including the not-found error path.

## Hard Dependencies

- a.1 command surface and `ops::generate` / `ops::show`

## Deliverables

- `src/bin/cargo-docli.rs` — argv after the binary name matches `docli`; the file calls `ops::generate` and `ops::show` and does not duplicate their write or envelope logic
- `Cargo.toml` — `[[bin]]` name `cargo-docli` beside `docli`
- `tests/fixtures/minimal-model.json` — one valid `CliModel` used only by this sprint's tests
- `tests/cargo_docli.rs` — stdout and exit code parity for success and for a missing input

## Explicit Code Samples

```toml
[[bin]]
name = "docli"
path = "src/main.rs"

[[bin]]
name = "cargo-docli"
path = "src/bin/cargo-docli.rs"
```

`cargo docli generate --input model.json --html site/cli` is cargo invoking binary `cargo-docli` with argv `generate --input model.json --html site/cli`.

## Out of Scope

- Publishing either binary to crates.io
- Discovering a package's clap types without `from_clap`
- Changing generate or show behavior
- Using `fixtures/contract/model.json` (that file is a.2)

## Acceptance Criteria

- `cargo run --bin cargo-docli -- generate --input tests/fixtures/minimal-model.json --html <dir> --json` and the same argv on `--bin docli` exit 0 with equal stdout
- The same pair holds for `show --html <dir> --json` on that directory
- Both binaries, given `--input tests/fixtures/does-not-exist.json --json`, exit 3 and both envelopes have `ok: false` and `error.code` `DOCLI.INPUT_NOT_FOUND`
- `cargo run --bin cargo-docli -- --help` exits 0 and lists `generate` and `show`

## Required Validation

- Phase A CI gate in [README.md](README.md)
- `cargo test --test cargo_docli`
