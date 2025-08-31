using System.ComponentModel.DataAnnotations;

namespace GameBooster.Common.Models;

/// <summary>
/// Represents detailed results of a memory cleanup operation
/// </summary>
public class MemoryCleanupResult : CleanupResult
{
    /// <summary>
    /// Memory freed from garbage collection (bytes)
    /// </summary>
    public long GarbageCollectionFreed { get; set; }

    /// <summary>
    /// Memory freed from system cache (bytes)
    /// </summary>
    public long SystemCacheFreed { get; set; }

    /// <summary>
    /// Memory freed from paged pool (bytes)
    /// </summary>
    public long PagedPoolFreed { get; set; }

    /// <summary>
    /// Memory freed from non-paged pool (bytes)
    /// </summary>
    public long NonPagedPoolFreed { get; set; }

    /// <summary>
    /// Memory freed from working set optimization (bytes)
    /// </summary>
    public long WorkingSetFreed { get; set; }

    /// <summary>
    /// Memory freed from standby memory (bytes)
    /// </summary>
    public long StandbyMemoryFreed { get; set; }

    /// <summary>
    /// Memory freed from DNS cache (bytes)
    /// </summary>
    public long DnsCacheFreed { get; set; }

    /// <summary>
    /// Memory freed from Windows caches (bytes)
    /// </summary>
    public long WindowsCachesFreed { get; set; }

    /// <summary>
    /// Memory freed from system processes (bytes)
    /// </summary>
    public long SystemProcessesFreed { get; set; }

    /// <summary>
    /// Memory freed from page file (bytes)
    /// </summary>
    public long PageFileFreed { get; set; }

    /// <summary>
    /// Memory state before cleanup
    /// </summary>
    public MemoryInfo? MemoryBefore { get; set; }

    /// <summary>
    /// Memory state after cleanup
    /// </summary>
    public MemoryInfo? MemoryAfter { get; set; }

    /// <summary>
    /// Number of processes optimized
    /// </summary>
    public int ProcessesOptimized { get; set; }

    /// <summary>
    /// Number of system caches cleared
    /// </summary>
    public int CachesCleared { get; set; }

    /// <summary>
    /// Options used for the cleanup
    /// </summary>
    public MemoryCleanupOptions? OptionsUsed { get; set; }

    /// <summary>
    /// Detailed information about processes that were cleaned
    /// </summary>
    public List<ProcessCleanupInfo> ProcessCleanupDetails { get; set; } = new();

    // Computed properties for display
    public double GarbageCollectionFreedMB => GarbageCollectionFreed / (1024.0 * 1024.0);
    public double SystemCacheFreedMB => SystemCacheFreed / (1024.0 * 1024.0);
    public double PagedPoolFreedMB => PagedPoolFreed / (1024.0 * 1024.0);
    public double NonPagedPoolFreedMB => NonPagedPoolFreed / (1024.0 * 1024.0);
    public double WorkingSetFreedMB => WorkingSetFreed / (1024.0 * 1024.0);
    public double StandbyMemoryFreedMB => StandbyMemoryFreed / (1024.0 * 1024.0);
    public double DnsCacheFreedMB => DnsCacheFreed / (1024.0 * 1024.0);
    public double WindowsCachesFreedMB => WindowsCachesFreed / (1024.0 * 1024.0);
    public double SystemProcessesFreedMB => SystemProcessesFreed / (1024.0 * 1024.0);
    public double PageFileFreedMB => PageFileFreed / (1024.0 * 1024.0);

    public double GarbageCollectionFreedGB => GarbageCollectionFreed / (1024.0 * 1024.0 * 1024.0);
    public double SystemCacheFreedGB => SystemCacheFreed / (1024.0 * 1024.0 * 1024.0);
    public double PagedPoolFreedGB => PagedPoolFreed / (1024.0 * 1024.0 * 1024.0);
    public double NonPagedPoolFreedGB => NonPagedPoolFreed / (1024.0 * 1024.0 * 1024.0);
    public double WorkingSetFreedGB => WorkingSetFreed / (1024.0 * 1024.0 * 1024.0);
    public double StandbyMemoryFreedGB => StandbyMemoryFreed / (1024.0 * 1024.0 * 1024.0);
    public double DnsCacheFreedGB => DnsCacheFreed / (1024.0 * 1024.0 * 1024.0);
    public double WindowsCachesFreedGB => WindowsCachesFreed / (1024.0 * 1024.0 * 1024.0);
    public double SystemProcessesFreedGB => SystemProcessesFreed / (1024.0 * 1024.0 * 1024.0);
    public double PageFileFreedGB => PageFileFreed / (1024.0 * 1024.0 * 1024.0);

    /// <summary>
    /// Total memory freed from all sources
    /// </summary>
    public long TotalMemoryFreed => GarbageCollectionFreed + SystemCacheFreed + PagedPoolFreed + 
                                    NonPagedPoolFreed + WorkingSetFreed + StandbyMemoryFreed + 
                                    DnsCacheFreed + WindowsCachesFreed + SystemProcessesFreed + PageFileFreed;

    public double TotalMemoryFreedMB => TotalMemoryFreed / (1024.0 * 1024.0);
    public double TotalMemoryFreedGB => TotalMemoryFreed / (1024.0 * 1024.0 * 1024.0);

    /// <summary>
    /// Memory improvement percentage
    /// </summary>
    public double MemoryImprovementPercentage
    {
        get
        {
            if (MemoryBefore?.TotalPhysical > 0 && MemoryAfter?.AvailablePhysical > 0)
            {
                var beforeAvailable = MemoryBefore.AvailablePhysical;
                var afterAvailable = MemoryAfter.AvailablePhysical;
                var improvement = afterAvailable - beforeAvailable;
                return (improvement / (double)MemoryBefore.TotalPhysical) * 100.0;
            }
            return 0.0;
        }
    }

    /// <summary>
    /// Available memory before cleanup in GB
    /// </summary>
    public double AvailableMemoryBeforeGB => MemoryBefore?.AvailablePhysicalGB ?? 0.0;

    /// <summary>
    /// Available memory after cleanup in GB
    /// </summary>
    public double AvailableMemoryAfterGB => MemoryAfter?.AvailablePhysicalGB ?? 0.0;

    /// <summary>
    /// Total system memory in GB
    /// </summary>
    public double TotalMemoryGB => MemoryBefore?.TotalPhysicalGB ?? 0.0;

    /// <summary>
    /// Memory usage percentage before cleanup
    /// </summary>
    public double MemoryUsageBeforePercentage
    {
        get
        {
            if (MemoryBefore?.TotalPhysical > 0)
            {
                var used = MemoryBefore.TotalPhysical - MemoryBefore.AvailablePhysical;
                return (used / (double)MemoryBefore.TotalPhysical) * 100.0;
            }
            return 0.0;
        }
    }

    /// <summary>
    /// Memory usage percentage after cleanup
    /// </summary>
    public double MemoryUsageAfterPercentage
    {
        get
        {
            if (MemoryAfter?.TotalPhysical > 0)
            {
                var used = MemoryAfter.TotalPhysical - MemoryAfter.AvailablePhysical;
                return (used / (double)MemoryAfter.TotalPhysical) * 100.0;
            }
            return 0.0;
        }
    }

    /// <summary>
    /// Get processes sorted by memory freed (descending)
    /// </summary>
    public List<ProcessCleanupInfo> GetProcessesSortedByMemoryFreed()
    {
        return ProcessCleanupDetails
            .Where(p => p.Success && p.MemoryFreed > 0)
            .OrderByDescending(p => p.MemoryFreed)
            .ToList();
    }

    /// <summary>
    /// Total memory freed from processes only
    /// </summary>
    public long TotalProcessMemoryFreed => ProcessCleanupDetails
        .Where(p => p.Success)
        .Sum(p => p.MemoryFreed);

    public double TotalProcessMemoryFreedMB => TotalProcessMemoryFreed / (1024.0 * 1024.0);
    public double TotalProcessMemoryFreedGB => TotalProcessMemoryFreed / (1024.0 * 1024.0 * 1024.0);
}
