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

`REQ-DOCLI-NET-002` — the tool's own `docli model` JSON. Command name stays `docli`. It does not add the sample proof.

## Hard Dependencies

- d.3 — `CommandLineAdapter.FromCommand` on `integrate/phase-d`.

## Deliverables

- Add `dotnet/src/Docli.Tool/Docli.Tool.csproj` to `dotnet/Docli.sln`. SDK `Microsoft.NET.Sdk`, `OutputType` `Exe`, `TargetFramework` the ADR-005 TFM pin, `Nullable` enable, `ImplicitUsings` enable, `PackAsTool` true, `ToolCommandName` `docli`, `PackageId` `Docli.Tool`. `ProjectReference` to `../Docli/Docli.csproj`. No direct `System.CommandLine` package reference. No new test packages.
- Tests live in `dotnet/tests/Docli.Tests/ToolCommandTests.cs`. The repo root is `RepoRoot.Find()` from d.2.
- `Docli.Tests` gains a `ProjectReference` to `dotnet/src/Docli.Tool/Docli.Tool.csproj` and no other new packages. Tests call `ToolCommands.ToolVersion()` in-process.
- `dotnet/tests/Docli.Tests/RustDocli.cs` is the only Rust binary selection. d.5 calls it and does not restate the rule.

```csharp
namespace Docli.Tests;

public readonly record struct GenerateResult(int ExitCode, string Stdout, string Stderr);

public static class RustDocli
{
    public static GenerateResult Generate(string repoRoot, string inputPath, string htmlDir, string? markdownPath, TimeSpan timeout);
}
```

  If `target/debug/docli` exists under `repoRoot` (`docli.exe` on Windows), run that file. Otherwise run `cargo run --quiet --bin docli --` with working directory `repoRoot`. The helper still prefers that existing file, so a stale binary is a failed proof unless Required Validation has just run `cargo build --bin docli`. Pass arguments with `ProcessStartInfo.ArgumentList` (`generate`, `--input`, `inputPath`, `--html`, `htmlDir`, plus `--markdown` and `markdownPath` when that path is non-null). Do not build a shell string. `UseShellExecute` is false. Redirect stdout and stderr and read both with `ReadToEndAsync` before waiting, so a full pipe cannot deadlock. On timeout, `Process.Kill(entireProcessTree: true)` and throw `TimeoutException` whose message includes both streams. If the process cannot start, throw, and include the command and any captured streams. A non-zero exit returns `GenerateResult` and does not throw.
- The tool root is a `Command` named `docli`, not a `RootCommand`, so the model name stays `docli` when the process name differs.
- Public surface:

```csharp
namespace Docli.Tool;

public static class ToolCommands
{
    public static Command Create();
    public static string? ToolVersion();
}

public static class ModelCommand
{
    public const string Name = "model";

    public static readonly Option<string> OutputOption;

    public static int Write(CliModel model, TextWriter stdout, TextWriter stderr, string? outputPath);
}

public static class Program
{
    public static int Main(string[] args)
    {
        try
        {
            return ToolCommands.Create().Parse(args).Invoke();
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine("docli: model failed: " + ex.Message);
            return 2;
        }
    }
}
```

- `Create()` builds command `docli` (description `Emit a System.CommandLine tree as docli JSON`) and adds `new System.CommandLine.Help.HelpOption()` on that command. It does not add `VersionOption`. The tool's own command tree does not install a throwing `DefaultValueFactory` on any option or argument. Subcommand `model` (description `Write CliModel JSON for this tool`) uses `ModelCommand.OutputOption`, an `Option<string>("--output")` with `HelpName` `PATH`, description `Write JSON to this path instead of stdout`, not required. No other subcommands. `FromCommand` omits `HelpOption` from the JSON.
- The `model` action is `SetAction(Func<ParseResult, int>)`. It reads `--output` with `parseResult.GetValue(ModelCommand.OutputOption)`, builds `CommandLineAdapter.FromCommand(ToolCommands.Create(), ToolCommands.ToolVersion())`, and returns `ModelCommand.Write` of that `CliModel`. That int is the process exit code when `Invoke` returns. If `FromCommand` or `Serialize` throws anyway, `Program.Main` writes one stderr line prefixed `docli: model failed:` and returns 2, as in the `Main` body above. `Write` handles a `Serialize` throw itself and does not rethrow, so that row does not reach `Main`. For host trees, [d.3](d3-commandline-walker.md) says an exception from `DefaultValueFactory` propagates unchanged from the adapter.
- `ToolVersion()` returns `AssemblyInformationalVersion` of the `Docli.Tool` assembly when that attribute is non-empty; otherwise `AssemblyName.Version`; otherwise null. It does not read the entry assembly.
- `Write` serializes with `CliModelJson.Serialize` and appends one `\n`. UTF-8, no BOM. `outputPath` null: that text is the only stdout write, return 0. `outputPath` set: write the file, do not write the JSON to stdout, return 0. A failure writes no file and no stdout JSON, returns 2, and writes one stderr line that starts with the prefix in the row. Tests assert that prefix, exit code 2, and the recovery in the row. Exit code 2 with non-empty stderr is not enough.

| Failure | Stderr prefix | Exit | Recovery a caller or test asserts |
|---------|---------------|------|-------------------------------------|
| Missing parent directory | `docli: output parent missing:` | 2 | The output path is not created. Creating the parent directory and calling `Write` again with the same file path returns 0 and writes the file. |
| Path is a directory | `docli: output path is a directory:` | 2 | The directory is unchanged and stdout has no JSON object. A later call with a file path returns 0 and writes the file. |
| Unauthorized | `docli: output unauthorized:` | 2 | The failed call creates no file and stdout has no JSON object. A later call with a writable file path returns 0 and writes the file. |
| Serialize failure | `docli: serialize failed:` | 2 | `CliModelJson.Serialize` threw, no file is created, and stdout has no JSON object. `Write` does not rethrow. A later call with an acyclic `CliModel` that `Serialize` accepts, to a writable file path, returns 0 and writes the file. |

- The dotnet tool has no `--html`, `--markdown`, `generate`, or `show` command and does not print the Rust version `"1"` envelope. Automation must not depend on the parse-error exit code of `Parse().Invoke()`. That exit stays undocumented. Help and unknown-command behavior are System.CommandLine defaults and are not a Phase D acceptance gate.
- Tests start the built tool with `dotnet <RepoRoot>/dotnet/src/Docli.Tool/bin/Release/<tfm>/Docli.Tool.dll <args>`, where `<tfm>` is the ADR-005 pin. They require `dotnet test -c Release`. If that DLL is missing, the test fails with a message that names the path and says the run requires `-c Release`. They do not call `dotnet run`. Each test's timeout is 60 seconds, except `Model_json_generates_html` and `Pack_produces_tool_package`, which are 180 seconds. Redirect stdout and stderr.
- `Generate_captures_rust_failure` asserts a non-zero exit and non-empty stderr only. Phase D proofs use the human stderr from `docli generate`. They do not pass `--json` and do not assert `error.code`. `RustDocli.Generate` does not pass `--json`. `Generate_captures_rust_failure` and `Pack_produces_tool_package` are `[Fact]` methods in `ToolCommandTests`. `Missing_output_directory_exits_2`, `Write_failure_exits_2`, `Unauthorized_output_exits_2`, and `Serialize_failure_exits_2` are `[Fact]` methods in `ToolCommandTests` and assert the failure-mode row (stderr prefix, exit code 2, and that row's recovery). `Pack_produces_tool_package` runs `dotnet pack dotnet/src/Docli.Tool/Docli.Tool.csproj -c Release -o <temp>`, where `<temp>` is outside the repo. It unzips the `.nupkg` and asserts `tools/<tfm>/any/DotnetToolSettings.xml` contains `<Command Name="docli"` and the nuspec package id is `Docli.Tool`. It deletes `<temp>` before returning. The nupkg is not written under a tracked or non-ignored path.

## Out of Scope

- Phase D non-closures: see [README.md](README.md)
- `dotnet/samples/`, Rust edits under `src/` or `templates/`, and a loader that reads a third-party assembly
- NuGet publish
- Automation that depends on the parse-error exit code of `Parse().Invoke()`. That exit stays undocumented. Help and unknown-command behavior are System.CommandLine defaults and are not a Phase D acceptance gate.

## Acceptance Criteria

- `Model_command_writes_json`: `docli model` exits 0, stdout parses as `CliModel` with name `docli`, a subcommand `model`, and an option `long` `--output`, and `version` equals `ToolVersion()`
- `Output_flag_writes_a_file`: `--output` writes that JSON to the path and leaves stdout without a `CliModel` `name` key
- `Missing_output_directory_exits_2`: a missing parent directory exits 2, stderr is one line prefixed `docli: output parent missing:`, and the path is not created. Creating that parent and writing the same model to a file path there returns 0 and writes the file.
- `Write_failure_exits_2`: `--output` pointing at a directory exits 2, stderr is one line prefixed `docli: output path is a directory:`, stdout has no JSON object, and the directory is unchanged. A later call with a file path returns 0 and writes the file.
- `Unauthorized_output_exits_2`: `--output` under a parent directory that denies write to the current user exits 2, stderr is one line prefixed `docli: output unauthorized:`, stdout has no JSON object, and the path is not created. The same model written to a writable file path returns 0 and writes the file.
- `Serialize_failure_exits_2`: `ModelCommand.Write` of a `CliModel` that lists itself in `subcommands`, so `CliModelJson.Serialize` throws, exits 2, stderr is one line prefixed `docli: serialize failed:`, stdout has no JSON object, and no file is created. `Write` of an acyclic model to a writable file path returns 0 and writes the file.
- `Tool_has_no_html_or_markdown_command`: the test may run `docli --help` and `docli model --help` to read the command list. Stdout contains `model` and `--output`. Absence of `generate`, `show`, `--html`, and `--markdown` is an ordinal, case-sensitive substring check on stdout. The default help text `Show help and usage information` is allowed. Help and unknown-command behavior are System.CommandLine defaults and are not a Phase D acceptance gate. The test does not assert those exit codes.
- `Model_json_generates_html`: `docli model --output <file>` exits 0 and the file is the tool's own tree (`name` is `docli`). `RustDocli.Generate(RepoRoot.Find(), file, dir, null, TimeSpan.FromSeconds(180)).ExitCode` is 0 and `<dir>/index.html` exists. This check does not pass a third-party command tree.
- `Generate_captures_rust_failure` asserts a non-zero exit and non-empty stderr only. `RustDocli.Generate` with an input path that does not exist returns a non-zero `ExitCode` and a non-empty `Stderr`. Phase D proofs use the human stderr from `docli generate`. They do not pass `--json` and do not assert `error.code`.
- `Pack_produces_tool_package`: the packed `DotnetToolSettings.xml` contains `<Command Name="docli"` and the package id is `Docli.Tool`

## Required Validation

- Phase D host gate for d.4 and d.5 — [README.md](README.md)
- `cargo build --bin docli` immediately before the filtered `dotnet test` below. A filtered test without that build is not a valid proof.
- `dotnet test dotnet/Docli.sln -c Release --filter ToolCommandTests`
- `dotnet pack dotnet/src/Docli.Tool/Docli.Tool.csproj -c Release`
- `rg -n "<ToolCommandName>docli</ToolCommandName>" dotnet/src/Docli.Tool/Docli.Tool.csproj`
- `rg -n "<PackAsTool>true</PackAsTool>" dotnet/src/Docli.Tool/Docli.Tool.csproj`
