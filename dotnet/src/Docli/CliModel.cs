using System.Diagnostics.CodeAnalysis;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Docli;

public sealed class CliModel
{
    public CliModel()
    {
    }

    // Source generation sets every init-only property from constructor arguments and
    // passes null for omitted JSON members, which would erase the property initializers.
    [JsonConstructor]
    [SetsRequiredMembers]
    internal CliModel(
        string name,
        string? version = null,
        string? description = null,
        string? longDescription = null,
        string? epilogue = null,
        string? usage = null,
        List<OptionSpec>? options = null,
        List<ArgumentSpec>? arguments = null,
        List<CliModel>? subcommands = null)
    {
        if (name is null)
        {
            throw new JsonException("Missing required property 'name'.");
        }

        Name = name;
        Version = version;
        Description = description ?? "";
        LongDescription = longDescription ?? "";
        Epilogue = epilogue ?? "";
        Usage = usage ?? "";
        Options = options ?? [];
        Arguments = arguments ?? [];
        Subcommands = subcommands ?? [];
    }

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
    public OptionSpec()
    {
    }

    [JsonConstructor]
    [SetsRequiredMembers]
    internal OptionSpec(
        string name,
        string? @long = null,
        string? @short = null,
        string? help = null,
        string? longHelp = null,
        string? valueName = null,
        bool required = false,
        string? defaultValue = null,
        List<string>? choices = null,
        int? minValues = null,
        int? maxValues = null)
    {
        if (name is null)
        {
            throw new JsonException("Missing required property 'name'.");
        }

        Name = name;
        Long = @long;
        Short = @short;
        Help = help ?? "";
        LongHelp = longHelp ?? "";
        ValueName = valueName;
        Required = required;
        DefaultValue = defaultValue;
        Choices = choices ?? [];
        MinValues = minValues;
        MaxValues = maxValues;
    }

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
    public ArgumentSpec()
    {
    }

    [JsonConstructor]
    [SetsRequiredMembers]
    internal ArgumentSpec(
        string name,
        string? help = null,
        bool required = false,
        string? defaultValue = null,
        List<string>? choices = null)
    {
        if (name is null)
        {
            throw new JsonException("Missing required property 'name'.");
        }

        Name = name;
        Help = help ?? "";
        Required = required;
        DefaultValue = defaultValue;
        Choices = choices ?? [];
    }

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
    PropertyNamingPolicy = JsonKnownNamingPolicy.Unspecified)]
[JsonSerializable(typeof(CliModel))]
public sealed partial class DocliJsonContext : JsonSerializerContext
{
}

public static class CliModelJson
{
    public static string Serialize(CliModel model)
    {
        return JsonSerializer.Serialize(model, DocliJsonContext.Default.CliModel);
    }

    public static CliModel Deserialize(string json)
    {
        var model = JsonSerializer.Deserialize(json, DocliJsonContext.Default.CliModel);
        if (model is null)
        {
            throw new JsonException("CliModel JSON was null.");
        }

        return model;
    }
}
