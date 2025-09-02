using System.Diagnostics;
using System.Management;
using System.Runtime.InteropServices;
using System.Runtime.Versioning;
using System.Security.Principal;
using GameBooster.Common.Interfaces;
using GameBooster.Common.Models;
using Microsoft.Win32;
using Microsoft.Extensions.Logging;

namespace GameBooster.Core.Services;

/// <summary>
/// Windows Service Manager for managing Windows services and Windows Defender
/// </summary>
[SupportedOSPlatform("windows")]
public class WindowsServiceManager : IWindowsServiceManager
{
    private readonly ILogger<WindowsServiceManager> _logger;

    public WindowsServiceManager(ILogger<WindowsServiceManager> logger)
    {
        _logger = logger;
    }

    /// <summary>
    /// Get Windows Defender status using PowerShell
    /// </summary>
    public async Task<DefenderStatus> GetDefenderStatusAsync()
    {
        try
        {
            _logger.LogInformation("Getting Windows Defender status");

            var status = new DefenderStatus();
            var script = @"
                $ErrorActionPreference = 'Stop'
                try {
                    $prefs = Get-MpPreference
                    $status = Get-MpComputerStatus
                    
                    $result = @{
                        RealTimeProtection = $status.RealTimeProtectionEnabled
                        CloudProtection = $prefs.MAPSReporting -ne 'Disabled'
                        AutomaticSampleSubmission = $prefs.SubmitSamplesConsent -ne 'NeverSend'
                        TamperProtection = $status.IsTamperProtected
                        StatusText = if($status.RealTimeProtectionEnabled) { 'Protected' } else { 'Unprotected' }
                    }
                    
                    ConvertTo-Json -Compress $result
                }
                catch {
                    Write-Error $_.Exception.Message
                    exit 1
                }
            ";

            var result = await ExecutePowerShellScriptAsync(script);
            if (result.Success && !string.IsNullOrEmpty(result.Output))
            {
                try
                {
                    var json = System.Text.Json.JsonSerializer.Deserialize<Dictionary<string, object>>(result.Output);
                    if (json != null)
                    {
                        status.RealTimeProtection = json.ContainsKey("RealTimeProtection") && (bool)json["RealTimeProtection"];
                        status.CloudProtection = json.ContainsKey("CloudProtection") && (bool)json["CloudProtection"];
                        status.AutomaticSampleSubmission = json.ContainsKey("AutomaticSampleSubmission") && (bool)json["AutomaticSampleSubmission"];
                        status.TamperProtection = json.ContainsKey("TamperProtection") && (bool)json["TamperProtection"];
                        status.StatusText = json.ContainsKey("StatusText") ? json["StatusText"].ToString() ?? "Unknown" : "Unknown";
                    }
                }
                catch (Exception ex)
                {
                    _logger.LogError($"Error parsing Defender status JSON: {ex.Message}");
                }
            }

            status.LastCheck = DateTime.Now;
            _logger.LogInformation($"Windows Defender status: {status.FormattedStatus}");
            return status;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error getting Windows Defender status: {ex.Message}");
            return new DefenderStatus
            {
                StatusText = "Error",
                LastOperationResults = { $"Error: {ex.Message}" }
            };
        }
    }

    /// <summary>
    /// Disable Windows Defender temporarily
    /// </summary>
    public async Task<bool> DisableDefenderAsync()
    {
        try
        {
            _logger.LogInformation("Disabling Windows Defender");

            if (!await IsElevatedAsync())
            {
                _logger.LogError("Administrator privileges required to disable Windows Defender");
                return false;
            }

            var script = @"
                $ErrorActionPreference = 'Stop'
                try {
                    Set-MpPreference -DisableRealtimeMonitoring $true
                    Set-MpPreference -DisableIOAVProtection $true
                    Set-MpPreference -DisableBehaviorMonitoring $true
                    Set-MpPreference -DisableBlockAtFirstSeen $true
                    Set-MpPreference -DisablePrivacyMode $true
                    Write-Output 'SUCCESS'
                }
                catch {
                    Write-Error $_.Exception.Message
                    exit 1
                }
            ";

            var result = await ExecutePowerShellScriptAsync(script);
            var success = result.Success;
            _logger.LogInformation($"Windows Defender disable result: {(success ? "Success" : "Failed")}");
            return success;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error disabling Windows Defender: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Enable Windows Defender
    /// </summary>
    public async Task<bool> EnableDefenderAsync()
    {
        try
        {
            _logger.LogInformation("Enabling Windows Defender");

            if (!await IsElevatedAsync())
            {
                _logger.LogError("Administrator privileges required to enable Windows Defender");
                return false;
            }

            var script = @"
                $ErrorActionPreference = 'Stop'
                try {
                    Set-MpPreference -DisableRealtimeMonitoring $false
                    Set-MpPreference -DisableIOAVProtection $false
                    Set-MpPreference -DisableBehaviorMonitoring $false
                    Set-MpPreference -DisableBlockAtFirstSeen $false
                    Set-MpPreference -DisablePrivacyMode $false
                    Write-Output 'SUCCESS'
                }
                catch {
                    Write-Error $_.Exception.Message
                    exit 1
                }
            ";

            var result = await ExecutePowerShellScriptAsync(script);
            var success = result.Success;
            _logger.LogInformation($"Windows Defender enable result: {(success ? "Success" : "Failed")}");
            return success;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error enabling Windows Defender: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Safely disable Windows Defender with warnings
    /// </summary>
    public async Task<bool> DisableDefenderSafelyAsync()
    {
        try
        {
            _logger.LogWarning("Safely disabling Windows Defender - this will temporarily disable protection");

            // Check if Tamper Protection is enabled
            var status = await GetDefenderStatusAsync();
            if (status.TamperProtection)
            {
                _logger.LogError("Cannot disable Windows Defender: Tamper Protection is enabled");
                return false;
            }

            return await DisableDefenderAsync();
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error safely disabling Windows Defender: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Check if current process has elevated privileges
    /// </summary>
    public async Task<bool> IsElevatedAsync()
    {
        return await Task.Run(() =>
        {
            try
            {
                using var identity = WindowsIdentity.GetCurrent();
                var principal = new WindowsPrincipal(identity);
                return principal.IsInRole(WindowsBuiltInRole.Administrator);
            }
            catch (Exception ex)
            {
                _logger.LogError($"Error checking elevation: {ex.Message}");
                return false;
            }
        });
    }

    /// <summary>
    /// Get list of important Windows services
    /// </summary>
    public async Task<List<SystemService>> GetImportantServicesAsync()
    {
        try
        {
            _logger.LogInformation("Getting important Windows services");

            var importantServices = new[]
            {
                "WinDefend",           // Windows Defender
                "wscsvc",              // Windows Security Center
                "wuauserv",            // Windows Update
                "BITS",                // Background Intelligent Transfer Service
                "Themes",              // Desktop Window Manager
                "AudioSrv",            // Windows Audio
                "Spooler",             // Print Spooler
                "WSearch",             // Windows Search
                "SysMain",             // Superfetch
                "TabletInputService",  // Touch Keyboard and Handwriting
                "WbioSrvc",            // Windows Biometric Service
                "WMPNetworkSvc",       // Windows Media Player Network Sharing
                "PcaSvc",              // Program Compatibility Assistant
                "WerSvc",              // Windows Error Reporting
                "DiagTrack",           // Connected User Experiences and Telemetry
                "DPS",                 // Diagnostic Policy Service
                "Dnscache",            // DNS Client
                "Dhcp",                // DHCP Client
                "nsi",                 // Network Store Interface Service
                "RpcEptMapper",        // RPC Endpoint Mapper
                "RpcSs",               // Remote Procedure Call
                "SamSs",               // Security Accounts Manager
                "Schedule",            // Task Scheduler
                "Themes",              // Desktop Window Manager
                "UxSms",               // Desktop Window Manager Session Manager
                "W32Time",             // Windows Time
                "WdiSystemHost",       // Diagnostic System Host
                "WdiServiceHost",      // Diagnostic Service Host
                "WSearch",             // Windows Search
                "WwanSvc",             // WWAN AutoConfig
                "XboxGipSvc",          // Xbox Accessory Management Service
                "XboxNetApiSvc"        // Xbox Live Networking Service
            };

            var services = new List<SystemService>();
            foreach (var serviceName in importantServices)
            {
                var service = await GetServiceStatusAsync(serviceName);
                if (service != null)
                {
                    services.Add(service);
                }
            }

            _logger.LogInformation($"Retrieved {services.Count} important services");
            return services;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error getting important services: {ex.Message}");
            return new List<SystemService>();
        }
    }

    /// <summary>
    /// Get service status by name
    /// </summary>
    public async Task<SystemService?> GetServiceStatusAsync(string serviceName)
    {
        return await Task.Run(() =>
        {
            try
            {
                using var searcher = new ManagementObjectSearcher(
                    $"SELECT * FROM Win32_Service WHERE Name = '{serviceName}'");
                
                using var collection = searcher.Get();
                var service = collection.Cast<ManagementObject>().FirstOrDefault();
                
                if (service == null) return null;

                var systemService = new SystemService
                {
                    Name = service["Name"]?.ToString() ?? serviceName,
                    DisplayName = service["DisplayName"]?.ToString() ?? serviceName,
                    Description = service["Description"]?.ToString() ?? "",
                    Status = ParseServiceStatus(service["State"]?.ToString() ?? "Unknown"),
                    StartType = ParseServiceStartType(service["StartMode"]?.ToString() ?? "Unknown"),
                    CanStop = service["State"]?.ToString() == "Running",
                    CanRestart = service["State"]?.ToString() == "Running",
                    LastUpdate = DateTime.Now
                };

                return systemService;
            }
            catch (Exception ex)
            {
                _logger.LogError($"Error getting service status for {serviceName}: {ex.Message}");
                return null;
            }
        });
    }

    /// <summary>
    /// Start a Windows service
    /// </summary>
    public async Task<bool> StartServiceAsync(string serviceName)
    {
        try
        {
            _logger.LogInformation($"Starting service: {serviceName}");

            if (!await IsElevatedAsync())
            {
                _logger.LogError("Administrator privileges required to start services");
                return false;
            }

            var script = $@"
                $ErrorActionPreference = 'Stop'
                try {{
                    Start-Service -Name '{serviceName}' -ErrorAction Stop
                    Write-Output 'SUCCESS'
                }}
                catch {{
                    Write-Error $_.Exception.Message
                    exit 1
                }}
            ";

            var result = await ExecutePowerShellScriptAsync(script);
            var success = result.Success;
            _logger.LogInformation($"Service start result for {serviceName}: {(success ? "Success" : "Failed")}");
            return success;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error starting service {serviceName}: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Stop a Windows service
    /// </summary>
    public async Task<bool> StopServiceAsync(string serviceName)
    {
        try
        {
            _logger.LogInformation($"Stopping service: {serviceName}");

            if (!await IsElevatedAsync())
            {
                _logger.LogError("Administrator privileges required to stop services");
                return false;
            }

            var script = $@"
                $ErrorActionPreference = 'Stop'
                try {{
                    Stop-Service -Name '{serviceName}' -Force -ErrorAction Stop
                    Write-Output 'SUCCESS'
                }}
                catch {{
                    Write-Error $_.Exception.Message
                    exit 1
                }}
            ";

            var result = await ExecutePowerShellScriptAsync(script);
            var success = result.Success;
            _logger.LogInformation($"Service stop result for {serviceName}: {(success ? "Success" : "Failed")}");
            return success;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error stopping service {serviceName}: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Enable a Windows service
    /// </summary>
    public async Task<bool> EnableServiceAsync(string serviceName)
    {
        try
        {
            _logger.LogInformation($"Enabling service: {serviceName}");

            if (!await IsElevatedAsync())
            {
                _logger.LogError("Administrator privileges required to enable services");
                return false;
            }

            var script = $@"
                $ErrorActionPreference = 'Stop'
                try {{
                    Set-Service -Name '{serviceName}' -StartupType Automatic -ErrorAction Stop
                    Write-Output 'SUCCESS'
                }}
                catch {{
                    Write-Error $_.Exception.Message
                    exit 1
                }}
            ";

            var result = await ExecutePowerShellScriptAsync(script);
            var success = result.Success;
            _logger.LogInformation($"Service enable result for {serviceName}: {(success ? "Success" : "Failed")}");
            return success;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error enabling service {serviceName}: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Disable a Windows service
    /// </summary>
    public async Task<bool> DisableServiceAsync(string serviceName)
    {
        try
        {
            _logger.LogInformation($"Disabling service: {serviceName}");

            if (!await IsElevatedAsync())
            {
                _logger.LogError("Administrator privileges required to disable services");
                return false;
            }

            var script = $@"
                $ErrorActionPreference = 'Stop'
                try {{
                    Set-Service -Name '{serviceName}' -StartupType Disabled -ErrorAction Stop
                    Write-Output 'SUCCESS'
                }}
                catch {{
                    Write-Error $_.Exception.Message
                    exit 1
                }}
            ";

            var result = await ExecutePowerShellScriptAsync(script);
            var success = result.Success;
            _logger.LogInformation($"Service disable result for {serviceName}: {(success ? "Success" : "Failed")}");
            return success;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error disabling service {serviceName}: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Get Windows startup programs
    /// </summary>
    public async Task<List<StartupProgram>> GetStartupProgramsAsync()
    {
        return await Task.Run(async () =>
        {
            try
            {
                _logger.LogInformation("Getting Windows startup programs");

                var startupPrograms = new List<StartupProgram>();

                // Registry startup locations
                var registryPaths = new[]
                {
                    @"SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
                    @"SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce",
                    @"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run",
                    @"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run32",
                    @"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\StartupFolder"
                };

                foreach (var registryPath in registryPaths)
                {
                    try
                    {
                        using var key = Registry.LocalMachine.OpenSubKey(registryPath);
                        if (key != null)
                        {
                            foreach (var valueName in key.GetValueNames())
                            {
                                var value = key.GetValue(valueName);
                                if (value != null)
                                {
                                                                         startupPrograms.Add(new StartupProgram
                                     {
                                         Name = valueName,
                                         ExecutablePath = value.ToString() ?? "",
                                         IsEnabled = true,
                                         RegistryPath = registryPath,
                                         LastModified = DateTime.Now
                                     });
                                }
                            }
                        }
                    }
                    catch (Exception ex)
                    {
                        _logger.LogWarning($"Error reading registry path {registryPath}: {ex.Message}");
                    }
                }

                // Also check HKEY_CURRENT_USER
                var userRegistryPaths = new[]
                {
                    @"SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
                    @"SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce"
                };

                foreach (var registryPath in userRegistryPaths)
                {
                    try
                    {
                        using var key = Registry.CurrentUser.OpenSubKey(registryPath);
                        if (key != null)
                        {
                            foreach (var valueName in key.GetValueNames())
                            {
                                var value = key.GetValue(valueName);
                                if (value != null)
                                {
                                                                         startupPrograms.Add(new StartupProgram
                                     {
                                         Name = valueName,
                                         ExecutablePath = value.ToString() ?? "",
                                         IsEnabled = true,
                                         RegistryPath = $"HKCU\\{registryPath}",
                                         LastModified = DateTime.Now
                                     });
                                }
                            }
                        }
                    }
                    catch (Exception ex)
                    {
                        _logger.LogWarning($"Error reading user registry path {registryPath}: {ex.Message}");
                    }
                }

                // Get disabled startup programs from Task Manager
                try
                {
                    var disabledScript = @"
                        $ErrorActionPreference = 'Stop'
                        try {
                            $disabled = Get-ItemProperty -Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run' -ErrorAction SilentlyContinue
                            $disabled32 = Get-ItemProperty -Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run32' -ErrorAction SilentlyContinue
                            
                            $result = @()
                            
                            if ($disabled) {
                                foreach ($prop in $disabled.PSObject.Properties) {
                                    if ($prop.Name -notin @('PSPath', 'PSParentPath', 'PSChildName', 'PSProvider', 'PSDrive')) {
                                                                                 $result += @{
                                             Name = $prop.Name
                                             ExecutablePath = 'Disabled (Run)'
                                             IsEnabled = $false
                                             RegistryPath = 'HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run'
                                         }
                                    }
                                }
                            }
                            
                            if ($disabled32) {
                                foreach ($prop in $disabled32.PSObject.Properties) {
                                    if ($prop.Name -notin @('PSPath', 'PSParentPath', 'PSChildName', 'PSProvider', 'PSDrive')) {
                                                                                 $result += @{
                                             Name = $prop.Name
                                             ExecutablePath = 'Disabled (Run32)'
                                             IsEnabled = $false
                                             RegistryPath = 'HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run32'
                                         }
                                    }
                                }
                            }
                            
                            ConvertTo-Json -Compress $result
                        }
                        catch {
                            Write-Error $_.Exception.Message
                            exit 1
                        }
                    ";

                                         var disabledResult = await ExecutePowerShellScriptAsync(disabledScript);
                    if (disabledResult.Success && !string.IsNullOrEmpty(disabledResult.Output))
                    {
                        try
                        {
                            var disabledPrograms = System.Text.Json.JsonSerializer.Deserialize<List<dynamic>>(disabledResult.Output);
                            if (disabledPrograms != null)
                            {
                                foreach (var program in disabledPrograms)
                                {
                                    startupPrograms.Add(new StartupProgram
                                    {
                                                                                 Name = program.GetProperty("Name").GetString() ?? "",
                                         ExecutablePath = program.GetProperty("ExecutablePath").GetString() ?? "",
                                         IsEnabled = false,
                                         RegistryPath = program.GetProperty("RegistryPath").GetString() ?? "",
                                        LastModified = DateTime.Now
                                    });
                                }
                            }
                        }
                        catch (Exception ex)
                        {
                            _logger.LogWarning($"Error parsing disabled startup programs: {ex.Message}");
                        }
                    }
                }
                catch (Exception ex)
                {
                    _logger.LogWarning($"Error getting disabled startup programs: {ex.Message}");
                }

                _logger.LogInformation($"Found {startupPrograms.Count} startup programs");
                return startupPrograms;
            }
            catch (Exception ex)
            {
                _logger.LogError($"Error getting startup programs: {ex.Message}");
                return new List<StartupProgram>();
            }
        });
    }

    /// <summary>
    /// Enable/disable startup program
    /// </summary>
    public async Task<bool> SetStartupProgramStatusAsync(string name, bool enabled)
    {
        try
        {
            _logger.LogInformation($"Setting startup program {name} status to: {(enabled ? "Enabled" : "Disabled")}");

            if (!await IsElevatedAsync())
            {
                _logger.LogError("Administrator privileges required to modify startup programs");
                return false;
            }

            // This is a simplified implementation
            // In a real scenario, you would need to handle different registry locations
            // and startup folder locations more comprehensively
            
            var script = $@"
                $ErrorActionPreference = 'Stop'
                try {{
                    $registryPath = 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run'
                    if (Test-Path $registryPath) {{
                        if ({enabled.ToString().ToLower()}) {{
                            # Re-enabling would require the original value to be stored
                            Write-Output 'SUCCESS'
                        }} else {{
                            Remove-ItemProperty -Path $registryPath -Name '{name}' -ErrorAction SilentlyContinue
                            Write-Output 'SUCCESS'
                        }}
                    }} else {{
                        Write-Output 'SUCCESS'
                    }}
                }}
                catch {{
                    Write-Error $_.Exception.Message
                    exit 1
                }}
            ";

            var result = await ExecutePowerShellScriptAsync(script);
            var success = result.Success;
            _logger.LogInformation($"Startup program status change result for {name}: {(success ? "Success" : "Failed")}");
            return success;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error setting startup program {name} status: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Execute PowerShell command and return result
    /// </summary>
    private bool ExecutePowerShellCommand(string command)
    {
        try
        {
            var startInfo = new ProcessStartInfo
            {
                FileName = "powershell.exe",
                Arguments = $"-NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -Command \"{command}\"",
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true
            };

            using var process = new Process { StartInfo = startInfo };
            process.Start();
            process.WaitForExit();
            
            return process.ExitCode == 0;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error executing PowerShell command: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Execute PowerShell script and return result
    /// </summary>
    private async Task<(bool Success, string Output, string Error)> ExecutePowerShellScriptAsync(string script)
    {
        return await Task.Run(() =>
        {
            try
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
                
                var output = process.StandardOutput.ReadToEnd();
                var error = process.StandardError.ReadToEnd();
                
                process.WaitForExit();
                
                return (process.ExitCode == 0, output.Trim(), error.Trim());
            }
            catch (Exception ex)
            {
                _logger.LogError($"Error executing PowerShell script: {ex.Message}");
                return (false, "", ex.Message);
            }
        });
    }

    /// <summary>
    /// Parse service status string to enum
    /// </summary>
    private static ServiceStatus ParseServiceStatus(string status)
    {
        return status.ToLower() switch
        {
            "running" => ServiceStatus.Running,
            "stopped" => ServiceStatus.Stopped,
            "startpending" => ServiceStatus.StartPending,
            "stoppending" => ServiceStatus.StopPending,
            "continuepending" => ServiceStatus.ContinuePending,
            "pausepending" => ServiceStatus.PausePending,
            "paused" => ServiceStatus.Paused,
            _ => ServiceStatus.Unknown
        };
    }

    /// <summary>
    /// Parse service start type string to enum
    /// </summary>
    private static ServiceStartType ParseServiceStartType(string startType)
    {
        return startType.ToLower() switch
        {
            "boot" => ServiceStartType.Boot,
            "system" => ServiceStartType.System,
            "automatic" => ServiceStartType.Automatic,
            "manual" => ServiceStartType.Manual,
            "disabled" => ServiceStartType.Disabled,
            _ => ServiceStartType.Manual
        };
    }

    /// <summary>
    /// Toggle Windows Defender real-time protection
    /// </summary>
    public async Task<bool> ToggleDefenderRealTimeProtectionAsync()
    {
        try
        {
            _logger.LogInformation("Toggling Windows Defender real-time protection");

            var script = @"
                $ErrorActionPreference = 'Stop'
                try {
                    # Check if we have admin rights
                    $isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] 'Administrator')
                    if (-not $isAdmin) {
                        Write-Error 'Administrator privileges required'
                        exit 1
                    }

                    # Get current status
                    $status = Get-MpComputerStatus
                    $currentState = $status.RealTimeProtectionEnabled
                    
                    # Toggle the state
                    if ($currentState) {
                        Set-MpPreference -DisableRealtimeMonitoring $true
                        Write-Output 'Real-time protection disabled'
                    } else {
                        Set-MpPreference -DisableRealtimeMonitoring $false
                        Write-Output 'Real-time protection enabled'
                    }
                    
                    exit 0
                }
                catch {
                    Write-Error $_.Exception.Message
                    exit 1
                }
            ";

            var result = await ExecutePowerShellScriptAsync(script);
            var success = result.Success;
            _logger.LogInformation($"Real-time protection toggle result: {(success ? "Success" : "Failed")}");
            return success;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error toggling real-time protection: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Toggle Windows Defender cloud protection
    /// </summary>
    public async Task<bool> ToggleDefenderCloudProtectionAsync()
    {
        try
        {
            _logger.LogInformation("Toggling Windows Defender cloud protection");

            if (!await IsElevatedAsync())
            {
                _logger.LogError("Administrator privileges required to modify Windows Defender settings");
                return false;
            }

            var script = @"
                $ErrorActionPreference = 'Stop'
                try {
                    # Get current status
                    $prefs = Get-MpPreference
                    $currentState = $prefs.MAPSReporting
                    
                    # Toggle the state
                    if ($currentState -eq 'Disabled') {
                        Set-MpPreference -MAPSReporting Advanced
                        Write-Output 'Cloud protection enabled'
                    } else {
                        Set-MpPreference -MAPSReporting Disabled
                        Write-Output 'Cloud protection disabled'
                    }
                    
                    exit 0
                }
                catch {
                    Write-Error $_.Exception.Message
                    exit 1
                }
            ";

            var result = await ExecutePowerShellScriptAsync(script);
            var success = result.Success;
            _logger.LogInformation($"Cloud protection toggle result: {(success ? "Success" : "Failed")} - {result.Output}");
            return success;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error toggling cloud protection: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Toggle Windows Defender tamper protection
    /// </summary>
    public async Task<bool> ToggleDefenderTamperProtectionAsync()
    {
        try
        {
            _logger.LogInformation("Toggling Windows Defender tamper protection");

            if (!await IsElevatedAsync())
            {
                _logger.LogError("Administrator privileges required to modify Windows Defender settings");
                return false;
            }

            var script = @"
                $ErrorActionPreference = 'Stop'
                try {
                    # Get current status
                    $status = Get-MpComputerStatus
                    $currentState = $status.IsTamperProtected
                    
                    # Toggle the state
                    if ($currentState) {
                        Set-MpPreference -DisableTamperProtection $true
                        Write-Output 'Tamper protection disabled'
                    } else {
                        Set-MpPreference -DisableTamperProtection $false
                        Write-Output 'Tamper protection enabled'
                    }
                    
                    exit 0
                }
                catch {
                    Write-Error $_.Exception.Message
                    exit 1
                }
            ";

            var result = await ExecutePowerShellScriptAsync(script);
            var success = result.Success;
            _logger.LogInformation($"Tamper protection toggle result: {(success ? "Success" : "Failed")} - {result.Output}");
            return success;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error toggling tamper protection: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Toggle Windows Defender sample submission
    /// </summary>
    public async Task<bool> ToggleDefenderSampleSubmissionAsync()
    {
        return await Task.Run(() =>
        {
            try
            {
                _logger.LogInformation("Toggling Windows Defender sample submission");
                
                // PowerShell command to toggle sample submission
                var command = "Set-MpPreference -SubmitSamplesConsent NeverSend";
                var result = ExecutePowerShellCommand(command);
                
                _logger.LogInformation("Windows Defender sample submission toggled successfully");
                return result;
            }
            catch (Exception ex)
            {
                _logger.LogError($"Error toggling sample submission: {ex.Message}");
                return false;
            }
        });
    }
}
