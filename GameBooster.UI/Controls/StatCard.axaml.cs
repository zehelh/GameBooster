using Avalonia.Controls;
using Avalonia;
using Avalonia.Media;
using Avalonia.Markup.Xaml;

namespace GameBooster.UI.Controls;

public partial class StatCard : UserControl
{
    public static readonly StyledProperty<string> IconProperty =
        AvaloniaProperty.Register<StatCard, string>(nameof(Icon), "📊");

    public static readonly StyledProperty<string> LabelProperty =
        AvaloniaProperty.Register<StatCard, string>(nameof(Label), "Statistic");

    public static readonly StyledProperty<string> ValueProperty =
        AvaloniaProperty.Register<StatCard, string>(nameof(Value), "0");

    public static readonly StyledProperty<IBrush> ValueColorProperty =
        AvaloniaProperty.Register<StatCard, IBrush>(nameof(ValueColor), Brushes.White);

    public static readonly StyledProperty<string?> SubTextProperty =
        AvaloniaProperty.Register<StatCard, string?>(nameof(SubText));

    public static readonly StyledProperty<bool> ShowProgressProperty =
        AvaloniaProperty.Register<StatCard, bool>(nameof(ShowProgress), false);

    public static readonly StyledProperty<double> ProgressValueProperty =
        AvaloniaProperty.Register<StatCard, double>(nameof(ProgressValue), 0);

    public static readonly StyledProperty<IBrush> ProgressColorProperty =
        AvaloniaProperty.Register<StatCard, IBrush>(nameof(ProgressColor), Brush.Parse("#00D4FF"));

    public string Icon
    {
        get => GetValue(IconProperty);
        set => SetValue(IconProperty, value);
    }

    public string Label
    {
        get => GetValue(LabelProperty);
        set => SetValue(LabelProperty, value);
    }

    public string Value
    {
        get => GetValue(ValueProperty);
        set => SetValue(ValueProperty, value);
    }

    public IBrush ValueColor
    {
        get => GetValue(ValueColorProperty);
        set => SetValue(ValueColorProperty, value);
    }

    public string? SubText
    {
        get => GetValue(SubTextProperty);
        set => SetValue(SubTextProperty, value);
    }

    public bool ShowProgress
    {
        get => GetValue(ShowProgressProperty);
        set => SetValue(ShowProgressProperty, value);
    }

    public double ProgressValue
    {
        get => GetValue(ProgressValueProperty);
        set => SetValue(ProgressValueProperty, value);
    }

    public IBrush ProgressColor
    {
        get => GetValue(ProgressColorProperty);
        set => SetValue(ProgressColorProperty, value);
    }

    public StatCard()
    {
        InitializeComponent();
    }

    private void InitializeComponent()
    {
        AvaloniaXamlLoader.Load(this);
    }
}
