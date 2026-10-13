using System.CommandLine;
using System.CommandLine.Help;
using System.Globalization;
using System.Text.Json;

namespace Docli;

public static class CommandLineAdapter
{
    // WriteIndented is false. Every other setting stays at the constructor default,
    // including a null PropertyNamingPolicy.
    private static readonly JsonSerializerOptions CompactDefaultValueOptions = new()
    {
        WriteIndented = false,
    };

    public static CliModel FromCommand(Command command, string? version = null)
    {
        return MapCommand(command, version, []);
    }

    private static CliModel MapCommand(
        Command command,
        string? version,
        IReadOnlyList<string> ancestorNames)
    {
        var path = new List<string>(ancestorNames.Count + 1);
        path.AddRange(ancestorNames);
        path.Add(command.Name);

        var options = command.Options
            .Where(option => option is not HelpOption && option is not VersionOption)
            .Select(MapOption)
            .OrderBy(option => option.Name, StringComparer.Ordinal)
            .ToList();

        var arguments = command.Arguments
            .Select(MapArgument)
            .OrderBy(argument => argument.Name, StringComparer.Ordinal)
            .ToList();

        var visibleSubcommands = command.Subcommands.Where(subcommand => !subcommand.Hidden).ToList();
        var subcommands = visibleSubcommands
            .Select(subcommand => MapCommand(subcommand, null, path))
            .OrderBy(subcommand => subcommand.Name, StringComparer.Ordinal)
            .ToList();

        return new CliModel
        {
            Name = command.Name,
            Version = version,
            Description = command.Description ?? "",
            LongDescription = "",
            Epilogue = "",
            Usage = BuildUsage(path, options.Count > 0, command.Arguments, visibleSubcommands.Count > 0),
            Options = options,
            Arguments = arguments,
            Subcommands = subcommands,
        };
    }

    private static OptionSpec MapOption(Option option)
    {
        var longName = LongToken(option);
        var shortName = ShortToken(option);
        var flag = IsFlag(option);
        var arity = option.Arity;

        return new OptionSpec
        {
            Name = DerivedName(longName, shortName, option.Name),
            Long = longName,
            Short = shortName,
            Help = option.Description ?? "",
            LongHelp = "",
            ValueName = flag ? null : option.HelpName,
            Required = option.Required,
            DefaultValue = ConvertDefault(option.HasDefaultValue, option.GetDefaultValue),
            Choices = ChoicesFor(option.ValueType),
            MinValues = flag ? null : arity.MinimumNumberOfValues,
            MaxValues = flag ? null : arity.MaximumNumberOfValues,
        };
    }

    private static ArgumentSpec MapArgument(Argument argument)
    {
        return new ArgumentSpec
        {
            Name = argument.Name,
            Help = argument.Description ?? "",
            Required = argument.Arity.MinimumNumberOfValues > 0,
            DefaultValue = ConvertDefault(argument.HasDefaultValue, argument.GetDefaultValue),
            Choices = ChoicesFor(argument.ValueType),
        };
    }

    private static string BuildUsage(
        IReadOnlyList<string> path,
        bool hasOptions,
        IEnumerable<Argument> arguments,
        bool hasSubcommands)
    {
        var usage = "Usage: " + string.Join(' ', path);
        if (hasOptions)
        {
            usage += " [OPTIONS]";
        }

        foreach (var argument in arguments)
        {
            usage += argument.Arity.MinimumNumberOfValues > 0
                ? $" <{argument.Name}>"
                : $" [{argument.Name}]";
        }

        if (hasSubcommands)
        {
            usage += " <COMMAND>";
        }

        return usage;
    }

    private static string? LongToken(Option option)
    {
        if (option.Name.StartsWith("--", StringComparison.Ordinal))
        {
            return option.Name;
        }

        foreach (var alias in option.Aliases)
        {
            if (alias.StartsWith("--", StringComparison.Ordinal))
            {
                return alias;
            }
        }

        return null;
    }

    private static string? ShortToken(Option option)
    {
        if (IsShortToken(option.Name))
        {
            return option.Name;
        }

        foreach (var alias in option.Aliases)
        {
            if (IsShortToken(alias))
            {
                return alias;
            }
        }

        return null;
    }

    private static bool IsShortToken(string token)
    {
        return token.Length == 2 && token[0] == '-' && token[1] != '-';
    }

    private static string DerivedName(string? longName, string? shortName, string optionName)
    {
        if (longName is not null)
        {
            return longName.TrimStart('-');
        }

        if (shortName is not null)
        {
            return shortName.TrimStart('-');
        }

        return optionName.TrimStart('-');
    }

    private static bool IsFlag(Option option)
    {
        return option.ValueType == typeof(bool)
            && option.Arity.MinimumNumberOfValues == 0
            && option.Arity.MaximumNumberOfValues == 0;
    }

    private static List<string> ChoicesFor(Type valueType)
    {
        if (valueType.IsEnum)
        {
            return [.. Enum.GetNames(valueType)];
        }

        var underlying = Nullable.GetUnderlyingType(valueType);
        if (underlying is not null && underlying.IsEnum)
        {
            return [.. Enum.GetNames(underlying)];
        }

        return [];
    }

    private static string? ConvertDefault(bool hasDefaultValue, Func<object?> readDefault)
    {
        if (!hasDefaultValue)
        {
            return null;
        }

        var value = readDefault();
        if (value is null)
        {
            return null;
        }

        if (value is bool boolean)
        {
            return boolean ? "true" : "false";
        }

        if (value is string text)
        {
            return text;
        }

        if (value is IFormattable formattable)
        {
            return formattable.ToString(null, CultureInfo.InvariantCulture);
        }

        return JsonSerializer.Serialize(value, value.GetType(), CompactDefaultValueOptions);
    }
}
