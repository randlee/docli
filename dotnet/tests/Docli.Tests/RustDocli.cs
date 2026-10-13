using System.Diagnostics;
using System.Text;

namespace Docli.Tests;

public readonly record struct GenerateResult(int ExitCode, string Stdout, string Stderr);

public static class RustDocli
{
    public static GenerateResult Generate(
        string repoRoot,
        string inputPath,
        string htmlDir,
        string? markdownPath,
        TimeSpan timeout)
    {
        var args = new List<string>
        {
            "generate",
            "--input",
            inputPath,
            "--html",
            htmlDir,
        };
        if (markdownPath is not null)
        {
            args.Add("--markdown");
            args.Add(markdownPath);
        }

        return Run(repoRoot, timeout, args);
    }

    public static GenerateResult GenerateDotnetProject(
        string repoRoot,
        string csproj,
        string htmlDir,
        string? markdownPath,
        TimeSpan timeout)
    {
        var args = new List<string>
        {
            "generate",
            "--dotnet-project",
            csproj,
            "--html",
            htmlDir,
        };
        if (markdownPath is not null)
        {
            args.Add("--markdown");
            args.Add(markdownPath);
        }

        return Run(repoRoot, timeout, args);
    }

    private static GenerateResult Run(string repoRoot, TimeSpan timeout, IReadOnlyList<string> args)
    {
        var binaryName = OperatingSystem.IsWindows() ? "docli.exe" : "docli";
        var binaryPath = Path.Combine(repoRoot, "target", "debug", binaryName);
        var start = new ProcessStartInfo
        {
            WorkingDirectory = repoRoot,
            UseShellExecute = false,
            RedirectStandardInput = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            StandardOutputEncoding = Encoding.UTF8,
            StandardErrorEncoding = Encoding.UTF8,
        };

        if (!File.Exists(binaryPath))
        {
            throw new FileNotFoundException(
                $"'{binaryPath}' was not found. Run `cargo build --bin docli` from the repo root before dotnet test. RustDocli does not fall back to `cargo run`.");
        }

        start.FileName = binaryPath;

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
        catch (Exception ex) when (ex is not TimeoutException)
        {
            throw new InvalidOperationException(
                $"Failed to start '{start.FileName}'. arguments: {string.Join(" ", start.ArgumentList)}. stdout: '' stderr: '' {ex.Message}",
                ex);
        }

        using (process)
        {
            process.StandardInput.Close();
            var stdoutTask = process.StandardOutput.ReadToEndAsync();
            var stderrTask = process.StandardError.ReadToEndAsync();
            var milliseconds = (int)Math.Min(int.MaxValue, Math.Max(0, timeout.TotalMilliseconds));
            if (!process.WaitForExit(milliseconds))
            {
                try
                {
                    process.Kill(entireProcessTree: true);
                }
                catch (InvalidOperationException)
                {
                }

                process.WaitForExit(5_000);
                var stdout = CompletedText(stdoutTask);
                var stderr = CompletedText(stderrTask);
                throw new TimeoutException(
                    $"docli generate timed out after {timeout}. stdout: {stdout} stderr: {stderr}");
            }

            process.WaitForExit();
            return new GenerateResult(
                process.ExitCode,
                stdoutTask.GetAwaiter().GetResult(),
                stderrTask.GetAwaiter().GetResult());
        }
    }

    private static string CompletedText(Task<string> task)
    {
        if (task.Wait(TimeSpan.FromSeconds(5)))
        {
            return task.Result;
        }

        return "";
    }
}
