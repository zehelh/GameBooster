using System.ComponentModel.DataAnnotations;

namespace GameBooster.Common.Models;

/// <summary>
/// Représente une tâche planifiée dans le système
/// </summary>
public class ScheduledTask
{
    /// <summary>
    /// Identifiant unique de la tâche
    /// </summary>
    public Guid Id { get; set; } = Guid.NewGuid();

    /// <summary>
    /// Nom de la tâche
    /// </summary>
    [Required]
    [StringLength(100)]
    public string Name { get; set; } = string.Empty;

    /// <summary>
    /// Description de la tâche
    /// </summary>
    [StringLength(500)]
    public string? Description { get; set; }

    /// <summary>
    /// Type de tâche à exécuter
    /// </summary>
    public TaskType Type { get; set; }

    /// <summary>
    /// Fréquence d'exécution
    /// </summary>
    public TaskFrequency Frequency { get; set; }

    /// <summary>
    /// Heure d'exécution (pour les tâches quotidiennes)
    /// </summary>
    public TimeSpan? ExecutionTime { get; set; }

    /// <summary>
    /// Jour de la semaine (pour les tâches hebdomadaires)
    /// </summary>
    public DayOfWeek? DayOfWeek { get; set; }

    /// <summary>
    /// Jour du mois (pour les tâches mensuelles)
    /// </summary>
    public int? DayOfMonth { get; set; }

    /// <summary>
    /// Indique si la tâche est activée
    /// </summary>
    public bool IsEnabled { get; set; } = true;

    /// <summary>
    /// Date de création de la tâche
    /// </summary>
    public DateTime CreatedAt { get; set; } = DateTime.Now;

    /// <summary>
    /// Date de la dernière exécution
    /// </summary>
    public DateTime? LastExecuted { get; set; }

    /// <summary>
    /// Date de la prochaine exécution prévue
    /// </summary>
    public DateTime? NextExecution { get; set; }

    /// <summary>
    /// Paramètres spécifiques à la tâche (JSON)
    /// </summary>
    public string? Parameters { get; set; }

    /// <summary>
    /// Propriétés calculées pour l'interface utilisateur
    /// </summary>
    public string StatusText => IsEnabled ? "Enabled" : "Disabled";
    public string StatusColor => IsEnabled ? "#4CAF50" : "#FF4444";
    public string Schedule => GetScheduleDescription();
    
    private string GetScheduleDescription()
    {
        return Frequency switch
        {
            TaskFrequency.Once => "Once",
            TaskFrequency.Daily => $"Daily at {ExecutionTime:hh\\:mm}",
            TaskFrequency.Weekly => $"Weekly on {DayOfWeek} at {ExecutionTime:hh\\:mm}",
            TaskFrequency.Monthly => $"Monthly on day {DayOfMonth} at {ExecutionTime:hh\\:mm}",
            TaskFrequency.OnStartup => "On Startup",
            TaskFrequency.OnShutdown => "On Shutdown",
            _ => "Unknown"
        };
    }
}

/// <summary>
/// Types de tâches disponibles
/// </summary>
public enum TaskType
{
    MemoryCleanup,
    DiskCleanup,
    BrowserCacheCleanup,
    TempFilesCleanup,
    RecycleBinEmpty,
    ThumbnailsCleanup,
    SystemOptimization,
    CustomCommand
}

/// <summary>
/// Fréquences d'exécution disponibles
/// </summary>
public enum TaskFrequency
{
    Once,
    Daily,
    Weekly,
    Monthly,
    OnStartup,
    OnShutdown
}
