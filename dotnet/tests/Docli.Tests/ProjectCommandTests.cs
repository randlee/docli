using System.Diagnostics;
using System.Text;
using System.Text.RegularExpressions;
using Docli;
using Docli.Tool;
using Xunit;

namespace Docli.Tests;

[CollectionDefinition("ExportHost")]
public sealed class ExportHostCollection
{
}

[Collection("ExportHost")]
public sealed class ProjectCommandTests
{
    [Fact(Timeout = 180_000)]
    public async Task Project_stdout_json_is_sample_tree()
    {
        var result = await RunTool("project", "--project", ExportCsproj());

        Assert.Equal(0, result.ExitCode);
        Assert.True(result.Stderr.Length == 0, result.Stderr);
        var model = CliModelJson.Deserialize(result.Stdout);
        Assert.Equal("sample", model.Name);
        Assert.Contains(model.Subcommands, subcommand => subcommand.Name == "run");
        Assert.Contains(model.Options, option => option.Long == "--output");
    }

    [Fact(Timeout = 180_000)]
    public async Task Project_output_flag_writes_file()
    {
        var directory = Directory.CreateTempSubdirectory("docli-d6-output-");
        try
        {
            var path = Path.Combine(directory.FullName, "sample.json");
            var result = await RunTool("project", "--project", ExportCsproj(), "--output", path);

            Assert.Equal(0, result.ExitCode);
            Assert.DoesNotContain("\"name\"", result.Stdout, StringComparison.Ordinal);
            Assert.True(File.Exists(path));
            var model = CliModelJson.Deserialize(File.ReadAllText(path));
            Assert.Equal("sample", model.Name);
            Assert.Contains(model.Subcommands, subcommand => subcommand.Name == "run");
            Assert.Contains(model.Options, option => option.Long == "--output");
        }
        finally
        {
            directory.Delete(recursive: true);
        }
    }

    [Fact(Timeout = 60_000)]
    public async Task Project_missing_csproj_exits_2()
    {
        var missing = Path.Combine(
            Path.GetTempPath(),
            "docli-d6-missing-" + Guid.NewGuid().ToString("N") + ".csproj");
        var result = await RunTool("project", "--project", missing);

        Assert.Equal(2, result.ExitCode);
        Assert.StartsWith(ModelCommand.StderrPrefix, result.Stderr, StringComparison.Ordinal);
        Assert.Contains("project file not found:", result.Stderr, StringComparison.Ordinal);
    }

    [Fact(Timeout = 60_000)]
    public async Task Project_help_lists_project_and_output()
    {
        var root = await RunTool("--help");
        var project = await RunTool("project", "--help");
        var stdout = root.Stdout + project.Stdout;

        Assert.Contains("model", root.Stdout, StringComparison.Ordinal);
        Assert.Contains("project", root.Stdout, StringComparison.Ordinal);
        Assert.Contains("--project", project.Stdout, StringComparison.Ordinal);
        Assert.Contains("--output", project.Stdout, StringComparison.Ordinal);
        Assert.DoesNotContain("generate", stdout, StringComparison.Ordinal);
        Assert.DoesNotContain("show", stdout, StringComparison.Ordinal);
        Assert.DoesNotContain("--html", stdout, StringComparison.Ordinal);
        Assert.DoesNotContain("--markdown", stdout, StringComparison.Ordinal);
    }

    private static string ExportCsproj()
    {
        return Path.Combine(
            RepoRoot.Find(),
            "dotnet",
            "samples",
            "Docli.Sample.Export",
            "Docli.Sample.Export.csproj");
    }

    private static string ToolTfm(string csproj)
    {
        var text = File.ReadAllText(csproj);
        var match = Regex.Match(text, "<TargetFramework>([^<]+)</TargetFramework>");
        if (!match.Success)
        {
            throw new InvalidOperationException($"No TargetFramework in '{csproj}'.");
        }

        return match.Groups[1].Value;
    }

    private static Task<(int ExitCode, string Stdout, string Stderr)> RunTool(params string[] args)
    {
        var repo = RepoRoot.Find();
        var csproj = Path.Combine(repo, "dotnet", "src", "Docli.Tool", "Docli.Tool.csproj");
        var dll = Path.Combine(
            repo,
            "dotnet",
            "src",
            "Docli.Tool",
            "bin",
            "Release",
            ToolTfm(csproj),
            "Docli.Tool.dll");
        if (!File.Exists(dll))
        {
            throw new FileNotFoundException(
                $"'{dll}' was not found. This run requires -c Release.");
        }

        var command = new List<string> { dll };
        command.AddRange(args);
        return RunProcess("dotnet", repo, TimeSpan.FromSeconds(170), command.ToArray());
    }

    private static async Task<(int ExitCode, string Stdout, string Stderr)> RunProcess(
        string fileName,
        string workingDirectory,
        TimeSpan timeout,
        params string[] args)
    {
        var start = new ProcessStartInfo
        {
            FileName = fileName,
            WorkingDirectory = workingDirectory,
            UseShellExecute = false,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            StandardOutputEncoding = Encoding.UTF8,
            StandardErrorEncoding = Encoding.UTF8,
        };
        foreach (var arg in args)
        {
            start.ArgumentList.Add(arg);
        }

        using var process = Process.Start(start)
            ?? throw new InvalidOperationException($"Failed to start '{fileName}'.");
        var stdoutTask = process.StandardOutput.ReadToEndAsync();
        var stderrTask = process.StandardError.ReadToEndAsync();
        using var timeoutSource = new CancellationTokenSource(timeout);
        try
        {
            await process.WaitForExitAsync(timeoutSource.Token);
        }
        catch (OperationCanceledException)
        {
            try
            {
                process.Kill(entireProcessTree: true);
            }
            catch (InvalidOperationException)
            {
            }

            await process.WaitForExitAsync();
            var stdout = stdoutTask.IsCompleted ? stdoutTask.Result : "";
            var stderr = stderrTask.IsCompleted ? stderrTask.Result : "";
            throw new TimeoutException(
                $"'{fileName}' timed out after {timeout}. stdout: {stdout} stderr: {stderr}");
        }

        return (process.ExitCode, await stdoutTask, await stderrTask);
    }
}
