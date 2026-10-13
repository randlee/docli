using Docli;
using Docli.Sample;
using Xunit;

namespace Docli.Tests;

public sealed class HtmlProofTests
{
    [Fact(Timeout = 180_000)]
    public async Task Sample_json_generates_html_and_markdown()
    {
        var directory = Directory.CreateTempSubdirectory("docli-d5-proof-");
        try
        {
            var jsonPath = Path.Combine(directory.FullName, "sample.json");
            var htmlDir = Path.Combine(directory.FullName, "html");
            var markdownPath = Path.Combine(directory.FullName, "manual.md");
            Directory.CreateDirectory(htmlDir);

            File.WriteAllText(
                jsonPath,
                CliModelJson.Serialize(CommandLineAdapter.FromCommand(SampleCommands.Create())));

            var result = RustDocli.Generate(
                RepoRoot.Find(),
                jsonPath,
                htmlDir,
                markdownPath,
                TimeSpan.FromSeconds(180));
            Assert.True(
                result.ExitCode == 0,
                $"docli generate exited {result.ExitCode}. stdout: {result.Stdout} stderr: {result.Stderr}");

            var htmlPath = Path.Combine(htmlDir, "index.html");
            var html = File.ReadAllText(htmlPath);
            Assert.Contains("<title>sample CLI Reference</title>", html, StringComparison.Ordinal);

            var model = CliModelJson.Deserialize(DocliDataInnerText(html));
            Assert.Equal("sample", model.Name);
            Assert.Contains(model.Options, option => option.Long == "--output");
            Assert.Contains(model.Subcommands, subcommand => subcommand.Name == "run");

            Assert.True(File.Exists(markdownPath));
            var markdown = File.ReadAllText(markdownPath);
            Assert.Contains("# sample CLI Reference", markdown, StringComparison.Ordinal);
            Assert.Contains("`sample run`", markdown, StringComparison.Ordinal);
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
