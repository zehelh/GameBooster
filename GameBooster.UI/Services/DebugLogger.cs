using System;
using System.IO;
using System.Threading.Tasks;
using Microsoft.Extensions.Logging;

namespace GameBooster.UI.Services;

/// <summary>
    /// Advanced logging service with separate files by type and automatic cleanup
/// </summary>
public static class DebugLogger
{
    private static readonly string LogBasePath = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "GameBooster", "Logs");

    private static readonly object LogLock = new object();

    static DebugLogger()
    {
        // Créer le répertoire de logs s'il n'existe pas
        Directory.CreateDirectory(LogBasePath);
        
        // Clean old logs at startup
        Task.Run(() => CleanOldLogsAsync());
    }

    /// <summary>
    /// Logger une action
    /// </summary>
    public static void LogAction(string message)
    {
        LogToFile("actions", message);
    }

    /// <summary>
    /// Logger une propriété
    /// </summary>
    public static void LogProperty(string property, object value)
    {
        LogToFile("properties", $"{property}: {value}");
    }

    /// <summary>
    /// Logger une erreur
    /// </summary>
    public static void LogError(string message, Exception? exception = null)
    {
        var errorMessage = exception != null 
            ? $"{message} - Exception: {exception.Message}\nStackTrace: {exception.StackTrace}"
            : message;
        LogToFile("errors", errorMessage);
    }

    /// <summary>
    /// Logger un warning
    /// </summary>
    public static void LogWarning(string message)
    {
        LogToFile("warnings", message);
    }

    /// <summary>
    /// Logger une information
    /// </summary>
    public static void LogInfo(string message)
    {
        LogToFile("info", message);
    }

    /// <summary>
    /// Logger une opération de mémoire
    /// </summary>
    public static void LogMemoryOperation(string operation, long bytesFreed, long durationMs)
    {
        var message = $"{operation} - Freed: {bytesFreed / (1024 * 1024):F2} MB - Duration: {durationMs}ms";
        LogToFile("memory", message);
    }

    /// <summary>
    /// Logger une opération de disque
    /// </summary>
    public static void LogDiskOperation(string operation, long bytesFreed, long durationMs)
    {
        var message = $"{operation} - Freed: {bytesFreed / (1024 * 1024):F2} MB - Duration: {durationMs}ms";
        LogToFile("disk", message);
    }

    /// <summary>
    /// Logger une opération de planification
    /// </summary>
    public static void LogSchedulerOperation(string operation, string details)
    {
        var message = $"{operation} - {details}";
        LogToFile("scheduler", message);
    }

    /// <summary>
    /// Logger une opération de service
    /// </summary>
    public static void LogServiceOperation(string serviceName, string operation, bool success)
    {
        var status = success ? "SUCCESS" : "FAILED";
        var message = $"{serviceName} - {operation} - {status}";
        LogToFile("services", message);
    }

    /// <summary>
    /// Logger une opération réseau
    /// </summary>
    public static void LogNetworkOperation(string operation, string details)
    {
        var message = $"{operation} - {details}";
        LogToFile("network", message);
    }

    /// <summary>
    /// Logger une opération UI
    /// </summary>
    public static void LogUIOperation(string operation, string details)
    {
        var message = $"{operation} - {details}";
        LogToFile("ui", message);
    }

    /// <summary>
    /// Logger une performance
    /// </summary>
    public static void LogPerformance(string operation, long durationMs, string details = "")
    {
        var message = $"{operation} - Duration: {durationMs}ms - {details}";
        LogToFile("performance", message);
    }

    /// <summary>
    /// Logger vers un fichier spécifique
    /// </summary>
    private static void LogToFile(string logType, string message)
    {
        try
        {
            var today = DateTime.Now.ToString("yyyy-MM-dd");
            var logFileName = $"{logType}_{today}.log";
            var logFilePath = Path.Combine(LogBasePath, logFileName);

            var timestamp = DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss.fff");
            var logEntry = $"[{timestamp}] {message}";

            lock (LogLock)
            {
                File.AppendAllText(logFilePath, logEntry + Environment.NewLine);
            }
        }
        catch (Exception ex)
        {
            // En cas d'erreur de logging, on ne peut pas logger l'erreur de logging
            // On utilise Console.WriteLine comme fallback
            Console.WriteLine($"Logging error: {ex.Message}");
        }
    }

    /// <summary>
    /// Clean old logs (older than 7 days)
    /// </summary>
    private static void CleanOldLogsAsync()
    {
        try
        {
            var cutoffDate = DateTime.Now.AddDays(-7);
            var logFiles = Directory.GetFiles(LogBasePath, "*.log");

            foreach (var logFile in logFiles)
            {
                var fileInfo = new FileInfo(logFile);
                if (fileInfo.CreationTime < cutoffDate)
                {
                    try
                    {
                        File.Delete(logFile);
                        LogInfo($"Deleted old log file: {Path.GetFileName(logFile)}");
                    }
                    catch (Exception ex)
                    {
                        LogError($"Failed to delete old log file {Path.GetFileName(logFile)}", ex);
                    }
                }
            }
        }
        catch (Exception ex)
        {
            LogError("Error cleaning old logs", ex);
        }
    }

    /// <summary>
    /// Obtenir le chemin des logs
    /// </summary>
    public static string GetLogPath()
    {
        return LogBasePath;
    }

    /// <summary>
    /// Obtenir la taille totale des logs
    /// </summary>
    public static long GetTotalLogSize()
    {
        try
        {
            var logFiles = Directory.GetFiles(LogBasePath, "*.log");
            long totalSize = 0;

            foreach (var logFile in logFiles)
            {
                var fileInfo = new FileInfo(logFile);
                totalSize += fileInfo.Length;
            }

            return totalSize;
        }
        catch
        {
            return 0;
        }
    }

    /// <summary>
    /// Vider tous les logs
    /// </summary>
    public static void ClearAllLogs()
    {
        try
        {
            var logFiles = Directory.GetFiles(LogBasePath, "*.log");
            foreach (var logFile in logFiles)
            {
                File.Delete(logFile);
            }
            LogInfo("All logs cleared");
        }
        catch (Exception ex)
        {
            LogError("Error clearing all logs", ex);
        }
    }
}
