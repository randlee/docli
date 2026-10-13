using Docli;
using Xunit;

namespace Docli.Tests;

[Collection("ExportHost")]
public sealed class DotnetGenerateOrchestrationTests
{
    [Fact(Timeout = 180_000)]
    public async Task Generate_dotnet_project_writes_html()
    {
        var directory = Directory.CreateTempSubdirectory("docli-d6-html-");
        try
        {
            var repo = RepoRoot.Find();
            var csproj = Path.Combine(
                repo,
                "dotnet",
                "samples",
                "Docli.Sample.Export",
                "Docli.Sample.Export.csproj");
            var htmlDir = Path.Combine(directory.FullName, "html");
            Directory.CreateDirectory(htmlDir);

            var result = RustDocli.GenerateDotnetProject(
                repo,
                csproj,
                htmlDir,
                null,
                TimeSpan.FromSeconds(170));
            Assert.True(
                result.ExitCode == 0,
                $"docli generate exited {result.ExitCode}. stdout: {result.Stdout} stderr: {result.Stderr}");

            var htmlPath = Path.Combine(htmlDir, "index.html");
            Assert.True(File.Exists(htmlPath));
            var html = File.ReadAllText(htmlPath);
            var model = CliModelJson.Deserialize(DocliDataInnerText(html));
            Assert.Equal("sample", model.Name);
            Assert.Contains(model.Subcommands, subcommand => subcommand.Name == "run");
            Assert.Contains(model.Options, option => option.Long == "--output");
            await Task.CompletedTask;
        }
        finally
        {
            directory.Delete(recursive: true);
        }
    }

    private static string DocliDataInnerText(string html)
    {
        const string open = "<script id=\"docli-data\" type=\"application/json\">";
        const string close = "</script>";
        var start = html.IndexOf(open, StringComparison.Ordinal);
        Assert.True(start >= 0, "index.html is missing <script id=\"docli-data\" type=\"application/json\">.");
        var jsonStart = start + open.Length;
        var end = html.IndexOf(close, jsonStart, StringComparison.Ordinal);
        Assert.True(end >= 0, "index.html docli-data script is not closed.");
        return html[jsonStart..end];
    }
}
