using Avalonia;
using System;
using System.Runtime.Versioning;

namespace GameBooster.UI;

[SupportedOSPlatform("windows")]
class Program
{
    // Initialization code. Don't use any Avalonia, third-party APIs or any
    // SynchronizationContext-reliant code before AppMain is called: things aren't initialized
    // yet and stuff might break.
    [STAThread]
    public static void Main(string[] args)
    {
        try
        {
            DebugLogger.Log("=== DÉMARRAGE APPLICATION ===");
            DebugLogger.Log("Arguments: " + string.Join(", ", args));
            
            BuildAvaloniaApp().StartWithClassicDesktopLifetime(args);
            
            DebugLogger.Log("=== ARRÊT APPLICATION ===");
        }
        catch (Exception ex)
        {
            DebugLogger.LogError("ERREUR CRITIQUE au démarrage", ex);
            throw;
        }
    }

    // Avalonia configuration, don't remove; also used by visual designer.
    public static AppBuilder BuildAvaloniaApp()
        => AppBuilder.Configure<App>()
            .UsePlatformDetect()
            .WithInterFont()
            .LogToTrace();
}
