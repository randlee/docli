using System.CommandLine;
using System.CommandLine.Help;
using System.Reflection;
using Docli;

namespace Docli.Tool;

public static class ToolCommands
{
    public static Command Create()
    {
        var root = new Command("docli", "Emit a System.CommandLine tree as docli JSON");
        root.Options.Add(new HelpOption());

        var model = new Command(ModelCommand.Name, "Write CliModel JSON for this tool");
        model.Options.Add(ModelCommand.OutputOption);
        model.SetAction((ParseResult parseResult) =>
        {
            var outputPath = parseResult.GetValue(ModelCommand.OutputOption);
            var tree = CommandLineAdapter.FromCommand(Create(), ToolVersion());
            return ModelCommand.Write(tree, Console.Out, Console.Error, outputPath);
        });
        root.Subcommands.Add(model);
        return root;
    }

    public static string? ToolVersion()
    {
        var assembly = typeof(ToolCommands).Assembly;
        var informational = assembly
            .GetCustomAttribute<AssemblyInformationalVersionAttribute>()
            ?.InformationalVersion;
        if (!string.IsNullOrEmpty(informational))
        {
            return informational;
        }

        return assembly.GetName().Version?.ToString();
    }
}
