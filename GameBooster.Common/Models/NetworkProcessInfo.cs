namespace GameBooster.Common.Models;

/// <summary>
/// Network process information
/// </summary>
public class NetworkProcessInfo
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
    /// Total bytes sent
    /// </summary>
    public long BytesSent { get; set; }

    /// <summary>
    /// Total bytes received
    /// </summary>
    public long BytesReceived { get; set; }

    /// <summary>
    /// Total packets sent
    /// </summary>
    public long PacketsSent { get; set; }

    /// <summary>
    /// Total packets received
    /// </summary>
    public long PacketsReceived { get; set; }

    /// <summary>
    /// Whether the process is limited
    /// </summary>
    public bool IsLimited { get; set; }

    /// <summary>
    /// Speed limit in Kbps (null if unlimited)
    /// </summary>
    public int? SpeedLimit { get; set; }

    /// <summary>
    /// Number of active connections
    /// </summary>
    public int Connections { get; set; }

    /// <summary>
    /// Current upload speed in bytes per second
    /// </summary>
    public long CurrentUploadSpeed { get; set; }

    /// <summary>
    /// Current download speed in bytes per second
    /// </summary>
    public long CurrentDownloadSpeed { get; set; }

    /// <summary>
    /// Total bytes (sent + received)
    /// </summary>
    public long TotalBytes => BytesSent + BytesReceived;

    /// <summary>
    /// Formatted total bytes
    /// </summary>
    public string TotalBytesFormatted
    {
        get
        {
            if (TotalBytes < 1024) return $"{TotalBytes} B";
            if (TotalBytes < 1024 * 1024) return $"{TotalBytes / 1024.0:F1} KB";
            if (TotalBytes < 1024 * 1024 * 1024) return $"{TotalBytes / (1024.0 * 1024.0):F1} MB";
            return $"{TotalBytes / (1024.0 * 1024.0 * 1024.0):F1} GB";
        }
    }

    /// <summary>
    /// Formatted current upload speed
    /// </summary>
    public string CurrentUploadSpeedFormatted
    {
        get
        {
            if (CurrentUploadSpeed < 1024) return $"{CurrentUploadSpeed} B/s";
            if (CurrentUploadSpeed < 1024 * 1024) return $"{CurrentUploadSpeed / 1024.0:F1} KB/s";
            return $"{CurrentUploadSpeed / (1024.0 * 1024.0):F1} MB/s";
        }
    }

    /// <summary>
    /// Formatted current download speed
    /// </summary>
    public string CurrentDownloadSpeedFormatted
    {
        get
        {
            if (CurrentDownloadSpeed < 1024) return $"{CurrentDownloadSpeed} B/s";
            if (CurrentDownloadSpeed < 1024 * 1024) return $"{CurrentDownloadSpeed / 1024.0:F1} KB/s";
            return $"{CurrentDownloadSpeed / (1024.0 * 1024.0):F1} MB/s";
        }
    }

    /// <summary>
    /// Formatted speed limit
    /// </summary>
    public string SpeedLimitFormatted => SpeedLimit.HasValue ? $"{SpeedLimit.Value} Kbps" : "Unlimited";

    /// <summary>
    /// Process ID formatted
    /// </summary>
    public string ProcessIdFormatted => $"PID: {ProcessId}";

    /// <summary>
    /// Status color for UI
    /// </summary>
    public string StatusColor => IsLimited ? "#FFA500" : "LightGreen";
}
