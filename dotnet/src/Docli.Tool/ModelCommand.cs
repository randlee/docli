using System.CommandLine;
using System.Text;
using Docli;

namespace Docli.Tool;

public static class ModelCommand
{
    public const string Name = "model";

    public static readonly Option<string> OutputOption = new("--output")
    {
        Description = "Write JSON to this path instead of stdout",
        HelpName = "PATH",
    };

    private static readonly UTF8Encoding Utf8NoBom = new(encoderShouldEmitUTF8Identifier: false);

    public static int Write(CliModel model, TextWriter stdout, TextWriter stderr, string? outputPath)
    {
        string json;
        try
        {
            json = CliModelJson.Serialize(model) + "\n";
        }
        catch (Exception ex)
        {
            stderr.WriteLine("docli: serialize failed: " + OneLine(ex.Message));
            return 2;
        }

        if (outputPath is null)
        {
            stdout.Write(json);
            return 0;
        }

        if (Directory.Exists(outputPath))
        {
            stderr.WriteLine("docli: output path is a directory: " + outputPath);
            return 2;
        }

        var parent = Path.GetDirectoryName(outputPath);
        if (!string.IsNullOrEmpty(parent) && !Directory.Exists(parent))
        {
            stderr.WriteLine("docli: output parent missing: " + outputPath);
            return 2;
        }

        try
        {
            File.WriteAllText(outputPath, json, Utf8NoBom);
            return 0;
        }
        catch (UnauthorizedAccessException)
        {
            stderr.WriteLine("docli: output unauthorized: " + outputPath);
            return 2;
        }
    }

    private static string OneLine(string message)
    {
        return message.Replace("\r\n", " ", StringComparison.Ordinal)
            .Replace('\n', ' ')
            .Replace('\r', ' ');
    }
}
