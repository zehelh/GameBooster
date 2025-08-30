using System;
using System.IO;
using System.Threading.Tasks;

namespace GameBooster.UI.Logging
{
    public static class DebugLogger
    {
        private static readonly string LogsDirectory = Path.Combine(
            AppDomain.CurrentDomain.BaseDirectory, 
            "Logs",
            DateTime.Now.ToString("yyyy-MM-dd")
        );

        private static readonly string InfoLogPath = Path.Combine(LogsDirectory, "info.log");
        private static readonly string WarningLogPath = Path.Combine(LogsDirectory, "warning.log");
        private static readonly string ErrorLogPath = Path.Combine(LogsDirectory, "error.log");
        private static readonly string DebugLogPath = Path.Combine(LogsDirectory, "debug.log");

        static DebugLogger()
        {
            // Créer le répertoire de logs s'il n'existe pas
            Directory.CreateDirectory(LogsDirectory);
            
            // Nettoyer les logs de plus de 7 jours
            CleanOldLogs();
        }

        public static void LogInfo(string message)
        {
            WriteToLog(InfoLogPath, "INFO", message);
        }

        public static void LogWarning(string message)
        {
            WriteToLog(WarningLogPath, "WARNING", message);
        }

        public static void LogError(string message, Exception? ex = null)
        {
            var fullMessage = message;
            if (ex != null)
            {
                fullMessage += $" | Exception: {ex.Message} | StackTrace: {ex.StackTrace}";
            }
            WriteToLog(ErrorLogPath, "ERROR", fullMessage);
        }

        public static void LogDebug(string message)
        {
            WriteToLog(DebugLogPath, "DEBUG", message);
        }

        public static void LogProperty(string propertyName, object? value)
        {
            LogDebug($"PROPRIÉTÉ [{propertyName}] = {value}");
        }

        public static void LogAction(string action)
        {
            LogDebug($"ACTION: {action}");
        }

        public static void ClearLog()
        {
            try
            {
                if (File.Exists(DebugLogPath))
                {
                    File.WriteAllText(DebugLogPath, string.Empty);
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Erreur lors du nettoyage du log: {ex.Message}");
            }
        }

        private static void WriteToLog(string filePath, string level, string message)
        {
            var timestamp = DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss.fff");
            var logEntry = $"[{timestamp}] [{level}] {message}";
            
            try
            {
                // Écrire de manière asynchrone pour éviter le blocage
                Task.Run(() =>
                {
                    try
                    {
                        File.AppendAllText(filePath, logEntry + Environment.NewLine);
                    }
                    catch (Exception ex)
                    {
                        Console.WriteLine($"Erreur écriture log: {ex.Message}");
                    }
                });
                
                // Afficher dans la console pour le débogage
                Console.WriteLine(logEntry);
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Erreur logging: {ex.Message}");
            }
        }

        private static void CleanOldLogs()
        {
            try
            {
                var logsBaseDirectory = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Logs");
                if (!Directory.Exists(logsBaseDirectory)) return;

                var directories = Directory.GetDirectories(logsBaseDirectory);
                var cutoffDate = DateTime.Now.AddDays(-7);

                foreach (var directory in directories)
                {
                    var dirName = Path.GetFileName(directory);
                    if (DateTime.TryParseExact(dirName, "yyyy-MM-dd", null, System.Globalization.DateTimeStyles.None, out var dirDate))
                    {
                        if (dirDate < cutoffDate)
                        {
                            try
                            {
                                Directory.Delete(directory, true);
                                Console.WriteLine($"Suppression des logs anciens: {dirName}");
                            }
                            catch (Exception ex)
                            {
                                Console.WriteLine($"Erreur lors de la suppression du répertoire {dirName}: {ex.Message}");
                            }
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Erreur lors du nettoyage des logs: {ex.Message}");
            }
        }

        /// <summary>
        /// Log les informations de performance
        /// </summary>
        public static void LogPerformance(string operation, TimeSpan duration)
        {
            LogInfo($"PERFORMANCE [{operation}] = {duration.TotalMilliseconds:F2}ms");
        }

        /// <summary>
        /// Log les informations de mémoire
        /// </summary>
        public static void LogMemoryInfo(string operation, long bytesFreed, long totalMemory)
        {
            var mbFreed = bytesFreed / (1024.0 * 1024.0);
            var totalGB = totalMemory / (1024.0 * 1024.0 * 1024.0);
            LogInfo($"MÉMOIRE [{operation}] = {mbFreed:F2} MB libérés / {totalGB:F2} GB total");
        }

        /// <summary>
        /// Log les informations de disque
        /// </summary>
        public static void LogDiskInfo(string operation, long bytesFreed, string diskPath)
        {
            var mbFreed = bytesFreed / (1024.0 * 1024.0);
            LogInfo($"DISQUE [{operation}] = {mbFreed:F2} MB libérés sur {diskPath}");
        }

        /// <summary>
        /// Log les erreurs de service
        /// </summary>
        public static void LogServiceError(string serviceName, string operation, Exception ex)
        {
            LogError($"SERVICE [{serviceName}] - {operation} échoué", ex);
        }

        /// <summary>
        /// Log les succès de service
        /// </summary>
        public static void LogServiceSuccess(string serviceName, string operation, object? result = null)
        {
            var message = $"SERVICE [{serviceName}] - {operation} réussi";
            if (result != null)
            {
                message += $" | Résultat: {result}";
            }
            LogInfo(message);
        }
    }
}
