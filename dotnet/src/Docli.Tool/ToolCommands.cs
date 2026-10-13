using System.CommandLine;
using System.CommandLine.Help;
using System.Reflection;
using Docli;

namespace Docli.Tool;

public static class ToolCommands
{
    public const string RootCommandName = "docli-dotnet";

    public static Command Create()
    {
        var root = new Command(RootCommandName, "Emit a System.CommandLine tree as docli JSON");
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

        var project = new Command(ProjectCommand.Name, "Run a .NET export host and write CliModel JSON");
        project.Options.Add(ProjectCommand.ProjectOption);
        project.Options.Add(ProjectCommand.OutputOption);
        project.SetAction((ParseResult parseResult) =>
        {
            var projectPath = parseResult.GetValue(ProjectCommand.ProjectOption);
            var outputPath = parseResult.GetValue(ProjectCommand.OutputOption);
            if (string.IsNullOrWhiteSpace(projectPath))
            {
                Console.Error.WriteLine(ModelCommand.StderrPrefix + "project file not found: " + projectPath);
                return 2;
            }

            return ProjectCommand.Run(projectPath, Console.Out, Console.Error, outputPath);
        });
        root.Subcommands.Add(project);
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
