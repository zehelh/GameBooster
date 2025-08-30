using System.ComponentModel.DataAnnotations;

namespace GameBooster.Common.Models;

/// <summary>
/// Représente les informations sur un disque système
/// </summary>
public class DiskInfo
{
    /// <summary>
    /// Nom du disque (ex: C:)
    /// </summary>
    [Required]
    public string Name { get; set; } = string.Empty;

    /// <summary>
    /// Nom complet du disque
    /// </summary>
    public string? FullName { get; set; }

    /// <summary>
    /// Type de système de fichiers
    /// </summary>
    public string? FileSystem { get; set; }

    /// <summary>
    /// Taille totale en bytes
    /// </summary>
    public long TotalSize { get; set; }

    /// <summary>
    /// Espace libre en bytes
    /// </summary>
    public long FreeSpace { get; set; }

    /// <summary>
    /// Espace utilisé en bytes
    /// </summary>
    public long UsedSpace => TotalSize - FreeSpace;

    /// <summary>
    /// Pourcentage d'utilisation
    /// </summary>
    public double UsagePercentage => TotalSize > 0 ? (UsedSpace * 100.0) / TotalSize : 0;

    /// <summary>
    /// Indique si le disque est prêt
    /// </summary>
    public bool IsReady { get; set; }

    /// <summary>
    /// Type de média (SSD, HDD, etc.)
    /// </summary>
    public DriveType DriveType { get; set; }

    /// <summary>
    /// Volume label
    /// </summary>
    public string? VolumeLabel { get; set; }

    /// <summary>
    /// Date de la dernière mise à jour
    /// </summary>
    public DateTime LastUpdate { get; set; } = DateTime.Now;

    /// <summary>
    /// Convertit les bytes en GB
    /// </summary>
    public static double BytesToGB(long bytes) => bytes / (1024.0 * 1024.0 * 1024.0);

    /// <summary>
    /// Taille totale en GB
    /// </summary>
    public double TotalSizeGB => BytesToGB(TotalSize);

    /// <summary>
    /// Espace libre en GB
    /// </summary>
    public double FreeSpaceGB => BytesToGB(FreeSpace);

    /// <summary>
    /// Espace utilisé en GB
    /// </summary>
    public double UsedSpaceGB => BytesToGB(UsedSpace);
}

/// <summary>
/// Types de lecteurs disponibles
/// </summary>
public enum DriveType
{
    Unknown,
    NoRootDirectory,
    Removable,
    Fixed,
    Network,
    Optical,
    Ram
}
