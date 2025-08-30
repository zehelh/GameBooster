using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml;

namespace GameBooster.UI.Controls;

public partial class SvgIcon : UserControl
{
    public static readonly StyledProperty<string> IconDataProperty =
        AvaloniaProperty.Register<SvgIcon, string>(nameof(IconData));

    public static readonly StyledProperty<Avalonia.Media.Brush> IconColorProperty =
        AvaloniaProperty.Register<SvgIcon, Avalonia.Media.Brush>(nameof(IconColor));

    public static readonly StyledProperty<double> IconSizeProperty =
        AvaloniaProperty.Register<SvgIcon, double>(nameof(IconSize));

    public string IconData
    {
        get => GetValue(IconDataProperty);
        set => SetValue(IconDataProperty, value);
    }

    public Avalonia.Media.Brush IconColor
    {
        get => GetValue(IconColorProperty);
        set => SetValue(IconColorProperty, value);
    }

    public double IconSize
    {
        get => GetValue(IconSizeProperty);
        set => SetValue(IconSizeProperty, value);
    }

    public SvgIcon()
    {
        InitializeComponent();
    }

    private void InitializeComponent()
    {
        AvaloniaXamlLoader.Load(this);
    }
}
