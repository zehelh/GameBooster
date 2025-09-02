using System.Diagnostics;
using System.Management;
using System.Runtime.Versioning;
using System.Runtime.InteropServices;
using GameBooster.Common.Interfaces;
using GameBooster.Common.Models;
using Microsoft.Extensions.Logging;
using System.Reactive.Subjects;
using System.Reactive.Linq;

namespace GameBooster.Core.Services;

/// <summary>
/// CPU service for managing process priorities and CPU affinity
/// </summary>
[SupportedOSPlatform("windows")]
public class CpuService : ICpuService
{
    private readonly ILogger<CpuService> _logger;
    private readonly Subject<CpuInfo> _cpuMonitorSubject;
    private readonly Dictionary<int, CpuProcessInfo> _processCache;
    private readonly Dictionary<int, DateTime> _lastUpdateTime;
    private Timer? _monitoringTimer;
    private bool _isMonitoring;

    // P/Invoke declarations for Windows API
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool SetProcessAffinityMask(IntPtr hProcess, ulong dwProcessAffinityMask);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool GetProcessAffinityMask(IntPtr hProcess, out ulong lpProcessAffinityMask, out ulong lpSystemAffinityMask);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool SetPriorityClass(IntPtr hProcess, uint dwPriorityClass);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern uint GetPriorityClass(IntPtr hProcess);

    public CpuService(ILogger<CpuService> logger)
    {
        _logger = logger;
        _cpuMonitorSubject = new Subject<CpuInfo>();
        _processCache = new Dictionary<int, CpuProcessInfo>();
        _lastUpdateTime = new Dictionary<int, DateTime>();
    }

    /// <summary>
    /// Get CPU information and statistics
    /// </summary>
    public async Task<CpuInfo> GetCpuInfoAsync()
    {
        try
        {
            _logger.LogInformation("Getting CPU information");

            var cpuInfo = new CpuInfo();

            // Get CPU information using WMI
            using var searcher = new ManagementObjectSearcher("SELECT * FROM Win32_Processor");
            using var collection = searcher.Get();

            foreach (ManagementObject obj in collection)
            {
                cpuInfo.Name = obj["Name"]?.ToString() ?? "Unknown CPU";
                cpuInfo.PhysicalCores = Convert.ToInt32(obj["NumberOfCores"] ?? 1);
                cpuInfo.LogicalCores = Convert.ToInt32(obj["NumberOfLogicalProcessors"] ?? 1);
                cpuInfo.MaxFrequency = Convert.ToDouble(obj["MaxClockSpeed"] ?? 0);
                break; // Take the first CPU (assuming single CPU system)
            }

            // Get current CPU usage using PerformanceCounter
            using var cpuCounter = new PerformanceCounter("Processor", "% Processor Time", "_Total");
            cpuCounter.NextValue(); // First call returns 0, second call returns actual value
            await Task.Delay(100); // Wait a bit to get accurate reading
            cpuInfo.UsagePercentage = cpuCounter.NextValue();

            // Get current frequency (approximation)
            cpuInfo.Frequency = cpuInfo.MaxFrequency * (cpuInfo.UsagePercentage / 100.0 + 0.3); // Rough estimation

            // Try to get temperature (requires additional libraries)
            cpuInfo.Temperature = null; // Not implemented yet

            // Try to get power consumption (requires additional libraries)
            cpuInfo.PowerConsumption = null; // Not implemented yet

            cpuInfo.LastUpdate = DateTime.Now;

            _logger.LogInformation("CPU info retrieved - Usage: {Usage}%, Cores: {Cores}", 
                cpuInfo.UsagePercentage, cpuInfo.LogicalCores);

            return cpuInfo;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error getting CPU information");
            return new CpuInfo
            {
                Name = "Unknown",
                PhysicalCores = Environment.ProcessorCount,
                LogicalCores = Environment.ProcessorCount,
                UsagePercentage = 0,
                LastUpdate = DateTime.Now
            };
        }
    }

    /// <summary>
    /// Get all processes with CPU usage information
    /// </summary>
    public async Task<List<CpuProcessInfo>> GetCpuProcessesAsync()
    {
        try
        {
            _logger.LogInformation("Getting CPU processes information");

            var processes = new List<CpuProcessInfo>();
            var now = DateTime.Now;

            foreach (var process in Process.GetProcesses())
            {
                try
                {
                    if (process.HasExited) continue;

                    var processInfo = new CpuProcessInfo
                    {
                        ProcessId = process.Id,
                        Name = process.ProcessName,
                        ThreadCount = process.Threads?.Count ?? 0,
                        StartTime = process.StartTime,
                        IsActive = !process.HasExited
                    };

                    // Get CPU usage using PerformanceCounter
                    try
                    {
                        using var cpuCounter = new PerformanceCounter("Process", "% Processor Time", process.ProcessName);
                        cpuCounter.NextValue(); // First call returns 0
                        await Task.Delay(50); // Wait a bit
                        processInfo.CpuUsage = cpuCounter.NextValue();
                    }
                    catch
                    {
                        processInfo.CpuUsage = 0;
                    }

                    // Get process priority
                    try
                    {
                        var priorityClass = GetPriorityClass(process.Handle);
                        processInfo.Priority = ConvertToProcessPriority(priorityClass);
                    }
                    catch
                    {
                        processInfo.Priority = ProcessPriority.Normal;
                    }

                    // Get CPU affinity
                    try
                    {
                        if (GetProcessAffinityMask(process.Handle, out ulong processAffinity, out _))
                        {
                            processInfo.AffinityMask = processAffinity;
                        }
                    }
                    catch
                    {
                        processInfo.AffinityMask = 0xFFFFFFFFFFFFFFFF; // All cores
                    }

                    // Update cache
                    _processCache[process.Id] = processInfo;
                    _lastUpdateTime[process.Id] = now;

                    processes.Add(processInfo);
                }
                catch (Exception ex)
                {
                    _logger.LogDebug("Error getting info for process {ProcessName} (PID: {ProcessId}): {Error}", 
                        process.ProcessName, process.Id, ex.Message);
                }
                finally
                {
                    process?.Dispose();
                }
            }

            // Sort by CPU usage (descending)
            processes = processes.OrderByDescending(p => p.CpuUsage).ToList();

            _logger.LogInformation("Retrieved CPU information for {Count} processes", processes.Count);
            return processes;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error getting CPU processes");
            return new List<CpuProcessInfo>();
        }
    }

    /// <summary>
    /// Set process priority
    /// </summary>
    public Task<bool> SetProcessPriorityAsync(int processId, ProcessPriority priority)
    {
        try
        {
            _logger.LogInformation("Setting priority for process {ProcessId} to {Priority}", processId, priority);

            using var process = Process.GetProcessById(processId);
            
            // Check if we have permission to modify this process
            if (process.HasExited)
            {
                _logger.LogWarning("Process {ProcessId} has already exited", processId);
                return Task.FromResult(false);
            }

            var priorityClass = ConvertToPriorityClass(priority);
            
            var success = SetPriorityClass(process.Handle, priorityClass);
            if (success)
            {
                _logger.LogInformation("Successfully set priority for process {ProcessId} to {Priority}", processId, priority);
                return Task.FromResult(true);
            }
            else
            {
                var error = Marshal.GetLastWin32Error();
                _logger.LogError("Failed to set priority for process {ProcessId}. Error code: {ErrorCode}", processId, error);
                return Task.FromResult(false);
            }
        }
        catch (UnauthorizedAccessException ex)
        {
            _logger.LogError(ex, "Access denied when setting priority for process {ProcessId}. Try running as administrator.", processId);
            return Task.FromResult(false);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error setting priority for process {ProcessId}", processId);
            return Task.FromResult(false);
        }
    }

    /// <summary>
    /// Get current process priority
    /// </summary>
    public Task<ProcessPriority> GetProcessPriorityAsync(int processId)
    {
        try
        {
            using var process = Process.GetProcessById(processId);
            var priorityClass = GetPriorityClass(process.Handle);
            return Task.FromResult(ConvertToProcessPriority(priorityClass));
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error getting priority for process {ProcessId}", processId);
            return Task.FromResult(ProcessPriority.Normal);
        }
    }

    /// <summary>
    /// Set process CPU affinity
    /// </summary>
    public Task<bool> SetProcessAffinityAsync(int processId, ulong affinityMask)
    {
        try
        {
            _logger.LogInformation("Setting affinity for process {ProcessId} to {AffinityMask:X}", processId, affinityMask);

            using var process = Process.GetProcessById(processId);
            var success = SetProcessAffinityMask(process.Handle, affinityMask);
            
            if (success)
            {
                _logger.LogInformation("Successfully set affinity for process {ProcessId}", processId);
                return Task.FromResult(true);
            }
            else
            {
                _logger.LogError("Failed to set affinity for process {ProcessId}", processId);
                return Task.FromResult(false);
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error setting affinity for process {ProcessId}", processId);
            return Task.FromResult(false);
        }
    }

    /// <summary>
    /// Get current process CPU affinity
    /// </summary>
    public Task<ulong> GetProcessAffinityAsync(int processId)
    {
        try
        {
            using var process = Process.GetProcessById(processId);
            if (GetProcessAffinityMask(process.Handle, out ulong processAffinity, out _))
            {
                return Task.FromResult(processAffinity);
            }
            return Task.FromResult(0xFFFFFFFFFFFFFFFFUL); // All cores
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error getting affinity for process {ProcessId}", processId);
            return Task.FromResult(0xFFFFFFFFFFFFFFFFUL);
        }
    }

    /// <summary>
    /// Set CPU usage limit for a process (experimental)
    /// </summary>
    public async Task<bool> SetProcessCpuLimitAsync(int processId, int percentageLimit)
    {
        try
        {
            _logger.LogInformation("Setting CPU limit for process {ProcessId} to {Limit}%", processId, percentageLimit);

            using var process = Process.GetProcessById(processId);
            
            // Set process priority based on CPU limit
            ProcessPriority priority;
            if (percentageLimit <= 25)
                priority = ProcessPriority.BelowNormal;
            else if (percentageLimit <= 50)
                priority = ProcessPriority.Normal;
            else if (percentageLimit <= 75)
                priority = ProcessPriority.AboveNormal;
            else
                priority = ProcessPriority.High;

            var success = await SetProcessPriorityAsync(processId, priority);
            
            if (success)
            {
                _logger.LogInformation("CPU limit set successfully for process {ProcessId} with priority {Priority}", 
                    processId, priority);
            }
            
            return success;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error setting CPU limit for process {ProcessId}", processId);
            return false;
        }
    }

    /// <summary>
    /// Remove CPU usage limit for a process
    /// </summary>
    public async Task<bool> RemoveProcessCpuLimitAsync(int processId)
    {
        try
        {
            _logger.LogInformation("Removing CPU limit for process {ProcessId}", processId);
            
            // Reset to normal priority
            var success = await SetProcessPriorityAsync(processId, ProcessPriority.Normal);
            
            if (success)
            {
                _logger.LogInformation("CPU limit removed successfully for process {ProcessId}", processId);
            }
            
            return success;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error removing CPU limit for process {ProcessId}", processId);
            return false;
        }
    }

    /// <summary>
    /// Get CPU usage statistics for a specific process
    /// </summary>
    public async Task<CpuProcessInfo?> GetProcessCpuUsageAsync(int processId)
    {
        try
        {
            var processes = await GetCpuProcessesAsync();
            return processes.FirstOrDefault(p => p.ProcessId == processId);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error getting CPU usage for process {ProcessId}", processId);
            return null;
        }
    }

    /// <summary>
    /// Monitor CPU usage in real-time
    /// </summary>
    public IObservable<CpuInfo> MonitorCpuAsync()
    {
        if (!_isMonitoring)
        {
            StartMonitoring();
        }
        return _cpuMonitorSubject.AsObservable();
    }

    /// <summary>
    /// Apply gaming optimization profile
    /// </summary>
    public async Task<bool> ApplyGamingProfileAsync(string gameProcessName)
    {
        try
        {
            _logger.LogInformation("Applying gaming profile for {GameProcess}", gameProcessName);

            var processes = Process.GetProcessesByName(gameProcessName);
            if (processes.Length == 0)
            {
                _logger.LogWarning("Game process {GameProcess} not found", gameProcessName);
                return false;
            }

            var success = true;
            foreach (var process in processes)
            {
                try
                {
                    // Set high priority
                    success &= await SetProcessPriorityAsync(process.Id, ProcessPriority.High);
                    
                    // Set affinity to use all cores
                    var allCoresMask = (1UL << Environment.ProcessorCount) - 1;
                    success &= await SetProcessAffinityAsync(process.Id, allCoresMask);
                }
                catch (Exception ex)
                {
                    _logger.LogError(ex, "Error applying gaming profile to process {ProcessId}", process.Id);
                    success = false;
                }
                finally
                {
                    process?.Dispose();
                }
            }

            _logger.LogInformation("Gaming profile applied for {GameProcess}: {Success}", gameProcessName, success);
            return success;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error applying gaming profile for {GameProcess}", gameProcessName);
            return false;
        }
    }

    /// <summary>
    /// Reset all CPU optimizations
    /// </summary>
    public async Task<bool> ResetAllOptimizationsAsync()
    {
        try
        {
            _logger.LogInformation("Resetting all CPU optimizations");

            var processes = await GetCpuProcessesAsync();
            var success = true;

            foreach (var process in processes.Where(p => p.Priority != ProcessPriority.Normal))
            {
                try
                {
                    success &= await SetProcessPriorityAsync(process.ProcessId, ProcessPriority.Normal);
                }
                catch (Exception ex)
                {
                    _logger.LogError(ex, "Error resetting priority for process {ProcessId}", process.ProcessId);
                    success = false;
                }
            }

            _logger.LogInformation("CPU optimizations reset: {Success}", success);
            return success;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error resetting CPU optimizations");
            return false;
        }
    }

    /// <summary>
    /// Get available CPU cores count
    /// </summary>
    public Task<int> GetCpuCoresCountAsync()
    {
        return Task.FromResult(Environment.ProcessorCount);
    }

    /// <summary>
    /// Get CPU performance counters
    /// </summary>
    public Task<CpuPerformanceCounters> GetCpuPerformanceCountersAsync()
    {
        try
        {
            var counters = new CpuPerformanceCounters();

            // Get basic performance counters
            using var cpuCounter = new PerformanceCounter("Processor", "% Processor Time", "_Total");
            using var contextSwitchCounter = new PerformanceCounter("System", "Context Switches/sec");
            using var systemCallCounter = new PerformanceCounter("System", "System Calls/sec");

            counters.ContextSwitchesPerSec = contextSwitchCounter.NextValue();
            counters.SystemCallsPerSec = systemCallCounter.NextValue();

            counters.LastUpdate = DateTime.Now;

            return Task.FromResult(counters);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error getting CPU performance counters");
            return Task.FromResult(new CpuPerformanceCounters());
        }
    }

    private void StartMonitoring()
    {
        if (_isMonitoring) return;

        _isMonitoring = true;
        _monitoringTimer = new Timer(async _ =>
        {
            try
            {
                var cpuInfo = await GetCpuInfoAsync();
                _cpuMonitorSubject.OnNext(cpuInfo);
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Error in CPU monitoring");
            }
        }, null, TimeSpan.Zero, TimeSpan.FromSeconds(1));

        _logger.LogInformation("CPU monitoring started");
    }

    private ProcessPriority ConvertToProcessPriority(uint priorityClass)
    {
        return priorityClass switch
        {
            0x00000040 => ProcessPriority.Idle,
            0x00004000 => ProcessPriority.BelowNormal,
            0x00000020 => ProcessPriority.Normal,
            0x00008000 => ProcessPriority.AboveNormal,
            0x00000080 => ProcessPriority.High,
            0x00000100 => ProcessPriority.RealTime,
            _ => ProcessPriority.Normal
        };
    }

    private uint ConvertToPriorityClass(ProcessPriority priority)
    {
        return priority switch
        {
            ProcessPriority.Idle => 0x00000040,
            ProcessPriority.BelowNormal => 0x00004000,
            ProcessPriority.Normal => 0x00000020,
            ProcessPriority.AboveNormal => 0x00008000,
            ProcessPriority.High => 0x00000080,
            ProcessPriority.RealTime => 0x00000100,
            _ => 0x00000020
        };
    }

    public void Dispose()
    {
        _monitoringTimer?.Dispose();
        _cpuMonitorSubject?.Dispose();
    }
}
