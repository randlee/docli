using System.CommandLine;
using System.Diagnostics;
using System.Text;
using Docli;

namespace Docli.Tool;

public static class ProjectCommand
{
    public const string Name = "project";

    public static readonly Option<string> ProjectOption = new("--project", "-p")
    {
        Description = "Path to the export .csproj",
        HelpName = "CSPROJ",
        Required = true,
    };

    public static readonly Option<string> OutputOption = new("--output", "-o")
    {
        Description = "Write JSON to this path instead of stdout",
        HelpName = "FILE",
    };

    public static int Run(string projectPath, TextWriter stdout, TextWriter stderr, string? outputPath)
    {
        if (string.IsNullOrWhiteSpace(projectPath))
        {
            stderr.WriteLine(ModelCommand.StderrPrefix + "project file not found: " + projectPath);
            return 2;
        }

        var fullPath = Path.GetFullPath(projectPath);
        if (!File.Exists(fullPath))
        {
            stderr.WriteLine(ModelCommand.StderrPrefix + "project file not found: " + projectPath);
            return 2;
        }

        // Absolute project path plus the csproj directory as cwd, so `dotnet`
        // resolves the export host the same way from any tool working directory.
        var workDir = Path.GetDirectoryName(fullPath);
        if (string.IsNullOrEmpty(workDir))
        {
            stderr.WriteLine(ModelCommand.StderrPrefix + "project file not found: " + projectPath);
            return 2;
        }

        var build = RunDotnet(workDir, "build", fullPath, "-c", "Release");
        if (build.ExitCode != 0)
        {
            stderr.WriteLine(ModelCommand.StderrPrefix + Detail("build failed: " + projectPath, build.Stderr));
            return 2;
        }

        var run = RunDotnet(workDir, "run", "--project", fullPath, "-c", "Release", "--no-build");
        if (run.ExitCode != 0)
        {
            stderr.WriteLine(ModelCommand.StderrPrefix + Detail("run failed: " + projectPath, run.Stderr));
            return 2;
        }

        if (string.IsNullOrWhiteSpace(run.Stdout))
        {
            stderr.WriteLine(ModelCommand.StderrPrefix + "empty export stdout: " + projectPath);
            return 2;
        }

        CliModel model;
        try
        {
            model = CliModelJson.Deserialize(run.Stdout);
        }
        catch (Exception ex)
        {
            stderr.WriteLine(ModelCommand.StderrPrefix + "json parse failed: " + OneLine(ex.Message));
            return 2;
        }

        return ModelCommand.Write(model, stdout, stderr, outputPath);
    }

    private static string Detail(string message, string childStderr)
    {
        var detail = OneLine(childStderr);
        if (detail.Length == 0)
        {
            return message;
        }

        const int maxDetail = 240;
        if (detail.Length > maxDetail)
        {
            detail = detail[..maxDetail];
        }

        return message + ": " + detail;
    }

    private static string OneLine(string message)
    {
        return message.Replace("\r\n", " ", StringComparison.Ordinal)
            .Replace('\n', ' ')
            .Replace('\r', ' ')
            .Trim();
    }

    private static (int ExitCode, string Stdout, string Stderr) RunDotnet(
        string workingDirectory,
        params string[] args)
    {
        var start = new ProcessStartInfo
        {
            FileName = "dotnet",
            WorkingDirectory = workingDirectory,
            UseShellExecute = false,
            RedirectStandardInput = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            StandardOutputEncoding = Encoding.UTF8,
            StandardErrorEncoding = Encoding.UTF8,
        };
        // A `dotnet test` host keeps an MSBuild node. Reusing it from this child deadlocks.
        start.Environment["MSBUILDDISABLENODEREUSE"] = "1";
        foreach (var arg in args)
        {
            start.ArgumentList.Add(arg);
        }

        Process process;
        try
        {
            process = Process.Start(start)
                ?? throw new InvalidOperationException("Process.Start returned null.");
        }
        catch (Exception ex)
        {
            return (-1, "", ex.Message);
        }

        using (process)
        {
            process.StandardInput.Close();
            var stdoutTask = process.StandardOutput.ReadToEndAsync();
            var stderrTask = process.StandardError.ReadToEndAsync();
            process.WaitForExit();
            return (
                process.ExitCode,
                stdoutTask.GetAwaiter().GetResult(),
                stderrTask.GetAwaiter().GetResult());
        }
    }
}
