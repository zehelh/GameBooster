namespace GameBooster.Common.Models;

/// <summary>
/// CPU performance counters and metrics
/// </summary>
public class CpuPerformanceCounters
{
    /// <summary>
    /// Total CPU time
    /// </summary>
    public long TotalCpuTime { get; set; }

    /// <summary>
    /// Idle CPU time
    /// </summary>
    public long IdleCpuTime { get; set; }

    /// <summary>
    /// User CPU time
    /// </summary>
    public long UserCpuTime { get; set; }

    /// <summary>
    /// Kernel CPU time
    /// </summary>
    public long KernelCpuTime { get; set; }

    /// <summary>
    /// Interrupt time
    /// </summary>
    public long InterruptTime { get; set; }

    /// <summary>
    /// DPC (Deferred Procedure Call) time
    /// </summary>
    public long DpcTime { get; set; }

    /// <summary>
    /// Context switches per second
    /// </summary>
    public double ContextSwitchesPerSec { get; set; }

    /// <summary>
    /// System calls per second
    /// </summary>
    public double SystemCallsPerSec { get; set; }

    /// <summary>
    /// Last update timestamp
    /// </summary>
    public DateTime LastUpdate { get; set; } = DateTime.Now;

    // Formatted properties for display
    public string ContextSwitchesFormatted => $"{ContextSwitchesPerSec:F0}/s";
    public string SystemCallsFormatted => $"{SystemCallsPerSec:F0}/s";
}
