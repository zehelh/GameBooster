using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using System.Windows.Input;
using ReactiveUI;
using GameBooster.Common.Interfaces;
using GameBooster.Common.Models;
using Microsoft.Extensions.Logging;
using Avalonia.Threading;
using System.Reactive;
using System.ComponentModel;
using System.Windows;

namespace GameBooster.UI.ViewModels;

/// <summary>
/// ViewModel for memory management features with aggressive cleaning options
/// </summary>
public class MemoryViewModel : ViewModelBase
{
    private readonly IMemoryService _memoryService;
    private readonly ILogger<MemoryViewModel> _logger;
    
    private MemoryInfo _memoryInfo = new();
    private bool _isCleaning;
    private CleanupResult? _lastCleanupResult;
    private List<ProcessInfo> _topProcesses = new();
    private bool _isMonitoring = true;
    
    // Options de nettoyage avancées
    private bool _includeGarbageCollection = true;
    private bool _includeSystemCache = true;
    private bool _includeWorkingSet = true;
    private bool _includeStandbyMemory = true;
    private bool _includePagedPool = false;
    private bool _includeNonPagedPool = false;

    public MemoryInfo MemoryInfo
    {
        get => _memoryInfo;
        set => this.RaiseAndSetIfChanged(ref _memoryInfo, value);
    }

    public bool IsCleaning
    {
        get => _isCleaning;
        set => this.RaiseAndSetIfChanged(ref _isCleaning, value);
    }

    public CleanupResult? LastCleanupResult
    {
        get => _lastCleanupResult;
        set => this.RaiseAndSetIfChanged(ref _lastCleanupResult, value);
    }

    public List<ProcessInfo> TopProcesses
    {
        get => _topProcesses;
        set => this.RaiseAndSetIfChanged(ref _topProcesses, value);
    }

    public bool IsMonitoring
    {
        get => _isMonitoring;
        set => this.RaiseAndSetIfChanged(ref _isMonitoring, value);
    }

    // Options de nettoyage avancées
    public bool IncludeGarbageCollection
    {
        get => _includeGarbageCollection;
        set => this.RaiseAndSetIfChanged(ref _includeGarbageCollection, value);
    }

    public bool IncludeSystemCache
    {
        get => _includeSystemCache;
        set => this.RaiseAndSetIfChanged(ref _includeSystemCache, value);
    }

    public bool IncludeWorkingSet
    {
        get => _includeWorkingSet;
        set => this.RaiseAndSetIfChanged(ref _includeWorkingSet, value);
    }

    public bool IncludeStandbyMemory
    {
        get => _includeStandbyMemory;
        set => this.RaiseAndSetIfChanged(ref _includeStandbyMemory, value);
    }

    public bool IncludePagedPool
    {
        get => _includePagedPool;
        set => this.RaiseAndSetIfChanged(ref _includePagedPool, value);
    }

    public bool IncludeNonPagedPool
    {
        get => _includeNonPagedPool;
        set => this.RaiseAndSetIfChanged(ref _includeNonPagedPool, value);
    }

    // Commandes simples sans observables complexes
    public ICommand OptimizeMemoryCommand { get; }
    public ICommand RefreshCommand { get; }
    public ICommand ToggleMonitoringCommand { get; }

    public MemoryViewModel(IMemoryService memoryService, ILogger<MemoryViewModel> logger)
    {
        _memoryService = memoryService;
        _logger = logger;

        // Commandes simples pour éviter les problèmes de thread
        OptimizeMemoryCommand = new RelayCommand(async () => await OptimizeMemoryAsync(), () => !IsCleaning);
        RefreshCommand = new RelayCommand(async () => await RefreshMemoryInfoAsync());
        ToggleMonitoringCommand = new RelayCommand(() => ToggleMonitoring());

        // Start monitoring
        StartMonitoring();
    }

    private async Task OptimizeMemoryAsync()
    {
        // Vérification thread-safe de l'état
        if (IsCleaning) return;

        try
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsCleaning = true);
            _logger.LogInformation("Starting aggressive memory optimization");

            // Créer les options de nettoyage basées sur les sélections de l'utilisateur
            var options = new MemoryCleanupOptions
            {
                IncludeGarbageCollection = IncludeGarbageCollection,
                IncludeSystemCache = IncludeSystemCache,
                IncludeWorkingSet = IncludeWorkingSet,
                IncludeStandbyMemory = IncludeStandbyMemory,
                IncludePagedPool = IncludePagedPool,
                IncludeNonPagedPool = IncludeNonPagedPool
            };

            var result = await _memoryService.CleanMemoryAsync(options);
            
            // Mise à jour sur le thread UI
            await Dispatcher.UIThread.InvokeAsync(() => LastCleanupResult = result);

            // Also load top processes
            await LoadTopProcessesAsync();
            
            _logger.LogInformation("Aggressive memory optimization completed. Freed: {0} MB", result.SpaceFreedMB);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error during aggressive memory optimization");
        }
        finally
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsCleaning = false);
        }
    }

    private async Task RefreshMemoryInfoAsync()
    {
        try
        {
            var memoryInfo = await _memoryService.GetMemoryInfoAsync();
            await Dispatcher.UIThread.InvokeAsync(() => MemoryInfo = memoryInfo);
            
            await LoadTopProcessesAsync();
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error refreshing memory information");
        }
    }

    private async Task LoadTopProcessesAsync()
    {
        try
        {
            var processes = await _memoryService.GetTopMemoryProcessesAsync(10);
            await Dispatcher.UIThread.InvokeAsync(() => TopProcesses = processes);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error loading top memory processes");
            await Dispatcher.UIThread.InvokeAsync(() => TopProcesses = new List<ProcessInfo>());
        }
    }

    private void ToggleMonitoring()
    {
        IsMonitoring = !IsMonitoring;
        if (IsMonitoring)
        {
            StartMonitoring();
        }
    }

    private void StartMonitoring()
    {
        if (!IsMonitoring) return;

        // Start real-time monitoring
        Task.Run(async () =>
        {
            while (IsMonitoring)
            {
                try
                {
                    await RefreshMemoryInfoAsync();
                    await Task.Delay(2000); // Update every 2 seconds
                }
                catch (Exception ex)
                {
                    _logger.LogError(ex, "Error in memory monitoring loop");
                    await Task.Delay(5000); // Wait longer on error
                }
            }
        });
    }
}

// Classe helper pour les commandes simples
public class RelayCommand : ICommand
{
    private readonly Action _execute;
    private readonly Func<bool>? _canExecute;

    public RelayCommand(Action execute, Func<bool>? canExecute = null)
    {
        _execute = execute ?? throw new ArgumentNullException(nameof(execute));
        _canExecute = canExecute;
    }

#pragma warning disable CS0067
    public event EventHandler? CanExecuteChanged;
#pragma warning restore CS0067

    public bool CanExecute(object? parameter) => _canExecute?.Invoke() ?? true;

    public void Execute(object? parameter) => _execute();
}
