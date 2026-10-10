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

`REQ-DOCLI-NET-002` — the sample command tree and the Rust `docli generate` HTML and Markdown proof. The Rust generator is already on `develop`. This sprint does not close `REQ-DOCLI-NET-001`.

## Hard Dependencies

- d.4 — `RustDocli` (test helper) on `integrate/phase-d`. d.5 depends on d.4 because it calls `RustDocli.Generate`, which d.4 owns. This proof does not invoke the tool and does not read `docli model` output. It calls `CommandLineAdapter.FromCommand(SampleCommands.Create())`.

## Deliverables

- Add class library `dotnet/samples/Docli.Sample/Docli.Sample.csproj` to `dotnet/Docli.sln`. `TargetFramework` is the ADR-005 TFM pin, `Nullable` enable, `ImplicitUsings` enable. `ProjectReference` to `dotnet/src/Docli/Docli.csproj`. Not `PackAsTool`. No direct `System.CommandLine` package reference. No new test packages.
- `Docli.Tests` gains a `ProjectReference` to the sample and `HtmlProofTests.cs`. The repo root is `RepoRoot.Find()` from d.2.
- The sample tree is a `Command`, not a `RootCommand`:

```csharp
namespace Docli.Sample;

public static class SampleCommands
{
    public static Command Create();
}
```

- `Create()` returns command name `sample`, description `Sample command tree`, with option `Option<string>("--output", "-o")` (`Description` `Write output to PATH`, `HelpName` `PATH`, `Required` false) and subcommand `run`, description `Run the thing`. No arguments and no further subcommands.
- `Sample_json_generates_html_and_markdown` writes `CliModelJson.Serialize(CommandLineAdapter.FromCommand(SampleCommands.Create()))` to a temp file and calls `RustDocli.Generate` from d.4: `RustDocli.Generate(RepoRoot.Find(), json, dir, md, TimeSpan.FromSeconds(180))`. It does not choose between `target/debug/docli` and `cargo run` itself. It does not compare HTML bytes to `fixtures/contract/index.html`. `cargo build --bin docli` in Required Validation runs immediately before this test.

## Out of Scope

- Phase D non-closures: see [README.md](README.md)
- Changes under `src/`, `templates/`, or `fixtures/contract/`
- Invoking `Docli.Tool` to produce the proof JSON
- A byte match against the Phase A HTML corpus

## Acceptance Criteria

- `RustDocli.Generate(...).ExitCode` is 0
- The `id="docli-data"` JSON deserializes to the same `CliModel` the sample produced: name `sample`, an option whose `long` is `--output`, and a subcommand named `run`
- `<dir>/index.html` contains `<title>sample CLI Reference</title>`
- `<md>` exists and contains `# sample CLI Reference` and `` `sample run` ``
- The `ARCH-RULE-010` gate from d.1 passes on `dotnet/src` and `dotnet/samples`

## Required Validation

- Phase D host gate for d.4 and d.5 — [README.md](README.md)
- `cargo build --bin docli` immediately before the filtered `dotnet test` below. A filtered test without that build is not a valid proof.
- `dotnet test dotnet/Docli.sln -c Release --filter Sample_json_generates_html_and_markdown`
- `rg -n 'html::render|markdown::render|MiniJinja|\.j2|<!DOCTYPE|<html|CLI Reference|Scriban|Fluid|Razor' dotnet/src dotnet/samples` exits 1
- `find dotnet -type d -name templates -print` prints nothing
- `rg -n 'Include=.*\.(html|md)' dotnet/src dotnet/samples -g '*.csproj'` exits 1. These three commands are the `ARCH-RULE-010` gate. They do not scan `dotnet/tests`.
