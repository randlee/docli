---
id: d.3
title: System.CommandLine walker
status: planned
branch: feature/phase-d-d3-adapter
worktree: ../docli-worktrees/feature/phase-d-d3-adapter
target: integrate/phase-d
---

# Sprint d.3 — System.CommandLine walker

## Goal

`Docli.CommandLineAdapter.FromCommand` maps a `System.CommandLine` tree (ADR-005 package pin) onto the d.2 JSON types. No reflection.

## Closes

No requirement id. This sprint implements the walker named by `ADR-005`. It does not add the tool or the HTML proof.

## Hard Dependencies

- d.2 — `CliModel`, `OptionSpec`, `ArgumentSpec`, and `CliModelJson` on `integrate/phase-d`.

## Deliverables

- `dotnet/src/Docli/Docli.csproj` gains `PackageReference` `System.CommandLine` at the ADR-005 package pin only. No 3.0 preview. Do not restate the version number in this sprint.
- `dotnet/src/Docli/CommandLineAdapter.cs` and `dotnet/tests/Docli.Tests/CommandLineAdapterTests.cs`. Every test below is a `[Fact]` in `CommandLineAdapterTests`. The class is not split.
- `Docli.Tests` references `Docli.csproj` only. It does not take its own `System.CommandLine` package reference and does not add test packages (d.2 pins those).
- This is the only signature source for the adapter:

```csharp
namespace Docli;

public static class CommandLineAdapter
{
    public static CliModel FromCommand(
        System.CommandLine.Command command,
        string? version = null);
}
```

- No reflection. Read only the public members in the table. Do not call `GetField`, `GetProperty`, `GetCustomAttributes`, or `BindingFlags`. Do not cast to `Option<T>` or `Argument<T>`. Do not read the internal `Option.Argument` property. `Nullable.GetUnderlyingType` and `Type.IsEnum` are the enum checks.
- The `command` argument is the walk root. Its parents are not walked. `Hidden` on that root does not drop it. `version` is written only on the returned root. Every nested subcommand model has `version` null. The method does not read the entry assembly.
- Walk `command.Options`, `command.Arguments`, and `command.Subcommands` only. Do not copy a parent option whose `Option.Recursive` is true onto a child. Do not read `RootCommand.Directives`.
- Drop an option when `option is System.CommandLine.Help.HelpOption` or `option is VersionOption`. Drop a subcommand when `Command.Hidden` is true, and do not walk its descendants. A hidden option stays. A hidden argument stays.
- After mapping, sort `options`, `arguments`, and `subcommands` by `name` with `StringComparer.Ordinal`.
- `ArgumentSpec` has only `name`, `help`, `required`, `default_value`, and `choices`. Argument `HelpName` is not mapped. Arguments have no `value_name`, `min_values`, or `max_values`. The flag rule applies only to options.

| Kind | JSON field | Public member | Rule |
|------|------------|---------------|------|
| command | `name` | `Command.Name` | Leaf name, unchanged |
| command | `version` | `FromCommand` argument | Walk root only; nested models null |
| command | `description` | `Command.Description` | Null becomes `""` |
| command | `long_description` | none | Always `""` |
| command | `epilogue` | none | Always `""` |
| command | `usage` | computed | Formula under the table |
| command | `options` | `Command.Options` | Option rows, after the help/version drop |
| command | `arguments` | `Command.Arguments` | Argument rows |
| command | `subcommands` | `Command.Subcommands` | Skip `Hidden`; do not walk those descendants |
| option | `name` | derived | `string.TrimStart('-')` of `long`, else of `short`, else of `Option.Name`. `TrimStart('-')` removes every leading `-` |
| option | `long` | `Option.Name`, then `Option.Aliases` | `Name` when it starts with `--`; else the first alias that starts with `--`; else null |
| option | `short` | `Option.Name`, then `Option.Aliases` | First token of length 2 that starts with `-` and whose second character is not `-`. Keep the dash. Else null |
| option | `help` | `Option.Description` | Null becomes `""` |
| option | `long_help` | none | Always `""` |
| option | `value_name` | `Option.HelpName` | Null when `HelpName` is null. Also null under the flag rule |
| option | `required` | `Option.Required` | That bool. Not arity |
| option | `default_value` | `Option.HasDefaultValue`, `Option.GetDefaultValue()` | Conversion below |
| option | `choices` | `Option.ValueType` | Enum rule below |
| option | `min_values`, `max_values` | `Option.Arity` | Flag rule, otherwise the two ints |
| argument | `name` | `Argument.Name` | Unchanged. Not trimmed |
| argument | `help` | `Argument.Description` | Null becomes `""` |
| argument | `required` | `Argument.Arity.MinimumNumberOfValues` | True when that int is greater than 0. `Argument` has no `Required` property |
| argument | `default_value` | `Argument.HasDefaultValue`, `Argument.GetDefaultValue()` | Same conversion as options |
| argument | `choices` | `Argument.ValueType` | Same enum rule as options |

- Flag rule, options only: `Option.ValueType == typeof(bool)` and both `Option.Arity.MinimumNumberOfValues` and `Option.Arity.MaximumNumberOfValues` are 0. Then `value_name`, `min_values`, and `max_values` are null. Otherwise those counts are the arity ints as JSON numbers. Do not rewrite large maxima. A default-arity `Option<bool>` leaves arity unset, so `min_values` and `max_values` are that option's `Option.Arity` minimum and maximum, and `value_name` is null when `HelpName` is null.
- Default conversion, for `Option.GetDefaultValue()` or `Argument.GetDefaultValue()`: `HasDefaultValue` false yields null. A null return yields null. `bool` and `bool?` yield `"true"` or `"false"` (not `bool.ToString()`, which is `True`/`False`). `string` yields the raw string, so `x` stays `x` and is not `"\"x\""`. Any other `IFormattable` uses `ToString(null, CultureInfo.InvariantCulture)`. Anything else is the JSON text of the value, stored as one string. An exception from `GetDefaultValue()` or from `DefaultValueFactory` propagates unchanged.
- Choices: if `ValueType.IsEnum`, `Enum.GetNames(ValueType)` in metadata order. If `Nullable.GetUnderlyingType(ValueType)` is an enum, `Enum.GetNames` of that type. Otherwise `[]`. Do not read `CompletionSources`.
- Usage is the concatenation of `Usage: `, the full path, then the suffixes. The full path is `Command.Name` values from the walk root down to the current command, joined by single spaces. Append ` [OPTIONS]` when any option remains after the help/version drop. Append arguments in `Command.Arguments` order, before the name sort: ` <{name}>` when that argument is required, otherwise ` [{name}]`. Append ` <COMMAND>` when any subcommand remains after the hidden drop. Examples, exact: root `sample` with one option and subcommand `run` and no arguments is `Usage: sample [OPTIONS] <COMMAND>`; that child with nothing of its own is `Usage: sample run`; required argument `config` then optional argument `label`, with no options and no subcommands, is `Usage: sample <config> [label]`.

## Out of Scope

- Phase D non-closures: see [README.md](README.md)
- The dotnet tool, the sample, and edits under `src/` or `templates/`
- Treating completion sources as `choices`
- Reflection, including non-public `System.CommandLine` members

## Acceptance Criteria

- `Maps_option_name_long_and_short`: `Option<string>("--output", "-o")` with `HelpName` `PATH` yields `name` `output`, `long` `--output`, `short` `-o`, `value_name` `PATH`
- `Maps_long_from_alias_and_short_fallback`: `Name` `-o` plus alias `--output` yields `long` `--output`, `short` `-o`, `name` `output`. `Name` `output` plus only alias `-v` yields `long` null, `short` `-v`, `name` `v`
- `Skips_help_and_version_options`: a `RootCommand` model has no option whose `long` is `--help` or `--version`
- `Omits_hidden_subcommands`: a `Hidden` subcommand is absent; a hidden option remains
- `Hidden_argument_is_kept`: an argument with `Hidden` true is present
- `Hidden_root_is_kept`: `FromCommand` on a `Hidden` command still returns that command
- `Does_not_copy_recursive_options_to_children`: a parent option with `Recursive` true is absent from the child `options` list
- `Ignores_root_directives`: a `RootCommand` model does not copy `Directives` into `options`, `arguments`, or `subcommands`
- `Usage_string_matches_spec`: the three usage examples in Deliverables match exactly, including `Usage: sample run`, `Usage: sample [OPTIONS] <COMMAND>`, and `Usage: sample <config> [label]`
- `Flag_arity_zero_nulls_min_and_max`: `Option<bool>` with `Arity = ArgumentArity.Zero` has null `min_values`, `max_values`, and `value_name`
- `Default_bool_option_uses_arity`: `new Option<bool>("--verbose")` with arity left unset yields `min_values` and `max_values` equal to that option's `Arity` minimum and maximum, and `value_name` null
- `Enum_choices_are_enum_names`: an enum option's `choices` equal `Enum.GetNames` in that order
- `Nullable_enum_and_non_enum_choices`: `Option<MyEnum?>` choices equal `Enum.GetNames`; `Option<string>` choices are empty
- `Maps_default_values`: bool `false` yields `"false"`; int `12` yields `"12"`; string `x` yields `x` and not a quoted JSON string; an enum yields its invariant name; a `DefaultValueFactory` that throws `InvalidOperationException` propagates that exception
- `Required_option_maps_required`: `Option.Required` true maps to `required` true, and false maps to false, independent of arity
- `Null_description_becomes_empty_string`: null `Description` on a command, option, and argument becomes `""`, and `long_description`, `epilogue`, and `long_help` are `""`
- `Version_argument_is_only_on_the_root_model`: `FromCommand(cmd, "1.2.3")` sets root `version` `1.2.3` and a child `version` null. Passing a non-root command makes that command the walk root
- `Sorts_options_arguments_and_subcommands_by_name`: declaration order `b` then `a` serializes as `a` then `b`
- `Argument_required_follows_arity`: an argument with `Arity` `ArgumentArity.ExactlyOne` has `required` true; `ArgumentArity.ZeroOrOne` has `required` false
- `CommandLineAdapter.cs` contains no `GetField`, `GetProperty`, `BindingFlags`, or `NonPublic`

## Required Validation

- Phase D host gate for d.2–d.5 — [README.md](README.md)
- `dotnet test dotnet/Docli.sln -c Release --filter CommandLineAdapterTests`
- `rg -n "GetField|GetProperty|BindingFlags|NonPublic" dotnet/src/Docli/CommandLineAdapter.cs` returns no matches
