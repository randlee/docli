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

- `docli generate` and `docli show` speak one version `"1"` envelope on success and failure, and the same functions are callable in-process.

## Hard Dependencies

- [adr-001-ops-boundary.md](adr-001-ops-boundary.md)
- Renderer signatures stay `pub fn render(model: &CliModel) -> String` in `src/render/html.rs` and `src/render/markdown.rs`. a.1 calls those functions and does not change them.

## Deliverables

- `src/contract.rs` — envelope, error body, exit codes
- `src/ops.rs` — `generate` and `show`
- `src/main.rs` — clap commands call the operations; `--json` prints only the envelope
- `src/lib.rs` — `pub mod ops; pub mod contract; pub mod render;` and `pub use schema::{ArgumentSpec, CliModel, OptionSpec}`. `pub mod render` stays public per [adr-001-ops-boundary.md](adr-001-ops-boundary.md). `pub mod search` is a.2.
- `src/schema.rs` — `CliModel`, `OptionSpec`, and `ArgumentSpec` keep the fields below. None of them uses `deny_unknown_fields`. Each has `#[serde(flatten)] pub extra: serde_json::Map<String, serde_json::Value>` so unknown fields stay on the value and do not change how known fields render.
- `Cargo.toml` — `license = "MIT"` stays set. `LICENSE` at the repo root stays the MIT text.
- `tests/cli_contract.rs` — process tests and one in-process `ops::generate` test

`from_clap` re-export is a.4, not this sprint.

## Explicit Code Samples

```rust
#[derive(Serialize)]
pub struct Envelope<T> {
    pub version: &'static str, // "1"
    pub ok: bool,
    pub data: Option<T>,
    pub error: Option<ErrorBody>,
}
```

`serde_json` writes `None` as `null`, so both keys are always present. `skip_serializing_if = "Option::is_none"` is forbidden on `data` and `error`. Success JSON contains `"error": null`. Failure JSON contains `"data": null`.

```rust
pub struct ErrorBody {
    pub kind: &'static str, // validation | not_found | dependency | internal
    pub code: &'static str,
    pub message: String,
    pub details: serde_json::Value,
    pub suggested_action: String,
}

pub enum InputSource {
    Stdin,
    File(std::path::PathBuf),
}

pub struct ArtifactReport {
    pub kind: &'static str, // "html" | "markdown"
    pub path: std::path::PathBuf,
    pub bytes: u64,
    pub sha256: String,
}

pub struct GenerateRequest {
    pub input: InputSource,
    pub html_dir: Option<std::path::PathBuf>, // None -> site/cli
    pub markdown: Option<std::path::PathBuf>,
}

pub struct GenerateResponse {
    pub operation: &'static str, // "generate"
    pub input: String,
    pub model_name: String,
    pub html_dir: std::path::PathBuf,
    pub outputs: Vec<ArtifactReport>,
}

pub struct ShowRequest {
    pub html_dir: Option<std::path::PathBuf>,
    pub markdown: Option<std::path::PathBuf>,
}

pub struct ShowResponse {
    pub operation: &'static str, // "show"
    pub artifacts: Vec<ArtifactReport>,
}

pub fn generate(req: GenerateRequest) -> Envelope<GenerateResponse>;
pub fn show(req: ShowRequest) -> Envelope<ShowResponse>;
```

`suggested_action` stays one `String` because `REQ-DOCLI-CLI-003` names that field. It is one recovery sentence. There is no `cause` field on `ErrorBody`; an upstream OS or parse error is the string `details.cause` when one exists, and that key is omitted when it does not.

`generate` resolves `html_dir: None` to `site/cli` before writing. A write failure after another file landed returns an envelope with `ok: false`, code `DOCLI.IO`, and `error.details.outputs_written` listing the artifacts already written. `show` does not apply that default; both paths `None` is `DOCLI.USAGE`.

Model fields this sprint owns in `src/schema.rs`:

```rust
pub struct CliModel {
    pub name: String,
    pub version: Option<String>,
    pub description: String,
    pub long_description: String,
    pub epilogue: String,
    pub usage: String,
    pub options: Vec<OptionSpec>,
    pub arguments: Vec<ArgumentSpec>,
    pub subcommands: Vec<CliModel>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

pub struct OptionSpec {
    pub name: String,
    pub long: Option<String>,
    pub short: Option<String>,
    pub help: String,
    pub long_help: String,
    pub value_name: Option<String>,
    pub required: bool,
    pub default_value: Option<String>,
    pub choices: Vec<String>,
    pub min_values: Option<usize>,
    pub max_values: Option<usize>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

pub struct ArgumentSpec {
    pub name: String,
    pub help: String,
    pub required: bool,
    pub default_value: Option<String>,
    pub choices: Vec<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}
```

Exit codes: `0` success, `2` validation (`DOCLI.USAGE`, `DOCLI.INPUT_INVALID`), `3` not found (`DOCLI.INPUT_NOT_FOUND`, `DOCLI.OUTPUT_NOT_FOUND`), `4` dependency (`DOCLI.IO`), `1` internal (`DOCLI.INTERNAL`).

| Code | Kind | Cause | `details` | `suggested_action` |
|---|---|---|---|---|
| `DOCLI.USAGE` | validation | unknown command or invalid flags | `{}` | name the flag or command that was rejected |
| `DOCLI.INPUT_INVALID` | validation | model JSON is missing, empty, or does not match known field types | `{ "cause": "<parse error>" }` when serde reports one, otherwise `{}` | point at the input and the parse error |
| `DOCLI.INPUT_NOT_FOUND` | not_found | `--input` path does not exist | `{ "path": "<path>" }` | name the missing path |
| `DOCLI.OUTPUT_NOT_FOUND` | not_found | `show` asked for an artifact that is not on disk | `{ "artifacts": [{ "path": "<path>", "exists": false }] }` for every requested artifact | name each missing path |
| `DOCLI.IO` | dependency | reading or writing a file failed | `{ "cause": "<os error>", "outputs_written": [<ArtifactReport>, ...] }`. `outputs_written` is present only after a partial write | name the path that failed |
| `DOCLI.INTERNAL` | internal | an unexpected failure | `{ "cause": "<message>" }` | say to report the cause string |

## Out of Scope

- Caret, search, and other HTML interaction fixes
- `cargo-docli`
- `pub use clap_model::from_clap` (a.4)
- atm-core, sc-compose, and sc-observability fixtures
- Opening a file in a browser or default app

## Acceptance Criteria

- `docli generate --input <model.json> --html <dir> --markdown <out.md> --json` exits 0, stdout is one envelope, and each `outputs` sha256 matches the written file
- That success envelope contains the key `error` with JSON value `null`
- `docli generate --input <missing.json> --json` exits 3, code `DOCLI.INPUT_NOT_FOUND`, and the envelope contains the key `data` with JSON value `null`
- `docli generate --input <model.json> --json` with no `--html` writes `site/cli/index.html` and `html_dir` is that directory
- `docli generate --json` with stdin model JSON and no `--input` exits 0 and `input` is `"stdin"` (`--input` defaults to `-`)
- `docli show --json` with neither `--html` nor `--markdown` exits 2 with `error.code` `DOCLI.USAGE`
- `docli generate --input <model.json> --html <dir>` with no `--json` exits 0 and stdout contains that HTML path, its byte length, and its sha256
- `docli show --html <dir> --markdown <out.md> --json` exits 0 and its artifact hashes equal the `generate` hashes
- `docli show --html <dir>` with no `--json` exits 0 and stdout contains path, byte length, and sha256
- Invalid model JSON exits 2 with `error.code` `DOCLI.INPUT_INVALID`
- `docli show --html <missing> --json` exits 3 with `DOCLI.OUTPUT_NOT_FOUND` and `details` listing each requested artifact
- `docli::ops::generate(GenerateRequest { input: InputSource::File(model_path), html_dir: Some(tmp), markdown: None })` returns an `Envelope` with `ok: true`, `html_dir` equal to `tmp`, and `tmp/index.html` exists
- `--help` and `--version` exit 0 and print human text
- A model JSON object with an extra field `"future": 1` on the root, on one option, and on one argument deserializes with those keys in `extra`, and the generated page still renders the known fields
- `CliModel`, `OptionSpec`, and `ArgumentSpec` expose every field named by `REQ-DOCLI-INPUT-001`, `REQ-DOCLI-INPUT-002`, and `REQ-DOCLI-INPUT-003`
- `cargo metadata --no-deps --format-version 1` reports `"license":"MIT"`, and the repo-root `LICENSE` file begins with `MIT License`

## Required Validation

- Phase A CI gate in [README.md](README.md)
- `cargo test --test cli_contract`
