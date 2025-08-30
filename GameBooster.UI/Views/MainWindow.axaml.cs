using Avalonia.Controls;
using Avalonia.Markup.Xaml;
using GameBooster.UI.ViewModels;

namespace GameBooster.UI.Views;

public partial class MainWindow : Window
{
    // Constructeur parameterless requis pour le XAML loader et design-time
    public MainWindow()
    {
        DebugLogger.LogAction("CONSTRUCTEUR MainWindow() parameterless appelé (design-time)");
        InitializeComponent();
        DebugLogger.LogAction("InitializeComponent() terminé (design-time)");
    }
    
    // Constructeur avec injection de dépendance pour l'exécution
    public MainWindow(MainWindowViewModel viewModel) : this()
    {
        DebugLogger.LogAction("CONSTRUCTEUR MainWindow(viewModel) appelé (runtime DI)");
        DebugLogger.LogProperty("viewModel != null", viewModel != null);
        
        DataContext = viewModel;
        DebugLogger.LogAction("DataContext assigné via DI");
        DebugLogger.LogProperty("DataContext != null", DataContext != null);
        
        if (viewModel != null)
        {
            DebugLogger.LogProperty("ViewModel.Title", viewModel.Title);
            DebugLogger.LogProperty("ViewModel.IsInitializing", viewModel.IsInitializing);
            DebugLogger.LogProperty("ViewModel.MemoryUsagePercentage", viewModel.MemoryUsagePercentage);
        }
    }

    private void InitializeComponent()
    {
        AvaloniaXamlLoader.Load(this);
    }
}
