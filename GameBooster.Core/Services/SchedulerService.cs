using System.Text.Json;
using GameBooster.Common.Interfaces;
using GameBooster.Common.Models;
using Microsoft.Extensions.Logging;

namespace GameBooster.Core.Services;

/// <summary>
/// Service de planification des tâches
/// </summary>
public class SchedulerService : ISchedulerService
{
    private readonly ILogger<SchedulerService> _logger;
    private readonly List<ScheduledTask> _tasks;
    private readonly Timer _schedulerTimer;
    private readonly string _tasksFilePath;

    public SchedulerService(ILogger<SchedulerService> logger)
    {
        _logger = logger;
        _tasks = new List<ScheduledTask>();
        _tasksFilePath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), 
            "GameBooster", "scheduled_tasks.json");
        
        _schedulerTimer = new Timer(CheckScheduledTasks, null, TimeSpan.Zero, TimeSpan.FromMinutes(1));
        
        // Charge les tâches existantes
        _ = LoadTasksAsync(_tasksFilePath);
    }

    public async Task<bool> AddTaskAsync(ScheduledTask task)
    {
        try
        {
            if (_tasks.Any(t => t.Id == task.Id))
            {
                _logger.LogWarning("Une tâche avec l'ID {TaskId} existe déjà", task.Id);
                return false;
            }

            _tasks.Add(task);
            await SaveTasksAsync(_tasksFilePath);
            
            _logger.LogInformation("Tâche ajoutée: {TaskName}", task.Name);
            return true;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors de l'ajout de la tâche");
            return false;
        }
    }

    public async Task<bool> UpdateTaskAsync(ScheduledTask task)
    {
        try
        {
            var existingTask = _tasks.FirstOrDefault(t => t.Id == task.Id);
            if (existingTask == null)
            {
                _logger.LogWarning("Tâche non trouvée: {TaskId}", task.Id);
                return false;
            }

            var index = _tasks.IndexOf(existingTask);
            _tasks[index] = task;
            await SaveTasksAsync(_tasksFilePath);
            
            _logger.LogInformation("Tâche mise à jour: {TaskName}", task.Name);
            return true;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors de la mise à jour de la tâche");
            return false;
        }
    }

    public async Task<bool> RemoveTaskAsync(Guid taskId)
    {
        try
        {
            var task = _tasks.FirstOrDefault(t => t.Id == taskId);
            if (task == null)
            {
                _logger.LogWarning("Tâche non trouvée: {TaskId}", taskId);
                return false;
            }

            _tasks.Remove(task);
            await SaveTasksAsync(_tasksFilePath);
            
            _logger.LogInformation("Tâche supprimée: {TaskName}", task.Name);
            return true;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors de la suppression de la tâche");
            return false;
        }
    }

    public async Task<List<ScheduledTask>> GetAllTasksAsync()
    {
        return await Task.FromResult(_tasks.ToList());
    }

    public async Task<ScheduledTask?> GetTaskByIdAsync(Guid taskId)
    {
        return await Task.FromResult(_tasks.FirstOrDefault(t => t.Id == taskId));
    }

    public async Task<bool> SetTaskEnabledAsync(Guid taskId, bool enabled)
    {
        try
        {
            var task = _tasks.FirstOrDefault(t => t.Id == taskId);
            if (task == null)
            {
                _logger.LogWarning("Tâche non trouvée: {TaskId}", taskId);
                return false;
            }

            task.IsEnabled = enabled;
            await SaveTasksAsync(_tasksFilePath);
            
            _logger.LogInformation("Tâche {TaskName} {Status}", task.Name, enabled ? "activée" : "désactivée");
            return true;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors du changement d'état de la tâche");
            return false;
        }
    }

    public async Task<bool> ExecuteTaskNowAsync(Guid taskId)
    {
        try
        {
            var task = _tasks.FirstOrDefault(t => t.Id == taskId);
            if (task == null)
            {
                _logger.LogWarning("Tâche non trouvée: {TaskId}", taskId);
                return false;
            }

            await ExecuteTaskAsync(task);
            return true;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors de l'exécution immédiate de la tâche");
            return false;
        }
    }

    public async Task SaveTasksAsync(string filePath)
    {
        try
        {
            var directory = Path.GetDirectoryName(filePath);
            if (!string.IsNullOrEmpty(directory) && !Directory.Exists(directory))
            {
                Directory.CreateDirectory(directory);
            }

            var json = JsonSerializer.Serialize(_tasks, new JsonSerializerOptions 
            { 
                WriteIndented = true 
            });
            await File.WriteAllTextAsync(filePath, json);
            
            _logger.LogDebug("Tâches sauvegardées dans: {FilePath}", filePath);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors de la sauvegarde des tâches");
        }
    }

    public async Task LoadTasksAsync(string filePath)
    {
        try
        {
            if (!File.Exists(filePath))
            {
                _logger.LogInformation("Fichier de tâches non trouvé: {FilePath}", filePath);
                return;
            }

            var json = await File.ReadAllTextAsync(filePath);
            var loadedTasks = JsonSerializer.Deserialize<List<ScheduledTask>>(json);
            
            if (loadedTasks != null)
            {
                _tasks.Clear();
                _tasks.AddRange(loadedTasks);
                _logger.LogInformation("{Count} tâches chargées", _tasks.Count);
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors du chargement des tâches");
        }
    }

    private void CheckScheduledTasks(object? state)
    {
        var now = DateTime.Now;
        var tasksToExecute = _tasks.Where(t => 
            t.IsEnabled && 
            ShouldExecuteTask(t, now)).ToList();

        foreach (var task in tasksToExecute)
        {
            _ = Task.Run(async () => await ExecuteTaskAsync(task));
        }
    }

    private bool ShouldExecuteTask(ScheduledTask task, DateTime now)
    {
        if (task.ExecutionTime == null) return false;
        
        var taskTime = task.ExecutionTime.Value;
        var currentTime = now.TimeOfDay;
        
        // Vérifie si l'heure correspond (avec une marge de 1 minute)
        var timeMatch = Math.Abs((currentTime - taskTime).TotalMinutes) <= 1;
        
        return task.Frequency switch
        {
            TaskFrequency.Daily => timeMatch,
            TaskFrequency.Weekly => task.DayOfWeek == now.DayOfWeek && timeMatch,
            TaskFrequency.Monthly => now.Day == task.DayOfMonth && timeMatch,
            _ => false
        };
    }

    private async Task ExecuteTaskAsync(ScheduledTask task)
    {
        try
        {
            _logger.LogInformation("Exécution de la tâche: {TaskName}", task.Name);
            
            // Ici, on exécuterait la tâche selon son type
            // Pour l'instant, on simule l'exécution
            
            task.LastExecuted = DateTime.Now;
            task.NextExecution = CalculateNextExecution(task);
            
            await SaveTasksAsync(_tasksFilePath);
            
            _logger.LogInformation("Tâche exécutée avec succès: {TaskName}", task.Name);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors de l'exécution de la tâche: {TaskName}", task.Name);
        }
    }

    private DateTime? CalculateNextExecution(ScheduledTask task)
    {
        var now = DateTime.Now;
        
        return task.Frequency switch
        {
            TaskFrequency.Daily => now.AddDays(1).Date.Add(task.ExecutionTime ?? TimeSpan.Zero),
            TaskFrequency.Weekly => now.AddDays(7).Date.Add(task.ExecutionTime ?? TimeSpan.Zero),
            TaskFrequency.Monthly => now.AddMonths(1).Date.Add(task.ExecutionTime ?? TimeSpan.Zero),
            _ => null
        };
    }
}
