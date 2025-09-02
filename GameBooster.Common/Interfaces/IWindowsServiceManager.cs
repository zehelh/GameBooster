using GameBooster.Common.Models;

namespace GameBooster.Common.Interfaces;

/// <summary>
/// Interface for managing Windows services and Windows Defender
/// </summary>
public interface IWindowsServiceManager
{
    /// <summary>
    /// Get Windows Defender status
    /// </summary>
    Task<DefenderStatus> GetDefenderStatusAsync();

    /// <summary>
    /// Disable Windows Defender temporarily
    /// </summary>
    Task<bool> DisableDefenderAsync();

    /// <summary>
    /// Enable Windows Defender
    /// </summary>
    Task<bool> EnableDefenderAsync();

    /// <summary>
    /// Safely disable Windows Defender with warnings
    /// </summary>
    Task<bool> DisableDefenderSafelyAsync();

    /// <summary>
    /// Check if current process has elevated privileges
    /// </summary>
    Task<bool> IsElevatedAsync();

    /// <summary>
    /// Get list of important Windows services
    /// </summary>
    Task<List<SystemService>> GetImportantServicesAsync();

    /// <summary>
    /// Get service status by name
    /// </summary>
    Task<SystemService?> GetServiceStatusAsync(string serviceName);

    /// <summary>
    /// Start a Windows service
    /// </summary>
    Task<bool> StartServiceAsync(string serviceName);

    /// <summary>
    /// Stop a Windows service
    /// </summary>
    Task<bool> StopServiceAsync(string serviceName);

    /// <summary>
    /// Enable a Windows service
    /// </summary>
    Task<bool> EnableServiceAsync(string serviceName);

    /// <summary>
    /// Disable a Windows service
    /// </summary>
    Task<bool> DisableServiceAsync(string serviceName);

    /// <summary>
    /// Get Windows startup programs
    /// </summary>
    Task<List<StartupProgram>> GetStartupProgramsAsync();

    /// <summary>
    /// Enable/disable startup program
    /// </summary>
    Task<bool> SetStartupProgramStatusAsync(string name, bool enabled);

    /// <summary>
    /// Toggle Windows Defender real-time protection
    /// </summary>
    Task<bool> ToggleDefenderRealTimeProtectionAsync();

    /// <summary>
    /// Toggle Windows Defender cloud protection
    /// </summary>
    Task<bool> ToggleDefenderCloudProtectionAsync();

    /// <summary>
    /// Toggle Windows Defender tamper protection
    /// </summary>
    Task<bool> ToggleDefenderTamperProtectionAsync();

    /// <summary>
    /// Toggle Windows Defender sample submission
    /// </summary>
    Task<bool> ToggleDefenderSampleSubmissionAsync();
}
