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

    [Fact(Timeout = 120_000)]
    public async Task Project_build_failure_exits_2()
    {
        using var host = new ExportHost("buildfail", "this is not csharp;\n");
        var result = await RunTool("project", "--project", host.Csproj);

        Assert.Equal(2, result.ExitCode);
        Assert.StartsWith(ModelCommand.StderrPrefix, result.Stderr, StringComparison.Ordinal);
        Assert.Contains("build failed:", result.Stderr, StringComparison.Ordinal);
        Assert.Contains(host.Csproj, result.Stderr, StringComparison.Ordinal);
        Assert.Contains("error CS", result.Stderr, StringComparison.Ordinal);
        Assert.DoesNotContain("run failed:", result.Stderr, StringComparison.Ordinal);
        Assert.DoesNotContain("\"name\"", result.Stdout, StringComparison.Ordinal);
    }

    [Fact(Timeout = 120_000)]
    public async Task Project_run_failure_exits_2()
    {
        using var host = new ExportHost("runfail", "Environment.Exit(7);\n");
        var result = await RunTool("project", "--project", host.Csproj);

        Assert.Equal(2, result.ExitCode);
        Assert.StartsWith(ModelCommand.StderrPrefix, result.Stderr, StringComparison.Ordinal);
        Assert.Contains("run failed:", result.Stderr, StringComparison.Ordinal);
        Assert.Contains(host.Csproj, result.Stderr, StringComparison.Ordinal);
        Assert.DoesNotContain("build failed:", result.Stderr, StringComparison.Ordinal);
        Assert.DoesNotContain("empty export stdout:", result.Stderr, StringComparison.Ordinal);
        Assert.DoesNotContain("json parse failed:", result.Stderr, StringComparison.Ordinal);
        Assert.DoesNotContain("\"name\"", result.Stdout, StringComparison.Ordinal);
    }

    [Fact(Timeout = 120_000)]
    public async Task Project_empty_stdout_exits_2()
    {
        using var host = new ExportHost("empty", "Environment.Exit(0);\n");
        var result = await RunTool("project", "--project", host.Csproj);

        Assert.Equal(2, result.ExitCode);
        Assert.StartsWith(ModelCommand.StderrPrefix, result.Stderr, StringComparison.Ordinal);
        Assert.Contains("empty export stdout:", result.Stderr, StringComparison.Ordinal);
        Assert.Contains(host.Csproj, result.Stderr, StringComparison.Ordinal);
        Assert.DoesNotContain("json parse failed:", result.Stderr, StringComparison.Ordinal);
        Assert.DoesNotContain("\"name\"", result.Stdout, StringComparison.Ordinal);
    }

    [Fact(Timeout = 120_000)]
    public async Task Project_invalid_json_exits_2()
    {
        using var host = new ExportHost("badjson", "Console.Write(\"not-json\");\n");
        var result = await RunTool("project", "--project", host.Csproj);

        Assert.Equal(2, result.ExitCode);
        Assert.StartsWith(ModelCommand.StderrPrefix, result.Stderr, StringComparison.Ordinal);
        const string marker = "json parse failed:";
        Assert.Contains(marker, result.Stderr, StringComparison.Ordinal);
        var detail = result.Stderr[(result.Stderr.IndexOf(marker, StringComparison.Ordinal) + marker.Length)..].Trim();
        Assert.True(detail.Length > 0, result.Stderr);
        Assert.DoesNotContain("empty export stdout:", result.Stderr, StringComparison.Ordinal);
        Assert.DoesNotContain("\"name\"", result.Stdout, StringComparison.Ordinal);
    }

    [Fact(Timeout = 90_000)]
    public async Task Project_run_timeout_exits_2()
    {
        using var host = new ExportHost("hang", "Thread.Sleep(TimeSpan.FromSeconds(45));\n");
        var prebuild = await RunProcess(
            "dotnet",
            host.Directory,
            TimeSpan.FromSeconds(60),
            "build",
            host.Csproj,
            "-c",
            "Release",
            "--nologo");
        Assert.Equal(0, prebuild.ExitCode);

        var started = Stopwatch.StartNew();
        var result = await RunTool(
            new Dictionary<string, string>
            {
                ["DOCLI_DOTNET_PROJECT_TIMEOUT_SECS"] = "15",
            },
            "project",
            "--project",
            host.Csproj);

        Assert.Equal(2, result.ExitCode);
        Assert.StartsWith(ModelCommand.StderrPrefix, result.Stderr, StringComparison.Ordinal);
        Assert.Contains("run timed out:", result.Stderr, StringComparison.Ordinal);
        Assert.Contains(host.Csproj, result.Stderr, StringComparison.Ordinal);
        Assert.DoesNotContain("build timed out:", result.Stderr, StringComparison.Ordinal);
        Assert.DoesNotContain("build failed:", result.Stderr, StringComparison.Ordinal);
        Assert.True(
            started.Elapsed < TimeSpan.FromSeconds(40),
            $"deadline did not fire; elapsed {started.Elapsed}; stderr: {result.Stderr}");
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
        return RunTool(new Dictionary<string, string>(), args);
    }

    private static Task<(int ExitCode, string Stdout, string Stderr)> RunTool(
        IReadOnlyDictionary<string, string> environment,
        params string[] args)
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
        return RunProcess("dotnet", repo, TimeSpan.FromSeconds(170), environment, command.ToArray());
    }

    private static Task<(int ExitCode, string Stdout, string Stderr)> RunProcess(
        string fileName,
        string workingDirectory,
        TimeSpan timeout,
        params string[] args)
    {
        return RunProcess(fileName, workingDirectory, timeout, new Dictionary<string, string>(), args);
    }

    private static async Task<(int ExitCode, string Stdout, string Stderr)> RunProcess(
        string fileName,
        string workingDirectory,
        TimeSpan timeout,
        IReadOnlyDictionary<string, string> environment,
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
        foreach (var pair in environment)
        {
            start.Environment[pair.Key] = pair.Value;
        }

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

    private sealed class ExportHost : IDisposable
    {
        public ExportHost(string label, string program)
        {
            Directory = System.IO.Directory.CreateTempSubdirectory("docli-d6-" + label + "-").FullName;
            Csproj = Path.Combine(Directory, label + ".csproj");
            File.Copy(Path.Combine(RepoRoot.Find(), "global.json"), Path.Combine(Directory, "global.json"));
            File.WriteAllText(
                Csproj,
                """
                <Project Sdk="Microsoft.NET.Sdk">
                  <PropertyGroup>
                    <OutputType>Exe</OutputType>
                    <TargetFramework>net10.0</TargetFramework>
                    <Nullable>enable</Nullable>
                    <ImplicitUsings>enable</ImplicitUsings>
                  </PropertyGroup>
                </Project>
                """);
            File.WriteAllText(Path.Combine(Directory, "Program.cs"), program);
        }

        public string Directory { get; }

        public string Csproj { get; }

        public void Dispose()
        {
            try
            {
                System.IO.Directory.Delete(Directory, recursive: true);
            }
            catch (IOException)
            {
            }
        }
    }
}
