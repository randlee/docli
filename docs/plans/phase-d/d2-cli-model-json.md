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

- `dotnet/Docli.sln` with `dotnet/src/Docli/Docli.csproj` and `dotnet/tests/Docli.Tests/Docli.Tests.csproj` only. Both projects: `net10.0`, `Nullable` enable, `ImplicitUsings` enable. The library is not `PackAsTool`. Neither project references `System.CommandLine`.
- `dotnet/tests/Docli.Tests/CliModelJsonTests.cs` references the library and reads `fixtures/contract/model.json` by walking parents for `Cargo.toml`.
- Public types in namespace `Docli`. JSON names are the `[JsonPropertyName]` values. CLR names are the property names. Missing `name` fails deserialize. Every other field uses the `src/schema.rs` default: null for optional strings and counts, `""` for strings, `false` for `required`, empty lists for collections. Unknown JSON keys round-trip through `Extra` and must not reuse a known key.

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

- `System.CommandLine`, `CommandLineAdapter`, the dotnet tool, the sample, Rust edits under `src/` or `templates/`
- Byte-identical output against `fixtures/contract/model.json` (that file omits defaults; comparison is semantic)
- A .NET HTML or Markdown renderer, a template-engine port, Go or Python adapters, an MCP wrapper
- Closing `REQ-DOCLI-PRODUCT-004`, `REQ-DOCLI-NET-001`, or `REQ-DOCLI-GEN-003`

## Acceptance Criteria

- `Deserialize_contract_fixture` loads `fixtures/contract/model.json` and sees root name `demo`, version `1.0.0`, option `output` with `value_name` `PATH` and choices `json` then `yaml`, option `verbose` with `long` `--verbose` and `short` `-v`, subcommand `run` whose `usage` contains `demo run`, nested command `once`, and subcommand `check` argument `config` with `required` true
- `RoundTrip_preserves_unknown_fields`: a root key `vendor_ext` and an option key `vendor_opt` survive `Deserialize` then `Serialize`
- `Missing_name_is_rejected`: a command object without `name` throws
- `Omitted_defaults_match_serde`: omitted `version` is null, omitted `required` is false, omitted `options` is empty, and `Serialize` writes `"version": null` when `Version` is null

## Required Validation

- Phase D host gate for d.2–d.5 — [README.md](README.md)
- `dotnet test dotnet/Docli.sln -c Release --filter CliModelJsonTests`
