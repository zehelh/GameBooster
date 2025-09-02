using System.ComponentModel.DataAnnotations;

namespace GameBooster.Common.Models;

/// <summary>
/// Represents CPU usage information for a specific process
/// </summary>
public class CpuProcessInfo
{
    /// <summary>
    /// Process ID
    /// </summary>
    public int ProcessId { get; set; }

    /// <summary>
    /// Process name
    /// </summary>
    [Required]
    public string Name { get; set; } = string.Empty;

    /// <summary>
    /// Process full name/path
    /// </summary>
    public string? FullName { get; set; }

    /// <summary>
    /// Current CPU usage percentage
    /// </summary>
    public double CpuUsage { get; set; }

    /// <summary>
    /// Average CPU usage over time
    /// </summary>
    public double AverageCpuUsage { get; set; }

    /// <summary>
    /// Peak CPU usage
    /// </summary>
    public double PeakCpuUsage { get; set; }

    /// <summary>
    /// Current process priority
    /// </summary>
    public ProcessPriority Priority { get; set; }

    /// <summary>
    /// CPU affinity mask (which cores the process can use)
    /// </summary>
    public ulong AffinityMask { get; set; }

    /// <summary>
    /// Number of threads
    /// </summary>
    public int ThreadCount { get; set; }

    /// <summary>
    /// Process start time
    /// </summary>
    public DateTime StartTime { get; set; }

    /// <summary>
    /// Indicates if the process is currently running
    /// </summary>
    public bool IsActive { get; set; }

    /// <summary>
    /// CPU usage limit percentage (if set)
    /// </summary>
    public int? CpuLimit { get; set; }

    /// <summary>
    /// Indicates if CPU limit is active
    /// </summary>
    public bool IsCpuLimited => CpuLimit.HasValue;

    // Formatted properties for display
    public string CpuUsageFormatted => $"{CpuUsage:F1}%";
    public string AverageCpuUsageFormatted => $"{AverageCpuUsage:F1}%";
    public string PeakCpuUsageFormatted => $"{PeakCpuUsage:F1}%";
    public string PriorityFormatted => Priority.ToString();
    public string AffinityFormatted => GetAffinityDescription();
    public string ProcessIdFormatted => $"PID: {ProcessId}";
    public string CpuLimitFormatted => CpuLimit.HasValue ? $"{CpuLimit}%" : "No limit";

    private string GetAffinityDescription()
    {
        if (AffinityMask == 0) return "None";
        
        var cores = new List<int>();
        for (int i = 0; i < 64; i++)
        {
            if ((AffinityMask & (1UL << i)) != 0)
            {
                cores.Add(i);
            }
        }
        
        return cores.Count <= 4 ? string.Join(", ", cores) : $"Cores {cores.First()}-{cores.Last()}";
    }
}
