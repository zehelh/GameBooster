using System.ComponentModel.DataAnnotations;

namespace GameBooster.Common.Models;

/// <summary>
/// Represents CPU information and statistics
/// </summary>
public class CpuInfo
{
    /// <summary>
    /// CPU name/model
    /// </summary>
    public string Name { get; set; } = string.Empty;

    /// <summary>
    /// Number of physical cores
    /// </summary>
    public int PhysicalCores { get; set; }

    /// <summary>
    /// Number of logical cores (including hyper-threading)
    /// </summary>
    public int LogicalCores { get; set; }

    /// <summary>
    /// Current CPU usage percentage
    /// </summary>
    public double UsagePercentage { get; set; }

    /// <summary>
    /// Current CPU frequency in MHz
    /// </summary>
    public double Frequency { get; set; }

    /// <summary>
    /// Maximum CPU frequency in MHz
    /// </summary>
    public double MaxFrequency { get; set; }

    /// <summary>
    /// CPU temperature in Celsius (if available)
    /// </summary>
    public double? Temperature { get; set; }

    /// <summary>
    /// CPU power consumption in watts (if available)
    /// </summary>
    public double? PowerConsumption { get; set; }

    /// <summary>
    /// Last update timestamp
    /// </summary>
    public DateTime LastUpdate { get; set; } = DateTime.Now;

    // Formatted properties for display
    public string UsageFormatted => $"{UsagePercentage:F1}%";
    public string FrequencyFormatted => $"{Frequency:F0} MHz";
    public string MaxFrequencyFormatted => $"{MaxFrequency:F0} MHz";
    public string TemperatureFormatted => Temperature.HasValue ? $"{Temperature:F1}°C" : "N/A";
    public string PowerFormatted => PowerConsumption.HasValue ? $"{PowerConsumption:F1}W" : "N/A";
    public string CoresFormatted => $"{PhysicalCores} physical, {LogicalCores} logical";
}
