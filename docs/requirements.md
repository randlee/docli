# docli Requirements

**Status**: Authoritative requirements baseline for this repository.  
**Applies to**: `docli`  
**Consumers**: `req-qa` (compliance), implementers, sprint plans.

**Hard requirements**: every `REQ-DOCLI-*` id in this document is normative for
Phase A unless marked **(later)**. Use **MUST** / **MUST NOT** semantics when
implementing or reviewing.

**Source of truth (product)**

- This file — requirement ids and acceptance criteria
- [`architecture.md`](architecture.md) — ADRs and `ARCH-RULE-*` (validated by `arch-qa`)
- `src/schema.rs` — input model fields for `REQ-DOCLI-INPUT-*`
- `fixtures/contract/` — rendered HTML/Markdown byte lock for `REQ-DOCLI-GEN-*`

**QA validation (`req-qa`)**

1. Read this file, [`architecture.md`](architecture.md), and
   [`plans/project-plan.md`](plans/project-plan.md) before analysis.
2. Map each in-scope deliverable and acceptance criterion to one or more
   `REQ-DOCLI-*` ids (cite `requirements.md` line or section).
3. Map structural boundaries to `ADR-*` / `ARCH-RULE-*` in `architecture.md`
   (arch-qa primary; req-qa flags cross-doc conflicts).
4. **FAIL** on missing deliverables, unverifiable acceptance, or contradiction
   between sprint docs and these baselines.
5. External skill rules apply where cited (see `REQ-DOCLI-NORM-001`); skill text
   lives under `.claude/skills/creating-ai-clis/`.

**Mandatory CLI design basis**: the **creating-ai-clis** skill supplied for this
project (vendored at `.claude/skills/creating-ai-clis/`, from
`synaptic-canvas/packages/sc-ai-cli`). The `docli` and `cargo docli` binaries,
their `--json` envelope, typed actionable errors, and `generate` / `show`
readback pair MUST conform to that skill and its references (`core-contract.md`,
`error-contracts.md`, `mcp-compatibility.md`, `simulation-and-auditability.md`
where applicable). This file names stable requirement ids and docli-specific
product scope; it does not replace the skill.

Phase A is Rust only. Go, .NET, and Python stay in the contract so later phases match the same bytes and the same CLI envelope, and they are not Phase A deliverables.

## 1. Purpose And Scope

`docli` is a build tool that generates CLI documentation. It converts a
neutral command-tree description into a self-contained two-pane HTML reference
and a Markdown manual, mechanically and deterministically. Reference pages are
generated, never hand-authored.

Callers extract the command tree from their own CLI framework and hand `docli`
the neutral model. `docli` does not parse rendered `--help` text, and it is
not a site framework or a theme library.

## 2. Phases

| Phase | Closes | Does not close |
|---|---|---|
| A | Rust library, `docli` CLI, `cargo docli`, clap adapter, HTML and Markdown renderer, `generate` / `show` envelope, and `fixtures/contract/` produced by that Rust tool | Go, .NET, Python, crates.io publish, MCP wrapper |
| Later | `REQ-DOCLI-GO-001`, `REQ-DOCLI-NET-001`, `REQ-DOCLI-PY-001`, cross-language byte match (`REQ-DOCLI-GEN-003`), per-language installers (`REQ-DOCLI-MD-002`), crates.io (`REQ-DOCLI-PRODUCT-001` publish step) | — |

Phase A acceptance is the Rust rows in the sections below. A later-phase id is not a Phase A gap.

## 3. Product Definition

Rust is the reference implementation. Go, .NET, and Python are later peer
implementations of the same generator and the same command contract.

- `REQ-DOCLI-PRODUCT-001`: the Rust implementation is a library and a
  command-line tool. Phase A builds and tests it from this repo. Publishing
  to crates.io (`cargo install docli` from the registry) is a later release
  step.
- `REQ-DOCLI-PRODUCT-002`: a Rust project runs the tool in either of two ways,
  and both ways perform the same operations:
  - CLI: `docli <command> [flags]`
  - Cargo: `cargo docli <command> [flags]` (a `cargo-docli` binary; arguments
    after `cargo docli` match `docli`)
- `REQ-DOCLI-PRODUCT-003`: a Rust build script or test can call the library
  in-process (`from_clap` plus the HTML and Markdown renderers, and the
  `generate` / `show` operations) and get the same artifacts and the same
  result data the CLI would emit.
- `REQ-DOCLI-PRODUCT-004` (later): Go, .NET, and Python implementations
  generate the same HTML and Markdown from the same neutral model, and expose
  the same `generate` and `show` commands, flags, exit codes, and JSON envelope:
  - Go: a `docli` command
  - .NET: a `dotnet docli` tool
  - Python: a `docli` console script and `python -m docli`

The tool runs in CI and release pipelines. It is a local build tool, not a
service.

## 4. Neutral Input Model

The input is one JSON document describing a CLI command tree, carrying no
framework-specific concepts. Every implementation accepts this document.

- `REQ-DOCLI-INPUT-001`: a command carries `name`, `version`, `description`,
  `long_description`, `epilogue`, `usage`, `options`, `arguments`,
  `subcommands`.
- `REQ-DOCLI-INPUT-002`: an option carries `name`, `long`, `short`, `help`,
  `long_help`, `value_name`, `required`, `default_value`, `choices`,
  `min_values`, `max_values`.
- `REQ-DOCLI-INPUT-003`: an argument carries `name`, `help`, `required`,
  `default_value`, `choices`.
- `REQ-DOCLI-INPUT-004`: input is read from a file path or stdin (`-`).
- `REQ-DOCLI-INPUT-005`: unknown input fields are preserved for forward
  compatibility. Implementations still render the known fields the same way.

## 5. Language Adapters

Each language walks its native command definition and emits the neutral model,
then renders that model with that language's generator.

- `REQ-DOCLI-RUST-001`: Rust maps a live `clap::Command` through
  `docli::from_clap`. clap's `possible_values` map to `choices`,
  `value_names` to `value_name`, and `num_args` to `min_values`/`max_values`.
- `REQ-DOCLI-RUST-002`: the clap adapter excludes auto-injected
  `--help`/`--version` and preserves the full command path in `usage`.
- `REQ-DOCLI-GO-001` (later): a Go adapter emits the same JSON from a cobra
  command tree, and the Go generator renders that JSON to the shared HTML and
  Markdown.
- `REQ-DOCLI-NET-001` (later): a .NET adapter emits the same JSON from a
  System.CommandLine command tree, and the .NET generator renders that JSON
  to the shared HTML and Markdown.
- `REQ-DOCLI-PY-001` (later): a Python adapter emits the same JSON from the
  project's Python CLI definition (argparse, Typer, or click), and the Python
  generator renders that JSON to the shared HTML and Markdown.

## 6. HTML Output

These rules apply to every language implementation.

- `REQ-DOCLI-HTML-001`: output is a self-contained page (single file, inline
  CSS and JS, embedded data), hash-routed per command with readable slug
  anchors.
- `REQ-DOCLI-HTML-002`: layout is two-pane — an indented, collapsible command
  tree on the left and a detail panel on the right.
- `REQ-DOCLI-HTML-003`: the tree has indentation by depth, a selected
  highlight, a hover state, and expand/collapse carets (cli_doc-style).
  Activating a caret expands or collapses that node. Activating the command
  name selects it and leaves the expanded state unchanged.
- `REQ-DOCLI-HTML-004`: the detail panel shows description, usage, and an
  arguments/options table (name/short/value/required/default/choices/
  description).
- `REQ-DOCLI-HTML-005`: a search box filters commands and options. A match on
  a nested command or option stays visible, including its ancestor nodes.
- `REQ-DOCLI-HTML-006`: the default HTML directory is `site/cli`, which
  writes `site/cli/index.html`. The page's up-link (`../`) resolves to
  `site/index.html`. Callers pass `--html` explicitly in scripts, CI, and the
  test repos (atm-core, sc-compose, sc-observability all use `--html site/cli`).
  The default applies only when `--html` is omitted, and the resolved path is
  still reported in the `generate` envelope. The generator creates the
  directory when it is missing.

## 7. Markdown Output

- `REQ-DOCLI-MD-001`: a flat Markdown reference with one section per command,
  including description, usage, arguments, and options.
- `REQ-DOCLI-MD-002` (later): the Markdown manual is distributed with each
  language's installer for offline use.

## 8. Generation Guarantees

- `REQ-DOCLI-GEN-001`: output is deterministic and mechanically generated from
  the command model. The same input bytes produce the same HTML bytes and the
  same Markdown bytes on every run.
- `REQ-DOCLI-GEN-002`: consumers regenerate both outputs on every release; CI
  rejects missing or stale outputs.
- `REQ-DOCLI-GEN-003` (later): Go, .NET, and Python match the fixture corpus
  Phase A writes. `fixtures/contract/` holds input models plus the HTML and
  Markdown produced by the Rust tool. Phase A fails CI when the Rust renderer
  drifts from those fixtures. Later implementations fail CI when their bytes
  differ from the same files.

## 9. CLI Contract

The machine contract is primary. Human output is a presentation of the same
result. Section 9 implements the mandatory **creating-ai-clis** skill (see header):
read `.claude/skills/creating-ai-clis/SKILL.md` and
`references/core-contract.md` / `references/error-contracts.md` before changing
CLI behavior. Every command has a stable operation name, a request model, a
response model, `--json`, and one envelope for success and failure. Failures
must expose `kind`, stable `code`, structured `details`, and `suggested_action`
so automated callers (and wrapper scripts) can recover without guessing. Request
and response types live outside the CLI entrypoint so a later MCP wrapper can
call the same operations without reshaping the payload.

- `REQ-DOCLI-NORM-001`: `docli` CLI contract changes are invalid unless they
  remain conformant with the in-repo **creating-ai-clis** skill and references.

There is no interactive prompt. JSON mode emits no color, progress, or
prompts.

- `REQ-DOCLI-CLI-001`: every command accepts a global `--json` flag. With
  `--json`, stdout is only the envelope, on success and on failure. Exit
  status is still non-zero on failure.
- `REQ-DOCLI-CLI-002`: the envelope is the same shape for every command:

```json
{
  "version": "1",
  "ok": true,
  "data": {},
  "error": null
}
```

  `version` is the string `"1"`. `data` and `error` are always present.
  Success sets `ok` true, `data` to the response, and `error` null. Failure
  sets `ok` false, `data` null, and `error` to the error object.
- `REQ-DOCLI-CLI-003`: `error` carries `kind`, `code`, `message`, `details`,
  `suggested_action`, and `docs`. This matches sc-ai-cli actionable error
  guidance: callers must be able to branch on `kind`/`code` and act on
  `suggested_action` without parsing prose-only stderr. `kind` is one of
  `validation`, `not_found`, `dependency`, or `internal`. `code` is one of the
  stable strings below. `details` is an object. `docs` is a string or null.
  Codes are stable:

  | Code | Kind | When |
  |---|---|---|
  | `DOCLI.USAGE` | `validation` | unknown command or invalid flags |
  | `DOCLI.INPUT_INVALID` | `validation` | model JSON is missing, empty, or does not match the input model |
  | `DOCLI.INPUT_NOT_FOUND` | `not_found` | `--input` path does not exist |
  | `DOCLI.OUTPUT_NOT_FOUND` | `not_found` | `show` was asked for an artifact that is not on disk |
  | `DOCLI.IO` | `dependency` | reading or writing a file failed |
  | `DOCLI.INTERNAL` | `internal` | an unexpected failure |

- `REQ-DOCLI-CLI-004`: exit codes are `0` success, `2` validation, `3` not
  found, `4` dependency, `1` internal. `--help` and `--version` exit `0` and
  stay human-readable.
- `REQ-DOCLI-CLI-005`: `generate` is the mutating build operation.
  - Request: `--input` (path or `-`, default `-`), `--html DIR` (default
    `site/cli`), optional `--markdown FILE` (no default; omitted means no
    Markdown file).
  - It renders HTML from the model and writes `DIR/index.html`. It renders
    Markdown only when `--markdown` is set, and writes that file. Parent
    directories are created as needed.
  - Response `data` includes `operation` (`"generate"`), `input` (`"stdin"`
    or the path), `model_name`, `html_dir` (the resolved directory), and
    `outputs` (each written artifact's `kind`, `path`, `bytes`, and `sha256`).
  - `--json` writes the files and prints the envelope. It does not print the
    page on stdout.
  - A failed write after another output was already written is `ok: false`
    with `DOCLI.IO`. `error.details.outputs_written` lists the artifacts
    that were written.
- `REQ-DOCLI-CLI-006`: `show` is the readback for `generate`.
  - Request: at least one of `--html DIR` or `--markdown FILE`.
  - Response `data` includes `operation` (`"show"`) and `artifacts` with the
    same `kind`, `path`, `bytes`, and `sha256` fields `generate` reports.
  - A missing requested artifact is `DOCLI.OUTPUT_NOT_FOUND`. `details`
    lists every requested artifact and whether it exists, so a partial set
    is visible.
- `REQ-DOCLI-CLI-007`: human-readable output uses only fields that `--json`
  also returns. Human `generate` with output paths prints each path, byte
  length, and sha256. Human `show` prints the same fields.
- `REQ-DOCLI-CLI-008`: every stable error code in the table below **MUST**
  have integration coverage in `tests/error_contract.rs` that asserts:
  - exit code per `REQ-DOCLI-CLI-004`
  - envelope shape per `REQ-DOCLI-CLI-002` (`data: null` on failure)
  - `error.message` and `error.suggested_action` are non-empty strings agents
    can act on without parsing stderr
  - `error.details` matches the schema row for that code
- `REQ-DOCLI-CLI-009`: with `--json`, failures **MUST** emit exactly one JSON
  envelope on stdout. Contract fields **MUST NOT** appear only on stderr.
  Human mode **MUST** print the same stable `DOCLI.*` code and recovery text on
  stderr via `ErrorBody`'s display format (code, message, suggested_action,
  optional cause).
- `REQ-DOCLI-CLI-010`: `cargo docli` **MUST** match `docli` exit status and
  stdout bytes for each error scenario exercised in `tests/error_contract.rs`
  (see `cargo_docli_matches_docli_error_envelopes` and extended cases).

### Error code inventory (normative)

| Code | Exit | `kind` | `details` (required keys) | Covered by |
|------|------|--------|---------------------------|------------|
| `DOCLI.USAGE` | 2 | `validation` | `{}` | `docli_usage_*` tests |
| `DOCLI.INPUT_INVALID` | 2 | `validation` | `{}` or `{ "cause" }` | `docli_input_invalid_*`, `docli_stdin_empty_*` |
| `DOCLI.INPUT_NOT_FOUND` | 3 | `not_found` | `{ "path" }` | `docli_input_not_found_*` |
| `DOCLI.OUTPUT_NOT_FOUND` | 3 | `not_found` | `{ "artifacts": [{ "path", "exists" }] }` | `docli_output_not_found_*` |
| `DOCLI.IO` | 4 | `dependency` | `{ "cause" }`; optional `{ "outputs_written" }` after partial write | `docli_io_*` |
| `DOCLI.INTERNAL` | 1 | `internal` | `{ "cause" }` | `docli_internal_error_body_contract` |

Adding a new `DOCLI.*` code requires updating this table, `src/contract.rs`,
and a matching test in `tests/error_contract.rs` before merge.

## 10. Distribution

- `REQ-DOCLI-DIST-001`: every implementation is licensed MIT.

## 11. Requirement index (Phase A)

Use this table for traceability in QA findings (`source_refs` must cite ids).

| ID | Summary | ADR |
|----|---------|-----|
| `REQ-DOCLI-NORM-001` | CLI must conform to creating-ai-clis skill | ADR-003 |
| `REQ-DOCLI-PRODUCT-001` | Rust library + CLI in repo; crates.io later | — |
| `REQ-DOCLI-PRODUCT-002` | `docli` and `cargo docli` same operations | ADR-001 |
| `REQ-DOCLI-PRODUCT-003` | In-process `ops` / `from_clap` / render parity with CLI | ADR-001 |
| `REQ-DOCLI-INPUT-001`–`005` | Neutral `CliModel` JSON shape and stdin/file input | — |
| `REQ-DOCLI-RUST-001`–`002` | clap adapter mapping and exclusions | — |
| `REQ-DOCLI-HTML-001`–`006` | Self-contained two-pane HTML, search, default `site/cli` | ADR-002 |
| `REQ-DOCLI-MD-001` | Flat Markdown reference | — |
| `REQ-DOCLI-GEN-001`–`002` | Deterministic output; CI fixture lock | ADR-002 |
| `REQ-DOCLI-CLI-001`–`010` | `--json` envelope, errors, tests, cargo parity | ADR-001, ADR-003 |
| `REQ-DOCLI-DIST-001` | MIT license | — |
