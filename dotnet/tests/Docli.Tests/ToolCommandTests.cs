using System.Diagnostics;
using System.IO.Compression;
using System.Text;
using System.Text.RegularExpressions;
using Docli;
using Docli.Tool;
using Xunit;

namespace Docli.Tests;

[Collection("ExportHost")]
public sealed class ToolCommandTests
{
    [Fact(Timeout = 60_000)]
    public async Task Model_command_writes_json()
    {
        var result = await RunTool("model");

        Assert.Equal(0, result.ExitCode);
        var model = CliModelJson.Deserialize(result.Stdout);
        Assert.Equal(ToolCommands.RootCommandName, model.Name);
        var modelCommand = Assert.Single(model.Subcommands, subcommand => subcommand.Name == "model");
        Assert.Contains(modelCommand.Options, option => option.Long == "--output");
        Assert.Equal(ToolCommands.ToolVersion(), model.Version);
    }

    [Fact(Timeout = 60_000)]
    public async Task Output_flag_writes_a_file()
    {
        var directory = Directory.CreateTempSubdirectory("docli-d4-output-");
        try
        {
            var path = Path.Combine(directory.FullName, "model.json");
            var result = await RunTool("model", "--output", path);

            Assert.Equal(0, result.ExitCode);
            Assert.DoesNotContain("\"name\"", result.Stdout, StringComparison.Ordinal);
            Assert.True(File.Exists(path));
            var model = CliModelJson.Deserialize(File.ReadAllText(path));
            Assert.Equal(ToolCommands.RootCommandName, model.Name);
            Assert.Contains(model.Subcommands, subcommand => subcommand.Name == "model");
            Assert.Contains(
                Assert.Single(model.Subcommands, subcommand => subcommand.Name == "model").Options,
                option => option.Long == "--output");
        }
        finally
        {
            directory.Delete(recursive: true);
        }
    }

    [Fact(Timeout = 60_000)]
    public async Task Missing_output_directory_exits_2()
    {
        var parent = Path.Combine(Path.GetTempPath(), "docli-d4-missing-" + Guid.NewGuid().ToString("N"));
        var path = Path.Combine(parent, "model.json");
        try
        {
            Assert.False(Directory.Exists(parent));
            var result = await RunTool("model", "--output", path);

            Assert.Equal(2, result.ExitCode);
            Assert.StartsWith(ModelCommand.StderrPrefix + "output parent missing:", SingleLine(result.Stderr), StringComparison.Ordinal);
            Assert.False(File.Exists(path));
            Assert.False(Directory.Exists(parent));

            Directory.CreateDirectory(parent);
            var recovered = RecoverToFile(path);
            Assert.Equal(0, recovered);
            Assert.True(File.Exists(path));
            Assert.Equal(ToolCommands.RootCommandName, CliModelJson.Deserialize(File.ReadAllText(path)).Name);
        }
        finally
        {
            if (Directory.Exists(parent))
            {
                Directory.Delete(parent, recursive: true);
            }
        }
    }

    [Fact(Timeout = 60_000)]
    public async Task Write_failure_exits_2()
    {
        var directory = Directory.CreateTempSubdirectory("docli-d4-isdir-");
        try
        {
            var marker = Path.Combine(directory.FullName, "marker.txt");
            File.WriteAllText(marker, "unchanged");
            var before = Directory.GetFileSystemEntries(directory.FullName).Order(StringComparer.Ordinal).ToArray();

            var result = await RunTool("model", "--output", directory.FullName);

            Assert.Equal(2, result.ExitCode);
            Assert.StartsWith(
                ModelCommand.StderrPrefix + "output path is a directory:",
                SingleLine(result.Stderr),
                StringComparison.Ordinal);
            Assert.DoesNotContain("{", result.Stdout, StringComparison.Ordinal);
            Assert.Equal(before, Directory.GetFileSystemEntries(directory.FullName).Order(StringComparer.Ordinal).ToArray());
            Assert.Equal("unchanged", File.ReadAllText(marker));

            var file = Path.Combine(directory.FullName, "model.json");
            Assert.Equal(0, RecoverToFile(file));
            Assert.True(File.Exists(file));
            Assert.Equal(ToolCommands.RootCommandName, CliModelJson.Deserialize(File.ReadAllText(file)).Name);
        }
        finally
        {
            directory.Delete(recursive: true);
        }
    }

    [Fact(Timeout = 60_000)]
    public async Task Unauthorized_output_exits_2()
    {
        var directory = Directory.CreateTempSubdirectory("docli-d4-unauth-");
        var path = Path.Combine(directory.FullName, "model.json");
        try
        {
            await DenyCurrentUserWrite(directory.FullName);
            var result = await RunTool("model", "--output", path);

            Assert.Equal(2, result.ExitCode);
            Assert.StartsWith(ModelCommand.StderrPrefix + "output unauthorized:", SingleLine(result.Stderr), StringComparison.Ordinal);
            Assert.DoesNotContain("{", result.Stdout, StringComparison.Ordinal);
            Assert.False(File.Exists(path));

            await AllowCurrentUserWrite(directory.FullName);
            var writable = Path.Combine(Path.GetTempPath(), "docli-d4-writable-" + Guid.NewGuid().ToString("N") + ".json");
            try
            {
                Assert.Equal(0, RecoverToFile(writable));
                Assert.True(File.Exists(writable));
                Assert.Equal(ToolCommands.RootCommandName, CliModelJson.Deserialize(File.ReadAllText(writable)).Name);
            }
            finally
            {
                if (File.Exists(writable))
                {
                    File.Delete(writable);
                }
            }
        }
        finally
        {
            await AllowCurrentUserWrite(directory.FullName);
            directory.Delete(recursive: true);
        }
    }

    [Fact(Timeout = 60_000)]
    public async Task Serialize_failure_exits_2()
    {
        var directory = Directory.CreateTempSubdirectory("docli-d4-serialize-");
        try
        {
            var cyclic = new CliModel { Name = "cycle" };
            cyclic.Subcommands.Add(cyclic);
            var path = Path.Combine(directory.FullName, "cycle.json");
            var stdout = new StringWriter();
            var stderr = new StringWriter();

            var exit = ModelCommand.Write(cyclic, stdout, stderr, path);

            Assert.Equal(2, exit);
            Assert.StartsWith(ModelCommand.StderrPrefix + "serialize failed:", SingleLine(stderr.ToString()), StringComparison.Ordinal);
            Assert.DoesNotContain("{", stdout.ToString(), StringComparison.Ordinal);
            Assert.False(File.Exists(path));

            var acyclicPath = Path.Combine(directory.FullName, "ok.json");
            var recoveredStdout = new StringWriter();
            var recoveredStderr = new StringWriter();
            var recovered = ModelCommand.Write(
                new CliModel { Name = "ok" },
                recoveredStdout,
                recoveredStderr,
                acyclicPath);
            Assert.Equal(0, recovered);
            Assert.True(File.Exists(acyclicPath));
            Assert.Equal("ok", CliModelJson.Deserialize(File.ReadAllText(acyclicPath)).Name);
            await Task.CompletedTask;
        }
        finally
        {
            directory.Delete(recursive: true);
        }
    }

    [Fact(Timeout = 60_000)]
    public async Task Tool_has_no_html_or_markdown_command()
    {
        var root = await RunTool("--help");
        var model = await RunTool("model", "--help");
        var stdout = root.Stdout + model.Stdout;

        Assert.Contains("model", stdout, StringComparison.Ordinal);
        Assert.Contains("--output", stdout, StringComparison.Ordinal);
        Assert.DoesNotContain("generate", stdout, StringComparison.Ordinal);
        Assert.DoesNotContain("show", stdout, StringComparison.Ordinal);
        Assert.DoesNotContain("--html", stdout, StringComparison.Ordinal);
        Assert.DoesNotContain("--markdown", stdout, StringComparison.Ordinal);
    }

    // Same 10s slack as DotnetGenerateOrchestrationTests (170s inside 180s).
    private const int ModelHtmlRunToolSeconds = 55;
    private const int ModelHtmlGenerateSeconds = 150;
    private const int ModelHtmlTimeoutMarginSeconds = 10;

    [Fact(Timeout = (ModelHtmlRunToolSeconds + ModelHtmlGenerateSeconds + ModelHtmlTimeoutMarginSeconds) * 1_000)]
    public async Task Model_json_generates_html()
    {
        var directory = Directory.CreateTempSubdirectory("docli-d4-html-");
        try
        {
            var modelPath = Path.Combine(directory.FullName, "model.json");
            var htmlDir = Path.Combine(directory.FullName, "html");
            Directory.CreateDirectory(htmlDir);

            var result = await RunTool(TimeSpan.FromSeconds(ModelHtmlRunToolSeconds), "model", "--output", modelPath);
            Assert.Equal(0, result.ExitCode);
            var model = CliModelJson.Deserialize(File.ReadAllText(modelPath));
            Assert.Equal(ToolCommands.RootCommandName, model.Name);

            var generated = RustDocli.Generate(
                RepoRoot.Find(),
                modelPath,
                htmlDir,
                null,
                TimeSpan.FromSeconds(ModelHtmlGenerateSeconds));
            Assert.Equal(0, generated.ExitCode);
            Assert.True(File.Exists(Path.Combine(htmlDir, "index.html")));
        }
        finally
        {
            directory.Delete(recursive: true);
        }
    }

    [Fact(Timeout = 60_000)]
    public async Task Generate_captures_rust_failure()
    {
        var htmlDir = Directory.CreateTempSubdirectory("docli-d4-rust-fail-");
        try
        {
            var missing = Path.Combine(
                Path.GetTempPath(),
                "docli-d4-missing-input-" + Guid.NewGuid().ToString("N") + ".json");
            var result = RustDocli.Generate(
                RepoRoot.Find(),
                missing,
                htmlDir.FullName,
                null,
                TimeSpan.FromSeconds(40));

            Assert.NotEqual(0, result.ExitCode);
            Assert.False(string.IsNullOrEmpty(result.Stderr));
            await Task.CompletedTask;
        }
        finally
        {
            htmlDir.Delete(recursive: true);
        }
    }

    [Fact(Timeout = 180_000)]
    public async Task Pack_produces_tool_package()
    {
        var temp = Path.Combine(Path.GetTempPath(), "docli-d4-pack-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(temp);
        try
        {
            var repo = RepoRoot.Find();
            var csproj = Path.Combine(repo, "dotnet", "src", "Docli.Tool", "Docli.Tool.csproj");
            var packed = await RunProcess(
                "dotnet",
                repo,
                TimeSpan.FromSeconds(170),
                "pack",
                csproj,
                "-c",
                "Release",
                "-o",
                temp);
            Assert.Equal(0, packed.ExitCode);

            var nupkg = Assert.Single(Directory.GetFiles(temp, "*.nupkg"));
            using var archive = ZipFile.OpenRead(nupkg);
            var tfm = ToolTfm(csproj);
            var settingsEntry = archive.GetEntry($"tools/{tfm}/any/DotnetToolSettings.xml");
            Assert.NotNull(settingsEntry);
            using (var reader = new StreamReader(settingsEntry.Open()))
            {
                var settings = reader.ReadToEnd();
                Assert.Contains("<Command Name=\"docli-dotnet\"", settings, StringComparison.Ordinal);
            }

            var nuspec = Assert.Single(
                archive.Entries,
                entry => entry.FullName.EndsWith(".nuspec", StringComparison.OrdinalIgnoreCase));
            using var nuspecReader = new StreamReader(nuspec.Open());
            var nuspecXml = nuspecReader.ReadToEnd();
            Assert.Contains("<id>Docli.Tool</id>", nuspecXml, StringComparison.Ordinal);
        }
        finally
        {
            if (Directory.Exists(temp))
            {
                Directory.Delete(temp, recursive: true);
            }
        }
    }

    private static int RecoverToFile(string path)
    {
        var model = CommandLineAdapter.FromCommand(ToolCommands.Create(), ToolCommands.ToolVersion());
        return ModelCommand.Write(model, new StringWriter(), new StringWriter(), path);
    }

    private static string SingleLine(string stderr)
    {
        var line = stderr.TrimEnd('\r', '\n');
        Assert.DoesNotContain('\n', line);
        Assert.DoesNotContain('\r', line);
        return line;
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
        return RunTool(TimeSpan.FromSeconds(55), args);
    }

    private static Task<(int ExitCode, string Stdout, string Stderr)> RunTool(TimeSpan timeout, params string[] args)
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
        return RunProcess("dotnet", repo, timeout, command.ToArray());
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

    private static async Task DenyCurrentUserWrite(string directory)
    {
        if (OperatingSystem.IsWindows())
        {
            await RunChecked("icacls", directory, "/deny", WindowsAccount() + ":(OI)(CI)(W,DC)");
            return;
        }

        File.SetUnixFileMode(
            directory,
            UnixFileMode.UserRead
                | UnixFileMode.UserExecute
                | UnixFileMode.GroupRead
                | UnixFileMode.GroupExecute
                | UnixFileMode.OtherRead
                | UnixFileMode.OtherExecute);
    }

    private static async Task AllowCurrentUserWrite(string directory)
    {
        if (OperatingSystem.IsWindows())
        {
            await RunChecked("icacls", directory, "/remove:d", WindowsAccount());
            return;
        }

        File.SetUnixFileMode(
            directory,
            UnixFileMode.UserRead | UnixFileMode.UserWrite | UnixFileMode.UserExecute);
    }

    private static string WindowsAccount()
    {
        return Environment.UserDomainName + "\\" + Environment.UserName;
    }

    private static async Task RunChecked(string fileName, params string[] args)
    {
        var result = await RunProcess(fileName, Directory.GetCurrentDirectory(), TimeSpan.FromSeconds(30), args);
        if (result.ExitCode != 0)
        {
            throw new InvalidOperationException(
                $"{fileName} exited {result.ExitCode}. stdout: {result.Stdout} stderr: {result.Stderr}");
        }
    }
}
