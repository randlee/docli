---
id: d.2
title: CliModel JSON types and parity tests
status: planned
branch: feature/phase-d-d2-model
worktree: ../docli-worktrees/feature/phase-d-d2-model
target: integrate/phase-d
---

# Sprint d.2 — CliModel JSON types and parity tests

## Goal

The `Docli` class library deserializes and serializes the neutral model with the snake_case names in `src/schema.rs`.

## Closes

No requirement id. This sprint implements the JSON types named by `ADR-005`. It does not implement the walker, the tool, or the HTML proof.

## Hard Dependencies

- d.1 — adapter-versus-renderer wording and `ADR-005` are on `integrate/phase-d`.

## Deliverables

- Repo-root `global.json` pins the SDK. This is the only SDK pin:

```json
{
  "sdk": {
    "version": "10.0.401",
    "rollForward": "latestPatch",
    "allowPrerelease": false
  }
}
```

  Install the .NET SDK that satisfies `global.json` before the host gate. The version number stays in this sample only.
- `.gitignore` gains these two lines and no broader `bin/` or `obj/` rule:

```text
dotnet/**/bin/
dotnet/**/obj/
```

- `dotnet/Docli.sln` with `dotnet/src/Docli/Docli.csproj` and `dotnet/tests/Docli.Tests/Docli.Tests.csproj` only. Both projects use the ADR-005 TFM pin, `Nullable` enable, `ImplicitUsings` enable. The library is not `PackAsTool`. Neither project references `System.CommandLine`. d.3–d.5 reuse this TFM pin and these test packages; they do not restate the version numbers.
- `Docli.Tests.csproj` is an xUnit v2 VSTest project. Package versions were checked on nuget.org on 2026-10-09. `xunit` 2.9.3 is the current v2 package (NuGet marks it legacy). This sprint does not use `xunit.v3` 4.0.2, because v3's Microsoft Testing Platform does not accept `dotnet test --filter <ClassName>`. An expression with no operator is a contains match on `FullyQualifiedName` for this VSTest adapter (Microsoft Learn, selective unit tests), so `--filter CliModelJsonTests` is valid.

```xml
<PackageReference Include="Microsoft.NET.Test.Sdk" Version="18.10.1" />
<PackageReference Include="xunit" Version="2.9.3" />
<PackageReference Include="xunit.runner.visualstudio" Version="3.1.5">
  <IncludeAssets>runtime; build; native; contentfiles; analyzers; buildtransitive</IncludeAssets>
  <PrivateAssets>all</PrivateAssets>
</PackageReference>
```

- Tests are public classes with `[Fact]` methods. d.3–d.5 add classes to this project and do not add test packages.
- `dotnet/tests/Docli.Tests/RepoRoot.cs` is the only parent walk. d.4 and d.5 call it; they do not walk parents themselves. Rust binary selection is `RustDocli` in d.4, not this file.

```csharp
namespace Docli.Tests;

public static class RepoRoot
{
    public static string Find();
}
```

  `Find` starts at the test assembly directory and walks parents until a directory contains `Cargo.toml`. If none does, it throws `DirectoryNotFoundException`.
- `dotnet/tests/Docli.Tests/CliModelJsonTests.cs` references the library and opens `fixtures/contract/model.json` under `RepoRoot.Find()`.
- Public types in namespace `Docli`. JSON names are the `[JsonPropertyName]` values. CLR names are the property names. Missing `name` fails deserialize with `JsonException`. Every other field uses the `src/schema.rs` default: null for optional strings and counts, `""` for strings, `false` for `required`, empty lists for collections. Unknown keys (names that are not known properties) round-trip through `Extra`. A second copy of a known property name is a `JsonException` from the serializer, not an `Extra` entry.

```csharp
namespace Docli;

public sealed class CliModel
{
    [JsonPropertyName("name")]
    public required string Name { get; init; }

    [JsonPropertyName("version")]
    public string? Version { get; init; }

    [JsonPropertyName("description")]
    public string Description { get; init; } = "";

    [JsonPropertyName("long_description")]
    public string LongDescription { get; init; } = "";

    [JsonPropertyName("epilogue")]
    public string Epilogue { get; init; } = "";

    [JsonPropertyName("usage")]
    public string Usage { get; init; } = "";

    [JsonPropertyName("options")]
    public List<OptionSpec> Options { get; init; } = [];

    [JsonPropertyName("arguments")]
    public List<ArgumentSpec> Arguments { get; init; } = [];

    [JsonPropertyName("subcommands")]
    public List<CliModel> Subcommands { get; init; } = [];

    [JsonExtensionData]
    public Dictionary<string, JsonElement> Extra { get; set; } = new();
}

public sealed class OptionSpec
{
    [JsonPropertyName("name")]
    public required string Name { get; init; }

    [JsonPropertyName("long")]
    public string? Long { get; init; }

    [JsonPropertyName("short")]
    public string? Short { get; init; }

    [JsonPropertyName("help")]
    public string Help { get; init; } = "";

    [JsonPropertyName("long_help")]
    public string LongHelp { get; init; } = "";

    [JsonPropertyName("value_name")]
    public string? ValueName { get; init; }

    [JsonPropertyName("required")]
    public bool Required { get; init; }

    [JsonPropertyName("default_value")]
    public string? DefaultValue { get; init; }

    [JsonPropertyName("choices")]
    public List<string> Choices { get; init; } = [];

    [JsonPropertyName("min_values")]
    public int? MinValues { get; init; }

    [JsonPropertyName("max_values")]
    public int? MaxValues { get; init; }

    [JsonExtensionData]
    public Dictionary<string, JsonElement> Extra { get; set; } = new();
}

public sealed class ArgumentSpec
{
    [JsonPropertyName("name")]
    public required string Name { get; init; }

    [JsonPropertyName("help")]
    public string Help { get; init; } = "";

    [JsonPropertyName("required")]
    public bool Required { get; init; }

    [JsonPropertyName("default_value")]
    public string? DefaultValue { get; init; }

    [JsonPropertyName("choices")]
    public List<string> Choices { get; init; } = [];

    [JsonExtensionData]
    public Dictionary<string, JsonElement> Extra { get; set; } = new();
}

[JsonSourceGenerationOptions(
    WriteIndented = true,
    DefaultIgnoreCondition = JsonIgnoreCondition.Never,
    PropertyNamingPolicy = null)]
[JsonSerializable(typeof(CliModel))]
public sealed partial class DocliJsonContext : JsonSerializerContext;

public static class CliModelJson
{
    public static string Serialize(CliModel model);
    public static CliModel Deserialize(string json);
}
```

- `Serialize` uses `DocliJsonContext` and returns indented JSON with null known fields present. It does not append a trailing newline. `Deserialize` uses the same context.

## Out of Scope

- Phase D non-closures: see [README.md](README.md)
- `System.CommandLine`, `CommandLineAdapter`, the dotnet tool, the sample, Rust edits under `src/` or `templates/`
- A .NET CI job (host gate only)
- Byte-identical output against `fixtures/contract/model.json` (that file omits defaults; comparison is semantic)

## Acceptance Criteria

- `dotnet --version` satisfies the SDK pinned in `global.json`
- After the host gate, `git check-ignore` matches the file paths `dotnet/src/Docli/bin/Debug/x` and `dotnet/src/Docli/obj/x` via the two new gitignore lines. Those paths need not exist as directories beforehand.
- After the host gate, `git status --porcelain` lists no `dotnet/**/bin` or `dotnet/**/obj` path
- `Deserialize_contract_fixture` loads `fixtures/contract/model.json` and sees root name `demo`, version `1.0.0`, option `output` with `value_name` `PATH` and choices `json` then `yaml`, option `verbose` with `long` `--verbose` and `short` `-v`, subcommand `run` whose `usage` contains `demo run`, nested command `once`, and subcommand `check` argument `config` with `required` true
- `RoundTrip_preserves_unknown_fields`: keys `vendor_ext` on the root, `vendor_opt` on an option, `vendor_arg` on an argument, and `vendor_sub` on a nested subcommand survive `Deserialize` then `Serialize`
- `Missing_name_is_rejected`: a command object without `name` throws `JsonException`
- `Omitted_defaults_match_serde`: omitted `version` is null, omitted `required` is false, omitted `options` is empty, and `Serialize` writes `"version": null` when `Version` is null

## Required Validation

- Phase D host gate for d.2–d.5 — [README.md](README.md)
- `dotnet test dotnet/Docli.sln -c Release --filter CliModelJsonTests`
- `git check-ignore -v dotnet/src/Docli/bin/Debug/x dotnet/src/Docli/obj/x`
- `git status --porcelain`
