using System.CommandLine;

namespace Docli.Sample;

public static class SampleCommands
{
    public static Command Create()
    {
        var sample = new Command("sample", "Sample command tree");
        sample.Options.Add(new Option<string>("--output", "-o")
        {
            Description = "Write output to PATH",
            HelpName = "PATH",
            Required = false,
        });
        sample.Subcommands.Add(new Command("run", "Run the thing"));
        return sample;
    }
}
