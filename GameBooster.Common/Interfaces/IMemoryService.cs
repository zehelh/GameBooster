using GameBooster.Common.Models;

namespace GameBooster.Common.Interfaces;

/// <summary>
/// Interface pour les services de gestion de la mémoire système
/// </summary>
public interface IMemoryService
{
    /// <summary>
    /// Obtient les informations actuelles sur la mémoire
    /// </summary>
    Task<MemoryInfo> GetMemoryInfoAsync();

    /// <summary>
    /// Nettoie la mémoire système avec options avancées
    /// </summary>
    Task<CleanupResult> CleanMemoryAsync(MemoryCleanupOptions? options = null);

    /// <summary>
    /// Obtient la liste des processus utilisant le plus de mémoire
    /// </summary>
    Task<List<ProcessInfo>> GetTopMemoryProcessesAsync(int count = 10);

    /// <summary>
    /// Optimise la mémoire en libérant les ressources inutilisées
    /// </summary>
    Task<CleanupResult> OptimizeMemoryAsync();

    /// <summary>
    /// Surveille la mémoire en temps réel
    /// </summary>
    IObservable<MemoryInfo> MonitorMemoryAsync();
}
