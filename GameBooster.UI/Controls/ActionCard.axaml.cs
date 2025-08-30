using Avalonia.Controls;
using Avalonia;
using System.Windows.Input;
using GameBooster.Common;

namespace GameBooster.UI.Controls;

public partial class ActionCard : UserControl
{
    public static readonly StyledProperty<string> IconProperty =
        AvaloniaProperty.Register<ActionCard, string>(nameof(Icon), "🔧");

    public static readonly StyledProperty<string> TitleProperty =
        AvaloniaProperty.Register<ActionCard, string>(nameof(Title), "Action");

    public static readonly StyledProperty<string> DescriptionProperty =
        AvaloniaProperty.Register<ActionCard, string>(nameof(Description), "Description");

    public static readonly StyledProperty<string> ActionTextProperty =
        AvaloniaProperty.Register<ActionCard, string>(nameof(ActionText), "Exécuter");

    public static readonly StyledProperty<ICommand?> ActionCommandProperty =
        AvaloniaProperty.Register<ActionCard, ICommand?>(nameof(ActionCommand));

    public static readonly StyledProperty<bool> IsLoadingProperty =
        AvaloniaProperty.Register<ActionCard, bool>(nameof(IsLoading), false);

    public static readonly StyledProperty<string> LoadingTextProperty =
        AvaloniaProperty.Register<ActionCard, string>(nameof(LoadingText), "En cours...");

    public static readonly StyledProperty<string?> LastResultProperty =
        AvaloniaProperty.Register<ActionCard, string?>(nameof(LastResult));

    public static readonly new StyledProperty<object?> ContentProperty =
        AvaloniaProperty.Register<ActionCard, object?>(nameof(Content));

    public string Icon
    {
        get => GetValue(IconProperty);
        set => SetValue(IconProperty, value);
    }

    public string Title
    {
        get => GetValue(TitleProperty);
        set => SetValue(TitleProperty, value);
    }

    public string Description
    {
        get => GetValue(DescriptionProperty);
        set => SetValue(DescriptionProperty, value);
    }

    public string ActionText
    {
        get => GetValue(ActionTextProperty);
        set => SetValue(ActionTextProperty, value);
    }

    public ICommand? ActionCommand
    {
        get => GetValue(ActionCommandProperty);
        set => SetValue(ActionCommandProperty, value);
    }

    public bool IsLoading
    {
        get => GetValue(IsLoadingProperty);
        set 
        { 
            var oldValue = GetValue(IsLoadingProperty);
            DebugLogger.LogProperty($"ActionCard[{Title}].IsLoading AVANT", oldValue);
            SetValue(IsLoadingProperty, value);
            DebugLogger.LogProperty($"ActionCard[{Title}].IsLoading APRÈS", value);
        }
    }

    public string LoadingText
    {
        get => GetValue(LoadingTextProperty);
        set => SetValue(LoadingTextProperty, value);
    }

    public string? LastResult
    {
        get => GetValue(LastResultProperty);
        set => SetValue(LastResultProperty, value);
    }

    public new object? Content
    {
        get => GetValue(ContentProperty);
        set => SetValue(ContentProperty, value);
    }

    public ActionCard()
    {
        DebugLogger.LogAction("CONSTRUCTEUR ActionCard");
        InitializeComponent();
        DebugLogger.LogAction("ActionCard InitializeComponent terminé");
    }
}
