namespace GameBooster.Common.Models;

/// <summary>
/// Options pour le nettoyage agressif de la mémoire
/// </summary>
public class MemoryCleanupOptions
{
    /// <summary>
    /// Inclure le garbage collection agressif
    /// </summary>
    public bool IncludeGarbageCollection { get; set; } = true;
    
    /// <summary>
    /// Inclure la libération du cache système
    /// </summary>
    public bool IncludeSystemCache { get; set; } = true;
    
    /// <summary>
    /// Inclure l'optimisation de la mémoire de travail
    /// </summary>
    public bool IncludeWorkingSet { get; set; } = true;
    
    /// <summary>
    /// Inclure la libération de la mémoire standby
    /// </summary>
    public bool IncludeStandbyMemory { get; set; } = true;
    
    /// <summary>
    /// Inclure l'optimisation du pool paginé (risqué)
    /// </summary>
    public bool IncludePagedPool { get; set; } = false;
    
    /// <summary>
    /// Inclure l'optimisation du pool non paginé (risqué)
    /// </summary>
    public bool IncludeNonPagedPool { get; set; } = false;
}
