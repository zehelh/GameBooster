using System.Collections.ObjectModel;
using System.Windows.Input;
using Avalonia.Threading;
using GameBooster.Common.Interfaces;
using GameBooster.Common.Models;
using ReactiveUI;
using Microsoft.Extensions.Logging;
using System.Reactive;
using System.Linq;

namespace GameBooster.UI.ViewModels;

/// <summary>
/// ViewModel for CPU management
/// </summary>
public class CpuViewModel : ViewModelBase
{
    private readonly ICpuService _cpuService;
    private readonly ILogger<CpuViewModel> _logger;

    // Properties
    private ObservableCollection<CpuProcessInfo> _cpuProcesses;
    private CpuInfo? _cpuInfo;
    private CpuPerformanceCounters? _performanceCounters;
    private bool _isLoading;
    private bool _isMonitoring;
    private CpuProcessInfo? _selectedProcess;
    private ProcessPriority _selectedPriority = ProcessPriority.Normal;
    private int _selectedCpuLimit = 50;
    
    // Nouvelles propriétés pour les filtres
    private bool _showMicrosoftProcesses = true;
    private string _processSearchText = "";
    private ObservableCollection<CpuProcessInfo> _filteredProcesses;

    public CpuViewModel(ICpuService cpuService, ILogger<CpuViewModel> logger)
    {
        _cpuService = cpuService;
        _logger = logger;
        
        _cpuProcesses = new ObservableCollection<CpuProcessInfo>();
        _filteredProcesses = new ObservableCollection<CpuProcessInfo>();

        // Initialize commands
        RefreshCommand = new RelayCommand(async () => await RefreshAsync());
        StartMonitoringCommand = new RelayCommand(async () => await StartMonitoringAsync());
        StopMonitoringCommand = new RelayCommand(async () => await StopMonitoringAsync());
        SetPriorityCommand = new RelayCommand(async () => await SetPriorityAsync());
        SetAffinityCommand = new RelayCommand(async () => await SetAffinityAsync());
        SetCpuLimitCommand = new RelayCommand(async () => await SetCpuLimitAsync());
        RemoveCpuLimitCommand = new RelayCommand(async () => await RemoveCpuLimitAsync());
        ApplyGamingProfileCommand = new RelayCommand(async () => await ApplyGamingProfileAsync());
        ResetOptimizationsCommand = new RelayCommand(async () => await ResetOptimizationsAsync());

        // Initial load
        _ = Task.Run(async () => await InitializeAsync());
    }

    /// <summary>
    /// CPU processes (filtered)
    /// </summary>
    public ObservableCollection<CpuProcessInfo> CpuProcesses
    {
        get => _filteredProcesses;
        set => this.RaiseAndSetIfChanged(ref _filteredProcesses, value);
    }

    /// <summary>
    /// All CPU processes (unfiltered)
    /// </summary>
    public ObservableCollection<CpuProcessInfo> AllCpuProcesses
    {
        get => _cpuProcesses;
        set => this.RaiseAndSetIfChanged(ref _cpuProcesses, value);
    }

    /// <summary>
    /// CPU information
    /// </summary>
    public CpuInfo? CpuInfo
    {
        get => _cpuInfo;
        set => this.RaiseAndSetIfChanged(ref _cpuInfo, value);
    }

    /// <summary>
    /// CPU performance counters
    /// </summary>
    public CpuPerformanceCounters? PerformanceCounters
    {
        get => _performanceCounters;
        set => this.RaiseAndSetIfChanged(ref _performanceCounters, value);
    }

    /// <summary>
    /// Loading state
    /// </summary>
    public new bool IsLoading
    {
        get => _isLoading;
        set => this.RaiseAndSetIfChanged(ref _isLoading, value);
    }

    /// <summary>
    /// Monitoring state
    /// </summary>
    public bool IsMonitoring
    {
        get => _isMonitoring;
        set => this.RaiseAndSetIfChanged(ref _isMonitoring, value);
    }

    /// <summary>
    /// Selected process
    /// </summary>
    public CpuProcessInfo? SelectedProcess
    {
        get => _selectedProcess;
        set => this.RaiseAndSetIfChanged(ref _selectedProcess, value);
    }

    /// <summary>
    /// Selected priority
    /// </summary>
    public ProcessPriority SelectedPriority
    {
        get => _selectedPriority;
        set => this.RaiseAndSetIfChanged(ref _selectedPriority, value);
    }

    /// <summary>
    /// Selected CPU limit
    /// </summary>
    public int SelectedCpuLimit
    {
        get => _selectedCpuLimit;
        set => this.RaiseAndSetIfChanged(ref _selectedCpuLimit, value);
    }

    /// <summary>
    /// Show Microsoft processes
    /// </summary>
    public bool ShowMicrosoftProcesses
    {
        get => _showMicrosoftProcesses;
        set
        {
            this.RaiseAndSetIfChanged(ref _showMicrosoftProcesses, value);
            ApplyFilters();
        }
    }

    /// <summary>
    /// Process search text
    /// </summary>
    public string ProcessSearchText
    {
        get => _processSearchText;
        set
        {
            this.RaiseAndSetIfChanged(ref _processSearchText, value);
            ApplyFilters();
        }
    }

    // Commands
    public ICommand RefreshCommand { get; }
    public ICommand StartMonitoringCommand { get; }
    public ICommand StopMonitoringCommand { get; }
    public ICommand SetPriorityCommand { get; }
    public ICommand SetAffinityCommand { get; }
    public ICommand SetCpuLimitCommand { get; }
    public ICommand RemoveCpuLimitCommand { get; }
    public ICommand ApplyGamingProfileCommand { get; }
    public ICommand ResetOptimizationsCommand { get; }

    private async Task InitializeAsync()
    {
        try
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsLoading = true);

            // Get initial CPU info
            var cpuInfo = await _cpuService.GetCpuInfoAsync();
            await Dispatcher.UIThread.InvokeAsync(() => CpuInfo = cpuInfo);

            // Get initial processes
            await RefreshAsync();

            // Get performance counters
            var counters = await _cpuService.GetCpuPerformanceCountersAsync();
            await Dispatcher.UIThread.InvokeAsync(() => PerformanceCounters = counters);

            _logger.LogInformation("CPU ViewModel initialized successfully");
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error initializing CPU ViewModel");
        }
        finally
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsLoading = false);
        }
    }

    private async Task RefreshAsync()
    {
        try
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsLoading = true);

            var processes = await _cpuService.GetCpuProcessesAsync();
            
            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                AllCpuProcesses.Clear();
                foreach (var process in processes)
                {
                    AllCpuProcesses.Add(process);
                }
                ApplyFilters();
            });

            _logger.LogInformation("CPU processes refreshed: {Count} processes", processes.Count);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error refreshing CPU processes");
        }
        finally
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsLoading = false);
        }
    }

    private void ApplyFilters()
    {
        var filtered = AllCpuProcesses.AsEnumerable();

        // Filter Microsoft processes
        if (!ShowMicrosoftProcesses)
        {
            filtered = filtered.Where(p => !IsMicrosoftProcess(p.Name));
        }

        // Filter by search text
        if (!string.IsNullOrWhiteSpace(ProcessSearchText))
        {
            filtered = filtered.Where(p => 
                p.Name.Contains(ProcessSearchText, StringComparison.OrdinalIgnoreCase) ||
                p.ProcessId.ToString().Contains(ProcessSearchText));
        }

        // Update filtered collection
        CpuProcesses.Clear();
        foreach (var process in filtered)
        {
            CpuProcesses.Add(process);
        }
    }

    private bool IsMicrosoftProcess(string processName)
    {
        var microsoftProcesses = new[]
        {
            "svchost", "lsass", "csrss", "winlogon", "services", "spoolsv", "wininit",
            "dwm", "explorer", "taskmgr", "cmd", "powershell", "conhost", "rundll32",
            "regsvr32", "msiexec", "wuauserv", "bits", "cryptsvc", "dcomlaunch",
            "iphlpsvc", "netman", "nsi", "pla", "schedule", "sppsvc", "themeservice",
            "trkwks", "upnphost", "vss", "w32time", "wsearch", "wuauserv", "wudfsvc"
        };

        return microsoftProcesses.Contains(processName.ToLower());
    }

    private async Task StartMonitoringAsync()
    {
        try
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsMonitoring = true);

            var subscription = _cpuService.MonitorCpuAsync()
                .Subscribe(async cpuInfo =>
                {
                    await Dispatcher.UIThread.InvokeAsync(() => CpuInfo = cpuInfo);
                });

            _logger.LogInformation("CPU monitoring started");
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error starting CPU monitoring");
            await Dispatcher.UIThread.InvokeAsync(() => IsMonitoring = false);
        }
    }

    private async Task StopMonitoringAsync()
    {
        try
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsMonitoring = false);
            _logger.LogInformation("CPU monitoring stopped");
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error stopping CPU monitoring");
        }
    }

    private async Task SetPriorityAsync()
    {
        if (SelectedProcess == null)
        {
            _logger.LogWarning("No process selected for priority change");
            return;
        }

        try
        {
            var success = await _cpuService.SetProcessPriorityAsync(SelectedProcess.ProcessId, SelectedPriority);
            if (success)
            {
                _logger.LogInformation("Priority set for process {ProcessId} to {Priority}", 
                    SelectedProcess.ProcessId, SelectedPriority);
                
                // Refresh the process list to show updated priority
                await RefreshAsync();
            }
            else
            {
                _logger.LogError("Failed to set priority for process {ProcessId}", SelectedProcess.ProcessId);
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error setting priority for process {ProcessId}", SelectedProcess.ProcessId);
        }
    }

    private async Task SetAffinityAsync()
    {
        if (SelectedProcess == null)
        {
            _logger.LogWarning("No process selected for affinity change");
            return;
        }

        try
        {
            // For now, set affinity to all cores
            var coresCount = await _cpuService.GetCpuCoresCountAsync();
            var allCoresMask = (1UL << coresCount) - 1;
            
            var success = await _cpuService.SetProcessAffinityAsync(SelectedProcess.ProcessId, allCoresMask);
            if (success)
            {
                _logger.LogInformation("Affinity set for process {ProcessId} to all cores", SelectedProcess.ProcessId);
                
                // Refresh the process list to show updated affinity
                await RefreshAsync();
            }
            else
            {
                _logger.LogError("Failed to set affinity for process {ProcessId}", SelectedProcess.ProcessId);
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error setting affinity for process {ProcessId}", SelectedProcess.ProcessId);
        }
    }

    private async Task SetCpuLimitAsync()
    {
        if (SelectedProcess == null)
        {
            _logger.LogWarning("No process selected for CPU limit");
            return;
        }

        try
        {
            var success = await _cpuService.SetProcessCpuLimitAsync(SelectedProcess.ProcessId, SelectedCpuLimit);
            if (success)
            {
                _logger.LogInformation("CPU limit set for process {ProcessId} to {Limit}%", 
                    SelectedProcess.ProcessId, SelectedCpuLimit);
                
                // Refresh the process list to show updated limit
                await RefreshAsync();
            }
            else
            {
                _logger.LogError("Failed to set CPU limit for process {ProcessId}", SelectedProcess.ProcessId);
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error setting CPU limit for process {ProcessId}", SelectedProcess.ProcessId);
        }
    }

    private async Task RemoveCpuLimitAsync()
    {
        if (SelectedProcess == null)
        {
            _logger.LogWarning("No process selected for CPU limit removal");
            return;
        }

        try
        {
            var success = await _cpuService.RemoveProcessCpuLimitAsync(SelectedProcess.ProcessId);
            if (success)
            {
                _logger.LogInformation("CPU limit removed for process {ProcessId}", SelectedProcess.ProcessId);
                
                // Refresh the process list to show updated limit
                await RefreshAsync();
            }
            else
            {
                _logger.LogError("Failed to remove CPU limit for process {ProcessId}", SelectedProcess.ProcessId);
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error removing CPU limit for process {ProcessId}", SelectedProcess.ProcessId);
        }
    }

    private async Task ApplyGamingProfileAsync()
    {
        try
        {
            // For now, apply to a common game process name
            var gameProcessName = "game"; // This would be configurable
            var success = await _cpuService.ApplyGamingProfileAsync(gameProcessName);
            
            if (success)
            {
                _logger.LogInformation("Gaming profile applied successfully");
                
                // Refresh the process list to show updated priorities
                await RefreshAsync();
            }
            else
            {
                _logger.LogWarning("Gaming profile application failed or no game process found");
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error applying gaming profile");
        }
    }

    private async Task ResetOptimizationsAsync()
    {
        try
        {
            var success = await _cpuService.ResetAllOptimizationsAsync();
            if (success)
            {
                _logger.LogInformation("All CPU optimizations reset successfully");
                
                // Refresh the process list to show updated priorities
                await RefreshAsync();
            }
            else
            {
                _logger.LogError("Failed to reset CPU optimizations");
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error resetting CPU optimizations");
        }
    }
}
