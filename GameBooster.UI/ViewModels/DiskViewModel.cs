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
    /// Commande pour nettoyer les fichiers temporaires
    /// </summary>
    public ICommand CleanTempFilesCommand { get; }

    /// <summary>
    /// Commande pour nettoyer le cache navigateur
    /// </summary>
    public ICommand CleanBrowserCacheCommand { get; }

    /// <summary>
    /// Commande pour vider la corbeille
    /// </summary>
    public ICommand EmptyRecycleBinCommand { get; }

    /// <summary>
    /// Commande pour nettoyer les miniatures
    /// </summary>
    public ICommand CleanThumbnailsCommand { get; }

    /// <summary>
    /// Commande pour un nettoyage complet
    /// </summary>
    public ICommand FullCleanupCommand { get; }

    /// <summary>
    /// Commande pour rafraîchir les informations disque
    /// </summary>
    public ICommand RefreshDisksInfoCommand { get; }

    /// <summary>
    /// Nettoie les fichiers temporaires
    /// </summary>
    private async Task CleanTempFilesAsync()
    {
        await ExecuteCleanupAsync(_diskService.CleanTempFilesAsync, "fichiers temporaires");
    }

    /// <summary>
    /// Nettoie le cache navigateur
    /// </summary>
    private async Task CleanBrowserCacheAsync()
    {
        await ExecuteCleanupAsync(_diskService.CleanBrowserCacheAsync, "cache navigateur");
    }

    /// <summary>
    /// Vide la corbeille
    /// </summary>
    private async Task EmptyRecycleBinAsync()
    {
        await ExecuteCleanupAsync(_diskService.EmptyRecycleBinAsync, "corbeille");
    }

    /// <summary>
    /// Nettoie les miniatures
    /// </summary>
    private async Task CleanThumbnailsAsync()
    {
        await ExecuteCleanupAsync(_diskService.CleanThumbnailsAsync, "miniatures");
    }

    /// <summary>
    /// Effectue un nettoyage complet
    /// </summary>
    private async Task FullCleanupAsync()
    {
        await ExecuteCleanupAsync(_diskService.PerformFullCleanupAsync, "nettoyage complet");
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
