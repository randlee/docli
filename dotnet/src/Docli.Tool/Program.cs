using System.CommandLine;

namespace Docli.Tool;

public static class Program
{
    public static int Main(string[] args)
    {
        try
        {
            // System.CommandLine otherwise catches action exceptions, prints
            // "Unhandled exception", and returns 1. The sprint requires this
            // catch to write the model-failed line and return 2.
            return ToolCommands.Create().Parse(args).Invoke(new InvocationConfiguration
            {
                EnableDefaultExceptionHandler = false,
            });
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine(ModelCommand.StderrPrefix + "model failed: " + ex.Message);
            return 2;
        }
    }
}
