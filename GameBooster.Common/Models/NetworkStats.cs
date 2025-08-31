namespace GameBooster.Common.Models;

/// <summary>
/// Network statistics
/// </summary>
public class NetworkStats
{
    /// <summary>
    /// Total upload bytes
    /// </summary>
    public long TotalUploadBytes { get; set; }

    /// <summary>
    /// Total download bytes
    /// </summary>
    public long TotalDownloadBytes { get; set; }

    /// <summary>
    /// Total processes
    /// </summary>
    public int TotalProcesses { get; set; }

    /// <summary>
    /// Number of limited processes
    /// </summary>
    public int LimitedProcessesCount { get; set; }

    /// <summary>
    /// Current upload speed in bytes per second
    /// </summary>
    public long CurrentUploadSpeed { get; set; }

    /// <summary>
    /// Current download speed in bytes per second
    /// </summary>
    public long CurrentDownloadSpeed { get; set; }

    /// <summary>
    /// Total bytes (upload + download)
    /// </summary>
    public long TotalBytes => TotalUploadBytes + TotalDownloadBytes;

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
    /// Percentage of limited processes
    /// </summary>
    public double LimitedProcessesPercentage => TotalProcesses > 0 ? (LimitedProcessesCount * 100.0) / TotalProcesses : 0;

    /// <summary>
    /// Formatted limited processes percentage
    /// </summary>
    public string LimitedProcessesPercentageFormatted => $"{LimitedProcessesPercentage:F1}%";
}
