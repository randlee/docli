using System.Reflection;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Text.Json.Serialization;
using System.Text.RegularExpressions;
using Docli;
using Xunit;

namespace Docli.Tests;

public sealed class CliModelJsonTests
{
    [Fact]
    public void Deserialize_contract_fixture()
    {
        var model = CliModelJson.Deserialize(ReadRepoFile("fixtures", "contract", "model.json"));

        Assert.Equal("demo", model.Name);
        Assert.Equal("1.0.0", model.Version);

        var output = model.Options.Single(option => option.Name == "output");
        Assert.Equal("PATH", output.ValueName);
        Assert.Equal(new[] { "json", "yaml" }, output.Choices);

        var verbose = model.Options.Single(option => option.Name == "verbose");
        Assert.Equal("--verbose", verbose.Long);
        Assert.Equal("-v", verbose.Short);

        var run = model.Subcommands.Single(command => command.Name == "run");
        Assert.Contains("demo run", run.Usage, StringComparison.Ordinal);
        Assert.Contains(run.Subcommands, command => command.Name == "once");

        var check = model.Subcommands.Single(command => command.Name == "check");
        var config = check.Arguments.Single(argument => argument.Name == "config");
        Assert.True(config.Required);
    }

    [Fact]
    public void RoundTrip_preserves_unknown_fields()
    {
        const string json = """
            {
              "name": "demo",
              "vendor_ext": {"keep": true},
              "options": [{"name": "output", "vendor_opt": "opt"}],
              "arguments": [{"name": "config", "vendor_arg": 1}],
              "subcommands": [
                {
                  "name": "run",
                  "subcommands": [{"name": "once", "vendor_sub": "nested"}]
                }
              ]
            }
            """;

        var model = CliModelJson.Deserialize(json);
        var serialized = CliModelJson.Serialize(model);
        var again = CliModelJson.Deserialize(serialized);

        AssertJsonEqual(model.Extra["vendor_ext"], again.Extra["vendor_ext"]);
        Assert.Equal("opt", again.Options[0].Extra["vendor_opt"].GetString());
        Assert.Equal(1, again.Arguments[0].Extra["vendor_arg"].GetInt32());
        Assert.Equal(
            "nested",
            again.Subcommands[0].Subcommands[0].Extra["vendor_sub"].GetString());
        Assert.Contains("\"vendor_ext\"", serialized, StringComparison.Ordinal);
        Assert.Contains("\"vendor_opt\"", serialized, StringComparison.Ordinal);
        Assert.Contains("\"vendor_arg\"", serialized, StringComparison.Ordinal);
        Assert.Contains("\"vendor_sub\"", serialized, StringComparison.Ordinal);
    }

    [Fact]
    public void Missing_name_is_rejected()
    {
        Assert.Throws<JsonException>(() =>
            CliModelJson.Deserialize("""{"version":"1.0.0","description":"no name"}"""));
    }

    [Fact]
    public void Omitted_defaults_match_serde()
    {
        var omitted = CliModelJson.Deserialize("""{"name":"demo"}""");
        Assert.Null(omitted.Version);
        Assert.Equal("", omitted.Description);
        Assert.Equal("", omitted.LongDescription);
        Assert.Equal("", omitted.Epilogue);
        Assert.Equal("", omitted.Usage);
        Assert.Empty(omitted.Options);
        Assert.Empty(omitted.Arguments);
        Assert.Empty(omitted.Subcommands);

        var partial = CliModelJson.Deserialize(
            """{"name":"demo","options":[{"name":"verbose"}],"arguments":[{"name":"file"}]}""");
        var option = partial.Options.Single();
        Assert.False(option.Required);
        Assert.Null(option.Long);
        Assert.Null(option.Short);
        Assert.Equal("", option.Help);
        Assert.Equal("", option.LongHelp);
        Assert.Null(option.ValueName);
        Assert.Null(option.DefaultValue);
        Assert.Empty(option.Choices);
        Assert.Null(option.MinValues);
        Assert.Null(option.MaxValues);
        Assert.False(partial.Arguments.Single().Required);

        var serialized = CliModelJson.Serialize(omitted);
        Assert.Contains("\"version\": null", serialized, StringComparison.Ordinal);
        Assert.False(serialized.EndsWith('\n'));
    }

    [Fact]
    public void Schema_fields_match_src_schema_rs()
    {
        var schema = ReadRepoFile("src", "schema.rs");
        AssertFieldSet(schema, "CliModel", typeof(CliModel));
        AssertFieldSet(schema, "OptionSpec", typeof(OptionSpec));
        AssertFieldSet(schema, "ArgumentSpec", typeof(ArgumentSpec));
    }

    [Fact]
    public void RoundTrip_all_fields()
    {
        var original = CliModelJson.Deserialize(
            ReadRepoFile("dotnet", "tests", "Docli.Tests", "Fixtures", "all-fields.json"));
        var option = original.Options.Single();
        Assert.Equal("Write output to PATH", option.LongHelp);
        Assert.Equal(1, option.MinValues);
        Assert.Equal(2, option.MaxValues);
        Assert.Equal("out.json", option.DefaultValue);
        Assert.Equal(new[] { "json", "yaml" }, option.Choices);

        var serialized = CliModelJson.Serialize(original);
        var again = CliModelJson.Deserialize(serialized);
        AssertSameModel(original, again);
        Assert.Equal(serialized, CliModelJson.Serialize(again));
    }

    private static void AssertFieldSet(string schema, string structName, Type clrType)
    {
        var rustFields = PublicFieldNames(StructBody(schema, structName));
        var jsonNames = JsonPropertyNames(clrType);
        Assert.NotEmpty(rustFields);
        Assert.True(
            rustFields.SetEquals(jsonNames),
            $"{structName} fields differ. schema: [{string.Join(", ", rustFields.Order())}] json: [{string.Join(", ", jsonNames.Order())}]");
    }

    private static string StructBody(string source, string structName)
    {
        var marker = $"pub struct {structName}";
        var start = source.IndexOf(marker, StringComparison.Ordinal);
        if (start < 0)
        {
            throw new InvalidOperationException($"src/schema.rs has no {marker}.");
        }

        var open = source.IndexOf('{', start);
        if (open < 0)
        {
            throw new InvalidOperationException($"src/schema.rs {structName} has no body.");
        }

        var depth = 0;
        for (var i = open; i < source.Length; i++)
        {
            if (source[i] == '{')
            {
                depth++;
            }
            else if (source[i] == '}')
            {
                depth--;
                if (depth == 0)
                {
                    return source[(open + 1)..i];
                }
            }
        }

        throw new InvalidOperationException($"src/schema.rs {structName} body is unterminated.");
    }

    private static HashSet<string> PublicFieldNames(string body)
    {
        var names = new HashSet<string>(StringComparer.Ordinal);
        foreach (Match match in PublicField.Matches(body))
        {
            var name = match.Groups[1].Value;
            if (name != "extra")
            {
                names.Add(name);
            }
        }

        return names;
    }

    private static HashSet<string> JsonPropertyNames(Type type)
    {
        var names = new HashSet<string>(StringComparer.Ordinal);
        foreach (var property in type.GetProperties())
        {
            var attribute = property.GetCustomAttribute<JsonPropertyNameAttribute>();
            if (attribute is not null)
            {
                names.Add(attribute.Name);
            }
        }

        return names;
    }

    private static void AssertSameModel(CliModel expected, CliModel actual)
    {
        Assert.Equal(expected.Name, actual.Name);
        Assert.Equal(expected.Version, actual.Version);
        Assert.Equal(expected.Description, actual.Description);
        Assert.Equal(expected.LongDescription, actual.LongDescription);
        Assert.Equal(expected.Epilogue, actual.Epilogue);
        Assert.Equal(expected.Usage, actual.Usage);
        AssertSameExtras(expected.Extra, actual.Extra);
        Assert.Equal(expected.Options.Count, actual.Options.Count);
        for (var i = 0; i < expected.Options.Count; i++)
        {
            AssertSameOption(expected.Options[i], actual.Options[i]);
        }

        Assert.Equal(expected.Arguments.Count, actual.Arguments.Count);
        for (var i = 0; i < expected.Arguments.Count; i++)
        {
            AssertSameArgument(expected.Arguments[i], actual.Arguments[i]);
        }

        Assert.Equal(expected.Subcommands.Count, actual.Subcommands.Count);
        for (var i = 0; i < expected.Subcommands.Count; i++)
        {
            AssertSameModel(expected.Subcommands[i], actual.Subcommands[i]);
        }
    }

    private static void AssertSameOption(OptionSpec expected, OptionSpec actual)
    {
        Assert.Equal(expected.Name, actual.Name);
        Assert.Equal(expected.Long, actual.Long);
        Assert.Equal(expected.Short, actual.Short);
        Assert.Equal(expected.Help, actual.Help);
        Assert.Equal(expected.LongHelp, actual.LongHelp);
        Assert.Equal(expected.ValueName, actual.ValueName);
        Assert.Equal(expected.Required, actual.Required);
        Assert.Equal(expected.DefaultValue, actual.DefaultValue);
        Assert.Equal(expected.Choices, actual.Choices);
        Assert.Equal(expected.MinValues, actual.MinValues);
        Assert.Equal(expected.MaxValues, actual.MaxValues);
        AssertSameExtras(expected.Extra, actual.Extra);
    }

    private static void AssertSameArgument(ArgumentSpec expected, ArgumentSpec actual)
    {
        Assert.Equal(expected.Name, actual.Name);
        Assert.Equal(expected.Help, actual.Help);
        Assert.Equal(expected.Required, actual.Required);
        Assert.Equal(expected.DefaultValue, actual.DefaultValue);
        Assert.Equal(expected.Choices, actual.Choices);
        AssertSameExtras(expected.Extra, actual.Extra);
    }

    private static void AssertSameExtras(
        Dictionary<string, JsonElement> expected,
        Dictionary<string, JsonElement> actual)
    {
        Assert.Equal(expected.Keys.Order(), actual.Keys.Order());
        foreach (var (key, value) in expected)
        {
            AssertJsonEqual(value, actual[key]);
        }
    }

    private static void AssertJsonEqual(JsonElement expected, JsonElement actual)
    {
        Assert.True(
            JsonNode.DeepEquals(JsonNode.Parse(expected.GetRawText()), JsonNode.Parse(actual.GetRawText())),
            $"{expected.GetRawText()} != {actual.GetRawText()}");
    }

    private static string ReadRepoFile(params string[] parts)
    {
        var path = Path.Combine([RepoRoot.Find(), .. parts]);
        return File.ReadAllText(path);
    }

    private static readonly Regex PublicField = new(
        @"^\s*pub\s+([A-Za-z_][A-Za-z0-9_]*)\s*:",
        RegexOptions.Multiline | RegexOptions.CultureInvariant,
        TimeSpan.FromSeconds(1));
}
