using System.Collections.ObjectModel;
using System.Windows.Input;
using Avalonia.Threading;
using GameBooster.Common.Interfaces;
using GameBooster.Common.Models;
using GameBooster.UI.Logging;
using ReactiveUI;

namespace GameBooster.UI.ViewModels;

/// <summary>
/// ViewModel for Network management
/// </summary>
public class NetworkViewModel : ViewModelBase
{
    private readonly INetworkService _networkService;

    // Properties
    private ObservableCollection<NetworkProcessInfo> _networkProcesses;
    private ObservableCollection<QosPolicyInfo> _qosPolicies;
    private NetworkStats? _networkStats;
    private bool _isLoading;
    private bool _isMonitoring;
    private int _selectedSpeedLimit = 1000; // Default 1 Mbps
    private NetworkProcessInfo? _selectedProcess;
    private QosPolicyInfo? _selectedPolicy;

    public NetworkViewModel(INetworkService networkService)
    {
        _networkService = networkService;
        
        _networkProcesses = new ObservableCollection<NetworkProcessInfo>();
        _qosPolicies = new ObservableCollection<QosPolicyInfo>();

        // Initialize commands
        RefreshCommand = new RelayCommand(async () => await RefreshAsync());
        StartMonitoringCommand = new RelayCommand(async () => await StartMonitoringAsync());
        StopMonitoringCommand = new RelayCommand(async () => await StopMonitoringAsync());
        SetSpeedLimitCommand = new RelayCommand(async () => await SetSpeedLimitAsync());
        RemoveSpeedLimitCommand = new RelayCommand(async () => await RemoveSpeedLimitAsync());
        ClearAllLimitsCommand = new RelayCommand(async () => await ClearAllLimitsAsync());
        RemoveQosPolicyCommand = new RelayCommand(async () => await RemoveQosPolicyAsync());

        // Initial load
        _ = Task.Run(async () => await InitializeAsync());
    }

    /// <summary>
    /// Network processes
    /// </summary>
    public ObservableCollection<NetworkProcessInfo> NetworkProcesses
    {
        get => _networkProcesses;
        set => this.RaiseAndSetIfChanged(ref _networkProcesses, value);
    }

    /// <summary>
    /// QoS policies
    /// </summary>
    public ObservableCollection<QosPolicyInfo> QosPolicies
    {
        get => _qosPolicies;
        set => this.RaiseAndSetIfChanged(ref _qosPolicies, value);
    }

    /// <summary>
    /// Network statistics
    /// </summary>
    public NetworkStats? NetworkStats
    {
        get => _networkStats;
        set => this.RaiseAndSetIfChanged(ref _networkStats, value);
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
    /// Selected speed limit in Kbps
    /// </summary>
    public int SelectedSpeedLimit
    {
        get => _selectedSpeedLimit;
        set => this.RaiseAndSetIfChanged(ref _selectedSpeedLimit, value);
    }

    /// <summary>
    /// Selected process for speed limit operations
    /// </summary>
    public NetworkProcessInfo? SelectedProcess
    {
        get => _selectedProcess;
        set => this.RaiseAndSetIfChanged(ref _selectedProcess, value);
    }

    /// <summary>
    /// Selected policy for removal
    /// </summary>
    public QosPolicyInfo? SelectedPolicy
    {
        get => _selectedPolicy;
        set => this.RaiseAndSetIfChanged(ref _selectedPolicy, value);
    }

    // Commands
    public ICommand RefreshCommand { get; }
    public ICommand StartMonitoringCommand { get; }
    public ICommand StopMonitoringCommand { get; }
    public ICommand SetSpeedLimitCommand { get; }
    public ICommand RemoveSpeedLimitCommand { get; }
    public ICommand ClearAllLimitsCommand { get; }
    public ICommand RemoveQosPolicyCommand { get; }

    /// <summary>
    /// Initialize the ViewModel
    /// </summary>
    private async Task InitializeAsync()
    {
        try
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsLoading = true);

            // Load initial data
            await RefreshAsync();
        }
        catch (Exception)
        {
            // _logger.LogError($"Error initializing NetworkViewModel: {ex.Message}");
        }
        finally
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsLoading = false);
        }
    }

    /// <summary>
    /// Refresh all data
    /// </summary>
    private async Task RefreshAsync()
    {
        try
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsLoading = true);

            // Load data in parallel
            var processesTask = _networkService.GetNetworkProcessesAsync();
            var policiesTask = _networkService.GetQosPoliciesAsync();
            var statsTask = _networkService.GetNetworkStatsAsync();

            await Task.WhenAll(processesTask, policiesTask, statsTask);

            // Update UI
            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                NetworkProcesses.Clear();
                foreach (var process in processesTask.Result)
                {
                    NetworkProcesses.Add(process);
                }

                QosPolicies.Clear();
                foreach (var policy in policiesTask.Result)
                {
                    QosPolicies.Add(policy);
                }

                NetworkStats = statsTask.Result;
            });

            // _logger.LogInfo("Network data refreshed successfully");
        }
        catch (Exception)
        {
            // _logger.LogError($"Error refreshing network data: {ex.Message}");
        }
        finally
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsLoading = false);
        }
    }

    /// <summary>
    /// Start network monitoring
    /// </summary>
    private async Task StartMonitoringAsync()
    {
        try
        {
            await _networkService.StartNetworkMonitoringAsync();
            IsMonitoring = true;
            // _logger.LogInfo("Network monitoring started");
        }
        catch (Exception)
        {
            // _logger.LogError($"Error starting network monitoring: {ex.Message}");
        }
    }

    /// <summary>
    /// Stop network monitoring
    /// </summary>
    private async Task StopMonitoringAsync()
    {
        try
        {
            await _networkService.StopNetworkMonitoringAsync();
            IsMonitoring = false;
            // _logger.LogInfo("Network monitoring stopped");
        }
        catch (Exception)
        {
            // _logger.LogError($"Error stopping network monitoring: {ex.Message}");
        }
    }

    /// <summary>
    /// Set speed limit for a process
    /// </summary>
    private async Task SetSpeedLimitAsync()
    {
        if (SelectedProcess == null) return;

        try
        {
            var success = await _networkService.SetProcessSpeedLimitAsync(SelectedProcess.ProcessId, SelectedSpeedLimit);
            if (success)
            {
                // _logger.LogInfo($"Speed limit set for {SelectedProcess.Name} (PID: {SelectedProcess.ProcessId}) to {SelectedSpeedLimit} Kbps");
                await RefreshAsync(); // Refresh to update status
            }
        }
        catch (Exception)
        {
            // _logger.LogError($"Error setting speed limit for {SelectedProcess.Name}: {ex.Message}");
        }
    }

    /// <summary>
    /// Remove speed limit for a process
    /// </summary>
    private async Task RemoveSpeedLimitAsync()
    {
        if (SelectedProcess == null) return;

        try
        {
            var success = await _networkService.RemoveProcessLimitAsync(SelectedProcess.ProcessId);
            if (success)
            {
                // _logger.LogInfo($"Speed limit removed for {SelectedProcess.Name} (PID: {SelectedProcess.ProcessId})");
                await RefreshAsync(); // Refresh to update status
            }
        }
        catch (Exception)
        {
            // _logger.LogError($"Error removing speed limit for {SelectedProcess.Name}: {ex.Message}");
        }
    }

    /// <summary>
    /// Clear all network limits
    /// </summary>
    private async Task ClearAllLimitsAsync()
    {
        try
        {
            var success = await _networkService.ClearAllLimitsAsync();
            if (success)
            {
                // _logger.LogInfo("All network limits cleared successfully");
                await RefreshAsync(); // Refresh to update status
            }
        }
        catch (Exception)
        {
            // _logger.LogError($"Error clearing all network limits: {ex.Message}");
        }
    }

    /// <summary>
    /// Remove QoS policy
    /// </summary>
    private async Task RemoveQosPolicyAsync()
    {
        if (SelectedPolicy == null) return;

        try
        {
            var success = await _networkService.RemoveQosPolicyAsync(SelectedPolicy.Name);
            if (success)
            {
                // _logger.LogInfo($"QoS policy {SelectedPolicy.Name} removed successfully");
                await RefreshAsync(); // Refresh to update status
            }
        }
        catch (Exception)
        {
            // _logger.LogError($"Error removing QoS policy {SelectedPolicy.Name}: {ex.Message}");
        }
    }
}
