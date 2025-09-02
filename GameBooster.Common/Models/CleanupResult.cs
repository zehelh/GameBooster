using System.ComponentModel.DataAnnotations;

namespace GameBooster.Common.Models;

/// <summary>
/// Représente le résultat d'une opération de nettoyage
/// </summary>
public class CleanupResult
{
    /// <summary>
    /// Type d'opération effectuée
    /// </summary>
    public CleanupType Type { get; set; }

    /// <summary>
    /// Nombre de fichiers supprimés
    /// </summary>
    public int FilesDeleted { get; set; }

    /// <summary>
    /// Nombre de dossiers supprimés
    /// </summary>
    public int FoldersDeleted { get; set; }

    /// <summary>
    /// Espace libéré en bytes
    /// </summary>
    public long SpaceFreed { get; set; }

    /// <summary>
    /// Durée de l'opération en millisecondes
    /// </summary>
    public long DurationMs { get; set; }

    /// <summary>
    /// Indique si l'opération s'est bien déroulée
    /// </summary>
    public bool Success { get; set; }

    /// <summary>
    /// Message d'erreur (si applicable)
    /// </summary>
    public string? ErrorMessage { get; set; }

    /// <summary>
    /// Date et heure de l'opération
    /// </summary>
    public DateTime Timestamp { get; set; } = DateTime.Now;

    /// <summary>
    /// Détails supplémentaires (JSON)
    /// </summary>
    public string? Details { get; set; }

    /// <summary>
    /// Espace libéré en MB
    /// </summary>
    public double SpaceFreedMB => SpaceFreed / (1024.0 * 1024.0);

    /// <summary>
    /// Espace libéré en GB
    /// </summary>
    public double SpaceFreedGB => SpaceFreed / (1024.0 * 1024.0 * 1024.0);

    /// <summary>
    /// Durée de l'opération en secondes
    /// </summary>
    public double DurationSeconds => DurationMs / 1000.0;

    /// <summary>
    /// Espace libéré formaté (MB/GB)
    /// </summary>
    public string SpaceFreedFormatted
    {
        get
        {
            if (SpaceFreed >= 1024 * 1024 * 1024) // 1 GB
            {
                return $"{SpaceFreedGB:F2} GB";
            }
            else if (SpaceFreed >= 1024 * 1024) // 1 MB
            {
                return $"{SpaceFreedMB:F2} MB";
            }
            else
            {
                return $"{SpaceFreed} bytes";
            }
        }
    }
}

/// <summary>
/// Types d'opérations de nettoyage
/// </summary>
public enum CleanupType
{
    TempFiles,
    BrowserCache,
    RecycleBin,
    Thumbnails,
    WindowsUpdate,
    SystemLogs,
    Memory,
    FullAnalysis,
    History,
    Custom
}
