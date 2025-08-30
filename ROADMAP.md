# ROADMAP - Migration GameBooster Rust vers C#

## 📋 Vue d'ensemble du projet

**GameBooster** est une application d'optimisation PC complète pour Windows, spécialement conçue pour améliorer les performances de jeu. L'application fournit une interface graphique moderne et intuitive pour surveiller, nettoyer, optimiser et planifier des tâches de maintenance système en temps réel.

### 🎯 Objectif de la migration
Portage complet de l'application Rust vers C# en conservant toutes les fonctionnalités existantes et en améliorant la maintenabilité et l'extensibilité du code.

---

## 🏗️ Architecture technique

### 📊 Structure actuelle (Rust)
```
GameBooster/
├── src/
│   ├── main.rs                 # Point d'entrée principal
│   ├── disk/                   # Nettoyage disque
│   ├── memory/                 # Gestion mémoire RAM
│   ├── network/                # Limitation réseau QoS
│   ├── scheduler/              # Planificateur de tâches
│   ├── services/               # Gestion services Windows
│   ├── ui/                     # Interface utilisateur (egui)
│   └── utils.rs                # Utilitaires
├── assets/                     # Ressources (icônes, images)
├── build.rs                    # Script de build
├── app.rc                      # Ressources Windows
└── app.manifest               # Manifeste d'application
```

### 🎯 Architecture C# proposée
```
GameBooster.CSharp/
├── GameBooster.Core/           # Logique métier
├── GameBooster.Services/       # Services système
├── GameBooster.UI/             # Interface WPF/Avalonia
├── GameBooster.Common/         # Classes partagées
├── GameBooster.Tests/          # Tests unitaires
└── GameBooster.Installer/      # Installateur
```

---

## 🔧 Technologies et dépendances

### 📦 Dépendances Rust actuelles
```toml
# Interface utilisateur
eframe = "0.25"                 # Framework GUI
egui = "0.25"                   # Interface utilisateur
egui_extras = "0.25"            # Extensions egui

# Système et API Windows
windows-sys = "0.59"            # Bindings Windows API
winapi = "0.3"                  # API Windows natives
sysinfo = "0.30"                # Informations système

# Async et threading
tokio = "1"                     # Runtime async
async-process = "2.3"           # Processus asynchrones

# Sérialisation et configuration
serde = "1.0"                   # Sérialisation JSON
serde_json = "1.0"              # JSON

# Utilitaires
anyhow = "1.0"                  # Gestion d'erreurs
tracing = "0.1"                 # Logging
chrono = "0.4"                  # Dates et temps
walkdir = "2.4"                 # Parcours de répertoires
```

### 🎯 Équivalents C# recommandés
```xml
<!-- Interface utilisateur -->
<PackageReference Include="Avalonia" Version="11.0.0" />
<PackageReference Include="Avalonia.Desktop" Version="11.0.0" />
<PackageReference Include="Avalonia.Themes.Fluent" Version="11.0.0" />

<!-- API Windows -->
<PackageReference Include="Microsoft.Windows.SDK.Contracts" Version="10.0.22621.2428" />
<PackageReference Include="System.Management" Version="7.0.0" />
<PackageReference Include="Microsoft.Win32.Registry" Version="5.0.0" />

<!-- Async et threading -->
<PackageReference Include="Microsoft.Extensions.Hosting" Version="7.0.0" />
<PackageReference Include="Microsoft.Extensions.DependencyInjection" Version="7.0.0" />

<!-- Sérialisation -->
<PackageReference Include="Newtonsoft.Json" Version="13.0.3" />
<PackageReference Include="System.Text.Json" Version="7.0.0" />

<!-- Logging -->
<PackageReference Include="Microsoft.Extensions.Logging" Version="7.0.0" />
<PackageReference Include="Serilog" Version="3.0.1" />
<PackageReference Include="Serilog.Sinks.File" Version="5.0.0" />

<!-- Utilitaires -->
<PackageReference Include="System.IO.Abstractions" Version="19.0.0" />
<PackageReference Include="FluentValidation" Version="11.5.0" />
```

---

## 🎨 Interface utilisateur

### 📱 Framework GUI recommandé : **Avalonia UI**

**Justification :**
- Cross-platform (Windows, macOS, Linux)
- Performance native
- Syntaxe similaire à WPF
- Support du thème sombre/clair
- Architecture MVVM moderne

### 🎯 Structure UI C#
```
GameBooster.UI/
├── Views/
│   ├── MainWindow.axaml         # Fenêtre principale
│   ├── MemoryView.axaml         # Onglet mémoire
│   ├── DiskView.axaml           # Onglet disque
│   ├── NetworkView.axaml        # Onglet réseau
│   ├── SchedulerView.axaml      # Onglet planificateur
│   ├── ServicesView.axaml       # Onglet services
│   └── SettingsView.axaml       # Onglet paramètres
├── ViewModels/
│   ├── MainWindowViewModel.cs
│   ├── MemoryViewModel.cs
│   ├── DiskViewModel.cs
│   ├── NetworkViewModel.cs
│   ├── SchedulerViewModel.cs
│   ├── ServicesViewModel.cs
│   └── SettingsViewModel.cs
├── Models/
│   ├── ProcessInfo.cs
│   ├── MemoryInfo.cs
│   ├── DiskInfo.cs
│   └── NetworkInfo.cs
└── Controls/
    ├── ProgressRing.axaml
    ├── StatusIndicator.axaml
    └── CustomButtons.axaml
```

---

## 💾 Gestion de la mémoire

### 🔧 Fonctionnalités à migrer
- Surveillance temps réel de la RAM
- Nettoyage manuel et automatique
- Suivi des statistiques d'utilisation
- Libération du working set des processus

### 🎯 Implémentation C#
```csharp
// GameBooster.Core/Memory/
public class MemoryManager
{
    public async Task<MemoryInfo> GetMemoryInfoAsync()
    public async Task<CleaningResult> CleanMemoryAsync()
    public async Task<List<ProcessMemoryInfo>> GetProcessMemoryInfoAsync()
    public async Task<long> GetAvailableMemoryAsync()
}

public class MemoryInfo
{
    public long TotalPhysical { get; set; }
    public long AvailablePhysical { get; set; }
    public long UsedPhysical { get; set; }
    public double UsagePercentage { get; set; }
    public long TotalPageFile { get; set; }
    public long AvailablePageFile { get; set; }
}

public class CleaningResult
{
    public DateTime StartTime { get; set; }
    public DateTime EndTime { get; set; }
    public List<ProcessCleaned> Processes { get; set; }
    public long TotalMemoryBefore { get; set; }
    public long TotalMemoryAfter { get; set; }
    public long TotalFreed { get; set; }
    public bool HasError { get; set; }
    public string ErrorMessage { get; set; }
}
```

### 🔧 API Windows utilisées
```csharp
// P/Invoke pour les API Windows
[DllImport("kernel32.dll")]
public static extern bool GlobalMemoryStatusEx(ref MEMORYSTATUSEX lpBuffer);

[DllImport("psapi.dll")]
public static extern bool EmptyWorkingSet(IntPtr hProcess);

[DllImport("kernel32.dll")]
public static extern IntPtr OpenProcess(ProcessAccessFlags dwDesiredAccess, bool bInheritHandle, uint dwProcessId);
```

---

## 🗂️ Nettoyage de disque

### 🔧 Fonctionnalités à migrer
- Nettoyage des fichiers temporaires (système, utilisateur, navigateurs)
- Cache des navigateurs (Chrome, Firefox, Edge, Opera)
- Miniatures Windows
- Corbeille
- Cache système (expérimental)

### 🎯 Implémentation C#
```csharp
// GameBooster.Core/Disk/
public class DiskCleaner
{
    public async Task<DiskScanResult> ScanDiskAsync(DiskCleaningOptions options)
    public async Task<DiskCleanResult> CleanDiskAsync(DiskCleaningOptions options)
    public async Task<long> GetTempFilesSizeAsync()
    public async Task<long> GetBrowserCacheSizeAsync()
    public async Task<long> GetThumbnailsSizeAsync()
    public async Task<long> GetRecycleBinSizeAsync()
}

public class DiskCleaningOptions
{
    public bool CleanTempFiles { get; set; } = true;
    public bool CleanBrowserCache { get; set; } = true;
    public bool CleanThumbnails { get; set; } = true;
    public bool CleanRecycleBin { get; set; } = false;
    public bool CleanSystemCache { get; set; } = false;
}

public class DiskScanResult
{
    public long TotalSpaceToFree { get; set; }
    public long TempFilesSize { get; set; }
    public long CacheSize { get; set; }
    public long ThumbnailsSize { get; set; }
    public long RecycleBinSize { get; set; }
    public List<string> Errors { get; set; }
    public TimeSpan Duration { get; set; }
}

public class DiskCleanResult
{
    public DateTime StartTime { get; set; }
    public DateTime EndTime { get; set; }
    public long TotalSpaceFreed { get; set; }
    public long TempFilesFreed { get; set; }
    public long CacheFreed { get; set; }
    public long ThumbnailsFreed { get; set; }
    public long RecycleBinFreed { get; set; }
    public List<string> Errors { get; set; }
    public TimeSpan Duration { get; set; }
}
```

### 🌐 Nettoyage cache navigateurs
```csharp
public class BrowserCacheCleaner
{
    private readonly string[] _chromePaths = {
        @"%LOCALAPPDATA%\Google\Chrome\User Data\Default\Cache",
        @"%LOCALAPPDATA%\Google\Chrome\User Data\Default\Code Cache"
    };
    
    private readonly string[] _firefoxPaths = {
        @"%LOCALAPPDATA%\Mozilla\Firefox\Profiles"
    };
    
    private readonly string[] _edgePaths = {
        @"%LOCALAPPDATA%\Microsoft\Edge\User Data\Default\Cache"
    };
    
    public async Task<long> CleanBrowserCacheAsync()
    public async Task<long> CleanChromeCacheAsync()
    public async Task<long> CleanFirefoxCacheAsync()
    public async Task<long> CleanEdgeCacheAsync()
}
```

---

## ⏰ Planificateur de tâches

### 🔧 Fonctionnalités à migrer
- Types de tâches : Nettoyage RAM, Nettoyage disque, Toggle Defender
- Options de planification : Au démarrage, Intervalle, Quotidien, Hebdomadaire, Sur condition
- Gestion CRUD complète
- Statistiques de succès/échec
- Persistance automatique

### 🎯 Implémentation C#
```csharp
// GameBooster.Core/Scheduler/
public class TaskScheduler
{
    public async Task<List<ScheduledTask>> LoadTasksAsync()
    public async Task SaveTasksAsync(List<ScheduledTask> tasks)
    public async Task AddTaskAsync(ScheduledTask task)
    public async Task RemoveTaskAsync(string taskId)
    public async Task UpdateTaskAsync(ScheduledTask task)
    public async Task<List<ScheduledTask>> GetPendingTasksAsync()
    public async Task StartAsync()
    public async Task StopAsync()
}

public enum TaskType
{
    CleanRam,
    CleanDisk,
    DefenderToggle
}

public enum ScheduleRule
{
    OnStartup,
    Interval,
    Daily,
    Weekly,
    OnCondition
}

public class ScheduledTask
{
    public string Id { get; set; }
    public string Name { get; set; }
    public string Description { get; set; }
    public TaskType TaskType { get; set; }
    public ScheduleRule Schedule { get; set; }
    public bool Enabled { get; set; }
    public DateTime? LastRun { get; set; }
    public DateTime? NextRun { get; set; }
    public int RunCount { get; set; }
    public int SuccessCount { get; set; }
    public string LastError { get; set; }
    
    // Paramètres spécifiques aux tâches
    public int? RamThresholdPercentage { get; set; }
    public long? DiskThresholdMb { get; set; }
    public bool? DefenderEnable { get; set; }
    
    // Paramètres de planification
    public int? IntervalMinutes { get; set; }
    public TimeSpan? DailyTime { get; set; }
    public DayOfWeek? WeeklyDay { get; set; }
    public TimeSpan? WeeklyTime { get; set; }
}

public class TaskEngine
{
    private readonly IServiceProvider _serviceProvider;
    private readonly ILogger<TaskEngine> _logger;
    private readonly Timer _timer;
    
    public async Task StartAsync()
    public async Task StopAsync()
    public async Task ExecuteTaskAsync(ScheduledTask task)
    public async Task CheckAndExecutePendingTasksAsync()
}
```

### 💾 Persistance des données
```csharp
// GameBooster.Core/Storage/
public class TaskStorage
{
    private readonly string _configPath;
    
    public async Task<List<ScheduledTask>> LoadTasksAsync()
    public async Task SaveTasksAsync(List<ScheduledTask> tasks)
    public async Task BackupTasksAsync()
    public async Task RestoreTasksAsync()
}

// Configuration JSON
{
  "tasks": [
    {
      "id": "ram_cleanup_threshold",
      "name": "Nettoyage RAM (Seuil)",
      "description": "Nettoyer automatiquement la RAM quand l'utilisation dépasse 85%",
      "taskType": "CleanRam",
      "schedule": "OnCondition",
      "enabled": true,
      "ramThresholdPercentage": 85,
      "lastRun": "2024-01-15T10:30:00",
      "runCount": 5,
      "successCount": 5
    }
  ],
  "settings": {
    "autoStart": false,
    "startMinimized": false,
    "autoStartScheduler": false,
    "startupDelay": 0
  }
}
```

---

## 🛡️ Gestion des services Windows

### 🔧 Fonctionnalités à migrer
- Toggle Windows Defender (temporaire/permanent)
- Surveillance du statut des services
- Gestion des privilèges administrateur

### 🎯 Implémentation C#
```csharp
// GameBooster.Core/Services/
public class WindowsDefenderManager
{
    public async Task<DefenderStatus> GetStatusAsync()
    public async Task<bool> DisableAsync()
    public async Task<bool> EnableAsync()
    public async Task<bool> DisableSafelyAsync()
    public async Task<bool> IsElevatedAsync()
}

public class DefenderStatus
{
    public bool RealTimeProtection { get; set; }
    public bool CloudProtection { get; set; }
    public bool AutomaticSampleSubmission { get; set; }
    public bool TamperProtection { get; set; }
    public DateTime LastCheck { get; set; }
    public string StatusText { get; set; }
    public List<string> LastOperationResults { get; set; }
}

public class ServiceManager
{
    public async Task<bool> IsServiceRunningAsync(string serviceName)
    public async Task<string> GetServiceStatusAsync(string serviceName)
    public async Task<bool> StartServiceAsync(string serviceName)
    public async Task<bool> StopServiceAsync(string serviceName)
    public async Task<bool> DisableServiceAsync(string serviceName)
    public async Task<bool> EnableServiceAsync(string serviceName)
}
```

### 🔧 API Windows utilisées
```csharp
// P/Invoke pour les services Windows
[DllImport("advapi32.dll")]
public static extern IntPtr OpenSCManager(string lpMachineName, string lpDatabaseName, uint dwDesiredAccess);

[DllImport("advapi32.dll")]
public static extern IntPtr OpenService(IntPtr hSCManager, string lpServiceName, uint dwDesiredAccess);

[DllImport("advapi32.dll")]
public static extern bool ControlService(IntPtr hService, uint dwControl, ref SERVICE_STATUS lpServiceStatus);

// Registry pour Windows Defender
public class DefenderRegistryManager
{
    private const string DEFENDER_REGISTRY_PATH = @"SOFTWARE\Microsoft\Windows Defender";
    private const string POLICY_REGISTRY_PATH = @"SOFTWARE\Policies\Microsoft\Windows Defender\Real-Time Protection";
    
    public async Task<bool> SetDefenderPolicyAsync(string policyName, int value)
    public async Task<int?> GetDefenderPolicyAsync(string policyName)
    public async Task<bool> DeleteDefenderPolicyAsync(string policyName)
}
```

---

## 🌐 Gestion réseau et QoS

### 🔧 Fonctionnalités à migrer
- Surveillance temps réel des processus réseau
- Limitation de bande passante par processus
- Gestion des politiques QoS Windows
- Statistiques de trafic en temps réel

### 🎯 Implémentation C#
```csharp
// GameBooster.Core/Network/
public class NetworkLimiter
{
    public async Task<List<NetworkProcessInfo>> ScanNetworkProcessesAsync()
    public async Task<bool> SetProcessSpeedLimitAsync(int pid, int limitKbps)
    public async Task<bool> RemoveProcessLimitAsync(int pid)
    public async Task<List<QosPolicyInfo>> VerifyQosPoliciesAsync()
    public async Task<bool> ClearAllLimitsAsync()
    public async Task<NetworkStats> GetNetworkStatsAsync()
}

public class NetworkProcessInfo
{
    public int Pid { get; set; }
    public string Name { get; set; }
    public long BytesSent { get; set; }
    public long BytesReceived { get; set; }
    public long PacketsSent { get; set; }
    public long PacketsReceived { get; set; }
    public bool IsLimited { get; set; }
    public int? SpeedLimit { get; set; }
    public int Connections { get; set; }
    public long CurrentUploadSpeed { get; set; }
    public long CurrentDownloadSpeed { get; set; }
}

public class QosPolicyInfo
{
    public string Name { get; set; }
    public string AppName { get; set; }
    public long ThrottleBits { get; set; }
    public bool IsRegistryOnly { get; set; }
}

public class NetworkStats
{
    public long TotalUploadBytes { get; set; }
    public long TotalDownloadBytes { get; set; }
    public int TotalProcesses { get; set; }
    public int LimitedProcessesCount { get; set; }
}
```

### 🔧 PowerShell pour QoS
```csharp
public class QosManager
{
    public async Task<bool> CreateQosPolicyAsync(string policyName, string appName, long throttleBits)
    {
        var script = $@"
            $ErrorActionPreference = 'Stop'
            try {{
                Remove-NetQosPolicy -Name '{policyName}' -Confirm:$false -ErrorAction SilentlyContinue
                New-NetQosPolicy -Name '{policyName}' -AppPathNameMatchCondition '{appName}' -ThrottleRateActionBitsPerSecond {throttleBits} -Confirm:$false
                Write-Output 'SUCCESS'
            }}
            catch {{
                Write-Error $_.Exception.Message
                exit 1
            }}
        ";
        
        return await ExecutePowerShellScriptAsync(script);
    }
    
    public async Task<bool> RemoveQosPolicyAsync(string policyName)
    {
        var script = $@"
            try {{
                Remove-NetQosPolicy -Name '{policyName}' -Confirm:$false
                Write-Output 'SUCCESS'
            }}
            catch {{
                Write-Error $_.Exception.Message
                exit 1
            }}
        ";
        
        return await ExecutePowerShellScriptAsync(script);
    }
    
    private async Task<bool> ExecutePowerShellScriptAsync(string script)
    {
        var startInfo = new ProcessStartInfo
        {
            FileName = "powershell.exe",
            Arguments = $"-NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -Command \"{script}\"",
            UseShellExecute = false,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            CreateNoWindow = true
        };
        
        using var process = new Process { StartInfo = startInfo };
        process.Start();
        await process.WaitForExitAsync();
        
        return process.ExitCode == 0;
    }
}
```

---

## 🎨 Interface utilisateur Avalonia

### 📱 Structure des vues
```xml
<!-- MainWindow.axaml -->
<Window xmlns="https://github.com/avaloniaui"
        xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
        x:Class="GameBooster.UI.Views.MainWindow"
        Title="GameBooster - Optimiseur de Performance"
        Width="1200" Height="700"
        MinWidth="900" MinHeight="500"
        Icon="/Assets/logo.ico">
    
    <Grid>
        <Grid.RowDefinitions>
            <RowDefinition Height="Auto"/>
            <RowDefinition Height="*"/>
        </Grid.RowDefinitions>
        
        <!-- Header avec logo -->
        <Border Grid.Row="0" Background="{DynamicResource SystemAccentColor}">
            <StackPanel Orientation="Horizontal" Margin="20,10">
                <Image Source="/Assets/logo.png" Width="32" Height="32"/>
                <TextBlock Text="GameBooster" FontSize="24" FontWeight="Bold" 
                          VerticalAlignment="Center" Margin="10,0,0,0"/>
            </StackPanel>
        </Border>
        
        <!-- Contenu principal avec onglets -->
        <TabControl Grid.Row="1" Name="MainTabControl">
            <TabItem Header="💾 Mémoire">
                <local:MemoryView/>
            </TabItem>
            <TabItem Header="🗂️ Disque">
                <local:DiskView/>
            </TabItem>
            <TabItem Header="🌐 Réseau">
                <local:NetworkView/>
            </TabItem>
            <TabItem Header="⏰ Planificateur">
                <local:SchedulerView/>
            </TabItem>
            <TabItem Header="🛡️ Services">
                <local:ServicesView/>
            </TabItem>
            <TabItem Header="⚙️ Paramètres">
                <local:SettingsView/>
            </TabItem>
        </TabControl>
    </Grid>
</Window>
```

### 🎯 ViewModels avec MVVM
```csharp
// GameBooster.UI/ViewModels/MainWindowViewModel.cs
public class MainWindowViewModel : ViewModelBase
{
    private readonly IServiceProvider _serviceProvider;
    private readonly ILogger<MainWindowViewModel> _logger;
    
    public MemoryViewModel MemoryViewModel { get; }
    public DiskViewModel DiskViewModel { get; }
    public NetworkViewModel NetworkViewModel { get; }
    public SchedulerViewModel SchedulerViewModel { get; }
    public ServicesViewModel ServicesViewModel { get; }
    public SettingsViewModel SettingsViewModel { get; }
    
    public MainWindowViewModel(IServiceProvider serviceProvider)
    {
        _serviceProvider = serviceProvider;
        _logger = serviceProvider.GetRequiredService<ILogger<MainWindowViewModel>>();
        
        MemoryViewModel = serviceProvider.GetRequiredService<MemoryViewModel>();
        DiskViewModel = serviceProvider.GetRequiredService<DiskViewModel>();
        NetworkViewModel = serviceProvider.GetRequiredService<NetworkViewModel>();
        SchedulerViewModel = serviceProvider.GetRequiredService<SchedulerViewModel>();
        ServicesViewModel = serviceProvider.GetRequiredService<ServicesViewModel>();
        SettingsViewModel = serviceProvider.GetRequiredService<SettingsViewModel>();
    }
}

// GameBooster.UI/ViewModels/MemoryViewModel.cs
public class MemoryViewModel : ViewModelBase
{
    private readonly IMemoryManager _memoryManager;
    private readonly ILogger<MemoryViewModel> _logger;
    
    private MemoryInfo _memoryInfo;
    private bool _isCleaning;
    private CleaningResult _lastCleaningResult;
    
    public MemoryInfo MemoryInfo
    {
        get => _memoryInfo;
        set => SetProperty(ref _memoryInfo, value);
    }
    
    public bool IsCleaning
    {
        get => _isCleaning;
        set => SetProperty(ref _isCleaning, value);
    }
    
    public CleaningResult LastCleaningResult
    {
        get => _lastCleaningResult;
        set => SetProperty(ref _lastCleaningResult, value);
    }
    
    public ICommand CleanMemoryCommand { get; }
    public ICommand RefreshMemoryInfoCommand { get; }
    
    public MemoryViewModel(IMemoryManager memoryManager, ILogger<MemoryViewModel> logger)
    {
        _memoryManager = memoryManager;
        _logger = logger;
        
        CleanMemoryCommand = new AsyncRelayCommand(CleanMemoryAsync, CanCleanMemory);
        RefreshMemoryInfoCommand = new AsyncRelayCommand(RefreshMemoryInfoAsync);
        
        // Actualisation automatique toutes les 2 secondes
        var timer = new Timer(async _ => await RefreshMemoryInfoAsync(), null, TimeSpan.Zero, TimeSpan.FromSeconds(2));
    }
    
    private async Task CleanMemoryAsync()
    {
        try
        {
            IsCleaning = true;
            LastCleaningResult = await _memoryManager.CleanMemoryAsync();
            await RefreshMemoryInfoAsync();
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors du nettoyage de la mémoire");
        }
        finally
        {
            IsCleaning = false;
        }
    }
    
    private bool CanCleanMemory() => !IsCleaning;
    
    private async Task RefreshMemoryInfoAsync()
    {
        try
        {
            MemoryInfo = await _memoryManager.GetMemoryInfoAsync();
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors de l'actualisation des informations mémoire");
        }
    }
}
```

---

## 🔧 Configuration et injection de dépendances

### 📦 Configuration avec Microsoft.Extensions
```csharp
// Program.cs
public class Program
{
    [STAThread]
    public static void Main(string[] args)
    {
        var host = CreateHostBuilder(args).Build();
        
        var app = new App();
        app.InitializeWithHost(host);
        app.Run();
    }
    
    public static IHostBuilder CreateHostBuilder(string[] args) =>
        Host.CreateDefaultBuilder(args)
            .ConfigureServices((context, services) =>
            {
                // Services système
                services.AddSingleton<IMemoryManager, MemoryManager>();
                services.AddSingleton<IDiskCleaner, DiskCleaner>();
                services.AddSingleton<INetworkLimiter, NetworkLimiter>();
                services.AddSingleton<ITaskScheduler, TaskScheduler>();
                services.AddSingleton<IWindowsDefenderManager, WindowsDefenderManager>();
                services.AddSingleton<IServiceManager, ServiceManager>();
                
                // ViewModels
                services.AddTransient<MainWindowViewModel>();
                services.AddTransient<MemoryViewModel>();
                services.AddTransient<DiskViewModel>();
                services.AddTransient<NetworkViewModel>();
                services.AddTransient<SchedulerViewModel>();
                services.AddTransient<ServicesViewModel>();
                services.AddTransient<SettingsViewModel>();
                
                // Configuration
                services.Configure<AppSettings>(context.Configuration.GetSection("AppSettings"));
            })
            .ConfigureLogging((context, logging) =>
            {
                logging.ClearProviders();
                logging.AddSerilog(new LoggerConfiguration()
                    .WriteTo.File("logs/gamebooster.log", rollingInterval: RollingInterval.Day)
                    .WriteTo.Console()
                    .CreateLogger());
            });
    }
}

// appsettings.json
{
  "AppSettings": {
    "AutoStartWithWindows": false,
    "StartMinimized": false,
    "AutoStartScheduler": false,
    "StartupDelay": 0,
    "Theme": "Dark",
    "Language": "fr-FR"
  },
  "Logging": {
    "LogLevel": {
      "Default": "Information",
      "Microsoft": "Warning",
      "Microsoft.Hosting.Lifetime": "Information"
    }
  }
}
```

---

## 🚀 Démarrage automatique Windows

### 🔧 Implémentation du démarrage automatique
```csharp
// GameBooster.Core/Startup/
public class StartupManager
{
    private const string REGISTRY_KEY = @"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
    private const string APP_NAME = "GameBooster";
    
    public async Task<bool> EnableAutoStartAsync()
    {
        try
        {
            var exePath = Process.GetCurrentProcess().MainModule.FileName;
            using var key = Registry.CurrentUser.OpenSubKey(REGISTRY_KEY, true);
            key?.SetValue(APP_NAME, $"\"{exePath}\" --minimized");
            return true;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors de l'activation du démarrage automatique");
            return false;
        }
    }
    
    public async Task<bool> DisableAutoStartAsync()
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(REGISTRY_KEY, true);
            key?.DeleteValue(APP_NAME, false);
            return true;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Erreur lors de la désactivation du démarrage automatique");
            return false;
        }
    }
    
    public async Task<bool> IsAutoStartEnabledAsync()
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(REGISTRY_KEY);
            return key?.GetValue(APP_NAME) != null;
        }
        catch
        {
            return false;
        }
    }
}
```

---

## 📦 Gestion des ressources et build

### 🔧 Fichiers de ressources
```xml
<!-- GameBooster.UI/GameBooster.UI.csproj -->
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>WinExe</OutputType>
    <TargetFramework>net7.0-windows</TargetFramework>
    <UseWPF>false</UseWPF>
    <UseWindowsForms>false</UseWindowsForms>
    <ApplicationIcon>Assets\logo.ico</ApplicationIcon>
    <AssemblyTitle>GameBooster</AssemblyTitle>
    <AssemblyDescription>Optimiseur de performance PC pour Windows</AssemblyDescription>
    <AssemblyCompany>GameBooster Team</AssemblyCompany>
    <AssemblyProduct>GameBooster</AssemblyProduct>
    <AssemblyVersion>1.1.0.0</AssemblyVersion>
    <FileVersion>1.1.0.0</FileVersion>
  </PropertyGroup>
  
  <ItemGroup>
    <AvaloniaResource Include="Assets\**" />
  </ItemGroup>
  
  <ItemGroup>
    <PackageReference Include="Avalonia" Version="11.0.0" />
    <PackageReference Include="Avalonia.Desktop" Version="11.0.0" />
    <PackageReference Include="Avalonia.Themes.Fluent" Version="11.0.0" />
    <PackageReference Include="Avalonia.Fonts.Inter" Version="11.0.0" />
  </ItemGroup>
  
  <ItemGroup>
    <ProjectReference Include="..\GameBooster.Core\GameBooster.Core.csproj" />
    <ProjectReference Include="..\GameBooster.Services\GameBooster.Services.csproj" />
  </ItemGroup>
</Project>
```

### 🔧 Manifeste d'application
```xml
<!-- app.manifest -->
<?xml version="1.0" encoding="utf-8"?>
<assembly manifestVersion="1.0" xmlns="urn:schemas-microsoft-com:asm.v1">
  <assemblyIdentity version="1.1.0.0" name="GameBooster"/>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v2">
    <security>
      <requestedPrivileges xmlns="urn:schemas-microsoft-com:asm.v3">
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}" />
      <supportedOS Id="{1f676c76-80e1-4239-95bb-83d0f6d0da78}" />
      <supportedOS Id="{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}" />
      <supportedOS Id="{35138b9a-5d96-4fbd-8e2d-a2440225f93a}" />
    </application>
  </compatibility>
</assembly>
```

---

## 🧪 Tests unitaires

### 📦 Structure des tests
```csharp
// GameBooster.Tests/MemoryManagerTests.cs
public class MemoryManagerTests
{
    private readonly IMemoryManager _memoryManager;
    private readonly Mock<ILogger<MemoryManager>> _loggerMock;
    
    public MemoryManagerTests()
    {
        _loggerMock = new Mock<ILogger<MemoryManager>>();
        _memoryManager = new MemoryManager(_loggerMock.Object);
    }
    
    [Fact]
    public async Task GetMemoryInfoAsync_ShouldReturnValidMemoryInfo()
    {
        // Arrange
        
        // Act
        var result = await _memoryManager.GetMemoryInfoAsync();
        
        // Assert
        Assert.NotNull(result);
        Assert.True(result.TotalPhysical > 0);
        Assert.True(result.AvailablePhysical >= 0);
        Assert.True(result.UsagePercentage >= 0 && result.UsagePercentage <= 100);
    }
    
    [Fact]
    public async Task CleanMemoryAsync_ShouldReturnCleaningResult()
    {
        // Arrange
        
        // Act
        var result = await _memoryManager.CleanMemoryAsync();
        
        // Assert
        Assert.NotNull(result);
        Assert.True(result.StartTime <= result.EndTime);
        Assert.NotNull(result.Processes);
    }
}

// GameBooster.Tests/DiskCleanerTests.cs
public class DiskCleanerTests
{
    [Fact]
    public async Task ScanDiskAsync_ShouldReturnValidScanResult()
    {
        // Arrange
        var options = new DiskCleaningOptions
        {
            CleanTempFiles = true,
            CleanBrowserCache = true,
            CleanThumbnails = false,
            CleanRecycleBin = false
        };
        
        // Act
        var result = await _diskCleaner.ScanDiskAsync(options);
        
        // Assert
        Assert.NotNull(result);
        Assert.True(result.TotalSpaceToFree >= 0);
        Assert.NotNull(result.Errors);
    }
}
```

---

## 📋 Plan de migration étape par étape

### 🎯 Phase 1 : Infrastructure de base (2-3 semaines)
1. **Création de la solution C#**
   - Structure des projets
   - Configuration des dépendances
   - Setup de l'injection de dépendances

2. **Interface utilisateur de base**
   - Fenêtre principale Avalonia
   - Navigation par onglets
   - Thème sombre/clair

3. **Logging et configuration**
   - Setup Serilog
   - Configuration appsettings.json
   - Gestion des erreurs

### 🎯 Phase 2 : Fonctionnalités système (3-4 semaines)
1. **Gestion de la mémoire**
   - Surveillance temps réel
   - Nettoyage manuel
   - Statistiques d'utilisation

2. **Nettoyage de disque**
   - Fichiers temporaires
   - Cache navigateurs
   - Miniatures Windows
   - Corbeille

3. **Services Windows**
   - Gestion Windows Defender
   - Surveillance des services
   - Privilèges administrateur

### 🎯 Phase 3 : Fonctionnalités avancées (3-4 semaines)
1. **Planificateur de tâches**
   - Types de tâches
   - Options de planification
   - Persistance des données
   - Moteur d'exécution

2. **Gestion réseau**
   - Surveillance des processus
   - Limitation QoS
   - Statistiques temps réel

3. **Démarrage automatique**
   - Intégration Windows
   - Gestion du registre

### 🎯 Phase 4 : Finalisation (2-3 semaines)
1. **Tests et débogage**
   - Tests unitaires
   - Tests d'intégration
   - Tests de performance

2. **Optimisations**
   - Performance de l'interface
   - Gestion mémoire
   - Optimisations système

3. **Packaging et déploiement**
   - Installateur
   - Mise à jour automatique
   - Documentation utilisateur

---

## 🔧 Considérations techniques importantes

### ⚠️ Points d'attention
1. **Privilèges administrateur** : L'application nécessite des droits administrateur pour certaines fonctionnalités
2. **Compatibilité Windows** : Tester sur Windows 10/11 avec différentes versions
3. **Performance** : Optimiser les opérations système pour éviter le blocage de l'interface
4. **Sécurité** : Validation des entrées utilisateur et gestion sécurisée des API Windows

### 🎯 Améliorations possibles
1. **Architecture modulaire** : Séparation claire des responsabilités
2. **Tests automatisés** : Couverture de tests complète
3. **Internationalisation** : Support multi-langues
4. **Plugins** : Architecture extensible pour de nouvelles fonctionnalités
5. **Cloud sync** : Synchronisation des configurations entre appareils

### 📊 Métriques de succès
- **Performance** : Temps de réponse < 100ms pour les opérations UI
- **Fiabilité** : 99.9% de disponibilité des fonctionnalités système
- **Maintenabilité** : Code coverage > 80%
- **Expérience utilisateur** : Interface fluide et intuitive

---

## 🎯 Conclusion

Cette roadmap fournit un plan détaillé pour migrer GameBooster de Rust vers C# en conservant toutes les fonctionnalités existantes. L'utilisation d'Avalonia UI, de l'injection de dépendances Microsoft et des API Windows natives permettra de créer une application moderne, performante et maintenable.

La migration se fera en phases progressives, permettant de valider chaque étape avant de passer à la suivante. L'architecture proposée offre une base solide pour les évolutions futures de l'application.
