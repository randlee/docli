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

- `cargo docli` runs the same operations as `docli`.

## Hard Dependencies

- a.1 command surface and envelope

## Deliverables

- `src/bin/cargo-docli.rs` — cargo subcommand entry; arguments after the binary name match `docli`
- `Cargo.toml` — `[[bin]]` name `cargo-docli` beside the existing `docli` bin
- `tests/cargo_docli.rs` — the two binaries emit the same stdout and exit code for `generate` and `show`

## Explicit Code Samples

```toml
[[bin]]
name = "docli"
path = "src/main.rs"

[[bin]]
name = "cargo-docli"
path = "src/bin/cargo-docli.rs"
```

`cargo docli generate --input model.json --html site/cli` is cargo invoking the `cargo-docli` binary with argv `generate --input model.json --html site/cli`. The binary does not insert a different flag set.

## Out of Scope

- Publishing either binary to crates.io
- A cargo build-script that discovers a package's clap types without `from_clap`
- Changing generate or show behavior

## Acceptance Criteria

- `cargo run --bin cargo-docli -- generate --input <model.json> --html <dir> --json` and `cargo run --bin docli -- generate --input <model.json> --html <dir> --json` exit 0 with equal stdout
- The same pair holds for `show --html <dir> --json`
- `cargo run --bin cargo-docli -- --help` exits 0 and lists `generate` and `show`

## Required Validation

- Phase A CI gate in [README.md](README.md)
- `cargo test --test cargo_docli`
