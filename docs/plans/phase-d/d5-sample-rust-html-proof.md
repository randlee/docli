---
id: d.5
title: sample command tree and Rust HTML proof
status: planned
branch: feature/phase-d-d5-html-proof
worktree: ../docli-worktrees/feature/phase-d-d5-html-proof
target: integrate/phase-d
---

# Sprint d.5 — sample command tree and Rust HTML proof

## Goal

A sample `System.CommandLine` tree becomes JSON through `FromCommand`, and the Rust `docli generate` command writes HTML and Markdown from that file.

## Closes

No requirement id. This sprint implements the HTML proof named by `ADR-005`. The Rust generator is already on `develop`.

## Hard Dependencies

- d.4 — the tool is on `integrate/phase-d`. This proof does not invoke the tool. It calls `CommandLineAdapter.FromCommand` and the Rust `docli` binary.

## Deliverables

- Add class library `dotnet/samples/Docli.Sample/Docli.Sample.csproj` to `dotnet/Docli.sln`. `net10.0`, `Nullable` enable, `ImplicitUsings` enable. `ProjectReference` to `dotnet/src/Docli/Docli.csproj`. Not `PackAsTool`. No direct `System.CommandLine` package reference.
- `Docli.Tests` gains a `ProjectReference` to the sample and `HtmlProofTests.cs`.
- The sample tree is a `Command`, not a `RootCommand`:

```csharp
namespace Docli.Sample;

public static class SampleCommands
{
    public static Command Create();
}
```

- `Create()` returns command name `sample`, description `Sample command tree`, with option `Option<string>("--output", "-o")` (`Description` `Write output to PATH`, `HelpName` `PATH`, `Required` false) and subcommand `run`, description `Run the thing`. No arguments and no further subcommands.
- `Sample_json_generates_html_and_markdown` finds the repo root by walking parents for `Cargo.toml`, writes `CliModelJson.Serialize(CommandLineAdapter.FromCommand(SampleCommands.Create()))` to a temp file, and runs, with that root as the working directory:

```text
cargo run --quiet --bin docli -- generate --input <json> --html <dir> --markdown <md>
```

- The test fails when `cargo` is not on `PATH`. It does not compare HTML bytes to `fixtures/contract/index.html`.

## Out of Scope

- Changes under `src/`, `templates/`, or `fixtures/contract/`
- Invoking `Docli.Tool` to produce the proof JSON
- A byte match against the Phase A HTML corpus
- A .NET HTML or Markdown renderer, a template-engine port, Go or Python adapters, an MCP wrapper
- Closing `REQ-DOCLI-PRODUCT-004`, `REQ-DOCLI-NET-001`, or `REQ-DOCLI-GEN-003`

## Acceptance Criteria

- The cargo command exits 0
- `<dir>/index.html` exists and contains `id="docli-data"`, `sample`, `--output`, and `run`
- `<md>` exists and contains `# sample CLI Reference` and `` `sample run` ``
- `rg` under `dotnet/` finds no `html::render`, `page.html.j2`, or `MiniJinja`

## Required Validation

- Phase D host gate for d.2–d.5, including the d.5 Rust commands — [README.md](README.md)
- `dotnet test dotnet/Docli.sln -c Release --filter Sample_json_generates_html_and_markdown`
- `rg -n "html::render|page\\.html\\.j2|MiniJinja" dotnet` returns no matches
