---
id: a.1
title: generate and show machine contract
status: planned
branch: feature/phase-a-a1-cli
worktree: ../docli-worktrees/feature/phase-a-a1-cli
target: develop
---

# Sprint a.1 — generate and show machine contract

## Goal

- `docli generate` and `docli show` speak one version `"1"` envelope on success and failure.

## Hard Dependencies

- None. The prototype renderer in `src/render/` stays the writer of HTML and Markdown bytes.

## Deliverables

- `src/contract.rs` — envelope, error body, exit codes
- `src/ops.rs` — `generate` and `show` operations returning those types
- `src/main.rs` — clap commands call the operations; `--json` prints only the envelope
- `src/lib.rs` — re-exports the operation and contract types
- `tests/cli_contract.rs` — binary tests for the envelope, default HTML path, and `show` readback

## Explicit Code Samples

```rust
pub struct Envelope<T> {
    pub version: &'static str, // "1"
    pub ok: bool,
    pub data: Option<T>,
    pub error: Option<ErrorBody>,
}

pub struct ErrorBody {
    pub kind: &'static str, // validation | not_found | dependency | internal
    pub code: &'static str,
    pub message: String,
    pub details: serde_json::Value,
    pub suggested_action: String,
}
```

Exit codes: `0` success, `2` validation (`DOCLI.USAGE`, `DOCLI.INPUT_INVALID`), `3` not found (`DOCLI.INPUT_NOT_FOUND`, `DOCLI.OUTPUT_NOT_FOUND`), `4` dependency (`DOCLI.IO`), `1` internal (`DOCLI.INTERNAL`).

`generate` data fields: `operation` (`"generate"`), `input`, `model_name`, `html_dir`, `outputs[]` (`kind`, `path`, `bytes`, `sha256`). `--html` defaults to `site/cli`. `--markdown` omitted writes no Markdown file. A later write failure after an earlier file landed is `ok: false` and `error.details.outputs_written` lists what landed.

`show` data fields: `operation` (`"show"`), `artifacts[]` with the same `kind`, `path`, `bytes`, and `sha256`. At least one of `--html` or `--markdown` is required. `show` does not apply the `generate` HTML default.

## Out of Scope

- Caret, search, and other HTML interaction fixes
- `cargo-docli`
- Clap adapter mapping tests beyond the prototype
- atm-core, sc-compose, and sc-observability fixtures
- Opening a file in a browser or default app

## Acceptance Criteria

- `docli generate --input <model.json> --html site/cli --markdown out.md --json` exits 0 and stdout is one envelope whose `outputs` sha256 values match the written files
- `docli generate --input <model.json> --json` with no `--html` writes `site/cli/index.html` and reports that path in `html_dir`
- `docli show --html site/cli --markdown out.md --json` exits 0 and its artifact hashes equal the `generate` hashes
- Invalid model JSON exits 2 with `error.code` `DOCLI.INPUT_INVALID`
- A missing `--input` path exits 3 with `DOCLI.INPUT_NOT_FOUND`
- `docli show --html <missing> --json` exits 3 with `DOCLI.OUTPUT_NOT_FOUND` and `details` listing each requested artifact
- `--help` and `--version` exit 0 and print human text

## Required Validation

- Phase A CI gate in [README.md](README.md)
- `cargo test --test cli_contract`
