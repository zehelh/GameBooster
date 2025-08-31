using System.Collections.ObjectModel;
using System.Windows.Input;
using Avalonia.Threading;
using GameBooster.Common.Interfaces;
using GameBooster.Common.Models;
using GameBooster.UI.Logging;
using ReactiveUI;

namespace GameBooster.UI.ViewModels;

/// <summary>
/// ViewModel for Windows Services management
/// </summary>
public class ServicesViewModel : ViewModelBase
{
    private readonly IWindowsServiceManager _windowsServiceManager;

    // Properties
    private DefenderStatus? _defenderStatus;
    private ObservableCollection<SystemService> _services;
    private ObservableCollection<StartupProgram> _startupPrograms;
    private bool _isLoading;
    private bool _isElevated;
    private SystemService? _selectedService;
    private StartupProgram? _selectedStartupProgram;

    public ServicesViewModel(IWindowsServiceManager windowsServiceManager)
    {
        _windowsServiceManager = windowsServiceManager;
        
        _services = new ObservableCollection<SystemService>();
        _startupPrograms = new ObservableCollection<StartupProgram>();

        // Initialize commands
        RefreshCommand = new RelayCommand(async () => await RefreshAsync());
        ToggleDefenderCommand = new RelayCommand(async () => await ToggleDefenderAsync());
        StartServiceCommand = new RelayCommand(async () => await StartServiceAsync());
        StopServiceCommand = new RelayCommand(async () => await StopServiceAsync());
        EnableServiceCommand = new RelayCommand(async () => await EnableServiceAsync());
        DisableServiceCommand = new RelayCommand(async () => await DisableServiceAsync());
        ToggleStartupProgramCommand = new RelayCommand(async () => await ToggleStartupProgramAsync());

        // Initial load
        _ = Task.Run(async () => await InitializeAsync());
    }

    /// <summary>
    /// Windows Defender status
    /// </summary>
    public DefenderStatus? DefenderStatus
    {
        get => _defenderStatus;
        set => this.RaiseAndSetIfChanged(ref _defenderStatus, value);
    }

    /// <summary>
    /// List of Windows services
    /// </summary>
    public ObservableCollection<SystemService> Services
    {
        get => _services;
        set => this.RaiseAndSetIfChanged(ref _services, value);
    }

    /// <summary>
    /// List of startup programs
    /// </summary>
    public ObservableCollection<StartupProgram> StartupPrograms
    {
        get => _startupPrograms;
        set => this.RaiseAndSetIfChanged(ref _startupPrograms, value);
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
    /// Whether the application has elevated privileges
    /// </summary>
    public bool IsElevated
    {
        get => _isElevated;
        set => this.RaiseAndSetIfChanged(ref _isElevated, value);
    }

    /// <summary>
    /// Whether the application does NOT have elevated privileges
    /// </summary>
    public bool IsNotElevated => !IsElevated;

    /// <summary>
    /// Selected service for operations
    /// </summary>
    public SystemService? SelectedService
    {
        get => _selectedService;
        set => this.RaiseAndSetIfChanged(ref _selectedService, value);
    }

    /// <summary>
    /// Selected startup program for operations
    /// </summary>
    public StartupProgram? SelectedStartupProgram
    {
        get => _selectedStartupProgram;
        set => this.RaiseAndSetIfChanged(ref _selectedStartupProgram, value);
    }

    // Commands
    public ICommand RefreshCommand { get; }
    public ICommand ToggleDefenderCommand { get; }
    public ICommand StartServiceCommand { get; }
    public ICommand StopServiceCommand { get; }
    public ICommand EnableServiceCommand { get; }
    public ICommand DisableServiceCommand { get; }
    public ICommand ToggleStartupProgramCommand { get; }

    /// <summary>
    /// Initialize the ViewModel
    /// </summary>
    private async Task InitializeAsync()
    {
        try
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsLoading = true);

            // Check elevation status
            IsElevated = await _windowsServiceManager.IsElevatedAsync();

            // Load initial data
            await RefreshAsync();
        }
        catch (Exception)
        {
            // _logger.LogError($"Error initializing ServicesViewModel: {ex.Message}");
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
            var defenderTask = _windowsServiceManager.GetDefenderStatusAsync();
            var servicesTask = _windowsServiceManager.GetImportantServicesAsync();
            var startupTask = _windowsServiceManager.GetStartupProgramsAsync();

            await Task.WhenAll(defenderTask, servicesTask, startupTask);

            // Update UI
            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                DefenderStatus = defenderTask.Result;
                
                Services.Clear();
                foreach (var service in servicesTask.Result)
                {
                    Services.Add(service);
                }

                StartupPrograms.Clear();
                foreach (var program in startupTask.Result)
                {
                    StartupPrograms.Add(program);
                }
            });

            // _logger.LogInfo("Services data refreshed successfully");
        }
        catch (Exception)
        {
            // _logger.LogError($"Error refreshing services data: {ex.Message}");
        }
        finally
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsLoading = false);
        }
    }

    /// <summary>
    /// Toggle Windows Defender
    /// </summary>
    private async Task ToggleDefenderAsync()
    {
        try
        {
            if (DefenderStatus == null) return;

            var success = false;
            if (DefenderStatus.IsProtected)
            {
                success = await _windowsServiceManager.DisableDefenderSafelyAsync();
                if (success)
                {
                    // _logger.LogInfo("Windows Defender disabled successfully");
                }
            }
            else
            {
                success = await _windowsServiceManager.EnableDefenderAsync();
                if (success)
                {
                    // _logger.LogInfo("Windows Defender enabled successfully");
                }
            }

            if (success)
            {
                // Refresh defender status
                DefenderStatus = await _windowsServiceManager.GetDefenderStatusAsync();
            }
        }
        catch (Exception)
        {
            // _logger.LogError($"Error toggling Windows Defender: {ex.Message}");
        }
    }

    /// <summary>
    /// Start a Windows service
    /// </summary>
    private async Task StartServiceAsync()
    {
        if (SelectedService == null) return;

        try
        {
            var success = await _windowsServiceManager.StartServiceAsync(SelectedService.Name);
            if (success)
            {
                // _logger.LogInfo($"Service {SelectedService.Name} started successfully");
                await RefreshAsync(); // Refresh to update status
            }
        }
        catch (Exception)
        {
            // _logger.LogError($"Error starting service {SelectedService.Name}: {ex.Message}");
        }
    }

    /// <summary>
    /// Stop a Windows service
    /// </summary>
    private async Task StopServiceAsync()
    {
        if (SelectedService == null) return;

        try
        {
            var success = await _windowsServiceManager.StopServiceAsync(SelectedService.Name);
            if (success)
            {
                // _logger.LogInfo($"Service {SelectedService.Name} stopped successfully");
                await RefreshAsync(); // Refresh to update status
            }
        }
        catch (Exception)
        {
            // _logger.LogError($"Error stopping service {SelectedService.Name}: {ex.Message}");
        }
    }

    /// <summary>
    /// Enable a Windows service
    /// </summary>
    private async Task EnableServiceAsync()
    {
        if (SelectedService == null) return;

        try
        {
            var success = await _windowsServiceManager.EnableServiceAsync(SelectedService.Name);
            if (success)
            {
                // _logger.LogInfo($"Service {SelectedService.Name} enabled successfully");
                await RefreshAsync(); // Refresh to update status
            }
        }
        catch (Exception)
        {
            // _logger.LogError($"Error enabling service {SelectedService.Name}: {ex.Message}");
        }
    }

    /// <summary>
    /// Disable a Windows service
    /// </summary>
    private async Task DisableServiceAsync()
    {
        if (SelectedService == null) return;

        try
        {
            var success = await _windowsServiceManager.DisableServiceAsync(SelectedService.Name);
            if (success)
            {
                // _logger.LogInfo($"Service {SelectedService.Name} disabled successfully");
                await RefreshAsync(); // Refresh to update status
            }
        }
        catch (Exception)
        {
            // _logger.LogError($"Error disabling service {SelectedService.Name}: {ex.Message}");
        }
    }

    /// <summary>
    /// Toggle startup program status
    /// </summary>
    private async Task ToggleStartupProgramAsync()
    {
        if (SelectedStartupProgram == null) return;

        try
        {
            var newStatus = !SelectedStartupProgram.IsEnabled;
            var success = await _windowsServiceManager.SetStartupProgramStatusAsync(SelectedStartupProgram.Name, newStatus);
            if (success)
            {
                SelectedStartupProgram.IsEnabled = newStatus;
                // _logger.LogInfo($"Startup program {SelectedStartupProgram.Name} {(newStatus ? "enabled" : "disabled")} successfully");
            }
        }
        catch (Exception)
        {
            // _logger.LogError($"Error toggling startup program {SelectedStartupProgram.Name}: {ex.Message}");
        }
    }
}
