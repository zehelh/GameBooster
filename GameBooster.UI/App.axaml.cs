using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;
using GameBooster.UI.ViewModels;
using GameBooster.UI.Views;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using GameBooster.Core.Services;
using GameBooster.Common.Interfaces;

namespace GameBooster.UI;

public partial class App : Application
{
    private IHost? _host;

    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
    }

    public override void OnFrameworkInitializationCompleted()
    {
        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            // Configuration de l'injection de dépendances
            _host = Host.CreateDefaultBuilder()
                .ConfigureServices(services =>
                {
                    // Services de base
                    services.AddLogging(builder =>
                    {
                        builder.AddConsole();
                        builder.AddDebug();
                    });

                    // Services métier
                    services.AddSingleton<IMemoryService, MemoryService>();
                    services.AddSingleton<IDiskService, DiskService>();
                    services.AddSingleton<ISchedulerService, SchedulerService>();

                    // ViewModels des onglets
                    services.AddTransient<MemoryViewModel>();
                    services.AddTransient<DiskViewModel>();
                    services.AddTransient<SchedulerViewModel>();

                    // ViewModel principal avec injection des ViewModels des onglets
                    services.AddTransient<MainWindowViewModel>(provider =>
                    {
                        var memoryService = provider.GetRequiredService<IMemoryService>();
                        var diskService = provider.GetRequiredService<IDiskService>();
                        var schedulerService = provider.GetRequiredService<ISchedulerService>();
                        var memoryViewModel = provider.GetRequiredService<MemoryViewModel>();
                        var diskViewModel = provider.GetRequiredService<DiskViewModel>();
                        var schedulerViewModel = provider.GetRequiredService<SchedulerViewModel>();
                        var logger = provider.GetRequiredService<ILogger<MainWindowViewModel>>();

                        return new MainWindowViewModel(
                            memoryService,
                            diskService,
                            schedulerService,
                            memoryViewModel,
                            diskViewModel,
                            schedulerViewModel,
                            logger);
                    });

                    // Views - Configuration explicite pour forcer l'injection du ViewModel
                    services.AddTransient<MainWindow>(provider =>
                    {
                        var viewModel = provider.GetRequiredService<MainWindowViewModel>();
                        return new MainWindow(viewModel);
                    });
                })
                .Build();

            // Démarrage de l'host
            _host.Start();
            DebugLogger.LogAction("Host DI démarré");

            // Création de la fenêtre principale avec DI (qui injecte automatiquement le ViewModel)
            DebugLogger.LogAction("Récupération MainWindow depuis DI avec injection automatique");
            var mainWindow = _host.Services.GetRequiredService<MainWindow>();
            DebugLogger.LogAction("MainWindow récupéré avec DataContext injecté");
            
            DebugLogger.LogProperty("mainWindow.DataContext != null", mainWindow.DataContext != null);
            
            desktop.MainWindow = mainWindow;
            DebugLogger.LogAction("MainWindow assignée à Desktop.MainWindow");
        }

        base.OnFrameworkInitializationCompleted();
    }

    public void OnShutdown()
    {
        if (_host != null)
        {
            _host.StopAsync().GetAwaiter().GetResult();
            _host.Dispose();
        }
    }
}
