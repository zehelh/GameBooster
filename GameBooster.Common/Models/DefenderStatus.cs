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

    // Propriétés calculées pour l'interface utilisateur
    public string RealTimeProtectionText => RealTimeProtection ? "Enabled" : "Disabled";
    public string RealTimeProtectionColor => RealTimeProtection ? "LightGreen" : "#FF4444";
    
    public string CloudProtectionText => CloudProtection ? "Enabled" : "Disabled";
    public string CloudProtectionColor => CloudProtection ? "LightGreen" : "#FF4444";
    
    public string TamperProtectionText => TamperProtection ? "Enabled" : "Disabled";
    public string TamperProtectionColor => TamperProtection ? "LightGreen" : "#FF4444";
    
    public string AutomaticSampleSubmissionText => AutomaticSampleSubmission ? "Enabled" : "Disabled";
    public string AutomaticSampleSubmissionColor => AutomaticSampleSubmission ? "LightGreen" : "#FF4444";
    
    public string ProtectionButtonText => IsProtected ? "Disable Protection" : "Enable Protection";
    public string ProtectionButtonColor => IsProtected ? "#FF4444" : "LightGreen";

    // Descriptions for each protection feature
    public string RealTimeProtectionDescription => "Monitors files and programs in real-time to detect and block threats as they occur.";
    public string CloudProtectionDescription => "Sends suspicious files to Microsoft for analysis and receives threat intelligence updates.";
    public string TamperProtectionDescription => "Prevents unauthorized changes to Windows Defender settings and other security features.";
    public string SampleSubmissionDescription => "Automatically sends sample files to Microsoft to help improve threat detection.";
}
