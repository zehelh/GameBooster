using System.ComponentModel.DataAnnotations;

namespace GameBooster.Common.Models;

/// <summary>
/// Représente un service système Windows
/// </summary>
public class SystemService
{
    /// <summary>
    /// Nom du service
    /// </summary>
    [Required]
    public string Name { get; set; } = string.Empty;

    /// <summary>
    /// Nom d'affichage du service
    /// </summary>
    public string? DisplayName { get; set; }

    /// <summary>
    /// Description du service
    /// </summary>
    public string? Description { get; set; }

    /// <summary>
    /// État actuel du service
    /// </summary>
    public ServiceStatus Status { get; set; }

    /// <summary>
    /// Type de démarrage du service
    /// </summary>
    public ServiceStartType StartType { get; set; }

    /// <summary>
    /// Chemin de l'exécutable du service
    /// </summary>
    public string? ExecutablePath { get; set; }

    /// <summary>
    /// Nombre de processus associés
    /// </summary>
    public int ProcessCount { get; set; }

    /// <summary>
    /// Utilisation mémoire en bytes
    /// </summary>
    public long MemoryUsage { get; set; }

    /// <summary>
    /// Indique si le service est critique pour le système
    /// </summary>
    public bool IsCritical { get; set; }

    /// <summary>
    /// Indique si le service peut être arrêté
    /// </summary>
    public bool CanStop { get; set; }

    /// <summary>
    /// Indique si le service peut être redémarré
    /// </summary>
    public bool CanRestart { get; set; }

    /// <summary>
    /// Date de la dernière mise à jour
    /// </summary>
    public DateTime LastUpdate { get; set; } = DateTime.Now;

    /// <summary>
    /// Utilisation mémoire en MB
    /// </summary>
    public double MemoryUsageMB => MemoryUsage / (1024.0 * 1024.0);

    /// <summary>
    /// Indique si le service est en cours d'exécution
    /// </summary>
    public bool IsRunning => Status == ServiceStatus.Running;

    /// <summary>
    /// Indique si le service peut être démarré
    /// </summary>
    public bool CanStart => Status == ServiceStatus.Stopped;

    /// <summary>
    /// Couleur du statut pour l'interface utilisateur
    /// </summary>
    public string StatusColor => IsRunning ? "LightGreen" : "#FF4444";
}

/// <summary>
/// États possibles d'un service
/// </summary>
public enum ServiceStatus
{
    Stopped,
    StartPending,
    StopPending,
    Running,
    ContinuePending,
    PausePending,
    Paused,
    Unknown
}

/// <summary>
/// Types de démarrage d'un service
/// </summary>
public enum ServiceStartType
{
    Boot,
    System,
    Automatic,
    Manual,
    Disabled
}
