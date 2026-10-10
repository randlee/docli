---
id: d.4
title: dotnet tool emits JSON
status: planned
branch: feature/phase-d-d4-tool
worktree: ../docli-worktrees/feature/phase-d-d4-tool
target: integrate/phase-d
---

# Sprint d.4 — dotnet tool emits JSON

## Goal

`dotnet/src/Docli.Tool` packs as a dotnet tool named `docli` and writes `CliModel` JSON for its own command tree. Rust `docli generate --input` accepts that file. The tool does not document any other app.

## Closes

No requirement id. This sprint implements the tool argv named by `ADR-005`. It does not add the sample proof.

## Hard Dependencies

- d.3 — `CommandLineAdapter.FromCommand` on `integrate/phase-d`.

## Deliverables

- Add `dotnet/src/Docli.Tool/Docli.Tool.csproj` to `dotnet/Docli.sln`. SDK `Microsoft.NET.Sdk`, `OutputType` `Exe`, `TargetFramework` the ADR-005 TFM pin, `Nullable` enable, `ImplicitUsings` enable, `PackAsTool` true, `ToolCommandName` `docli`, `PackageId` `Docli.Tool`. `ProjectReference` to `../Docli/Docli.csproj`. No direct `System.CommandLine` package reference. No new test packages.
- Tests live in `dotnet/tests/Docli.Tests/ToolCommandTests.cs`. The repo root is `RepoRoot.Find()` from d.2.
- The tool root is a `Command` named `docli`, not a `RootCommand`, so the model name stays `docli` when the process name differs.
- Public surface:

```csharp
namespace Docli.Tool;

public static class ToolCommands
{
    public static Command Create();
    public static string ToolVersion();
}

public static class ModelCommand
{
    public const string Name = "model";

    public static int Write(Command source, TextWriter stdout, TextWriter stderr, string? outputPath);
}
```

- `Create()` builds command `docli` (description `Emit a System.CommandLine tree as docli JSON`) and adds `new System.CommandLine.Help.HelpOption()` on that command. It does not add `VersionOption`. Subcommand `model` (description `Write CliModel JSON for this tool`) has option `--output` (`HelpName` `PATH`, description `Write JSON to this path instead of stdout`, not required). No other subcommands. `FromCommand` omits `HelpOption` from the JSON.
- `ToolVersion()` returns `AssemblyInformationalVersion` of the `Docli.Tool` assembly when that attribute is non-empty; otherwise `AssemblyName.Version`; otherwise null. It does not read the entry assembly.
- `Program` invokes `ToolCommands.Create().Parse(args).Invoke()`. The `model` action calls `ModelCommand.Write` with `FromCommand(ToolCommands.Create(), ToolCommands.ToolVersion())`.
- `Write` serializes with `CliModelJson.Serialize` and appends one `\n`. UTF-8, no BOM. `--output` omitted: that text is the only stdout write, return 0. `--output` set: write the file, do not write the JSON to stdout, return 0. Any `Write` failure, including a missing parent directory, a path that is a directory, or an unauthorized path: one line on stderr, no file, no stdout JSON, return 2. The dotnet tool has no `--html`, `--markdown`, `generate`, or `show` command and does not print the Rust version `"1"` envelope. The exit code of `Parse().Invoke()` for a parse error is not covered.
- Tests start the built tool with `dotnet <RepoRoot>/dotnet/src/Docli.Tool/bin/Release/<tfm>/Docli.Tool.dll <args>`, where `<tfm>` is the ADR-005 pin. They do not call `dotnet run`. Each test's timeout is 60 seconds, except `Model_json_generates_html`, which is 180 seconds. Redirect stdout and stderr.

## Out of Scope

- Phase D non-closures: see [README.md](README.md)
- `dotnet/samples/`, Rust edits under `src/` or `templates/`, and a loader that reads a third-party assembly
- NuGet publish
- The parse-error exit code of `System.CommandLine` `Parse().Invoke()`

## Acceptance Criteria

- `Model_command_writes_json`: `docli model` exits 0, stdout parses as `CliModel` with name `docli`, a subcommand `model`, and an option `long` `--output`, and `version` equals `ToolVersion()`
- `Output_flag_writes_a_file`: `--output` writes that JSON to the path and leaves stdout without a `CliModel` `name` key
- `Missing_output_directory_exits_2`: a missing parent directory exits 2, stderr is non-empty, and the path is not created
- `Write_failure_exits_2`: `--output` pointing at a directory exits 2, stderr is non-empty, and stdout has no JSON object
- `Tool_has_no_html_or_markdown_command`: `docli --help` exits 0 and stdout contains `model`; `docli model --help` exits 0 and stdout contains `--output`; neither stdout contains `generate`, `show`, `--html`, or `--markdown`
- `Model_json_generates_html`: `docli model --output <file>` exits 0 and the file is the tool's own tree (`name` is `docli`). Rust `docli generate --input <file> --html <dir>` then exits 0 and `<dir>/index.html` exists. Use `target/debug/docli` when that binary is already built; otherwise `cargo run --quiet --bin docli --` from `RepoRoot.Find()`. This check does not pass a third-party command tree

## Required Validation

- Phase D host gate for d.2–d.5, including the d.4 Rust commands — [README.md](README.md)
- `dotnet test dotnet/Docli.sln -c Release --filter ToolCommandTests`
- `dotnet pack dotnet/src/Docli.Tool/Docli.Tool.csproj -c Release`
- `rg -n "<ToolCommandName>docli</ToolCommandName>" dotnet/src/Docli.Tool/Docli.Tool.csproj`
- `rg -n "<PackAsTool>true</PackAsTool>" dotnet/src/Docli.Tool/Docli.Tool.csproj`
