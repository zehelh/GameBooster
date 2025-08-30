using GameBooster.Common.Models;

namespace GameBooster.Common.Interfaces;

/// <summary>
/// Interface pour les services de gestion des disques
/// </summary>
public interface IDiskService
{
    /// <summary>
    /// Obtient les informations sur tous les disques
    /// </summary>
    Task<List<DiskInfo>> GetDisksInfoAsync();

    /// <summary>
    /// Nettoie les fichiers temporaires
    /// </summary>
    Task<CleanupResult> CleanTempFilesAsync();

    /// <summary>
    /// Nettoie le cache des navigateurs
    /// </summary>
    Task<CleanupResult> CleanBrowserCacheAsync();

    /// <summary>
    /// Vide la corbeille
    /// </summary>
    Task<CleanupResult> EmptyRecycleBinAsync();

    /// <summary>
    /// Nettoie les miniatures
    /// </summary>
    Task<CleanupResult> CleanThumbnailsAsync();

    /// <summary>
    /// Nettoie les fichiers de mise à jour Windows
    /// </summary>
    Task<CleanupResult> CleanWindowsUpdateFilesAsync();

    /// <summary>
    /// Nettoie les logs système
    /// </summary>
    Task<CleanupResult> CleanSystemLogsAsync();

    /// <summary>
    /// Effectue un nettoyage complet du disque
    /// </summary>
    Task<CleanupResult> PerformFullCleanupAsync();
}
