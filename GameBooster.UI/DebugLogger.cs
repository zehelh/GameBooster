using System;
using System.IO;

namespace GameBooster.UI
{
    public static class DebugLogger
    {
        private static readonly string LogFilePath = Path.Combine(
            AppDomain.CurrentDomain.BaseDirectory, 
            "GameBooster_Debug.log"
        );

        public static void Log(string message)
        {
            var timestamp = DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss.fff");
            var logEntry = $"[{timestamp}] {message}";
            
            try
            {
                File.AppendAllText(LogFilePath, logEntry + Environment.NewLine);
                Console.WriteLine(logEntry);
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Erreur logging: {ex.Message}");
            }
        }

        public static void LogProperty(string propertyName, object value)
        {
            Log($"PROPRIÉTÉ [{propertyName}] = {value}");
        }

        public static void LogAction(string action)
        {
            Log($"ACTION: {action}");
        }

        public static void LogError(string error, Exception? ex = null)
        {
            Log($"ERREUR: {error}");
            if (ex != null)
            {
                Log($"EXCEPTION: {ex}");
            }
        }

        public static void ClearLog()
        {
            try
            {
                File.WriteAllText(LogFilePath, $"=== GameBooster Debug Log - {DateTime.Now:dd/MM/yyyy HH:mm:ss} ==={Environment.NewLine}");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Erreur clear log: {ex.Message}");
            }
        }
    }
}
