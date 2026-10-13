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
        var binaryName = OperatingSystem.IsWindows() ? "docli.exe" : "docli";
        var binaryPath = Path.Combine(repoRoot, "target", "debug", binaryName);
        var start = new ProcessStartInfo
        {
            WorkingDirectory = repoRoot,
            UseShellExecute = false,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            StandardOutputEncoding = Encoding.UTF8,
            StandardErrorEncoding = Encoding.UTF8,
        };

        if (File.Exists(binaryPath))
        {
            start.FileName = binaryPath;
        }
        else
        {
            start.FileName = "cargo";
            start.ArgumentList.Add("run");
            start.ArgumentList.Add("--quiet");
            start.ArgumentList.Add("--bin");
            start.ArgumentList.Add("docli");
            start.ArgumentList.Add("--");
        }

        start.ArgumentList.Add("generate");
        start.ArgumentList.Add("--input");
        start.ArgumentList.Add(inputPath);
        start.ArgumentList.Add("--html");
        start.ArgumentList.Add(htmlDir);
        if (markdownPath is not null)
        {
            start.ArgumentList.Add("--markdown");
            start.ArgumentList.Add(markdownPath);
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
