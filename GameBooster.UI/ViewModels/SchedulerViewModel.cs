using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Threading.Tasks;
using System.Windows.Input;
using ReactiveUI;
using GameBooster.Common.Interfaces;
using GameBooster.Common.Models;
using Microsoft.Extensions.Logging;
using Avalonia.Threading;
using System.Reactive;

namespace GameBooster.UI.ViewModels;

/// <summary>
/// ViewModel pour la gestion du planificateur de tâches
/// </summary>
public class SchedulerViewModel : ViewModelBase
{
    private readonly ISchedulerService _schedulerService;
    private readonly ILogger<SchedulerViewModel> _logger;
    
    private ObservableCollection<ScheduledTask> _tasks = new();
    private ScheduledTask? _selectedTask;
    private bool _isLoadingTasks;
    private string _statusMessage = "Prêt";

    public ObservableCollection<ScheduledTask> Tasks
    {
        get => _tasks;
        set => this.RaiseAndSetIfChanged(ref _tasks, value);
    }

    public ScheduledTask? SelectedTask
    {
        get => _selectedTask;
        set => this.RaiseAndSetIfChanged(ref _selectedTask, value);
    }

    public new bool IsLoading
    {
        get => _isLoadingTasks;
        set => this.RaiseAndSetIfChanged(ref _isLoadingTasks, value);
    }

    public string StatusMessage
    {
        get => _statusMessage;
        set => this.RaiseAndSetIfChanged(ref _statusMessage, value);
    }

    // Commandes simples
    public ICommand LoadTasksCommand { get; }
    public ICommand AddTaskCommand { get; }
    public ICommand EditTaskCommand { get; }
    public ICommand DeleteTaskCommand { get; }
    public ICommand ExecuteTaskCommand { get; }
    public ICommand ToggleTaskCommand { get; }
    public ICommand RefreshTasksCommand { get; }

    public SchedulerViewModel(ISchedulerService schedulerService, ILogger<SchedulerViewModel> logger)
    {
        _schedulerService = schedulerService;
        _logger = logger;

        // Commandes simples pour éviter les problèmes de thread
        LoadTasksCommand = new RelayCommand(async () => await LoadTasksAsync());
        AddTaskCommand = new RelayCommand(async () => await AddTaskAsync());
        EditTaskCommand = new RelayCommand(async () => await EditTaskAsync(), () => SelectedTask != null);
        DeleteTaskCommand = new RelayCommand(async () => await DeleteTaskAsync(), () => SelectedTask != null);
        ExecuteTaskCommand = new RelayCommand(async () => await ExecuteTaskAsync(), () => SelectedTask != null);
        ToggleTaskCommand = new RelayCommand(async () => await ToggleTaskAsync(), () => SelectedTask != null);
        RefreshTasksCommand = new RelayCommand(async () => await RefreshTasksAsync());

        // Charger les tâches au démarrage
        _ = LoadTasksAsync();
    }

    private async Task LoadTasksAsync()
    {
        try
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsLoading = true);
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Chargement des tâches...");

            var tasks = await _schedulerService.GetAllTasksAsync();
            
            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                Tasks.Clear();
                foreach (var task in tasks)
                {
                    Tasks.Add(task);
                }
                StatusMessage = $"{Tasks.Count} tâches chargées";
            });

            _logger.LogInformation("Tâches chargées: {Count}", Tasks.Count);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors du chargement des tâches");
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = $"Erreur: {ex.Message}");
        }
        finally
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsLoading = false);
        }
    }

    private async Task AddTaskAsync()
    {
        try
        {
            // Créer une nouvelle tâche par défaut
            var newTask = new ScheduledTask
            {
                Id = Guid.NewGuid(),
                Name = "Nouvelle tâche",
                Description = "Description de la nouvelle tâche",
                Type = TaskType.MemoryCleanup,
                Frequency = TaskFrequency.Daily,
                ExecutionTime = TimeSpan.FromHours(9), // 9h00 par défaut
                IsEnabled = true,
                CreatedAt = DateTime.Now
            };

            var success = await _schedulerService.AddTaskAsync(newTask);
            if (success)
            {
                await Dispatcher.UIThread.InvokeAsync(() =>
                {
                    Tasks.Add(newTask);
                    SelectedTask = newTask;
                    StatusMessage = "Tâche ajoutée avec succès";
                });

                _logger.LogInformation("Nouvelle tâche ajoutée: {TaskName}", newTask.Name);
            }
            else
            {
                await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Erreur lors de l'ajout de la tâche");
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors de l'ajout de la tâche");
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = $"Erreur: {ex.Message}");
        }
    }

    private async Task EditTaskAsync()
    {
        if (SelectedTask == null)
        {
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Aucune tâche sélectionnée");
            return;
        }

        try
        {
            // Ici on pourrait ouvrir une fenêtre de dialogue pour éditer la tâche
            // Pour l'instant, on simule une modification
            var originalName = SelectedTask.Name;
            SelectedTask.Name = $"{originalName} (modifiée)";

            var success = await _schedulerService.UpdateTaskAsync(SelectedTask);
            if (success)
            {
                await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Tâche mise à jour avec succès");

                _logger.LogInformation("Tâche mise à jour: {TaskName}", SelectedTask.Name);
            }
            else
            {
                await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Erreur lors de la mise à jour de la tâche");
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors de la modification de la tâche");
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = $"Erreur: {ex.Message}");
        }
    }

    private async Task DeleteTaskAsync()
    {
        if (SelectedTask == null)
        {
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Aucune tâche sélectionnée");
            return;
        }

        try
        {
            var taskToDelete = SelectedTask;
            var success = await _schedulerService.RemoveTaskAsync(taskToDelete.Id);
            if (success)
            {
                await Dispatcher.UIThread.InvokeAsync(() =>
                {
                    Tasks.Remove(taskToDelete);
                    SelectedTask = null;
                    StatusMessage = "Tâche supprimée avec succès";
                });

                _logger.LogInformation("Tâche supprimée: {TaskName}", taskToDelete.Name);
            }
            else
            {
                await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Erreur lors de la suppression de la tâche");
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors de la suppression de la tâche");
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = $"Erreur: {ex.Message}");
        }
    }

    private async Task ExecuteTaskAsync()
    {
        if (SelectedTask == null)
        {
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Aucune tâche sélectionnée");
            return;
        }

        try
        {
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = $"Exécution de la tâche: {SelectedTask.Name}");
            
            var success = await _schedulerService.ExecuteTaskNowAsync(SelectedTask.Id);
            if (success)
            {
                await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = $"Tâche exécutée: {SelectedTask.Name}");

                _logger.LogInformation("Tâche exécutée manuellement: {TaskName}", SelectedTask.Name);
            }
            else
            {
                await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Erreur lors de l'exécution de la tâche");
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors de l'exécution de la tâche");
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = $"Erreur: {ex.Message}");
        }
    }

    private async Task ToggleTaskAsync()
    {
        if (SelectedTask == null)
        {
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "No task selected");
            return;
        }

        try
        {
            var newEnabledState = !SelectedTask.IsEnabled;
            var success = await _schedulerService.SetTaskEnabledAsync(SelectedTask.Id, newEnabledState);
            if (success)
            {
                await Dispatcher.UIThread.InvokeAsync(() =>
                {
                    SelectedTask.IsEnabled = newEnabledState;
                    StatusMessage = $"Task {(newEnabledState ? "enabled" : "disabled")}: {SelectedTask.Name}";
                });

                _logger.LogInformation("Task {State}: {TaskName}", 
                    newEnabledState ? "enabled" : "disabled", SelectedTask.Name);
            }
            else
            {
                await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Error changing task status");
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error changing task status");
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = $"Error: {ex.Message}");
        }
    }

    private async Task RefreshTasksAsync()
    {
        await LoadTasksAsync();
    }

    /// <summary>
    /// Obtient le texte d'affichage pour la fréquence d'une tâche
    /// </summary>
    public static string GetFrequencyDisplayText(TaskFrequency frequency)
    {
        return frequency switch
        {
            TaskFrequency.Daily => "Daily",
            TaskFrequency.Weekly => "Weekly",
            TaskFrequency.Monthly => "Monthly",
            TaskFrequency.Once => "Once",
            TaskFrequency.OnStartup => "On startup",
            TaskFrequency.OnShutdown => "On shutdown",
            _ => "Unknown"
        };
    }

    /// <summary>
    /// Obtient le texte d'affichage pour le type d'une tâche
    /// </summary>
    public static string GetTaskTypeDisplayText(TaskType taskType)
    {
        return taskType switch
        {
                        TaskType.MemoryCleanup => "Memory cleanup",
            TaskType.DiskCleanup => "Disk cleanup",
            TaskType.BrowserCacheCleanup => "Browser cache",
            TaskType.TempFilesCleanup => "Temporary files",
            TaskType.RecycleBinEmpty => "Recycle bin",
            TaskType.ThumbnailsCleanup => "Thumbnails",
            TaskType.SystemOptimization => "System optimization",
            TaskType.CustomCommand => "Custom command",
            _ => "Unknown"
        };
    }

    /// <summary>
    /// Obtient le texte d'affichage pour le statut d'une tâche
    /// </summary>
    public static string GetTaskStatusText(ScheduledTask task)
    {
        if (!task.IsEnabled)
            return "Disabled";

        if (task.NextExecution.HasValue)
        {
            var timeUntilNext = task.NextExecution.Value - DateTime.Now;
            if (timeUntilNext.TotalMinutes < 0)
                return "Overdue";
            else if (timeUntilNext.TotalMinutes < 1)
                return "Execution imminent";
            else if (timeUntilNext.TotalHours < 1)
                return $"In {timeUntilNext.TotalMinutes:F0} min";
            else if (timeUntilNext.TotalDays < 1)
                return $"In {timeUntilNext.TotalHours:F0} h";
            else
                return $"In {timeUntilNext.TotalDays:F0} days";
        }

        return "Not scheduled";
    }
}
