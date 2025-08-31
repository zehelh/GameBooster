namespace GameBooster.Common.Models;

/// <summary>
/// QoS policy information
/// </summary>
public class QosPolicyInfo
{
    /// <summary>
    /// Policy name
    /// </summary>
    public string Name { get; set; } = string.Empty;

    /// <summary>
    /// Application name
    /// </summary>
    public string AppName { get; set; } = string.Empty;

    /// <summary>
    /// Throttle rate in bits per second
    /// </summary>
    public long ThrottleBits { get; set; }

    /// <summary>
    /// Whether the policy is registry only
    /// </summary>
    public bool IsRegistryOnly { get; set; }

    /// <summary>
    /// Policy priority
    /// </summary>
    public int Priority { get; set; }

    /// <summary>
    /// Policy status
    /// </summary>
    public string Status { get; set; } = string.Empty;

    /// <summary>
    /// Formatted throttle rate
    /// </summary>
    public string ThrottleRateFormatted
    {
        get
        {
            if (ThrottleBits < 1024) return $"{ThrottleBits} bps";
            if (ThrottleBits < 1024 * 1024) return $"{ThrottleBits / 1024.0:F1} Kbps";
            if (ThrottleBits < 1024 * 1024 * 1024) return $"{ThrottleBits / (1024.0 * 1024.0):F1} Mbps";
            return $"{ThrottleBits / (1024.0 * 1024.0 * 1024.0):F1} Gbps";
        }
    }

    /// <summary>
    /// Status color for UI
    /// </summary>
    public string StatusColor => Status == "Active" ? "LightGreen" : "#FFA500";
}
