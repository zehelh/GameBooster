using System.ComponentModel.DataAnnotations;

namespace GameBooster.Common.Models;

/// <summary>
/// Represents detailed information about a process that was cleaned during memory optimization
/// </summary>
public class ProcessCleanupInfo
{
    /// <summary>
    /// Process ID
    /// </summary>
    public int ProcessId { get; set; }

    /// <summary>
    /// Process name
    /// </summary>
    public string Name { get; set; } = string.Empty;

    /// <summary>
    /// Memory freed from this process in bytes
    /// </summary>
    public long MemoryFreed { get; set; }

    /// <summary>
    /// Memory freed from this process in MB
    /// </summary>
    public double MemoryFreedMB => MemoryFreed / (1024.0 * 1024.0);

    /// <summary>
    /// Memory freed from this process in GB
    /// </summary>
    public double MemoryFreedGB => MemoryFreed / (1024.0 * 1024.0 * 1024.0);

    /// <summary>
    /// Working set size before cleanup in bytes
    /// </summary>
    public long WorkingSetBefore { get; set; }

    /// <summary>
    /// Working set size after cleanup in bytes
    /// </summary>
    public long WorkingSetAfter { get; set; }

    /// <summary>
    /// Whether the cleanup was successful
    /// </summary>
    public bool Success { get; set; }

    /// <summary>
    /// Error message if cleanup failed
    /// </summary>
    public string? ErrorMessage { get; set; }

    /// <summary>
    /// Timestamp when the cleanup was performed
    /// </summary>
    public DateTime Timestamp { get; set; } = DateTime.Now;

    // Formatted properties for display
    public string MemoryFreedGBFormatted => MemoryFreedGB.ToString("F2") + " GB";
    public string WorkingSetBeforeFormatted => (WorkingSetBefore / (1024.0 * 1024.0)).ToString("F1") + " MB";
    public string ProcessIdFormatted => $"PID: {ProcessId}";
}
