using Docli;
using Docli.Sample;

namespace Docli.Sample.Export;

public static class Program
{
    public static int Main()
    {
        var json = CliModelJson.Serialize(CommandLineAdapter.FromCommand(SampleCommands.Create()));
        Console.Out.Write(json);
        return 0;
    }
}
