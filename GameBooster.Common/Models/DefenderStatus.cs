namespace GameBooster.Common.Models;

/// <summary>
/// Windows Defender status information
/// </summary>
public class DefenderStatus
{
    /// <summary>
    /// Real-time protection status
    /// </summary>
    public bool RealTimeProtection { get; set; }

    /// <summary>
    /// Cloud protection status
    /// </summary>
    public bool CloudProtection { get; set; }

    /// <summary>
    /// Automatic sample submission status
    /// </summary>
    public bool AutomaticSampleSubmission { get; set; }

    /// <summary>
    /// Tamper protection status
    /// </summary>
    public bool TamperProtection { get; set; }

    /// <summary>
    /// Last check timestamp
    /// </summary>
    public DateTime LastCheck { get; set; } = DateTime.Now;

    /// <summary>
    /// Human-readable status text
    /// </summary>
    public string StatusText { get; set; } = "Unknown";

    /// <summary>
    /// Last operation results
    /// </summary>
    public List<string> LastOperationResults { get; set; } = new();

    /// <summary>
    /// Overall protection status
    /// </summary>
    public bool IsProtected => RealTimeProtection && CloudProtection;

    /// <summary>
    /// Formatted status text
    /// </summary>
    public string FormattedStatus => IsProtected ? "Protected" : "Unprotected";

    /// <summary>
    /// Status color for UI
    /// </summary>
    public string StatusColor => IsProtected ? "LightGreen" : "#FF4444";
}
