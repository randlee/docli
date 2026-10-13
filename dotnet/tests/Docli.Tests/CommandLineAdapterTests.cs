using System.CommandLine;
using System.CommandLine.Help;
using Docli;
using Xunit;

namespace Docli.Tests;

public sealed class CommandLineAdapterTests
{
    [Fact]
    public void Maps_option_name_long_and_short()
    {
        var command = new Command("sample");
        command.Options.Add(new Option<string>("--output", "-o") { HelpName = "PATH" });

        var option = Assert.Single(CommandLineAdapter.FromCommand(command).Options);

        Assert.Equal("output", option.Name);
        Assert.Equal("--output", option.Long);
        Assert.Equal("-o", option.Short);
        Assert.Equal("PATH", option.ValueName);
    }

    [Fact]
    public void Maps_long_from_alias_and_short_fallback()
    {
        var command = new Command("sample");
        command.Options.Add(new Option<string>("-o", "--output"));
        command.Options.Add(new Option<string>("output", "-v"));

        var model = CommandLineAdapter.FromCommand(command);
        var aliased = model.Options.Single(option => option.Long == "--output");
        var fallback = model.Options.Single(option => option.Short == "-v");

        Assert.Equal("--output", aliased.Long);
        Assert.Equal("-o", aliased.Short);
        Assert.Equal("output", aliased.Name);
        Assert.Null(fallback.Long);
        Assert.Equal("-v", fallback.Short);
        Assert.Equal("v", fallback.Name);
    }

    [Fact]
    public void Skips_help_and_version_options()
    {
        var command = new RootCommand();
        Assert.Contains(command.Options, option => option is HelpOption);
        Assert.Contains(command.Options, option => option is VersionOption);

        var model = CommandLineAdapter.FromCommand(command);

        Assert.DoesNotContain(model.Options, option => option.Long is "--help" or "--version");
    }

    [Fact]
    public void Omits_hidden_subcommands()
    {
        var command = new Command("root");
        var hidden = new Command("secret") { Hidden = true };
        hidden.Subcommands.Add(new Command("nested"));
        command.Subcommands.Add(hidden);
        command.Subcommands.Add(new Command("show"));
        command.Options.Add(new Option<string>("--hidden") { Hidden = true });

        var model = CommandLineAdapter.FromCommand(command);

        var visible = Assert.Single(model.Subcommands);
        Assert.Equal("show", visible.Name);
        Assert.Empty(visible.Subcommands);
        Assert.Equal("hidden", Assert.Single(model.Options).Name);
    }

    [Fact]
    public void Hidden_argument_is_kept()
    {
        var command = new Command("sample");
        command.Arguments.Add(new Argument<string>("config") { Hidden = true });

        Assert.Equal("config", Assert.Single(CommandLineAdapter.FromCommand(command).Arguments).Name);
    }

    [Fact]
    public void Hidden_root_is_kept()
    {
        var command = new Command("sample") { Hidden = true };

        Assert.Equal("sample", CommandLineAdapter.FromCommand(command).Name);
    }

    [Fact]
    public void Does_not_copy_recursive_options_to_children()
    {
        var parent = new Command("parent");
        parent.Options.Add(new Option<string>("--global") { Recursive = true });
        parent.Subcommands.Add(new Command("child"));

        var model = CommandLineAdapter.FromCommand(parent);

        Assert.Equal("global", Assert.Single(model.Options).Name);
        Assert.Empty(Assert.Single(model.Subcommands).Options);
    }

    [Fact]
    public void Ignores_root_directives()
    {
        var command = new RootCommand();
        command.Directives.Add(new Directive("custom-dir"));

        var model = CommandLineAdapter.FromCommand(command);

        Assert.DoesNotContain(
            model.Options,
            option => option.Name is "suggest" or "custom-dir" || option.Long is "suggest" or "custom-dir");
        Assert.DoesNotContain(model.Arguments, argument => argument.Name is "suggest" or "custom-dir");
        Assert.DoesNotContain(model.Subcommands, subcommand => subcommand.Name is "suggest" or "custom-dir");
    }

    [Fact]
    public void Usage_string_matches_spec()
    {
        var run = new Command("run");
        var sample = new Command("sample");
        sample.Options.Add(new Option<string>("--output"));
        sample.Subcommands.Add(run);

        var withCommand = CommandLineAdapter.FromCommand(sample);
        Assert.Equal("Usage: sample [OPTIONS] <COMMAND>", withCommand.Usage);
        Assert.Equal("Usage: sample run", Assert.Single(withCommand.Subcommands).Usage);

        var withArguments = new Command("sample");
        withArguments.Arguments.Add(new Argument<string>("config") { Arity = ArgumentArity.ExactlyOne });
        withArguments.Arguments.Add(new Argument<string>("label") { Arity = ArgumentArity.ZeroOrOne });
        Assert.Equal("Usage: sample <config> [label]", CommandLineAdapter.FromCommand(withArguments).Usage);

        var declarationOrder = new Command("sample");
        declarationOrder.Arguments.Add(new Argument<string>("zeta") { Arity = ArgumentArity.ExactlyOne });
        declarationOrder.Arguments.Add(new Argument<string>("alpha") { Arity = ArgumentArity.ZeroOrOne });
        Assert.Equal(
            "Usage: sample <zeta> [alpha]",
            CommandLineAdapter.FromCommand(declarationOrder).Usage);
    }

    [Fact]
    public void Flag_arity_zero_nulls_min_and_max()
    {
        var command = new Command("sample");
        command.Options.Add(new Option<bool>("--verbose")
        {
            Arity = ArgumentArity.Zero,
            HelpName = "VALUE",
        });

        var option = Assert.Single(CommandLineAdapter.FromCommand(command).Options);

        Assert.Null(option.MinValues);
        Assert.Null(option.MaxValues);
        Assert.Null(option.ValueName);
    }

    [Fact]
    public void Default_bool_option_uses_arity()
    {
        var option = new Option<bool>("--verbose");
        var command = new Command("sample");
        command.Options.Add(option);

        var mapped = Assert.Single(CommandLineAdapter.FromCommand(command).Options);

        Assert.Equal(option.Arity.MinimumNumberOfValues, mapped.MinValues);
        Assert.Equal(option.Arity.MaximumNumberOfValues, mapped.MaxValues);
        Assert.Null(mapped.ValueName);
    }

    [Fact]
    public void Enum_choices_are_enum_names()
    {
        var command = new Command("sample");
        command.Options.Add(new Option<NamedColor>("--color"));
        command.Arguments.Add(new Argument<NamedColor>("mode"));

        var model = CommandLineAdapter.FromCommand(command);
        var names = Enum.GetNames(typeof(NamedColor));

        Assert.Equal(names, Assert.Single(model.Options).Choices);
        Assert.Equal(names, Assert.Single(model.Arguments).Choices);
    }

    [Fact]
    public void Nullable_enum_and_non_enum_choices()
    {
        var command = new Command("sample");
        command.Options.Add(new Option<NamedColor?>("--color"));
        command.Options.Add(new Option<string>("--text"));

        var model = CommandLineAdapter.FromCommand(command);

        Assert.Equal(
            Enum.GetNames(typeof(NamedColor)),
            model.Options.Single(option => option.Name == "color").Choices);
        Assert.Empty(model.Options.Single(option => option.Name == "text").Choices);
    }

    [Fact]
    public void Maps_default_values()
    {
        var command = new Command("sample");
        command.Options.Add(new Option<bool>("--off") { DefaultValueFactory = _ => false });
        command.Options.Add(new Option<int>("--count") { DefaultValueFactory = _ => 12 });
        command.Options.Add(new Option<string>("--label") { DefaultValueFactory = _ => "x" });
        command.Options.Add(new Option<NamedColor>("--color") { DefaultValueFactory = _ => NamedColor.Beta });
        command.Arguments.Add(new Argument<int>("times") { DefaultValueFactory = _ => 12 });

        var model = CommandLineAdapter.FromCommand(command);

        Assert.Equal("false", model.Options.Single(option => option.Name == "off").DefaultValue);
        Assert.Equal("12", model.Options.Single(option => option.Name == "count").DefaultValue);
        var label = model.Options.Single(option => option.Name == "label").DefaultValue;
        Assert.Equal("x", label);
        Assert.DoesNotContain("\"", label);
        Assert.Equal(nameof(NamedColor.Beta), model.Options.Single(option => option.Name == "color").DefaultValue);
        Assert.Equal("12", Assert.Single(model.Arguments).DefaultValue);

        var throwing = new Command("sample");
        throwing.Options.Add(new Option<int>("--bad")
        {
            DefaultValueFactory = _ => throw new InvalidOperationException("boom"),
        });
        var exception = Assert.Throws<InvalidOperationException>(() => CommandLineAdapter.FromCommand(throwing));
        Assert.Equal("boom", exception.Message);
    }

    [Fact]
    public void Required_option_maps_required()
    {
        var command = new Command("sample");
        command.Options.Add(new Option<string>("--need")
        {
            Required = true,
            Arity = ArgumentArity.Zero,
        });
        command.Options.Add(new Option<string>("--skip")
        {
            Required = false,
            Arity = ArgumentArity.OneOrMore,
        });

        var model = CommandLineAdapter.FromCommand(command);

        Assert.True(model.Options.Single(option => option.Name == "need").Required);
        Assert.False(model.Options.Single(option => option.Name == "skip").Required);
    }

    [Fact]
    public void Null_description_becomes_empty_string()
    {
        var command = new Command("sample") { Description = null };
        command.Options.Add(new Option<string>("--output") { Description = null });
        command.Arguments.Add(new Argument<string>("config") { Description = null });

        var model = CommandLineAdapter.FromCommand(command);
        var option = Assert.Single(model.Options);
        var argument = Assert.Single(model.Arguments);

        Assert.Equal("", model.Description);
        Assert.Equal("", model.LongDescription);
        Assert.Equal("", model.Epilogue);
        Assert.Equal("", option.Help);
        Assert.Equal("", option.LongHelp);
        Assert.Equal("", argument.Help);
    }

    [Fact]
    public void Version_argument_is_only_on_the_root_model()
    {
        var child = new Command("run");
        var root = new Command("sample");
        root.Subcommands.Add(child);

        var model = CommandLineAdapter.FromCommand(root, "1.2.3");

        Assert.Equal("1.2.3", model.Version);
        Assert.Null(Assert.Single(model.Subcommands).Version);

        var walkedChild = CommandLineAdapter.FromCommand(child, "9.9.9");

        Assert.Equal("run", walkedChild.Name);
        Assert.Equal("9.9.9", walkedChild.Version);
        Assert.Equal("Usage: run", walkedChild.Usage);
    }

    [Fact]
    public void Sorts_options_arguments_and_subcommands_by_name()
    {
        var command = new Command("root");
        command.Options.Add(new Option<string>("--b"));
        command.Options.Add(new Option<string>("--a"));
        command.Arguments.Add(new Argument<string>("b"));
        command.Arguments.Add(new Argument<string>("a"));
        command.Subcommands.Add(new Command("b"));
        command.Subcommands.Add(new Command("a"));

        var model = CommandLineAdapter.FromCommand(command);
        var serialized = CliModelJson.Deserialize(CliModelJson.Serialize(model));

        Assert.Equal(["a", "b"], model.Options.Select(option => option.Name));
        Assert.Equal(["a", "b"], model.Arguments.Select(argument => argument.Name));
        Assert.Equal(["a", "b"], model.Subcommands.Select(subcommand => subcommand.Name));
        Assert.Equal(["a", "b"], serialized.Options.Select(option => option.Name));
        Assert.Equal(["a", "b"], serialized.Arguments.Select(argument => argument.Name));
        Assert.Equal(["a", "b"], serialized.Subcommands.Select(subcommand => subcommand.Name));
    }

    [Fact]
    public void Duplicate_names_keep_declaration_order()
    {
        var command = new Command("sample");
        command.Options.Add(new Option<string>("--v"));
        command.Options.Add(new Option<string>("-v"));

        var options = CommandLineAdapter.FromCommand(command).Options;

        Assert.Equal(["v", "v"], options.Select(option => option.Name));
        Assert.Equal("--v", options[0].Long);
        Assert.Null(options[0].Short);
        Assert.Null(options[1].Long);
        Assert.Equal("-v", options[1].Short);
    }

    [Fact]
    public void Maps_collection_and_custom_defaults()
    {
        var command = new Command("sample");
        command.Options.Add(new Option<string[]>("--items")
        {
            DefaultValueFactory = _ => ["a", "b"],
        });
        command.Options.Add(new Option<NumberedDefault>("--numbered")
        {
            DefaultValueFactory = _ => new NumberedDefault { N = 3 },
        });

        var model = CommandLineAdapter.FromCommand(command);

        Assert.Equal("[\"a\",\"b\"]", model.Options.Single(option => option.Name == "items").DefaultValue);
        Assert.Equal("{\"N\":3}", model.Options.Single(option => option.Name == "numbered").DefaultValue);
    }

    [Fact]
    public void Argument_required_follows_arity()
    {
        var command = new Command("sample");
        command.Arguments.Add(new Argument<string>("config") { Arity = ArgumentArity.ExactlyOne });
        command.Arguments.Add(new Argument<string>("label") { Arity = ArgumentArity.ZeroOrOne });

        var model = CommandLineAdapter.FromCommand(command);

        Assert.True(model.Arguments.Single(argument => argument.Name == "config").Required);
        Assert.False(model.Arguments.Single(argument => argument.Name == "label").Required);
    }

    private enum NamedColor
    {
        Beta,
        Alpha,
    }

    public sealed class NumberedDefault
    {
        public int N { get; set; }
    }
}
