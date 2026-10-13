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

        var budget = ProjectTimeout();
        var clock = Stopwatch.StartNew();
        var build = RunDotnet(workDir, Remaining(budget, clock), "build", fullPath, "-c", "Release");
        if (build.TimedOut)
        {
            stderr.WriteLine(ModelCommand.StderrPrefix + Detail("build timed out: " + projectPath, build.Stdout, build.Stderr));
            return 2;
        }

        if (build.ExitCode != 0)
        {
            stderr.WriteLine(ModelCommand.StderrPrefix + Detail("build failed: " + projectPath, build.Stdout, build.Stderr));
            return 2;
        }

        var run = RunDotnet(workDir, Remaining(budget, clock), "run", "--project", fullPath, "-c", "Release", "--no-build");
        if (run.TimedOut)
        {
            stderr.WriteLine(ModelCommand.StderrPrefix + Detail("run timed out: " + projectPath, run.Stdout, run.Stderr));
            return 2;
        }

        if (run.ExitCode != 0)
        {
            stderr.WriteLine(ModelCommand.StderrPrefix + Detail("run failed: " + projectPath, run.Stdout, run.Stderr));
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

    /// Characters kept from the end of a collapsed child diagnostic.
    /// The build banner is at the start; the compiler error is at the end.
    private const int MaxDetailChars = 1024;

    private const int DefaultTimeoutSeconds = 180;

    private static TimeSpan ProjectTimeout()
    {
        var raw = Environment.GetEnvironmentVariable("DOCLI_DOTNET_PROJECT_TIMEOUT_SECS");
        if (raw is not null
            && int.TryParse(raw.Trim(), out var seconds)
            && seconds is > 0 and <= 86_400)
        {
            return TimeSpan.FromSeconds(seconds);
        }

        return TimeSpan.FromSeconds(DefaultTimeoutSeconds);
    }

    private static TimeSpan Remaining(TimeSpan budget, Stopwatch clock)
    {
        var left = budget - clock.Elapsed;
        return left > TimeSpan.Zero ? left : TimeSpan.Zero;
    }

    private static string Detail(string message, string stdout, string stderr)
    {
        var detail = OneLine(DiagnosticText(stdout, stderr));
        if (detail.Length == 0)
        {
            return message;
        }

        if (detail.Length > MaxDetailChars)
        {
            detail = detail[^MaxDetailChars..];
        }

        return message + ": " + detail;
    }

    /// `dotnet build` writes compiler errors on stdout and leaves stderr empty.
    private static string DiagnosticText(string stdout, string stderr)
    {
        var outHasError = stdout.Contains("error", StringComparison.OrdinalIgnoreCase);
        var errHasError = stderr.Contains("error", StringComparison.OrdinalIgnoreCase);
        if (errHasError && !outHasError)
        {
            return stderr;
        }

        if (!string.IsNullOrWhiteSpace(stdout) && (outHasError || string.IsNullOrWhiteSpace(stderr)))
        {
            return stdout;
        }

        return stderr;
    }

    private static string OneLine(string message)
    {
        var collapsed = string.Join(
            ' ',
            message.Split((char[]?)null, StringSplitOptions.RemoveEmptyEntries));
        if (collapsed.Length <= MaxDetailChars)
        {
            return collapsed;
        }

        return collapsed[^MaxDetailChars..];
    }

    private static (int ExitCode, string Stdout, string Stderr, bool TimedOut) RunDotnet(
        string workingDirectory,
        TimeSpan timeout,
        params string[] args)
    {
        if (timeout <= TimeSpan.Zero)
        {
            return (-1, "", "", true);
        }

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
            return (-1, "", ex.Message, false);
        }

        using (process)
        {
            process.StandardInput.Close();
            var stdoutTask = process.StandardOutput.ReadToEndAsync();
            var stderrTask = process.StandardError.ReadToEndAsync();
            var milliseconds = (int)Math.Clamp(timeout.TotalMilliseconds, 1, int.MaxValue);
            if (!process.WaitForExit(milliseconds))
            {
                try
                {
                    process.Kill(entireProcessTree: true);
                }
                catch (Exception ex) when (ex is InvalidOperationException or System.ComponentModel.Win32Exception)
                {
                }

                process.WaitForExit();
                return (
                    -1,
                    stdoutTask.GetAwaiter().GetResult(),
                    stderrTask.GetAwaiter().GetResult(),
                    true);
            }

            return (
                process.ExitCode,
                stdoutTask.GetAwaiter().GetResult(),
                stderrTask.GetAwaiter().GetResult(),
                false);
        }
    }
}
