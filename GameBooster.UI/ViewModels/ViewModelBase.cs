using ReactiveUI;

namespace GameBooster.UI.ViewModels;

/// <summary>
/// Classe de base pour tous les ViewModels
/// </summary>
public abstract class ViewModelBase : ReactiveObject
{
    /// <summary>
    /// Indique si le ViewModel est en cours de chargement
    /// </summary>
    private bool _isLoading;

    public bool IsLoading
    {
        get => _isLoading;
        set => this.RaiseAndSetIfChanged(ref _isLoading, value);
    }

    /// <summary>
    /// Message d'erreur à afficher
    /// </summary>
    private string? _errorMessage;

    public string? ErrorMessage
    {
        get => _errorMessage;
        set => this.RaiseAndSetIfChanged(ref _errorMessage, value);
    }

    /// <summary>
    /// Indique s'il y a une erreur
    /// </summary>
    public bool HasError => !string.IsNullOrEmpty(ErrorMessage);

    /// <summary>
    /// Efface le message d'erreur
    /// </summary>
    protected void ClearError()
    {
        ErrorMessage = null;
    }

    /// <summary>
    /// Définit un message d'erreur
    /// </summary>
    protected void SetError(string message)
    {
        ErrorMessage = message;
    }
}
