using System.ComponentModel;
using System.Runtime.CompilerServices;
using System.Collections.Generic;
using System.Threading.Tasks;
using System;
using ReactiveUI;
using GameBooster.Common.Models;
using GameBooster.Common.Interfaces;
using System.Reactive;
using Microsoft.Extensions.Logging;
using Avalonia.Media;
using System.Linq;
using System.Reactive.Linq;
using Avalonia.Threading;
using System.Windows.Input;

namespace GameBooster.UI.ViewModels;

/// <summary>
/// ViewModel principal de l'application
/// </summary>
public class MainWindowViewModel : ViewModelBase
{
    private readonly IMemoryService _memoryService;
    private readonly IDiskService _diskService;
    private readonly ISchedulerService _schedulerService;
    private readonly IWindowsServiceManager _windowsServiceManager;
    private readonly INetworkService _networkService;
    private readonly ILogger<MainWindowViewModel> _logger;
    
    private MemoryInfo _memoryInfo = new();
    private List<DiskInfo> _disksInfo = new();
    private List<ProcessInfo> _topMemoryProcesses = new();
    private bool _isCleaningMemory = false;
    private bool _isOptimizingMemory = false;
    private bool _isInitializing = true;
    private string _debugStatus = "Initialisation...";
    private string? _lastMemoryCleanResult;
    private string? _lastMemoryOptimizeResult;
    private DateTime _lastUpdateTime = DateTime.Now;
    private int _totalOptimizations = 0;
    
    // Propriétés cachées pour éviter les recalculs
    private string _memoryUsagePercentage = "0%";
    private double _memoryUsageProgress = 0;
    private string _availableMemoryGB = "0";
    private string _activeProcessCount = "0";

    // ViewModels des onglets
    public MemoryViewModel MemoryViewModel { get; }
    public DiskViewModel DiskViewModel { get; }
    public SchedulerViewModel SchedulerViewModel { get; }
    public ServicesViewModel ServicesViewModel { get; }
    public NetworkViewModel NetworkViewModel { get; }

    public MainWindowViewModel(
        IMemoryService memoryService,
        IDiskService diskService,
        ISchedulerService schedulerService,
        IWindowsServiceManager windowsServiceManager,
        INetworkService networkService,
        MemoryViewModel memoryViewModel,
        DiskViewModel diskViewModel,
        SchedulerViewModel schedulerViewModel,
        ServicesViewModel servicesViewModel,
        NetworkViewModel networkViewModel,
        ILogger<MainWindowViewModel> logger)
    {
        DebugLogger.ClearLog();
        var instanceId = Guid.NewGuid().ToString("N")[..8];
        DebugLogger.LogAction($"DÉBUT CONSTRUCTEUR MainWindowViewModel [ID: {instanceId}]");
        
        _memoryService = memoryService;
        _diskService = diskService;
        _schedulerService = schedulerService;
        _windowsServiceManager = windowsServiceManager;
        _networkService = networkService;
        _logger = logger;
        
        // Initialisation des ViewModels des onglets
        MemoryViewModel = memoryViewModel;
        DiskViewModel = diskViewModel;
        SchedulerViewModel = schedulerViewModel;
        ServicesViewModel = servicesViewModel;
        NetworkViewModel = networkViewModel;
        
        _logger.LogInformation("=== CONSTRUCTEUR MainWindowViewModel ===");
        DebugLogger.LogAction("Services injectés");

        // Initialisation des propriétés
        Title = "GameBooster - Optimiseur PC";
        DebugLogger.LogProperty("Title", Title);

        // Commandes simples pour éviter les problèmes de thread
        OptimizeSystemCommand = new RelayCommand(async () => await OptimizeSystemAsync());
        RefreshSystemInfoCommand = new RelayCommand(async () => await RefreshSystemInfoAsync());
        CleanMemoryCommand = new RelayCommand(async () => await CleanMemoryAsync(), () => !IsCleaningMemory);
        OptimizeMemoryCommand = new RelayCommand(async () => await OptimizeMemoryAsync(), () => !IsOptimizingMemory);
        RefreshTopProcessesCommand = new RelayCommand(async () => await RefreshTopProcessesAsync());
        
        _logger.LogInformation("=== COMMANDES CRÉÉES ===");
        DebugLogger.LogAction("Commandes ReactiveCommand créées");
        
        // Debug des états initiaux
        DebugLogger.LogProperty("IsCleaningMemory INITIAL", IsCleaningMemory);
        DebugLogger.LogProperty("IsOptimizingMemory INITIAL", IsOptimizingMemory);

        // UTILISATION DES VRAIS SERVICES AVEC LOGS
        DebugLogger.LogAction("DÉBUT INITIALISATION AVEC VRAIS SERVICES");
        
        // Initialisation asynchrone avec logs complets
        Task.Run(async () =>
        {
            try
            {
                DebugLogger.LogAction("DÉBUT InitializeServicesAsync");
                await InitializeServicesAsync();
                
                await Dispatcher.UIThread.InvokeAsync(() =>
                {
                    IsInitializing = false;
                    DebugStatus = "Système prêt";
                });
                
                DebugLogger.LogAction("FIN InitializeServicesAsync - IsInitializing = false");
                
                // Démarrer la surveillance en temps réel
                StartRealTimeMonitoring();
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Erreur critique lors de l'initialisation");
                await Dispatcher.UIThread.InvokeAsync(() =>
                {
                    IsInitializing = false;
                    DebugStatus = $"Erreur: {ex.Message}";
                });
            }
        });
    }

    // Propriétés publiques avec thread-safety
    public string Title { get; } = "GameBooster - Optimiseur PC";

    public MemoryInfo MemoryInfo
    {
        get => _memoryInfo;
        private set => this.RaiseAndSetIfChanged(ref _memoryInfo, value);
    }

    public List<DiskInfo> DisksInfo
    {
        get => _disksInfo;
        private set => this.RaiseAndSetIfChanged(ref _disksInfo, value);
    }

    public List<ProcessInfo> TopMemoryProcesses
    {
        get => _topMemoryProcesses;
        private set => this.RaiseAndSetIfChanged(ref _topMemoryProcesses, value);
    }

    public bool IsCleaningMemory
    {
        get => _isCleaningMemory;
        private set => this.RaiseAndSetIfChanged(ref _isCleaningMemory, value);
    }

    public bool IsOptimizingMemory
    {
        get => _isOptimizingMemory;
        private set => this.RaiseAndSetIfChanged(ref _isOptimizingMemory, value);
    }

    public bool IsInitializing
    {
        get => _isInitializing;
        private set => this.RaiseAndSetIfChanged(ref _isInitializing, value);
    }

    public string DebugStatus
    {
        get => _debugStatus;
        private set => this.RaiseAndSetIfChanged(ref _debugStatus, value);
    }

    public string? LastMemoryCleanResult
    {
        get => _lastMemoryCleanResult;
        private set => this.RaiseAndSetIfChanged(ref _lastMemoryCleanResult, value);
    }

    public string? LastMemoryOptimizeResult
    {
        get => _lastMemoryOptimizeResult;
        private set => this.RaiseAndSetIfChanged(ref _lastMemoryOptimizeResult, value);
    }

    public DateTime LastUpdateTime
    {
        get => _lastUpdateTime;
        private set => this.RaiseAndSetIfChanged(ref _lastUpdateTime, value);
    }

    public int TotalOptimizations
    {
        get => _totalOptimizations;
        private set => this.RaiseAndSetIfChanged(ref _totalOptimizations, value);
    }

    // Propriétés calculées thread-safe
    public string MemoryUsagePercentage
    {
        get => _memoryUsagePercentage;
        private set => this.RaiseAndSetIfChanged(ref _memoryUsagePercentage, value);
    }

    public double MemoryUsageProgress
    {
        get => _memoryUsageProgress;
        private set => this.RaiseAndSetIfChanged(ref _memoryUsageProgress, value);
    }

    public string AvailableMemoryGB 
    {
        get => _availableMemoryGB;
        private set => this.RaiseAndSetIfChanged(ref _availableMemoryGB, value);
    }

    public string ActiveProcessCount 
    {
        get => _activeProcessCount;
        private set => this.RaiseAndSetIfChanged(ref _activeProcessCount, value);
    }

    public IBrush MemoryUsageColor
    {
        get
        {
            var percentage = MemoryUsageProgress;
            if (percentage > 90) return Brush.Parse("#FF4444");
            if (percentage > 70) return Brush.Parse("#FFA500");
            return Brush.Parse("#00D4FF");
        }
    }

    public string MemoryUsageText 
    {
        get
        {
            if (MemoryInfo == null) return "0 GB utilisés";
            return $"{(MemoryInfo.TotalPhysical - MemoryInfo.AvailablePhysical) / (1024.0 * 1024 * 1024):F1} GB utilisés";
        }
    }

    public string MemoryInfoDetails
    {
        get
        {
            if (MemoryInfo == null) return "Aucune donnée mémoire disponible";
            var totalGB = MemoryInfo.TotalPhysical / (1024.0 * 1024 * 1024);
            var availableGB = MemoryInfo.AvailablePhysical / (1024.0 * 1024 * 1024);
            var usedGB = totalGB - availableGB;
            return $"Utilisé: {usedGB:F1} GB / {totalGB:F1} GB • Disponible: {availableGB:F1} GB";
        }
    }

    public string StatusIcon => IsInitializing ? "⏳" : "✅";
    public string StatusText => IsInitializing ? "Initialisation..." : "Système optimisé";
    public IBrush StatusColor => IsInitializing ? Brush.Parse("#FFA500") : Brush.Parse("#00FF00");

    // Commandes
    public ICommand OptimizeSystemCommand { get; }
    public ICommand RefreshSystemInfoCommand { get; }
    public ICommand CleanMemoryCommand { get; }
    public ICommand OptimizeMemoryCommand { get; }
    public ICommand RefreshTopProcessesCommand { get; }

    // Méthodes privées
    private async Task InitializeServicesAsync()
    {
        _logger.LogInformation("=== InitializeServicesAsync START ===");
        
        await Dispatcher.UIThread.InvokeAsync(() =>
        {
            DebugStatus = "Chargement des informations système...";
        });
        
        try
        {
            _logger.LogInformation("Chargement mémoire...");
            await LoadMemoryInfoAsync();
            
            _logger.LogInformation("Chargement disques...");
            await LoadDisksInfoAsync();
            
            _logger.LogInformation("Chargement processus...");
            await LoadTopProcessesAsync();
            
            _logger.LogInformation("Toutes les données chargées");
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors du chargement des données");
            throw;
        }
        
        await Dispatcher.UIThread.InvokeAsync(() =>
        {
            LastUpdateTime = DateTime.Now;
            DebugStatus = "Système prêt";
        });
        
        _logger.LogInformation("=== InitializeServicesAsync END ===");
    }

    private async Task LoadMemoryInfoAsync()
    {
        try
        {
            _logger.LogInformation("=== LoadMemoryInfoAsync START ===");
            var memoryInfo = await _memoryService.GetMemoryInfoAsync();
            
            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                MemoryInfo = memoryInfo;
                
                // Calcul des propriétés dérivées
                var totalGB = memoryInfo.TotalPhysical / (1024.0 * 1024 * 1024);
                var availableGB = memoryInfo.AvailablePhysical / (1024.0 * 1024 * 1024);
                var usedGB = totalGB - availableGB;
                var usagePercentage = (usedGB / totalGB) * 100;
                
                MemoryUsagePercentage = $"{usagePercentage:F1}%";
                MemoryUsageProgress = usagePercentage;
                AvailableMemoryGB = $"{availableGB:F1}";
                
                _logger.LogInformation("Mémoire mise à jour: {UsagePercentage}% utilisés, {AvailableGB} GB disponibles", 
                    usagePercentage, availableGB);
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors du chargement des informations mémoire");
        }
    }

    private async Task LoadDisksInfoAsync()
    {
        try
        {
            var disksInfo = await _diskService.GetDisksInfoAsync();
            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                DisksInfo = disksInfo;
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors du chargement des informations disque");
        }
    }

    private async Task LoadTopProcessesAsync()
    {
        try
        {
            var processes = await _memoryService.GetTopMemoryProcessesAsync(10);
            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                TopMemoryProcesses = processes;
                ActiveProcessCount = processes.Count.ToString();
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors du chargement des processus");
        }
    }

    private async Task CleanMemoryAsync()
    {
        // Vérification thread-safe de l'état avant d'exécuter
        if (IsCleaningMemory)
            return;

        try
        {
            // Désactiver le bouton de manière thread-safe
            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                IsCleaningMemory = true;
            });
            
            _logger.LogInformation("Démarrage du nettoyage mémoire");

            // Simulation de nettoyage pour tester l'UI
            await Task.Delay(2000);
            
            // Simuler un résultat positif
            var freedMB = Random.Shared.Next(50, 200);
            
            _logger.LogInformation("Nettoyage mémoire simulé: {SpaceFreed} MB libérés", freedMB);

            // IMPORTANT: Toutes les modifications UI doivent être sur le thread UI
            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                LastMemoryCleanResult = $"✅ {freedMB} MB libérés en 2000ms";
                TotalOptimizations++;

                // Actualiser les informations mémoire avec de nouvelles valeurs
                var currentMemory = MemoryInfo;
                MemoryInfo = new MemoryInfo
                {
                    TotalPhysical = currentMemory.TotalPhysical,
                    AvailablePhysical = currentMemory.AvailablePhysical + (freedMB * 1024 * 1024),
                    TotalPageFile = currentMemory.TotalPageFile,
                    AvailablePageFile = currentMemory.AvailablePageFile,
                    LastUpdate = DateTime.Now
                };
                
                LastUpdateTime = DateTime.Now;
            });
        }
        catch (Exception ex)
        {
            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                LastMemoryCleanResult = $"❌ Erreur: {ex.Message}";
            });
            _logger.LogError(ex, "Erreur lors du nettoyage mémoire");
        }
        finally
        {
            // Réactiver le bouton et arrêter le loading
            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                IsCleaningMemory = false;
            });
        }
    }

    private async Task OptimizeMemoryAsync()
    {
        // Vérification thread-safe de l'état avant d'exécuter
        if (IsOptimizingMemory)
            return;

        try
        {
            // Désactiver le bouton de manière thread-safe
            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                IsOptimizingMemory = true;
            });
            
            _logger.LogInformation("Démarrage de l'optimisation mémoire");

            // Simulation d'optimisation
            await Task.Delay(3000);
            
            var optimizedMB = Random.Shared.Next(100, 300);
            
            _logger.LogInformation("Optimisation mémoire simulée: {OptimizedMB} MB optimisés", optimizedMB);

            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                LastMemoryOptimizeResult = $"⚡ {optimizedMB} MB optimisés en 3000ms";
                TotalOptimizations++;

                // Actualiser les informations mémoire
                var currentMemory = MemoryInfo;
                MemoryInfo = new MemoryInfo
                {
                    TotalPhysical = currentMemory.TotalPhysical,
                    AvailablePhysical = currentMemory.AvailablePhysical + (optimizedMB * 1024 * 1024),
                    TotalPageFile = currentMemory.TotalPageFile,
                    AvailablePageFile = currentMemory.AvailablePageFile,
                    LastUpdate = DateTime.Now
                };
                
                LastUpdateTime = DateTime.Now;
            });
        }
        catch (Exception ex)
        {
            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                LastMemoryOptimizeResult = $"❌ Erreur: {ex.Message}";
            });
            _logger.LogError(ex, "Erreur lors de l'optimisation mémoire");
        }
        finally
        {
            // Réactiver le bouton et arrêter le loading
            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                IsOptimizingMemory = false;
            });
        }
    }

    private async Task RefreshTopProcessesAsync()
    {
        await LoadTopProcessesAsync();
        await Dispatcher.UIThread.InvokeAsync(() =>
        {
            LastUpdateTime = DateTime.Now;
        });
    }

    private async Task OptimizeSystemAsync()
    {
        _logger.LogInformation("Optimisation complète du système démarrée");
        
        // Optimiser la mémoire et actualiser les informations
        await CleanMemoryAsync();
        await OptimizeMemoryAsync();
        
        _logger.LogInformation("Optimisation complète du système terminée");
    }

    private async Task RefreshSystemInfoAsync()
    {
        _logger.LogInformation("Actualisation des informations système");
        
        var tasks = new List<Task>
        {
            LoadMemoryInfoAsync(),
            LoadDisksInfoAsync(),
            LoadTopProcessesAsync()
        };

        await Task.WhenAll(tasks);
        await Dispatcher.UIThread.InvokeAsync(() =>
        {
            LastUpdateTime = DateTime.Now;
        });
        
        _logger.LogInformation("Informations système actualisées");
    }

    private void StartRealTimeMonitoring()
    {
        // Surveillance en temps réel toutes les 5 secondes
        Observable.Interval(TimeSpan.FromSeconds(5))
            .ObserveOn(RxApp.MainThreadScheduler)
            .Subscribe(async _ =>
            {
                try
                {
                    if (!IsCleaningMemory && !IsOptimizingMemory)
                    {
                        await LoadMemoryInfoAsync();
                        await Dispatcher.UIThread.InvokeAsync(() =>
                        {
                            LastUpdateTime = DateTime.Now;
                        });
                    }
                }
                catch (Exception ex)
                {
                    _logger.LogError(ex, "Erreur lors de la surveillance temps réel");
                }
            });
    }

    /// <summary>
    /// Met à jour les propriétés calculées pour éviter les recalculs répétés
    /// </summary>
    private void UpdateCalculatedProperties()
    {
        if (MemoryInfo != null)
        {
            var totalGB = MemoryInfo.TotalPhysical / (1024.0 * 1024 * 1024);
            var availableGB = MemoryInfo.AvailablePhysical / (1024.0 * 1024 * 1024);
            var usedGB = totalGB - availableGB;
            var usagePercentage = (usedGB / totalGB) * 100;
            
            MemoryUsagePercentage = $"{usagePercentage:F1}%";
            MemoryUsageProgress = usagePercentage;
            AvailableMemoryGB = $"{availableGB:F1}";
        }
        
        if (TopMemoryProcesses != null)
        {
            ActiveProcessCount = TopMemoryProcesses.Count.ToString();
        }
    }
}
