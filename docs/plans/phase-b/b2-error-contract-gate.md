---
id: b.2
title: error contract integration gate
status: planned
branch: feature/phase-b-b2-errors
worktree: ../docli-worktrees/feature/phase-b-b2-errors
target: integrate/phase-b
---

# Sprint b.2 — error contract integration gate

## Goal

Every stable `DOCLI.*` code is integration-tested so `docli --json` and `cargo docli --json` return one actionable envelope, and human mode prints the same code and recovery text on stderr.

## Closes

- `REQ-DOCLI-CLI-008`
- `REQ-DOCLI-CLI-009`
- `REQ-DOCLI-CLI-010`
- `ADR-003` (machine contract; skill reference `.claude/skills/creating-ai-clis/references/error-contracts.md`)
- `ARCH-RULE-004` (`Envelope`, `ErrorBody`, and exit codes stay in `src/contract.rs`)
- `ARCH-RULE-007` (error-shape changes require `REQ-DOCLI-CLI-*` and the vendored skill)

## Hard Dependencies

- b.1 merged to `integrate/phase-b`
- Requirements section 9 error inventory and `REQ-DOCLI-CLI-008`–`010`
- `ADR-001` — operations return `Envelope<T>`; the CLI serializes that value
- Constructors in `src/contract.rs`: `ErrorBody::usage`, `input_invalid`, `input_not_found`, `output_not_found`, `io`, `internal`

## Deliverables

- [`docs/requirements.md`](../../requirements.md) section 9 **Covered by** cells name the tests in the manifest below.
- [`tests/common/mod.rs`](../../../tests/common/mod.rs), [`tests/common/envelope.rs`](../../../tests/common/envelope.rs), [`tests/common/harness.rs`](../../../tests/common/harness.rs) with the signatures in Explicit Code Samples.
- [`tests/error_contract.rs`](../../../tests/error_contract.rs) — the manifest tests. Each `--json` test calls `assert_failure` and `assert_json_mode_stdout_only_envelope`. Each human test calls `assert_human_actionable`.
- [`tests/cargo_docli.rs`](../../../tests/cargo_docli.rs) — `cargo_docli_matches_docli_error_envelopes` runs every **Parity** `yes` row against `docli` and `cargo-docli` and compares process exit and stdout bytes.
- [`tests/cli_contract.rs`](../../../tests/cli_contract.rs) — failure-envelope checks call `tests/common/envelope.rs`. Success-path tests stay. This file does not define a second envelope schema.

### Error-test manifest

| Code | Test | Exit | kind | details | Channel | Parity |
|------|------|------|------|---------|---------|--------|
| `DOCLI.USAGE` | `docli_usage_unknown_command_json` | 2 | `validation` | `{}`. `suggested_action` contains `not-a-command` | `--json` argv `not-a-command --json` | yes |
| `DOCLI.USAGE` | `docli_usage_invalid_flag_json` | 2 | `validation` | `{}`. `suggested_action` contains `--not-a-flag` | `--json` argv `generate --not-a-flag --json` | yes |
| `DOCLI.USAGE` | `docli_usage_show_without_paths_json` | 2 | `validation` | `{}`. `suggested_action` contains `--html` and `--markdown` | `--json` argv `show --json` | yes |
| `DOCLI.USAGE` | `docli_usage_show_without_paths_human` | 2 | `validation` | human stderr | argv `show` | no |
| `DOCLI.INPUT_INVALID` | `docli_input_invalid_parse_error_json` | 2 | `validation` | `{ "cause": <non-empty> }`. `suggested_action` contains the input path and that cause. File bytes: `{"name": 1}` | `--json` | yes |
| `DOCLI.INPUT_INVALID` | `docli_input_invalid_empty_file_json` | 2 | `validation` | `{}` | `--json`, empty file | yes |
| `DOCLI.INPUT_INVALID` | `docli_stdin_empty_is_input_invalid` | 2 | `validation` | `{}` | `--json` argv `generate --json`, stdin empty | yes |
| `DOCLI.INPUT_INVALID` | `docli_input_invalid_human` | 2 | `validation` | human stderr contains `DOCLI.INPUT_INVALID` and the parse cause | file bytes `{"name": 1}` | no |
| `DOCLI.INPUT_NOT_FOUND` | `docli_input_not_found_json` | 3 | `not_found` | `{ "path": <missing path> }`. `suggested_action` contains that path | `--json` | yes |
| `DOCLI.INPUT_NOT_FOUND` | `docli_input_not_found_human` | 3 | `not_found` | human stderr contains the code and the missing path | human | no |
| `DOCLI.OUTPUT_NOT_FOUND` | `docli_output_not_found_single_html_json` | 3 | `not_found` | `artifacts` is one object `{ "path": <dir>/index.html, "exists": false }`. `suggested_action` contains that path | `--json` `show --html <missing-dir>` | yes |
| `DOCLI.OUTPUT_NOT_FOUND` | `docli_output_not_found_html_and_markdown_json` | 3 | `not_found` | `artifacts` length 2, every `exists` is false | `--json` `show --html <missing-dir> --markdown <missing.md>` | yes |
| `DOCLI.OUTPUT_NOT_FOUND` | `docli_output_not_found_human` | 3 | `not_found` | human stderr contains the code and the missing `index.html` path | human | no |
| `DOCLI.IO` | `docli_io_generate_html_dir_not_writable_json` | 4 | `dependency` | `cause` present. `outputs_written` absent. `suggested_action` contains the failed path. `--html` is a file, not a directory | `--json` | yes |
| `DOCLI.IO` | `docli_io_generate_partial_write_lists_outputs_written_json` | 4 | `dependency` | `cause` present. `outputs_written` length 1, `kind` `html`. `index.html` exists on disk. Markdown parent path is a file | `--json` | yes |
| `DOCLI.IO` | `docli_io_show_unreadable_index_json` | 4 | `dependency` | `cause` present. Setup: generate HTML, replace `index.html` with a directory, then `show` | `--json` | yes |
| `DOCLI.IO` | `docli_io_show_unreadable_index_human` | 4 | `dependency` | human stderr contains `DOCLI.IO` and `index.html` | human | no |
| `DOCLI.INTERNAL` | `docli_internal_error_body_contract` | 1 | `internal` | `{ "cause": "serialization failed" }` from `ErrorBody::internal`. In-process only. `Envelope::failure` serializes `data: null` and `exit_code()` is `1` | in-process | no |

Adding a `DOCLI.*` variant requires a new manifest row, a requirements section 9 row, and a test before merge.

## Explicit Code Samples

Failure envelope (`REQ-DOCLI-CLI-002`, `REQ-DOCLI-CLI-003`). `data` and `error` are always present. `docs` is a string or `null`. `skip_serializing_if` is forbidden on `data`, `error`, and `docs`.

```json
{
  "version": "1",
  "ok": false,
  "data": null,
  "error": {
    "kind": "validation",
    "code": "DOCLI.INPUT_INVALID",
    "message": "model JSON is missing, empty, or does not match known field types",
    "details": { "cause": "invalid type: integer `1`, expected a string" },
    "suggested_action": "Fix the model JSON at /path/to/file: invalid type: integer `1`, expected a string",
    "docs": null
  }
}
```

Human stderr is `ErrorBody`'s `Display` (`REQ-DOCLI-CLI-009`). Contract fields are on stderr in human mode. They are on stdout in `--json` mode.

```text
DOCLI.INPUT_INVALID: model JSON is missing, empty, or does not match known field types
Fix the model JSON at /path/to/file: <cause>
<cause>
```

`DOCLI.USAGE` with empty details omits the cause line:

```text
DOCLI.USAGE: unknown command or invalid flags
Pass --html DIR and/or --markdown FILE
```

```rust
pub fn parse_envelope(stdout: &[u8]) -> serde_json::Value;
pub fn assert_success(envelope: &serde_json::Value);

/// Asserts process_exit == Some(expected_exit). The exit argument is checked.
/// Asserts version "1", ok false, data null, kind, code, details object,
/// non-empty message, non-empty suggested_action, docs string or null.
pub fn assert_failure(
    process_exit: Option<i32>,
    envelope: &serde_json::Value,
    expected_exit: i32,
    kind: &str,
    code: &str,
) -> serde_json::Value;

/// stderr contains `code` and every needle (the recovery text).
pub fn assert_human_actionable(stderr: &[u8], code: &str, needles: &[&str]);

/// stdout is one JSON value. stderr is empty or whitespace.
pub fn assert_json_mode_stdout_only_envelope(stdout: &[u8], stderr: &[u8]);
```

`parse_envelope` rejects empty stdout and rejects a second JSON value after the first.

```rust
pub fn docli_bin() -> std::process::Command;
pub fn cargo_docli_bin() -> std::process::Command;
pub fn run(cmd: &mut std::process::Command, args: &[&str]) -> std::process::Output;
pub fn unique_dir() -> std::path::PathBuf;
pub fn write_demo_model(dir: &std::path::Path) -> std::path::PathBuf;
```

Parity (`REQ-DOCLI-CLI-010`). `cargo-docli` is the binary `cargo docli` forwards to. For stdin rows, both processes receive the same empty stdin.

```rust
fn cargo_docli_matches_docli_error_envelopes() {
    // For every manifest row with Parity yes:
    //   let docli = run(&mut docli_bin(), args);
    //   let cargo = run(&mut cargo_docli_bin(), args);
    //   assert_eq!(docli.status.code(), cargo.status.code());
    //   assert_eq!(docli.stdout, cargo.stdout);
}
```

Exit codes stay `ErrorCode::exit_code`: `DOCLI.USAGE` and `DOCLI.INPUT_INVALID` → `2`, `DOCLI.INPUT_NOT_FOUND` and `DOCLI.OUTPUT_NOT_FOUND` → `3`, `DOCLI.IO` → `4`, `DOCLI.INTERNAL` → `1`.

## Paths to delete

- Delete `fn cargo_docli_matches_docli_error_envelopes` from `tests/error_contract.rs` after the full function is in `tests/cargo_docli.rs`. One parity test.

## Out of Scope

- New `DOCLI.*` codes
- A process argv that produces `DOCLI.INTERNAL` (the in-process test is the gate for that code)
- MCP wrapping of errors
- Translated messages
- Changing `ErrorBody` field names or the details keys in the manifest

## Acceptance Criteria

- Each manifest row's test asserts that row's exit, kind, code, details, channel, and non-empty `message` and `suggested_action`
- Each `--json` row leaves stderr empty or whitespace and puts the envelope on stdout, with `data` JSON `null` and `docs` a string or `null`
- Each human row prints the `DOCLI.*` code and the recovery text on stderr via `ErrorBody` `Display`
- `cargo_docli_matches_docli_error_envelopes` covers every Parity `yes` row, including empty stdin, and compares exit and stdout bytes
- `assert_failure` checks `process_exit` against `expected_exit`
- Requirements section 9 **Covered by** names the same test functions as the manifest
- `tests/cli_contract.rs` does not assert a conflicting envelope shape
- `DOCLI.INTERNAL` is covered by `docli_internal_error_body_contract` and is not given a false CLI argv

## As-built note

Much of this sprint may already be on `integrate/phase-b` (PR #13). Close **b.2** when acceptance criteria pass at the stack layer; do not revert landed work to re-enact the sprint.

## Required Validation

- Phase B host gate — [README.md](README.md)
- `cargo test --test error_contract`
- `cargo test --test cargo_docli cargo_docli_matches_docli_error_envelopes`
