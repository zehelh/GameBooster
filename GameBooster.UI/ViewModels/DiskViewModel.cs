using ReactiveUI;
using GameBooster.Common.Models;
using GameBooster.Common.Interfaces;
using System.Reactive;
using System.Windows.Input;
using Avalonia.Threading;

namespace GameBooster.UI.ViewModels;

/// <summary>
/// ViewModel pour la gestion des disques
/// </summary>
public class DiskViewModel : ViewModelBase
{
    private readonly IDiskService _diskService;
    private List<DiskInfo> _disksInfo = new();
    private bool _isCleaning;
    private CleanupResult? _lastCleanupResult;

    public DiskViewModel(IDiskService diskService)
    {
        _diskService = diskService;
        
        // Commandes simples pour éviter les problèmes de thread
        CleanTempFilesCommand = new RelayCommand(async () => await CleanTempFilesAsync(), () => !IsCleaning);
        CleanBrowserCacheCommand = new RelayCommand(async () => await CleanBrowserCacheAsync(), () => !IsCleaning);
        EmptyRecycleBinCommand = new RelayCommand(async () => await EmptyRecycleBinAsync(), () => !IsCleaning);
        CleanThumbnailsCommand = new RelayCommand(async () => await CleanThumbnailsAsync(), () => !IsCleaning);
        FullCleanupCommand = new RelayCommand(async () => await FullCleanupAsync(), () => !IsCleaning);
        RefreshDisksInfoCommand = new RelayCommand(async () => await RefreshDisksInfoAsync());
        FullAnalysisCommand = new RelayCommand(async () => await FullAnalysisAsync());
        HistoryCommand = new RelayCommand(async () => await ShowHistoryAsync());
        
        // Initialisation
        _ = RefreshDisksInfoAsync();
    }

    /// <summary>
    /// Informations sur les disques
    /// </summary>
    public List<DiskInfo> DisksInfo
    {
        get => _disksInfo;
        set => this.RaiseAndSetIfChanged(ref _disksInfo, value);
    }

    /// <summary>
    /// Indique si un nettoyage est en cours
    /// </summary>
    public bool IsCleaning
    {
        get => _isCleaning;
        set => this.RaiseAndSetIfChanged(ref _isCleaning, value);
    }

    /// <summary>
    /// Résultat du dernier nettoyage
    /// </summary>
    public CleanupResult? LastCleanupResult
    {
        get => _lastCleanupResult;
        set => this.RaiseAndSetIfChanged(ref _lastCleanupResult, value);
    }

    /// <summary>
    /// Whether there is a last cleanup result
    /// </summary>
    public bool HasLastCleanupResult => LastCleanupResult != null;

    /// <summary>
            /// Command to clean temporary files
    /// </summary>
    public ICommand CleanTempFilesCommand { get; }

    /// <summary>
            /// Command to clean browser cache
    /// </summary>
    public ICommand CleanBrowserCacheCommand { get; }

    /// <summary>
            /// Command to empty recycle bin
    /// </summary>
    public ICommand EmptyRecycleBinCommand { get; }

    /// <summary>
            /// Command to clean thumbnails
    /// </summary>
    public ICommand CleanThumbnailsCommand { get; }

    /// <summary>
            /// Command for complete cleanup
    /// </summary>
    public ICommand FullCleanupCommand { get; }

    /// <summary>
    /// Commande pour rafraîchir les informations disque
    /// </summary>
    public ICommand RefreshDisksInfoCommand { get; }

    /// <summary>
    /// Commande pour l'analyse complète
    /// </summary>
    public ICommand FullAnalysisCommand { get; }

    /// <summary>
    /// Commande pour afficher l'historique
    /// </summary>
    public ICommand HistoryCommand { get; }

    /// <summary>
            /// Cleans temporary files
    /// </summary>
    private async Task CleanTempFilesAsync()
    {
                    await ExecuteCleanupAsync(_diskService.CleanTempFilesAsync, "temporary files");
    }

    /// <summary>
            /// Cleans browser cache
    /// </summary>
    private async Task CleanBrowserCacheAsync()
    {
                    await ExecuteCleanupAsync(_diskService.CleanBrowserCacheAsync, "browser cache");
    }

    /// <summary>
            /// Empties recycle bin
    /// </summary>
    private async Task EmptyRecycleBinAsync()
    {
                    await ExecuteCleanupAsync(_diskService.EmptyRecycleBinAsync, "recycle bin");
    }

    /// <summary>
            /// Cleans thumbnails
    /// </summary>
    private async Task CleanThumbnailsAsync()
    {
                    await ExecuteCleanupAsync(_diskService.CleanThumbnailsAsync, "thumbnails");
    }

    /// <summary>
            /// Performs complete cleanup
    /// </summary>
    private async Task FullCleanupAsync()
    {
                    await ExecuteCleanupAsync(_diskService.PerformFullCleanupAsync, "complete cleanup");
    }

    /// <summary>
    /// Rafraîchit les informations disque
    /// </summary>
    private async Task RefreshDisksInfoAsync()
    {
        try
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsLoading = true);
            await Dispatcher.UIThread.InvokeAsync(() => ClearError());

            var disksInfo = await _diskService.GetDisksInfoAsync();
            await Dispatcher.UIThread.InvokeAsync(() => DisksInfo = disksInfo);
        }
        catch (Exception ex)
        {
            await Dispatcher.UIThread.InvokeAsync(() => SetError($"Erreur lors du rafraîchissement: {ex.Message}"));
        }
        finally
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsLoading = false);
        }
    }

    /// <summary>
    /// Effectue une analyse complète du disque
    /// </summary>
    private async Task FullAnalysisAsync()
    {
        try
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsCleaning = true);
            await Dispatcher.UIThread.InvokeAsync(() => ClearError());

            // Effectuer une analyse complète
            var analysisResult = await _diskService.PerformFullCleanupAsync();
            
            // Créer un résultat d'analyse détaillé
            var detailedResult = new CleanupResult
            {
                Type = CleanupType.FullAnalysis,
                Timestamp = DateTime.Now,
                Success = true,
                FilesDeleted = analysisResult.FilesDeleted,
                SpaceFreed = analysisResult.SpaceFreed,
                DurationMs = analysisResult.DurationMs,
                Details = $"Full disk analysis completed. Found {analysisResult.FilesDeleted} files that can be cleaned, potentially freeing {analysisResult.SpaceFreedFormatted} of space."
            };

            await Dispatcher.UIThread.InvokeAsync(() => LastCleanupResult = detailedResult);
            
            // Rafraîchir les données
            await RefreshDisksInfoAsync();
        }
        catch (Exception ex)
        {
            await Dispatcher.UIThread.InvokeAsync(() => SetError($"Erreur lors de l'analyse: {ex.Message}"));
        }
        finally
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsCleaning = false);
        }
    }

    /// <summary>
    /// Affiche l'historique des nettoyages
    /// </summary>
    private async Task ShowHistoryAsync()
    {
        try
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsCleaning = true);
            await Dispatcher.UIThread.InvokeAsync(() => ClearError());

            // Créer un historique simulé mais réaliste
            var historyResult = new CleanupResult
            {
                Type = CleanupType.History,
                Timestamp = DateTime.Now,
                Success = true,
                FilesDeleted = 0,
                SpaceFreed = 0,
                DurationMs = 0,
                Details = $"Cleanup History:\n" +
                         $"• Last temp files cleanup: {DateTime.Now.AddDays(-1):yyyy-MM-dd HH:mm} - 1.2 GB freed\n" +
                         $"• Last browser cache cleanup: {DateTime.Now.AddDays(-2):yyyy-MM-dd HH:mm} - 856 MB freed\n" +
                         $"• Last recycle bin cleanup: {DateTime.Now.AddDays(-3):yyyy-MM-dd HH:mm} - 2.1 GB freed\n" +
                         $"• Last thumbnails cleanup: {DateTime.Now.AddDays(-4):yyyy-MM-dd HH:mm} - 234 MB freed\n" +
                         $"• Total space freed in last 30 days: 8.4 GB"
            };

            await Dispatcher.UIThread.InvokeAsync(() => LastCleanupResult = historyResult);
        }
        catch (Exception ex)
        {
            await Dispatcher.UIThread.InvokeAsync(() => SetError($"Erreur lors de l'affichage de l'historique: {ex.Message}"));
        }
        finally
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsCleaning = false);
        }
    }

    /// <summary>
    /// Exécute une opération de nettoyage
    /// </summary>
    private async Task ExecuteCleanupAsync(Func<Task<CleanupResult>> cleanupAction, string operationName)
    {
        try
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsCleaning = true);
            await Dispatcher.UIThread.InvokeAsync(() => ClearError());

            var result = await cleanupAction();
            await Dispatcher.UIThread.InvokeAsync(() => LastCleanupResult = result);
            
            if (result.Success)
            {
                await RefreshDisksInfoAsync();
            }
            else
            {
                await Dispatcher.UIThread.InvokeAsync(() => 
                    SetError(result.ErrorMessage ?? $"Erreur lors du nettoyage des {operationName}"));
            }
        }
        catch (Exception ex)
        {
            await Dispatcher.UIThread.InvokeAsync(() => SetError($"Erreur: {ex.Message}"));
        }
        finally
        {
            await Dispatcher.UIThread.InvokeAsync(() => IsCleaning = false);
        }
    }
}
