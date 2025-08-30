using GameBooster.Common.Models;

namespace GameBooster.Common.Interfaces;

/// <summary>
/// Interface pour le service de planification des tâches
/// </summary>
public interface ISchedulerService
{
    /// <summary>
    /// Ajoute une nouvelle tâche planifiée
    /// </summary>
    Task<bool> AddTaskAsync(ScheduledTask task);

    /// <summary>
    /// Met à jour une tâche existante
    /// </summary>
    Task<bool> UpdateTaskAsync(ScheduledTask task);

    /// <summary>
    /// Supprime une tâche
    /// </summary>
    Task<bool> RemoveTaskAsync(Guid taskId);

    /// <summary>
    /// Obtient toutes les tâches planifiées
    /// </summary>
    Task<List<ScheduledTask>> GetAllTasksAsync();

    /// <summary>
    /// Obtient une tâche par son ID
    /// </summary>
    Task<ScheduledTask?> GetTaskByIdAsync(Guid taskId);

    /// <summary>
    /// Active ou désactive une tâche
    /// </summary>
    Task<bool> SetTaskEnabledAsync(Guid taskId, bool enabled);

    /// <summary>
    /// Exécute une tâche immédiatement
    /// </summary>
    Task<bool> ExecuteTaskNowAsync(Guid taskId);

    /// <summary>
    /// Sauvegarde les tâches dans un fichier
    /// </summary>
    Task SaveTasksAsync(string filePath);

    /// <summary>
    /// Charge les tâches depuis un fichier
    /// </summary>
    Task LoadTasksAsync(string filePath);
}
