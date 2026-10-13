namespace Docli.Tests;

public static class RepoRoot
{
    public static string Find()
    {
        var start = new DirectoryInfo(AppContext.BaseDirectory);
        for (var dir = start; dir is not null; dir = dir.Parent)
        {
            if (File.Exists(Path.Combine(dir.FullName, "Cargo.toml")))
            {
                return dir.FullName;
            }
        }

        throw new DirectoryNotFoundException(
            $"No directory from '{start.FullName}' upward contains Cargo.toml.");
    }
}
