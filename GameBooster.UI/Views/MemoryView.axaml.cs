using Avalonia.Controls;
using Avalonia.Markup.Xaml;

namespace GameBooster.UI.Views;

public partial class MemoryView : UserControl
{
    public MemoryView()
    {
        InitializeComponent();
    }

    private void InitializeComponent()
    {
        AvaloniaXamlLoader.Load(this);
    }
}
