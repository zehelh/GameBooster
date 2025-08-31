using GameBooster.Common.Models;

namespace GameBooster.Common.Interfaces;

/// <summary>
/// Interface for system memory management services
/// </summary>
public interface IMemoryService
{
    /// <summary>
    /// Gets current memory information
    /// </summary>
    Task<MemoryInfo> GetMemoryInfoAsync();

    /// <summary>
    /// Cleans system memory with advanced options
    /// </summary>
    Task<MemoryCleanupResult> CleanMemoryAsync(MemoryCleanupOptions? options = null);

    /// <summary>
    /// Gets the list of processes using the most memory
    /// </summary>
    Task<List<ProcessInfo>> GetTopMemoryProcessesAsync(int count = 10);

    /// <summary>
    /// Optimizes memory by freeing unused resources
    /// </summary>
    Task<MemoryCleanupResult> OptimizeMemoryAsync();

    /// <summary>
    /// Monitors memory in real-time
    /// </summary>
    IObservable<MemoryInfo> MonitorMemoryAsync();
}
