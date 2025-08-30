using System.ComponentModel.DataAnnotations;

namespace GameBooster.Common.Models;

/// <summary>
/// Représente les informations sur la mémoire système
/// </summary>
public class MemoryInfo
{
    /// <summary>
    /// Mémoire physique totale en bytes
    /// </summary>
    public long TotalPhysical { get; set; }

    /// <summary>
    /// Mémoire physique disponible en bytes
    /// </summary>
    public long AvailablePhysical { get; set; }

    /// <summary>
    /// Mémoire physique utilisée en bytes
    /// </summary>
    public long UsedPhysical => TotalPhysical - AvailablePhysical;

    /// <summary>
    /// Pourcentage d'utilisation de la mémoire physique
    /// </summary>
    public double UsagePercentage => TotalPhysical > 0 ? (UsedPhysical * 100.0) / TotalPhysical : 0;

    /// <summary>
    /// Mémoire virtuelle totale en bytes
    /// </summary>
    public long TotalPageFile { get; set; }

    /// <summary>
    /// Mémoire virtuelle disponible en bytes
    /// </summary>
    public long AvailablePageFile { get; set; }

    /// <summary>
    /// Mémoire virtuelle utilisée en bytes
    /// </summary>
    public long UsedPageFile => TotalPageFile - AvailablePageFile;

    /// <summary>
    /// Pourcentage d'utilisation de la mémoire virtuelle
    /// </summary>
    public double PageFileUsagePercentage => TotalPageFile > 0 ? (UsedPageFile * 100.0) / TotalPageFile : 0;

    /// <summary>
    /// Mémoire cache en bytes
    /// </summary>
    public long CacheMemory { get; set; }

    /// <summary>
    /// Mémoire kernel en bytes
    /// </summary>
    public long KernelMemory { get; set; }

    /// <summary>
    /// Mémoire non paginée en bytes
    /// </summary>
    public long NonPagedMemory { get; set; }

    /// <summary>
    /// Mémoire paginée en bytes
    /// </summary>
    public long PagedMemory { get; set; }

    /// <summary>
    /// Nombre de handles ouverts
    /// </summary>
    public int HandleCount { get; set; }

    /// <summary>
    /// Nombre de threads actifs
    /// </summary>
    public int ThreadCount { get; set; }

    /// <summary>
    /// Nombre de processus actifs
    /// </summary>
    public int ProcessCount { get; set; }

    /// <summary>
    /// Date et heure de la dernière mise à jour
    /// </summary>
    public DateTime LastUpdate { get; set; } = DateTime.Now;

    /// <summary>
    /// Convertit les bytes en GB
    /// </summary>
    public static double BytesToGB(long bytes) => bytes / (1024.0 * 1024.0 * 1024.0);

    /// <summary>
    /// Convertit les bytes en MB
    /// </summary>
    public static double BytesToMB(long bytes) => bytes / (1024.0 * 1024.0);

    /// <summary>
    /// Mémoire physique totale en GB
    /// </summary>
    public double TotalPhysicalGB => BytesToGB(TotalPhysical);

    /// <summary>
    /// Mémoire physique disponible en GB
    /// </summary>
    public double AvailablePhysicalGB => BytesToGB(AvailablePhysical);

    /// <summary>
    /// Mémoire physique utilisée en GB
    /// </summary>
    public double UsedPhysicalGB => BytesToGB(UsedPhysical);
}
