# Phase A — Rust build tool (`develop`)

Phase A implementation PRs target **`develop`**. This directory is the sole authority for Phase A sprint deliverables, acceptance criteria, and validation. `docs/requirements.md` is the product contract. A requirement id marked later there is not a Phase A gap.

## What Phase A closes

- `docli generate` and `docli show` with the version `"1"` JSON envelope
- HTML written to `site/cli/index.html` by default, and to an explicit `--html` directory when the caller passes one
- Markdown written only when `--markdown` is set
- HTML tree behavior: separate caret and command controls, search over commands and options, ancestors of a match stay visible
- `cargo docli` via a `cargo-docli` binary with the same arguments as `docli`
- `docli::from_clap` mapping required by `REQ-DOCLI-RUST-001` and `REQ-DOCLI-RUST-002`
- In-process `generate` / `show` operations the CLI serializes
- `fixtures/contract/` byte lock for the renderer, plus checked-in command models from atm-core and sc-compose

## What Phase A does not close

- Go, .NET, and Python generators (`REQ-DOCLI-GO-001`, `REQ-DOCLI-NET-001`, `REQ-DOCLI-PY-001`, `REQ-DOCLI-GEN-003`)
- crates.io publish (`REQ-DOCLI-PRODUCT-001` registry install)
- Markdown bundled into language installers (`REQ-DOCLI-MD-002`)
- An MCP server or wrapper
- Opening HTML or Markdown in a browser or default app
- Committing `site/cli/` into atm-core, sc-compose, or sc-observability
- A sc-observability command model (that repo has no clap `Command`)

## Commands

```text
docli generate --input <FILE|-> [--html DIR] [--markdown FILE] [--json]
docli show (--html DIR | --markdown FILE) [--json]
cargo docli generate …
cargo docli show …
```

Command syntax and the envelope live in [a1-cli-contract.md](a1-cli-contract.md). Product ids live in `docs/requirements.md`. `--html` defaults to `site/cli`. Callers in scripts and CI pass `--html` explicitly. `--markdown` has no default.

Library boundary: [adr-001-ops-boundary.md](adr-001-ops-boundary.md). Search embed: [adr-002-search-index.md](adr-002-search-index.md).

## Sprint index (5 active: a.1–a.5)

| Sprint | Doc | Branch | Closes |
|--------|-----|--------|--------|
| a.1 | [a1-cli-contract.md](a1-cli-contract.md) | `feature/phase-a-a1-cli` | `REQ-DOCLI-CLI-001`–`007`, `REQ-DOCLI-HTML-006`, in-process `ops::generate` / `ops::show` |
| a.2 | [a2-html-behavior.md](a2-html-behavior.md) | `feature/phase-a-a2-html` | `REQ-DOCLI-HTML-001`–`005`, `REQ-DOCLI-MD-001`, `REQ-DOCLI-GEN-001` fixture lock |
| a.3 | [a3-cargo-docli.md](a3-cargo-docli.md) | `feature/phase-a-a3-cargo` | `REQ-DOCLI-PRODUCT-002` |
| a.4 | [a4-clap-adapter.md](a4-clap-adapter.md) | `feature/phase-a-a4-clap` | `REQ-DOCLI-RUST-001`, `REQ-DOCLI-RUST-002`, `pub use from_clap` |
| a.5 | [a5-repo-fixtures.md](a5-repo-fixtures.md) | `feature/phase-a-a5-fixtures` | atm-core and sc-compose models rendered with explicit `--html site/cli` |

## CI gate

Every sprint runs this from the repo root. Do not replace it with a narrower check.

```text
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
git diff --check
```

Phase A has no OS matrix beyond the host that runs those commands. There is no headed-browser gate.
