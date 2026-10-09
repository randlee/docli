# docli Requirements

**Status**: Authoritative requirements baseline for this repository.  
**Applies to**: `docli`  
**Consumers**: `req-qa` (compliance), implementers, sprint plans.

**Hard requirements**: every `REQ-DOCLI-*` id with a requirement sentence in
this document is normative for Phase A unless marked **(later)**. Use **MUST** /
**MUST NOT** semantics when implementing or reviewing. Section 11 rows whose
text lands in a later sprint are traceability entries only; the normative
sentence arrives with that sprint.

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
  `generate` / `show` / `templates_list` / `templates_show` /
  `templates_validate` operations) and get the same artifacts and the same
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
- `REQ-DOCLI-HTML-007`: the `default` HTML template pack is embedded in the
  binary from `templates/html/default/`. `html::render` MUST render that pack
  with its `theme_schema` defaults and MUST match `fixtures/contract/index.html`
  when the caller does not pass `--template`. The page MUST include
  `id="docli-default-pack"`, and that marker MUST be defined only in the
  default pack. Optional extra packs MAY live under `share/docli/templates`;
  they are not the source of `default`. A failure rendering the embedded
  default pack MUST surface from `ops::generate` as `DOCLI.INTERNAL`.
  `resolve_pack` MUST classify a filesystem read failure as `DOCLI.IO`. This
  requirement MUST NOT introduce a `DOCLI.TEMPLATE_*` code.

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

### Fixture policy

CI (`cargo test`, including `tests/repo_fixtures.rs`) reads only the committed files `fixtures/repos/atm-core.json` and `fixtures/repos/sc-compose.json`.

`fixtures/repos/sc-observability.json` is not a file in this repository. sc-observability has no clap `Command` to capture.

`./scripts/verify-candidate-repos.sh` has three modes:

1. `DOCLI_SKIP_GEN_FIXTURES=1` — do not run `gen-fixtures`. `generate` and `show` use the committed JSON. This is the pre-publish proof command. When `DOCLI_REFRESH_FIXTURES` is also set, skip still wins and the script does not write fixtures.
2. Default (neither flag set) — snapshot `fixtures/repos/*.json`, run `gen-fixtures`, and diff against the snapshot. If the bytes differ, call `restore_committed_fixtures`, print the diff, and exit non-zero. The script does not leave a dirty fixture behind.
3. `DOCLI_REFRESH_FIXTURES=1` (and skip unset) — write the captured JSON over the committed files. That write is a reviewed pull request. It is not the proof command.

Live capture may differ from committed JSON when upstream help text is empty. That difference is a refresh pull request, not a silent verify success.

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
  | `DOCLI.TEMPLATE_INVALID` | `validation` | template pack manifest or templates are invalid, or `templates show` was given an unknown bundled id |

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
- `REQ-DOCLI-CLI-011`: `docli templates` discovers and validates template packs.
  - `templates list` takes no arguments. Response `data` includes `operation`
    (`"templates_list"`), `install_root`, and `templates`. `install_root` is
    `{prefix}/share/docli/templates`, where `{prefix}` is the parent of the
    directory that contains the executable (`/usr/local/bin/docli` →
    `/usr/local/share/docli/templates`). When the executable path is
    unavailable, `{prefix}` is `/usr/local`. The directory is reported even
    when it is absent. Each template has `id`, `name`, `version`, and `path`.
    The embedded `default` pack is always listed with path `embedded:default`.
    An install-root directory is listed only when it contains `template.toml`
    and the pack resolves. A pack whose `id` matches a bundled pack is omitted.
  - `templates show <ID|PATH>` loads a bundled id (`default`), an
    `embedded:<id>` path, or a pack directory. Response `data` includes
    `operation` (`"templates_show"`), `id`, `manifest`, `theme_schema`, and
    `example_generate_argv`
    (`["docli", "generate", "--input", "model.json", "--html", "site/cli"]`).
  - `templates validate <PATH>` checks a pack directory or `embedded:<id>`.
    Success `data` includes `operation` (`"templates_validate"`), `path` (the
    argument as passed), `id`, `version`, and `valid` (`true`).
  - An on-disk manifest or template that does not compile is
    `DOCLI.TEMPLATE_INVALID` (`validation`, exit `2`). `details` is
    `{ "cause" }`. `suggested_action` names the pack directory and
    `templates validate`. A filesystem read failure remains `DOCLI.IO`.
    Rendering the embedded default pack from `generate` remains
    `DOCLI.INTERNAL`. An unknown bundled id passed to `templates show` is
    `DOCLI.TEMPLATE_INVALID` and tells the caller to run `templates list`.
    `DOCLI.TEMPLATE_NOT_FOUND` for `generate --template` arrives in sprint b.9.

### Error code inventory (normative)

| Code | Exit | `kind` | `details` (required keys) | Covered by |
|------|------|--------|---------------------------|------------|
| `DOCLI.USAGE` | 2 | `validation` | `{}` | `docli_usage_unknown_command_json`, `docli_usage_invalid_flag_json`, `docli_usage_show_without_paths_json`, `docli_usage_show_without_paths_human` |
| `DOCLI.INPUT_INVALID` | 2 | `validation` | `{}` or `{ "cause" }` | `docli_input_invalid_parse_error_json`, `docli_input_invalid_empty_file_json`, `docli_stdin_empty_is_input_invalid`, `docli_input_invalid_human` |
| `DOCLI.INPUT_NOT_FOUND` | 3 | `not_found` | `{ "path" }` | `docli_input_not_found_json`, `docli_input_not_found_human` |
| `DOCLI.OUTPUT_NOT_FOUND` | 3 | `not_found` | `{ "artifacts": [{ "path", "exists" }] }` | `docli_output_not_found_single_html_json`, `docli_output_not_found_html_and_markdown_json`, `docli_output_not_found_human` |
| `DOCLI.IO` | 4 | `dependency` | `{ "cause" }`; optional `{ "outputs_written" }` after partial write | `docli_io_generate_html_dir_not_writable_json`, `docli_io_generate_partial_write_lists_outputs_written_json`, `docli_io_show_unreadable_index_json`, `docli_io_show_unreadable_index_human` |
| `DOCLI.INTERNAL` | 1 | `internal` | `{ "cause" }` | `docli_internal_error_body_contract` |
| `DOCLI.TEMPLATE_INVALID` | 2 | `validation` | `{ "cause" }` | `docli_template_invalid_validate_json`, `docli_template_invalid_validate_human`, `templates_show_unknown_id_is_template_invalid` |

Adding a new `DOCLI.*` code requires updating this table, `src/contract.rs`,
and a matching test in `tests/error_contract.rs` before merge.

## 10. Distribution

- `REQ-DOCLI-DIST-001`: every implementation is licensed MIT.

## 11. Requirement index

Use this table for traceability in QA findings (`source_refs` must cite ids).
Every `REQ-DOCLI-*` id in this repository is one row. Ids named in a Phase B
sprint **Closes** section include a “text lands in sprint …” note. Sprints
**b.7–b.11** add the requirement body and confirm the same index row in that
sprint’s PR. An index row is not a substitute for the body.

| ID | Summary | Text | ADR |
|----|---------|------|-----|
| `REQ-DOCLI-NORM-001` | CLI conforms to the vendored creating-ai-clis skill | text lands in sprint b.1 (section 9) | ADR-003 |
| `REQ-DOCLI-PRODUCT-001` | Rust library and CLI in this repo; registry install is later | product text is in section 3; registry-install text lands in sprint b.11 (b.5 records pre-publish evidence only) | — |
| `REQ-DOCLI-PRODUCT-002` | `docli` and `cargo docli` perform the same operations | section 3 | ADR-001 |
| `REQ-DOCLI-PRODUCT-003` | In-process `ops`, `from_clap`, and render match the CLI | section 3 | ADR-001 |
| `REQ-DOCLI-PRODUCT-004` | Go, .NET, and Python peer implementations (later) | section 3 | — |
| `REQ-DOCLI-INPUT-001` | Command fields on the neutral model | section 4 | — |
| `REQ-DOCLI-INPUT-002` | Option fields on the neutral model | section 4 | — |
| `REQ-DOCLI-INPUT-003` | Argument fields on the neutral model | section 4 | — |
| `REQ-DOCLI-INPUT-004` | Input from a file path or stdin (`-`) | section 4 | — |
| `REQ-DOCLI-INPUT-005` | Unknown input fields are preserved | section 4 | — |
| `REQ-DOCLI-RUST-001` | `docli::from_clap` maps clap fields | section 5 | — |
| `REQ-DOCLI-RUST-002` | Clap adapter drops auto `--help`/`--version` and keeps usage paths | section 5 | — |
| `REQ-DOCLI-GO-001` | Go cobra adapter and generator (later) | section 5 | — |
| `REQ-DOCLI-NET-001` | .NET System.CommandLine adapter and generator (later) | section 5 | — |
| `REQ-DOCLI-PY-001` | Python adapter and generator (later) | section 5 | — |
| `REQ-DOCLI-HTML-001` | Self-contained single-file HTML with hash routes | section 6 | ADR-002 |
| `REQ-DOCLI-HTML-002` | Two-pane tree and detail layout | section 6 | ADR-002 |
| `REQ-DOCLI-HTML-003` | Indented tree, selection, and expand/collapse | section 6 | ADR-002 |
| `REQ-DOCLI-HTML-004` | Detail panel fields for arguments and options | section 6 | ADR-002 |
| `REQ-DOCLI-HTML-005` | Search keeps matching nested commands and ancestors | section 6 | ADR-002 |
| `REQ-DOCLI-HTML-006` | Default HTML directory is `site/cli` | section 6 | ADR-002 |
| `REQ-DOCLI-HTML-007` | Embedded `default` template pack; byte match without `--template` | section 6 | ADR-004 |
| `REQ-DOCLI-HTML-008` | `cli-doc` pack layout markers, still self-contained | text lands in sprint b.10 | — |
| `REQ-DOCLI-HTML-009` | `_skeleton` validates and is omitted from `templates list` | text lands in sprint b.10 | — |
| `REQ-DOCLI-HTML-010` | Author docs match the template pack schema | text lands in sprint b.10 | ADR-004 (b.7) |
| `REQ-DOCLI-MD-001` | Flat Markdown reference, one section per command | section 7 | — |
| `REQ-DOCLI-MD-002` | Markdown shipped with each language installer (later) | section 7 | — |
| `REQ-DOCLI-GEN-001` | Same input bytes produce the same HTML and Markdown bytes | section 8 | ADR-002 |
| `REQ-DOCLI-GEN-002` | Consumers regenerate on release; CI rejects stale output | generation text is in section 8; fixture-policy text lands in sprint b.5 | ADR-002 |
| `REQ-DOCLI-GEN-003` | Later languages match the Phase A fixture corpus (later) | section 8 | — |
| `REQ-DOCLI-CLI-001` | Global `--json`; stdout is only the envelope | section 9 | ADR-001, ADR-003 |
| `REQ-DOCLI-CLI-002` | Envelope version `"1"` with `ok`, `data`, and `error` | section 9 | ADR-003 |
| `REQ-DOCLI-CLI-003` | Actionable `error` object and stable `DOCLI.*` codes | section 9 | ADR-003 |
| `REQ-DOCLI-CLI-004` | Exit codes `0`, `2`, `3`, `4`, `1` by outcome | section 9 | ADR-003 |
| `REQ-DOCLI-CLI-005` | `generate` writes HTML under `--html` (default `site/cli`) | section 9 | ADR-001 |
| `REQ-DOCLI-CLI-006` | `show` readback of written artifacts | section 9 | ADR-001 |
| `REQ-DOCLI-CLI-007` | Human output uses only `--json` fields | section 9 | ADR-001 |
| `REQ-DOCLI-CLI-008` | Every stable code has `tests/error_contract.rs` coverage | text lands in sprint b.1 (section 9); test closure is sprint b.2 | ADR-003 |
| `REQ-DOCLI-CLI-009` | `--json` failures are one stdout envelope; human mode uses stderr | text lands in sprint b.1 (section 9); test closure is sprint b.2 (also cited by b.3) | ADR-003 |
| `REQ-DOCLI-CLI-010` | `cargo docli` matches `docli` exit and stdout on error scenarios | text lands in sprint b.1 (section 9); test closure is sprint b.2 | ADR-003 |
| `REQ-DOCLI-CLI-011` | `docli templates` list, show, and validate | section 9 | ADR-001 |
| `REQ-DOCLI-CLI-012` | `generate --template`, `--theme`, and `--preview` | text lands in sprint b.9 | ADR-001 |
| `REQ-DOCLI-DIST-001` | MIT license | section 10 | — |
