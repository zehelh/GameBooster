using System.ComponentModel.DataAnnotations;

namespace GameBooster.Common.Models;

/// <summary>
/// Représente les informations d'un processus système
/// </summary>
public class ProcessInfo
{
    /// <summary>
    /// ID du processus
    /// </summary>
    public int ProcessId { get; set; }

    /// <summary>
    /// Nom du processus
    /// </summary>
    [Required]
    public string Name { get; set; } = string.Empty;

    /// <summary>
    /// Nom complet du processus
    /// </summary>
    public string? FullName { get; set; }

    /// <summary>
    /// Utilisation CPU en pourcentage
    /// </summary>
    public double CpuUsage { get; set; }

    /// <summary>
    /// Utilisation mémoire en bytes
    /// </summary>
    public long MemoryUsage { get; set; }

    /// <summary>
    /// Utilisation mémoire en MB
    /// </summary>
    public double MemoryUsageMB => MemoryUsage / (1024.0 * 1024.0);

    /// <summary>
    /// Nombre de threads
    /// </summary>
    public int ThreadCount { get; set; }

    /// <summary>
    /// Priorité du processus
    /// </summary>
    public int Priority { get; set; }

    /// <summary>
    /// Date de démarrage du processus
    /// </summary>
    public DateTime StartTime { get; set; }

    // Formatted properties for display
    public string MemoryUsageFormatted => (MemoryUsage / (1024.0 * 1024.0)).ToString("F1") + " MB";
    public string ProcessIdFormatted => $"PID: {ProcessId}";
    public string CpuUsageFormatted => $"{CpuUsage:F1}%";

    /// <summary>
    /// Indique si le processus est actif
    /// </summary>
    public bool IsActive { get; set; }

    /// <summary>
    /// Chemin de l'exécutable
    /// </summary>
    public string? ExecutablePath { get; set; }

    /// <summary>
    /// Nombre de connexions réseau (si applicable)
    /// </summary>
    public int NetworkConnections { get; set; }

    /// <summary>
    /// Vitesse de téléchargement en bytes/s
    /// </summary>
    public long DownloadSpeed { get; set; }

    /// <summary>
    /// Vitesse d'upload en bytes/s
    /// </summary>
    public long UploadSpeed { get; set; }

    /// <summary>
    /// Indique si le processus est limité en bande passante
    /// </summary>
    public bool IsNetworkLimited { get; set; }

    /// <summary>
    /// Limite de bande passante en KB/s (si applicable)
    /// </summary>
    public int? NetworkSpeedLimit { get; set; }
}
