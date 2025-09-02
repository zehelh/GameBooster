using GameBooster.Common.Models;

namespace GameBooster.Common.Interfaces;

/// <summary>
/// Interface for CPU management and process priority control
/// </summary>
public interface ICpuService
{
    /// <summary>
    /// Get CPU information and statistics
    /// </summary>
    Task<CpuInfo> GetCpuInfoAsync();

    /// <summary>
    /// Get all processes with CPU usage information
    /// </summary>
    Task<List<CpuProcessInfo>> GetCpuProcessesAsync();

    /// <summary>
    /// Set process priority
    /// </summary>
    Task<bool> SetProcessPriorityAsync(int processId, ProcessPriority priority);

    /// <summary>
    /// Get current process priority
    /// </summary>
    Task<ProcessPriority> GetProcessPriorityAsync(int processId);

    /// <summary>
    /// Set process CPU affinity (which cores the process can use)
    /// </summary>
    Task<bool> SetProcessAffinityAsync(int processId, ulong affinityMask);

    /// <summary>
    /// Get current process CPU affinity
    /// </summary>
    Task<ulong> GetProcessAffinityAsync(int processId);

    /// <summary>
    /// Set CPU usage limit for a process (experimental)
    /// </summary>
    Task<bool> SetProcessCpuLimitAsync(int processId, int percentageLimit);

    /// <summary>
    /// Remove CPU usage limit for a process
    /// </summary>
    Task<bool> RemoveProcessCpuLimitAsync(int processId);

    /// <summary>
    /// Get CPU usage statistics for a specific process
    /// </summary>
    Task<CpuProcessInfo?> GetProcessCpuUsageAsync(int processId);

    /// <summary>
    /// Monitor CPU usage in real-time
    /// </summary>
    IObservable<CpuInfo> MonitorCpuAsync();

    /// <summary>
    /// Apply gaming optimization profile
    /// </summary>
    Task<bool> ApplyGamingProfileAsync(string gameProcessName);

    /// <summary>
    /// Reset all CPU optimizations
    /// </summary>
    Task<bool> ResetAllOptimizationsAsync();

    /// <summary>
    /// Get available CPU cores count
    /// </summary>
    Task<int> GetCpuCoresCountAsync();

    /// <summary>
    /// Get CPU performance counters
    /// </summary>
    Task<CpuPerformanceCounters> GetCpuPerformanceCountersAsync();
}
