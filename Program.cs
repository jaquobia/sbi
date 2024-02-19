// See https://aka.ms/new-console-template for more information
using Spectre.Console;

public static class Program
{

    static Table table = new Table().Centered();

	public static void start(LiveDisplayContext ctx) {
		table.AddColumn("Foo");
		table.AddColumn("Bar");
		ctx.Refresh();
		var run = true;
		while (run) {
			ctx.Refresh();
			var key_info = AnsiConsole.Console.Input.ReadKey(true);
			if (key_info is ConsoleKeyInfo key && key.Key == ConsoleKey.Escape) {
				run = false;
			} else {
				table.AddRow("A");
			}
		}
	}

    public static void Main(string[] args)
    {
		// Animate
		AnsiConsole.Live(table)
			.AutoClear(true)
			.Overflow(VerticalOverflow.Ellipsis)
			.Cropping(VerticalOverflowCropping.Top)
			.Start(ctx => { start(ctx); });
	}
}
