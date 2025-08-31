namespace GameBooster.Common.Models;

/// <summary>
/// Windows startup program information
/// </summary>
public class StartupProgram
{
    /// <summary>
    /// Program name
    /// </summary>
    public string Name { get; set; } = string.Empty;

    /// <summary>
    /// Program description
    /// </summary>
    public string Description { get; set; } = string.Empty;

    /// <summary>
    /// Executable path
    /// </summary>
    public string ExecutablePath { get; set; } = string.Empty;

    /// <summary>
    /// Registry key path
    /// </summary>
    public string RegistryPath { get; set; } = string.Empty;

    /// <summary>
    /// Whether the program is enabled
    /// </summary>
    public bool IsEnabled { get; set; }

    /// <summary>
    /// Startup type (Registry, Startup folder, etc.)
    /// </summary>
    public string StartupType { get; set; } = string.Empty;

    /// <summary>
    /// Program publisher
    /// </summary>
    public string Publisher { get; set; } = string.Empty;

    /// <summary>
    /// File size in bytes
    /// </summary>
    public long FileSize { get; set; }

    /// <summary>
    /// Last modified date
    /// </summary>
    public DateTime LastModified { get; set; }

    /// <summary>
    /// Formatted file size
    /// </summary>
    public string FileSizeFormatted
    {
        get
        {
            if (FileSize < 1024) return $"{FileSize} B";
            if (FileSize < 1024 * 1024) return $"{FileSize / 1024.0:F1} KB";
            if (FileSize < 1024 * 1024 * 1024) return $"{FileSize / (1024.0 * 1024.0):F1} MB";
            return $"{FileSize / (1024.0 * 1024.0 * 1024.0):F1} GB";
        }
    }

    /// <summary>
    /// Formatted last modified date
    /// </summary>
    public string LastModifiedFormatted => LastModified.ToString("yyyy-MM-dd HH:mm:ss");

    /// <summary>
    /// Status text for UI
    /// </summary>
    public string StatusText => IsEnabled ? "Enabled" : "Disabled";

    /// <summary>
    /// Status color for UI
    /// </summary>
    public string StatusColor => IsEnabled ? "#FF4444" : "LightGreen";

    /// <summary>
    /// Toggle button text for UI
    /// </summary>
    public string ToggleButtonText => IsEnabled ? "Disable" : "Enable";

    /// <summary>
    /// Toggle button color for UI
    /// </summary>
    public string ToggleButtonColor => IsEnabled ? "LightGreen" : "#FF4444";
}
