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

`Docli.CommandLineAdapter.FromCommand` maps a `System.CommandLine` 2.0.12 tree onto the d.2 JSON types.

## Closes

No requirement id. This sprint implements the walker named by `ADR-005`. It does not add the tool or the HTML proof.

## Hard Dependencies

- d.2 — `CliModel`, `OptionSpec`, `ArgumentSpec`, and `CliModelJson` on `integrate/phase-d`.

## Deliverables

- `dotnet/src/Docli/Docli.csproj` gains `PackageReference` `System.CommandLine` version `2.0.12` only. No 3.0 preview.
- `dotnet/src/Docli/CommandLineAdapter.cs` and `dotnet/tests/Docli.Tests/CommandLineAdapterTests.cs`.
- `Docli.Tests` references `Docli.csproj` only. It does not take its own `System.CommandLine` package reference.
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

- `version` is written only on the returned root `CliModel`. Every nested `subcommands` element has `version` null. The method does not read the entry assembly.
- Walk `command.Options`, `command.Arguments`, and `command.Subcommands` only. Do not copy a parent's `Recursive` options onto a child. Ignore `RootCommand.Directives`.
- Drop an option when `option is HelpOption` or `option is VersionOption` (`HelpOption` is in `System.CommandLine.Help`). Drop a subcommand when `Hidden` is true, and do not walk its descendants. Hidden options and arguments stay.
- After mapping, sort `options`, `arguments`, and `subcommands` by `name` with ordinal compare.
- `long_description`, `epilogue`, and option `long_help` are `""`. System.CommandLine has one `Description`.
- `description` and option or argument `help` are `Description ?? ""`.
- Option `long`: `Name` when it starts with `--`; otherwise the first `Aliases` entry that starts with `--`; otherwise null.
- Option `short`: the first of `Name`, then `Aliases` in collection order, whose length is 2, that starts with `-`, and whose second character is not `-`. Include the dash. Otherwise null.
- Option `name`: `long` with leading `-` removed; if there is no `long`, `short` with the leading `-` removed; otherwise `Name` with leading `-` removed.
- Argument `name` is `Argument.Name`. `value_name` is `HelpName` (null when `HelpName` is null). For a bool option whose arity minimum and maximum are both 0, `value_name`, `min_values`, and `max_values` are null.
- Otherwise `min_values` is `Arity.MinimumNumberOfValues` and `max_values` is `Arity.MaximumNumberOfValues`, as JSON numbers. Do not rewrite large maxima.
- `required` is `Option.Required`. Arguments are required when `Arity.MinimumNumberOfValues` is greater than 0.
- `default_value` is null when `HasDefaultValue` is false. A bool becomes `"true"` or `"false"`. Any other `IFormattable` uses invariant `ToString`. Anything else is the JSON text of the value, stored as one string. Exceptions from `GetDefaultValue()` propagate.
- `choices` is `Enum.GetNames` when `ValueType` is an enum or `Nullable<TEnum>`, in metadata order. Otherwise `[]`. Do not read `CompletionSources`.
- `usage` is `Usage: {fullPath}{options}{args}{command}` where `fullPath` is ancestor names from the `FromCommand` root to this command, joined by single spaces. Append ` [OPTIONS]` when any option remains after the help/version drop. Append arguments in `Command.Arguments` order (not sorted name order): ` <{name}>` when that argument is required, otherwise ` [{name}]`. Append ` <COMMAND>` when any subcommand remains after the hidden drop.

## Out of Scope

- The dotnet tool, the sample, and edits under `src/` or `templates/`
- Treating completion sources as `choices`
- A .NET HTML or Markdown renderer, a template-engine port, Go or Python adapters, an MCP wrapper
- Closing `REQ-DOCLI-PRODUCT-004`, `REQ-DOCLI-NET-001`, or `REQ-DOCLI-GEN-003`

## Acceptance Criteria

- `Maps_option_name_long_and_short`: `Option<string>("--output", "-o")` with `HelpName` `PATH` yields `name` `output`, `long` `--output`, `short` `-o`, `value_name` `PATH`
- `Skips_help_and_version_options`: a `RootCommand` model has no option whose `long` is `--help` or `--version`
- `Omits_hidden_subcommands`: a `Hidden` subcommand is absent; a hidden option remains
- `Usage_includes_parent_path`: child `run` under `sample` has `usage` containing `sample run`
- `Flag_arity_zero_nulls_min_and_max`: `Option<bool>` with `Arity = ArgumentArity.Zero` has null `min_values`, `max_values`, and `value_name`
- `Enum_choices_are_enum_names`: an enum option's `choices` equal `Enum.GetNames` in that order
- `Version_argument_is_only_on_the_root_model`: `FromCommand(cmd, "1.2.3")` sets root `version` `1.2.3` and a child `version` null
- `Sorts_options_arguments_and_subcommands_by_name`: declaration order `b` then `a` serializes as `a` then `b`
- `Argument_required_follows_arity`: an argument with `Arity` `ArgumentArity.ExactlyOne` has `required` true; `ArgumentArity.ZeroOrOne` has `required` false

## Required Validation

- Phase D host gate for d.2–d.5 — [README.md](README.md)
- `dotnet test dotnet/Docli.sln -c Release --filter CommandLineAdapterTests`
