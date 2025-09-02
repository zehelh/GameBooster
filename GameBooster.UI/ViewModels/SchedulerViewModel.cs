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
using System.Linq;

namespace GameBooster.UI.ViewModels;

/// <summary>
/// ViewModel pour la gestion du planificateur de tâches
/// </summary>
public class SchedulerViewModel : ViewModelBase
{
    private readonly ISchedulerService _schedulerService;
    private readonly ILogger<SchedulerViewModel> _logger;
    
    private ObservableCollection<ScheduledTask> _scheduledTasks = new();
    private ScheduledTask? _selectedTask;
    private bool _isLoading;
    private string _statusMessage = "Ready";

    public ObservableCollection<ScheduledTask> ScheduledTasks
    {
        get => _scheduledTasks;
        set => this.RaiseAndSetIfChanged(ref _scheduledTasks, value);
    }

    public ScheduledTask? SelectedTask
    {
        get => _selectedTask;
        set => this.RaiseAndSetIfChanged(ref _selectedTask, value);
    }

    public new bool IsLoading
    {
        get => _isLoading;
        set => this.RaiseAndSetIfChanged(ref _isLoading, value);
    }

    public string StatusMessage
    {
        get => _statusMessage;
        set => this.RaiseAndSetIfChanged(ref _statusMessage, value);
    }

    // Propriétés calculées pour les statistiques
    public int EnabledTasksCount => ScheduledTasks.Count(t => t.IsEnabled);
    public int DisabledTasksCount => ScheduledTasks.Count(t => !t.IsEnabled);
    public bool HasTasks => ScheduledTasks.Count > 0;

    // Commandes pour les actions rapides
    public ICommand ScheduleMemoryCleanupCommand { get; }
    public ICommand ScheduleDiskCleanupCommand { get; }
    public ICommand ScheduleSystemOptimizationCommand { get; }
    public ICommand CreateCustomTaskCommand { get; }

    // Commandes pour la gestion des tâches
    public ICommand RefreshTasksCommand { get; }
    public ICommand StartAllTasksCommand { get; }
    public ICommand PauseAllTasksCommand { get; }
    public ICommand ClearAllTasksCommand { get; }
    public ICommand EditTaskCommand { get; }
    public ICommand DeleteTaskCommand { get; }

    public SchedulerViewModel(ISchedulerService schedulerService, ILogger<SchedulerViewModel> logger)
    {
        _schedulerService = schedulerService;
        _logger = logger;

        // Commandes pour les actions rapides
        ScheduleMemoryCleanupCommand = new RelayCommand(async () => await ScheduleMemoryCleanupAsync());
        ScheduleDiskCleanupCommand = new RelayCommand(async () => await ScheduleDiskCleanupAsync());
        ScheduleSystemOptimizationCommand = new RelayCommand(async () => await ScheduleSystemOptimizationAsync());
        CreateCustomTaskCommand = new RelayCommand(async () => await CreateCustomTaskAsync());

        // Commandes pour la gestion des tâches
        RefreshTasksCommand = new RelayCommand(async () => await RefreshTasksAsync());
        StartAllTasksCommand = new RelayCommand(async () => await StartAllTasksAsync());
        PauseAllTasksCommand = new RelayCommand(async () => await PauseAllTasksAsync());
        ClearAllTasksCommand = new RelayCommand(async () => await ClearAllTasksAsync());
        EditTaskCommand = new RelayCommand<ScheduledTask>(async (task) => await EditTaskAsync(task), (task) => true);
        DeleteTaskCommand = new RelayCommand<ScheduledTask>(async (task) => await DeleteTaskAsync(task), (task) => true);

        // Charger les tâches au démarrage
        _ = LoadTasksAsync();
    }

    private async Task LoadTasksAsync()
    {
        try
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsLoading = true);
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Loading tasks...");

            var tasks = await _schedulerService.GetAllTasksAsync();
            
            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                ScheduledTasks.Clear();
                foreach (var task in tasks)
                {
                    ScheduledTasks.Add(task);
                }
                StatusMessage = $"{ScheduledTasks.Count} tasks loaded";
                
                // Notifier les changements pour les propriétés calculées
                this.RaisePropertyChanged(nameof(EnabledTasksCount));
                this.RaisePropertyChanged(nameof(DisabledTasksCount));
                this.RaisePropertyChanged(nameof(HasTasks));
            });

            _logger.LogInformation("Tasks loaded: {Count}", ScheduledTasks.Count);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error loading tasks");
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = $"Error: {ex.Message}");
        }
        finally
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsLoading = false);
        }
    }

    private async Task ScheduleMemoryCleanupAsync()
    {
        try
        {
            var task = new ScheduledTask
            {
                Id = Guid.NewGuid(),
                Name = "Memory Cleanup",
                Description = "Automatic memory optimization",
                Type = TaskType.MemoryCleanup,
                Frequency = TaskFrequency.Daily,
                ExecutionTime = TimeSpan.FromHours(2), // 2:00 AM
                IsEnabled = true,
                CreatedAt = DateTime.Now,
                NextExecution = DateTime.Today.AddDays(1).AddHours(2)
            };

            var success = await _schedulerService.AddTaskAsync(task);
            if (success)
            {
                await Dispatcher.UIThread.InvokeAsync(() =>
                {
                    ScheduledTasks.Add(task);
                    StatusMessage = "Memory cleanup task scheduled successfully";
                    this.RaisePropertyChanged(nameof(EnabledTasksCount));
                    this.RaisePropertyChanged(nameof(DisabledTasksCount));
                    this.RaisePropertyChanged(nameof(HasTasks));
                });

                _logger.LogInformation("Memory cleanup task scheduled: {TaskName}", task.Name);
            }
            else
            {
                await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Error scheduling memory cleanup task");
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error scheduling memory cleanup task");
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = $"Error: {ex.Message}");
        }
    }

    private async Task ScheduleDiskCleanupAsync()
    {
        try
        {
            var task = new ScheduledTask
            {
                Id = Guid.NewGuid(),
                Name = "Disk Cleanup",
                Description = "Automatic disk space cleanup",
                Type = TaskType.DiskCleanup,
                Frequency = TaskFrequency.Weekly,
                ExecutionTime = TimeSpan.FromHours(3), // 3:00 AM
                IsEnabled = true,
                CreatedAt = DateTime.Now,
                NextExecution = DateTime.Today.AddDays(7).AddHours(3)
            };

            var success = await _schedulerService.AddTaskAsync(task);
            if (success)
            {
                await Dispatcher.UIThread.InvokeAsync(() =>
                {
                    ScheduledTasks.Add(task);
                    StatusMessage = "Disk cleanup task scheduled successfully";
                    this.RaisePropertyChanged(nameof(EnabledTasksCount));
                    this.RaisePropertyChanged(nameof(DisabledTasksCount));
                    this.RaisePropertyChanged(nameof(HasTasks));
                });

                _logger.LogInformation("Disk cleanup task scheduled: {TaskName}", task.Name);
            }
            else
            {
                await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Error scheduling disk cleanup task");
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error scheduling disk cleanup task");
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = $"Error: {ex.Message}");
        }
    }

    private async Task ScheduleSystemOptimizationAsync()
    {
        try
        {
            var task = new ScheduledTask
            {
                Id = Guid.NewGuid(),
                Name = "System Optimization",
                Description = "Complete system optimization",
                Type = TaskType.SystemOptimization,
                Frequency = TaskFrequency.Daily,
                ExecutionTime = TimeSpan.FromHours(1), // 1:00 AM
                IsEnabled = true,
                CreatedAt = DateTime.Now,
                NextExecution = DateTime.Today.AddDays(1).AddHours(1)
            };

            var success = await _schedulerService.AddTaskAsync(task);
            if (success)
            {
                await Dispatcher.UIThread.InvokeAsync(() =>
                {
                    ScheduledTasks.Add(task);
                    StatusMessage = "System optimization task scheduled successfully";
                    this.RaisePropertyChanged(nameof(EnabledTasksCount));
                    this.RaisePropertyChanged(nameof(DisabledTasksCount));
                    this.RaisePropertyChanged(nameof(HasTasks));
                });

                _logger.LogInformation("System optimization task scheduled: {TaskName}", task.Name);
            }
            else
            {
                await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Error scheduling system optimization task");
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error scheduling system optimization task");
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = $"Error: {ex.Message}");
        }
    }

    private async Task CreateCustomTaskAsync()
    {
        try
        {
            var task = new ScheduledTask
            {
                Id = Guid.NewGuid(),
                Name = "Custom Task",
                Description = "Custom scheduled task",
                Type = TaskType.CustomCommand,
                Frequency = TaskFrequency.Once,
                ExecutionTime = TimeSpan.FromHours(12), // 12:00 PM
                IsEnabled = true,
                CreatedAt = DateTime.Now,
                NextExecution = DateTime.Today.AddHours(12)
            };

            var success = await _schedulerService.AddTaskAsync(task);
            if (success)
            {
                await Dispatcher.UIThread.InvokeAsync(() =>
                {
                    ScheduledTasks.Add(task);
                    SelectedTask = task;
                    StatusMessage = "Custom task created successfully";
                    this.RaisePropertyChanged(nameof(EnabledTasksCount));
                    this.RaisePropertyChanged(nameof(DisabledTasksCount));
                    this.RaisePropertyChanged(nameof(HasTasks));
                });

                _logger.LogInformation("Custom task created: {TaskName}", task.Name);
            }
            else
            {
                await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Error creating custom task");
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error creating custom task");
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = $"Error: {ex.Message}");
        }
    }

    private async Task RefreshTasksAsync()
    {
        await LoadTasksAsync();
    }

    private async Task StartAllTasksAsync()
    {
        try
        {
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Enabling all tasks...");
            
            var tasksToEnable = ScheduledTasks.Where(t => !t.IsEnabled).ToList();
            var successCount = 0;

            foreach (var task in tasksToEnable)
            {
                var success = await _schedulerService.SetTaskEnabledAsync(task.Id, true);
                if (success)
                {
                    task.IsEnabled = true;
                    successCount++;
                }
            }

            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                // Forcer la mise à jour de la collection
                var currentTasks = ScheduledTasks.ToList();
                ScheduledTasks.Clear();
                foreach (var task in currentTasks)
                {
                    ScheduledTasks.Add(task);
                }
                
                StatusMessage = $"{successCount} tasks enabled";
                this.RaisePropertyChanged(nameof(EnabledTasksCount));
                this.RaisePropertyChanged(nameof(DisabledTasksCount));
                this.RaisePropertyChanged(nameof(HasTasks));
            });

            _logger.LogInformation("Enabled {Count} tasks", successCount);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error enabling all tasks");
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = $"Error: {ex.Message}");
        }
    }

    private async Task PauseAllTasksAsync()
    {
        try
        {
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Disabling all tasks...");
            
            var tasksToDisable = ScheduledTasks.Where(t => t.IsEnabled).ToList();
            var successCount = 0;

            foreach (var task in tasksToDisable)
            {
                var success = await _schedulerService.SetTaskEnabledAsync(task.Id, false);
                if (success)
                {
                    task.IsEnabled = false;
                    successCount++;
                }
            }

            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                // Forcer la mise à jour de la collection
                var currentTasks = ScheduledTasks.ToList();
                ScheduledTasks.Clear();
                foreach (var task in currentTasks)
                {
                    ScheduledTasks.Add(task);
                }
                
                StatusMessage = $"{successCount} tasks disabled";
                this.RaisePropertyChanged(nameof(EnabledTasksCount));
                this.RaisePropertyChanged(nameof(DisabledTasksCount));
                this.RaisePropertyChanged(nameof(HasTasks));
            });

            _logger.LogInformation("Disabled {Count} tasks", successCount);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error disabling all tasks");
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = $"Error: {ex.Message}");
        }
    }

    private async Task ClearAllTasksAsync()
    {
        try
        {
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Clearing all tasks...");
            
            var tasksToDelete = ScheduledTasks.ToList();
            var successCount = 0;

            foreach (var task in tasksToDelete)
            {
                var success = await _schedulerService.RemoveTaskAsync(task.Id);
                if (success)
                {
                    successCount++;
                }
            }

            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                ScheduledTasks.Clear();
                SelectedTask = null;
                StatusMessage = $"{successCount} tasks cleared";
                this.RaisePropertyChanged(nameof(EnabledTasksCount));
                this.RaisePropertyChanged(nameof(DisabledTasksCount));
                this.RaisePropertyChanged(nameof(HasTasks));
            });

            _logger.LogInformation("Cleared {Count} tasks", successCount);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error clearing all tasks");
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = $"Error: {ex.Message}");
        }
    }

    private async Task EditTaskAsync(ScheduledTask? task = null)
    {
        var taskToEdit = task ?? SelectedTask;
        if (taskToEdit == null)
        {
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "No task selected");
            return;
        }

        try
        {
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Editing task...");

            // Créer une copie modifiée de la tâche
            var modifiedTask = new ScheduledTask
            {
                Id = taskToEdit.Id,
                Name = $"{taskToEdit.Name} (Modified)",
                Description = taskToEdit.Description,
                Type = taskToEdit.Type,
                Frequency = taskToEdit.Frequency,
                ExecutionTime = taskToEdit.ExecutionTime,
                DayOfWeek = taskToEdit.DayOfWeek,
                DayOfMonth = taskToEdit.DayOfMonth,
                IsEnabled = !taskToEdit.IsEnabled, // Toggle l'état
                LastExecuted = taskToEdit.LastExecuted,
                NextExecution = taskToEdit.NextExecution,
                CreatedAt = taskToEdit.CreatedAt,
                Parameters = taskToEdit.Parameters
            };

            var success = await _schedulerService.UpdateTaskAsync(modifiedTask);
            if (success)
            {
                // Mettre à jour la tâche dans la collection
                var index = ScheduledTasks.IndexOf(taskToEdit);
                if (index >= 0)
                {
                    await Dispatcher.UIThread.InvokeAsync(() =>
                    {
                        ScheduledTasks[index] = modifiedTask;
                        SelectedTask = modifiedTask;
                        StatusMessage = $"Task '{modifiedTask.Name}' updated successfully";
                        this.RaisePropertyChanged(nameof(EnabledTasksCount));
                        this.RaisePropertyChanged(nameof(DisabledTasksCount));
                    });
                }

                _logger.LogInformation("Task updated: {TaskName}", modifiedTask.Name);
            }
            else
            {
                await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Error updating task");
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error updating task");
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = $"Error: {ex.Message}");
        }
    }

    private async Task DeleteTaskAsync(ScheduledTask? task = null)
    {
        var taskToDelete = task ?? SelectedTask;
        if (taskToDelete == null)
        {
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "No task selected");
            return;
        }

        try
        {
            var success = await _schedulerService.RemoveTaskAsync(taskToDelete.Id);
            if (success)
            {
                await Dispatcher.UIThread.InvokeAsync(() =>
                {
                    ScheduledTasks.Remove(taskToDelete);
                    if (SelectedTask == taskToDelete)
                        SelectedTask = null;
                    StatusMessage = "Task deleted successfully";
                    this.RaisePropertyChanged(nameof(EnabledTasksCount));
                    this.RaisePropertyChanged(nameof(DisabledTasksCount));
                    this.RaisePropertyChanged(nameof(HasTasks));
                });

                _logger.LogInformation("Task deleted: {TaskName}", taskToDelete.Name);
            }
            else
            {
                await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = "Error deleting task");
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error deleting task");
            await Dispatcher.UIThread.InvokeAsync(() => StatusMessage = $"Error: {ex.Message}");
        }
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
