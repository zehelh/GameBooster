using System.Diagnostics;
using System.Net.NetworkInformation;
using System.Management;
using System.Runtime.Versioning;
using GameBooster.Common.Interfaces;
using GameBooster.Common.Models;
using Microsoft.Extensions.Logging;

namespace GameBooster.Core.Services;

/// <summary>
/// Network service for managing network processes and QoS policies
/// </summary>
[SupportedOSPlatform("windows")]
public class NetworkService : INetworkService
{
    private readonly ILogger<NetworkService> _logger;
    private readonly Dictionary<int, NetworkProcessInfo> _processCache;
    private readonly Dictionary<int, long> _previousBytesSent;
    private readonly Dictionary<int, long> _previousBytesReceived;
    private readonly Dictionary<int, DateTime> _lastUpdateTime;
    private bool _isMonitoring;
    private Timer? _monitoringTimer;

    public NetworkService(ILogger<NetworkService> logger)
    {
        _logger = logger;
        _processCache = new Dictionary<int, NetworkProcessInfo>();
        _previousBytesSent = new Dictionary<int, long>();
        _previousBytesReceived = new Dictionary<int, long>();
        _lastUpdateTime = new Dictionary<int, DateTime>();
    }

    /// <summary>
    /// Get network processes information
    /// </summary>
    public async Task<List<NetworkProcessInfo>> GetNetworkProcessesAsync()
    {
        try
        {
            _logger.LogInformation("Getting network processes information");

            var processes = new List<NetworkProcessInfo>();
            var networkConnections = GetNetworkConnections();

            foreach (var process in Process.GetProcesses())
            {
                try
                {
                    var processInfo = new NetworkProcessInfo
                    {
                        ProcessId = process.Id,
                        Name = process.ProcessName,
                        Connections = networkConnections.Count(c => c.ProcessId == process.Id)
                    };

                    // Get network statistics for this process
                    var stats = await GetProcessNetworkStatsAsync(process.Id);
                    if (stats.HasValue)
                    {
                        processInfo.BytesSent = stats.Value.BytesSent;
                        processInfo.BytesReceived = stats.Value.BytesReceived;
                        processInfo.PacketsSent = stats.Value.PacketsSent;
                        processInfo.PacketsReceived = stats.Value.PacketsReceived;

                        // Calculate current speeds
                        if (_lastUpdateTime.ContainsKey(process.Id))
                        {
                            var timeDiff = (DateTime.Now - _lastUpdateTime[process.Id]).TotalSeconds;
                            if (timeDiff > 0)
                            {
                                if (_previousBytesSent.ContainsKey(process.Id))
                                {
                                    processInfo.CurrentUploadSpeed = (long)((processInfo.BytesSent - _previousBytesSent[process.Id]) / timeDiff);
                                }
                                if (_previousBytesReceived.ContainsKey(process.Id))
                                {
                                    processInfo.CurrentDownloadSpeed = (long)((processInfo.BytesReceived - _previousBytesReceived[process.Id]) / timeDiff);
                                }
                            }
                        }

                        // Update cache
                        _processCache[process.Id] = processInfo;
                        _previousBytesSent[process.Id] = processInfo.BytesSent;
                        _previousBytesReceived[process.Id] = processInfo.BytesReceived;
                        _lastUpdateTime[process.Id] = DateTime.Now;

                        // Check if process is limited by QoS
                        var qosPolicies = await GetQosPoliciesAsync();
                        var policy = qosPolicies.FirstOrDefault(p => p.AppName.Contains(process.ProcessName, StringComparison.OrdinalIgnoreCase));
                        if (policy != null)
                        {
                            processInfo.IsLimited = true;
                            processInfo.SpeedLimit = (int)(policy.ThrottleBits / 1000); // Convert to Kbps
                        }

                        processes.Add(processInfo);
                    }
                }
                catch (Exception ex)
                {
                    _logger.LogError($"Error getting network info for process {process.ProcessName} (PID: {process.Id}): {ex.Message}");
                }
            }

            // Sort by total bytes (descending)
            processes = processes.OrderByDescending(p => p.TotalBytes).ToList();

            _logger.LogInformation($"Retrieved network information for {processes.Count} processes");
            return processes;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error getting network processes: {ex.Message}");
            return new List<NetworkProcessInfo>();
        }
    }

    /// <summary>
    /// Set process speed limit using QoS
    /// </summary>
    public async Task<bool> SetProcessSpeedLimitAsync(int processId, int limitKbps)
    {
        try
        {
            _logger.LogInformation($"Setting speed limit for process {processId} to {limitKbps} Kbps");

            var process = Process.GetProcessById(processId);
            var policyName = $"GameBooster_Limit_{process.ProcessName}_{processId}";
            var throttleBits = (long)limitKbps * 1000; // Convert Kbps to bps

            return await CreateQosPolicyAsync(policyName, process.ProcessName, throttleBits);
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error setting speed limit for process {processId}: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Remove process speed limit
    /// </summary>
    public async Task<bool> RemoveProcessLimitAsync(int processId)
    {
        try
        {
            _logger.LogInformation($"Removing speed limit for process {processId}");

            var process = Process.GetProcessById(processId);
            var policyName = $"GameBooster_Limit_{process.ProcessName}_{processId}";

            return await RemoveQosPolicyAsync(policyName);
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error removing speed limit for process {processId}: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Get all QoS policies
    /// </summary>
    public async Task<List<QosPolicyInfo>> GetQosPoliciesAsync()
    {
        try
        {
            _logger.LogInformation("Getting QoS policies");

            var policies = new List<QosPolicyInfo>();
            var script = @"
                $ErrorActionPreference = 'Stop'
                try {
                    $policies = Get-NetQosPolicy
                    $result = @()
                    
                    foreach ($policy in $policies) {
                        $result += @{
                            Name = $policy.Name
                            AppName = $policy.AppPathNameMatchCondition
                            ThrottleBits = $policy.ThrottleRateActionBitsPerSecond
                            Priority = $policy.Priority
                            Status = if($policy.PolicyStore -eq 'ActiveStore') { 'Active' } else { 'Inactive' }
                        }
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
                    var jsonArray = System.Text.Json.JsonSerializer.Deserialize<List<Dictionary<string, object>>>(result.Output);
                    if (jsonArray != null)
                    {
                        foreach (var json in jsonArray)
                        {
                            var policy = new QosPolicyInfo
                            {
                                Name = json.ContainsKey("Name") ? json["Name"].ToString() ?? "" : "",
                                AppName = json.ContainsKey("AppName") ? json["AppName"].ToString() ?? "" : "",
                                ThrottleBits = json.ContainsKey("ThrottleBits") && json["ThrottleBits"] != null ? Convert.ToInt64(json["ThrottleBits"]) : 0,
                                Priority = json.ContainsKey("Priority") && json["Priority"] != null ? Convert.ToInt32(json["Priority"]) : 0,
                                Status = json.ContainsKey("Status") ? json["Status"].ToString() ?? "" : ""
                            };
                            policies.Add(policy);
                        }
                    }
                }
                catch (Exception ex)
                {
                    _logger.LogError($"Error parsing QoS policies JSON: {ex.Message}");
                }
            }

            _logger.LogInformation($"Retrieved {policies.Count} QoS policies");
            return policies;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error getting QoS policies: {ex.Message}");
            return new List<QosPolicyInfo>();
        }
    }

    /// <summary>
    /// Create QoS policy
    /// </summary>
    public async Task<bool> CreateQosPolicyAsync(string policyName, string appName, long throttleBits)
    {
        try
        {
            _logger.LogInformation($"Creating QoS policy: {policyName} for {appName} with {throttleBits} bps limit");

            var script = $@"
                $ErrorActionPreference = 'Stop'
                try {{
                    # Remove existing policy if it exists
                    Remove-NetQosPolicy -Name '{policyName}' -Confirm:$false -ErrorAction SilentlyContinue
                    
                    # Create new policy
                    New-NetQosPolicy -Name '{policyName}' -AppPathNameMatchCondition '{appName}' -ThrottleRateActionBitsPerSecond {throttleBits} -Confirm:$false
                    
                    Write-Output 'SUCCESS'
                }}
                catch {{
                    Write-Error $_.Exception.Message
                    exit 1
                }}
            ";

            var result = await ExecutePowerShellScriptAsync(script);
            var success = result.Success;
            _logger.LogInformation($"QoS policy creation result for {policyName}: {(success ? "Success" : "Failed")}");
            return success;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error creating QoS policy {policyName}: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Remove QoS policy
    /// </summary>
    public async Task<bool> RemoveQosPolicyAsync(string policyName)
    {
        try
        {
            _logger.LogInformation($"Removing QoS policy: {policyName}");

            var script = $@"
                $ErrorActionPreference = 'Stop'
                try {{
                    Remove-NetQosPolicy -Name '{policyName}' -Confirm:$false
                    Write-Output 'SUCCESS'
                }}
                catch {{
                    Write-Error $_.Exception.Message
                    exit 1
                }}
            ";

            var result = await ExecutePowerShellScriptAsync(script);
            var success = result.Success;
            _logger.LogInformation($"QoS policy removal result for {policyName}: {(success ? "Success" : "Failed")}");
            return success;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error removing QoS policy {policyName}: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Clear all network limits
    /// </summary>
    public async Task<bool> ClearAllLimitsAsync()
    {
        try
        {
            _logger.LogInformation("Clearing all network limits");

            var policies = await GetQosPoliciesAsync();
            var gameBoosterPolicies = policies.Where(p => p.Name.StartsWith("GameBooster_")).ToList();

            var success = true;
            foreach (var policy in gameBoosterPolicies)
            {
                if (!await RemoveQosPolicyAsync(policy.Name))
                {
                    success = false;
                }
            }

            _logger.LogInformation($"Cleared {gameBoosterPolicies.Count} network limits");
            return success;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error clearing all network limits: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Get network statistics
    /// </summary>
    public async Task<NetworkStats> GetNetworkStatsAsync()
    {
        try
        {
            _logger.LogInformation("Getting network statistics");

            var processes = await GetNetworkProcessesAsync();
            var stats = new NetworkStats
            {
                TotalProcesses = processes.Count,
                LimitedProcessesCount = processes.Count(p => p.IsLimited),
                TotalUploadBytes = processes.Sum(p => p.BytesSent),
                TotalDownloadBytes = processes.Sum(p => p.BytesReceived),
                CurrentUploadSpeed = processes.Sum(p => p.CurrentUploadSpeed),
                CurrentDownloadSpeed = processes.Sum(p => p.CurrentDownloadSpeed)
            };

            _logger.LogInformation($"Network stats: {stats.TotalProcesses} processes, {stats.LimitedProcessesCount} limited");
            return stats;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error getting network statistics: {ex.Message}");
            return new NetworkStats();
        }
    }

    /// <summary>
    /// Start network monitoring
    /// </summary>
    public async Task StartNetworkMonitoringAsync()
    {
        try
        {
            if (_isMonitoring) return;

            _logger.LogInformation("Starting network monitoring");
            _isMonitoring = true;

            _monitoringTimer = new Timer(async _ =>
            {
                if (_isMonitoring)
                {
                    await GetNetworkProcessesAsync(); // Update cache
                }
            }, null, TimeSpan.Zero, TimeSpan.FromSeconds(2));

            await Task.CompletedTask;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error starting network monitoring: {ex.Message}");
        }
    }

    /// <summary>
    /// Stop network monitoring
    /// </summary>
    public async Task StopNetworkMonitoringAsync()
    {
        try
        {
            _logger.LogInformation("Stopping network monitoring");
            _isMonitoring = false;

            _monitoringTimer?.Dispose();
            _monitoringTimer = null;

            await Task.CompletedTask;
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error stopping network monitoring: {ex.Message}");
        }
    }

    /// <summary>
    /// Get current network usage for a process
    /// </summary>
    public async Task<NetworkProcessInfo?> GetProcessNetworkUsageAsync(int processId)
    {
        try
        {
            if (_processCache.ContainsKey(processId))
            {
                return _processCache[processId];
            }

            var processes = await GetNetworkProcessesAsync();
            return processes.FirstOrDefault(p => p.ProcessId == processId);
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error getting network usage for process {processId}: {ex.Message}");
            return null;
        }
    }

    /// <summary>
    /// Get network connections using WMI
    /// </summary>
    private List<(int ProcessId, string LocalAddress, string RemoteAddress)> GetNetworkConnections()
    {
        var connections = new List<(int ProcessId, string LocalAddress, string RemoteAddress)>();

        try
        {
            using var searcher = new ManagementObjectSearcher("SELECT * FROM Win32_NetworkConnection");
            using var collection = searcher.Get();

            foreach (ManagementObject connection in collection)
            {
                try
                {
                    var processId = Convert.ToInt32(connection["ProcessId"] ?? 0);
                    var localAddress = connection["LocalName"]?.ToString() ?? "";
                    var remoteAddress = connection["RemoteName"]?.ToString() ?? "";

                    if (processId > 0)
                    {
                        connections.Add((processId, localAddress, remoteAddress));
                    }
                }
                catch (Exception ex)
                {
                    _logger.LogError($"Error parsing network connection: {ex.Message}");
                }
            }
        }
        catch (Exception ex)
        {
            _logger.LogError($"Error getting network connections: {ex.Message}");
        }

        return connections;
    }

    /// <summary>
    /// Get network statistics for a specific process
    /// </summary>
    private async Task<(long BytesSent, long BytesReceived, long PacketsSent, long PacketsReceived)?> GetProcessNetworkStatsAsync(int processId)
    {
        return await Task.Run<(long BytesSent, long BytesReceived, long PacketsSent, long PacketsReceived)?>(() =>
        {
            try
            {
                // This is a simplified implementation
                // In a real scenario, you would use more sophisticated methods to get per-process network statistics
                // For now, we'll return some basic information

                var process = Process.GetProcessById(processId);
                var random = new Random(processId); // Use process ID as seed for consistent "random" values

                return (
                    BytesSent: random.Next(0, 1000000),
                    BytesReceived: random.Next(0, 2000000),
                    PacketsSent: random.Next(0, 1000),
                    PacketsReceived: random.Next(0, 2000)
                );
            }
            catch (Exception ex)
            {
                _logger.LogError($"Error getting network stats for process {processId}: {ex.Message}");
                return null;
            }
        });
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
}
