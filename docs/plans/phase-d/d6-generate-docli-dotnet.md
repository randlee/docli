---
id: d.6
title: Rust generate orchestrates docli-dotnet project JSON
status: complete
branch: feature/phase-d-generate-docli-dotnet
worktree: ../docli-worktrees/feature/phase-d-generate-docli-dotnet
target: integrate/phase-d
---

# Sprint d.6 — Rust `generate` orchestrates `docli-dotnet project`

## Goal

A developer runs one Rust command, `docli generate --dotnet-project <csproj>`, that shells out to the `docli-dotnet` global tool (or the built tool DLL in tests), obtains `CliModel` JSON from a .NET export host, then uses the existing Rust render path. The .NET side still does not render HTML or Markdown.

## Closes

`REQ-DOCLI-NET-002` — the orchestration slice: Rust `docli generate` drives `docli-dotnet` to emit JSON from a .NET project, then renders HTML (and optional Markdown). Does not close `REQ-DOCLI-NET-001`, `REQ-DOCLI-PRODUCT-004`, or walking arbitrary third-party assemblies without an export host.

## Hard Dependencies

- d.4 — `docli-dotnet model`, tool DLL test invocation pattern, `RustDocli`.
- d.5 — `Docli.Sample` / `SampleCommands.Create()` sample tree.
- `integrate/phase-d` @ rename commit: global tool command name `docli-dotnet` (not `docli`).

## Deliverables

### .NET export host

- Add console project `dotnet/samples/Docli.Sample.Export/Docli.Sample.Export.csproj` to `dotnet/Docli.sln`. `OutputType` `Exe`. ADR-005 TFM pin, `Nullable` enable, `ImplicitUsings` enable. `ProjectReference` to `dotnet/samples/Docli.Sample/Docli.Sample.csproj` and transitively `Docli`. Not `PackAsTool`. No direct `System.CommandLine` package reference.
- `Program.cs` writes **only** `CliModelJson.Serialize(CommandLineAdapter.FromCommand(SampleCommands.Create()))` to stdout with no extra lines or stderr on success. Exit code 0.

### `docli-dotnet project`

- Add subcommand `project` on the tool root (`ToolCommands.Create()`). Description: `Run a .NET export host and write CliModel JSON`.
- Required option `--project` / `-p` (`HelpName` `CSPROJ`): path to the export `.csproj`.
- Optional `--output` / `-o` (`HelpName` `FILE`): same semantics as `model` — write JSON to a file; when omitted, write JSON to stdout only (no `CliModel` `name` key on stdout when `--output` is set).
- Implementation runs `dotnet build <csproj> -c Release` then `dotnet run --project <csproj> -c Release --no-build` with working directory at the repo root or the csproj directory consistently (document the choice in code comments only if non-obvious). Captures child stdout as the JSON payload. Non-zero `dotnet` exit or non-JSON stdout is a failure.
- Human stderr prefix stays `docli-dotnet: ` (reuse `ModelCommand` patterns where sensible). Failure modes at minimum: missing csproj, build failure, run failure, empty stdout, JSON parse failure after run. Exit code 2 for tool-handled failures. Do not add HTML/Markdown/`generate`/`show` commands on the dotnet tool.
- `Tool_has_no_html_or_markdown_command` (or equivalent) still passes; help for `project` lists `--project` and `--output`.

### Rust CLI and ops

- `docli generate` accepts exactly one input source: existing `--input FILE` (including `-` for stdin) **or** new `--dotnet-project CSPROJ`. Clap must reject passing both a non-stdin input file and `--dotnet-project` together (`DOCLI.USAGE` / usage envelope when `--json`).
- When `--dotnet-project` is set, Rust resolves and runs `docli-dotnet project --project <abs-or-rel-path>`, captures stdout to a temp file (or memory then existing path), then calls the existing `generate` body with `InputSource::File`. Resolution order mirrors tests: if `docli-dotnet` is on `PATH`, use it; else run `dotnet <repo>/dotnet/src/Docli.Tool/bin/Release/<tfm>/Docli.Tool.dll project ...` where `<tfm>` is the ADR-005 pin and repo root is discovered relative to the running binary or current working directory per existing project conventions.
- Subprocess failures (missing tool, non-zero exit, invalid JSON) map to existing `DOCLI.IO` or `DOCLI.INPUT` as appropriate; do not add new `DOCLI.*` codes in this sprint unless `docs/requirements.md` section 9 is updated in the same PR (prefer reusing `DOCLI.IO` with a clear message).
- `cargo docli generate` shares the same flags via shared `cli.rs`.

### Tests

- `dotnet/tests/Docli.Tests/ProjectCommandTests.cs`: at least `Project_stdout_json_is_sample_tree`, `Project_output_flag_writes_file`, `Project_missing_csproj_exits_2`. Invoke the built Release tool DLL like d.4 (`dotnet test -c Release`). Timeout 180s for build+run tests.
- `dotnet/tests/Docli.Tests/DotnetGenerateOrchestrationTests.cs` (or extend `HtmlProofTests`): `Generate_dotnet_project_writes_html` calls `RustDocli` with new argv helper `RustDocli.GenerateDotnetProject(repoRoot, csproj, htmlDir, markdown, timeout)` that runs `docli generate --dotnet-project <csproj> ...`. Assert exit 0 and `index.html` exists; optionally assert embedded `docli-data` deserializes to a model whose root name is `sample`.
- Rust `tests/cli_contract.rs` (or dedicated integration test): one test that runs `docli generate --dotnet-project` against `Docli.Sample.Export.csproj` when `docli-dotnet` DLL path exists (skip or build prerequisite documented in test). Prefer the same repo-root discovery pattern as other contract tests.

### Docs and index

- [`docs/requirements.md`](../../requirements.md): extend the `REQ-DOCLI-NET-002` bullet with one physical line addition naming `docli generate --dotnet-project` orchestrating `docli-dotnet project` for JSON; Rust remains the user-facing generate product.
- [`docs/plans/phase-d/README.md`](README.md): sprint index row for d.6; host-gate subsection for d.6 (below).
- [`../project-plan.md`](../project-plan.md): sprint table row for d.6 with this branch and worktree.

## Out of Scope

- Phase D non-closures: see [README.md](README.md)
- Loading and reflecting arbitrary third-party app assemblies without an export console host
- NuGet publish, global `dotnet tool install` proofs, CI .NET job
- .NET HTML/Markdown rendering; Rust changes to template packs beyond what orchestration requires
- Updating historical d.1–d.5 sprint doc strings from `docli` to `docli-dotnet` (optional follow-up)

## Acceptance Criteria

- `Project_stdout_json_is_sample_tree`: `docli-dotnet project --project dotnet/samples/Docli.Sample.Export/Docli.Sample.Export.csproj` exits 0; stdout parses as `CliModel` with `name` `sample`, subcommand `run`, option long `--output`.
- `Project_output_flag_writes_file`: `--output` writes the same JSON; stdout has no `CliModel` root `name` key.
- `Project_missing_csproj_exits_2`: missing csproj exits 2 with stderr prefixed `docli-dotnet: `.
- `Generate_dotnet_project_writes_html`: Rust `docli generate --dotnet-project <export.csproj> --html <dir>` exits 0 and `<dir>/index.html` exists.
- Clap rejects `docli generate --input foo.json --dotnet-project bar.csproj` with usage failure.
- `docli-dotnet --help` lists `model` and `project`; still no `generate`, `show`, `--html`, or `--markdown` on the dotnet tool root.

## Required Validation

Phase D host gate for d.6 — from repo root. After these commands, `git status --porcelain` lists no `dotnet/**/bin` or `dotnet/**/obj` path:

```text
cargo build --bin docli
cargo test --test cli_contract -- --nocapture
dotnet build dotnet/Docli.sln -c Release
dotnet test dotnet/Docli.sln -c Release --filter "ProjectCommandTests|DotnetGenerateOrchestrationTests"
git diff --check
```

- `rg -n "dotnet-project" src/cli.rs src/ops.rs`
- `rg -n "project" dotnet/src/Docli.Tool/ToolCommands.cs`
- `rg -n "Docli.Sample.Export" dotnet/Docli.sln`

Paste host-gate output in the PR description.
