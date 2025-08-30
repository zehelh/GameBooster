using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml;
using Avalonia.Media;
using GameBooster.UI.Services;

namespace GameBooster.UI.Controls;

public partial class HeroIcon : UserControl
{
    public static readonly StyledProperty<string> IconDataProperty =
        AvaloniaProperty.Register<HeroIcon, string>(nameof(IconData));

    public static readonly StyledProperty<string> IconColorProperty =
        AvaloniaProperty.Register<HeroIcon, string>(nameof(IconColor), "#FFFFFF");

    public static readonly StyledProperty<double> IconSizeProperty =
        AvaloniaProperty.Register<HeroIcon, double>(nameof(IconSize));

    public static readonly StyledProperty<string> IconNameProperty =
        AvaloniaProperty.Register<HeroIcon, string>(nameof(IconName));

    public string IconData
    {
        get => GetValue(IconDataProperty);
        set => SetValue(IconDataProperty, value);
    }

    public string IconColor
    {
        get => GetValue(IconColorProperty);
        set => SetValue(IconColorProperty, value);
    }

    public double IconSize
    {
        get => GetValue(IconSizeProperty);
        set => SetValue(IconSizeProperty, value);
    }

    public string IconName
    {
        get => GetValue(IconNameProperty);
        set
        {
            SetValue(IconNameProperty, value);
            LoadIconData();
        }
    }

    public HeroIcon()
    {
        InitializeComponent();
    }

    public HeroIcon(string iconName, double size = 16, string? color = null)
    {
        InitializeComponent();
        IconName = iconName;
        IconSize = size;
        IconColor = color ?? "#FFFFFF";
    }

    private void InitializeComponent()
    {
        AvaloniaXamlLoader.Load(this);
    }

    private void LoadIconData()
    {
        if (!string.IsNullOrEmpty(IconName))
        {
            IconData = IconService.GetIconPathData(IconName);
        }
    }
}
