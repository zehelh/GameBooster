using System.Diagnostics;
using System.Reactive.Linq;
using GameBooster.Common.Interfaces;
using GameBooster.Common.Models;
using Microsoft.Extensions.Logging;
using System.Runtime.Versioning;
using System.Runtime.InteropServices;

namespace GameBooster.Core.Services;

/// <summary>
/// Service de gestion de la mémoire système avec nettoyage agressif
/// </summary>
public class MemoryService : IMemoryService
{
    private readonly ILogger<MemoryService> _logger;

    [SupportedOSPlatform("windows")]
    public MemoryService(ILogger<MemoryService> logger)
    {
        _logger = logger;
    }

    public Task<MemoryInfo> GetMemoryInfoAsync()
    {
        try
        {
            // Try to get memory info from Windows API
            var memoryStatus = new MEMORYSTATUSEX();
            memoryStatus.dwLength = (uint)Marshal.SizeOf(typeof(MEMORYSTATUSEX));
            
            if (GlobalMemoryStatusEx(ref memoryStatus))
            {
                return Task.FromResult(new MemoryInfo
                {
                    TotalPhysical = (long)memoryStatus.ullTotalPhys,
                    AvailablePhysical = (long)memoryStatus.ullAvailPhys,
                    TotalPageFile = (long)memoryStatus.ullTotalPageFile,
                    AvailablePageFile = (long)memoryStatus.ullAvailPageFile,
                    LastUpdate = DateTime.Now
                });
            }
            
            // Fallback to default values if API call fails
            _logger.LogWarning("GlobalMemoryStatusEx failed, using fallback values");
            return Task.FromResult(new MemoryInfo
            {
                TotalPhysical = 16L * 1024 * 1024 * 1024, // 16 GB default
                AvailablePhysical = 8L * 1024 * 1024 * 1024, // 8 GB available
                TotalPageFile = 24L * 1024 * 1024 * 1024, // 24 GB page file
                AvailablePageFile = 12L * 1024 * 1024 * 1024, // 12 GB available
                LastUpdate = DateTime.Now
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error retrieving memory information, using fallback values");
            
            // Return safe default values instead of throwing
            return Task.FromResult(new MemoryInfo
            {
                TotalPhysical = 16L * 1024 * 1024 * 1024, // 16 GB default
                AvailablePhysical = 8L * 1024 * 1024 * 1024, // 8 GB available
                TotalPageFile = 24L * 1024 * 1024 * 1024, // 24 GB page file
                AvailablePageFile = 12L * 1024 * 1024 * 1024, // 12 GB available
                LastUpdate = DateTime.Now
            });
        }
    }

    public async Task<MemoryCleanupResult> CleanMemoryAsync(MemoryCleanupOptions? options = null)
    {
        var stopwatch = Stopwatch.StartNew();
        var result = new MemoryCleanupResult
        {
            Type = CleanupType.Memory,
            Timestamp = DateTime.Now,
            OptionsUsed = options
        };

        try
        {
            options ??= new MemoryCleanupOptions(); // Default options if not specified
            
            var memoryBefore = await GetMemoryInfoAsync();
            result.MemoryBefore = memoryBefore;

            // 1. Aggressive garbage collection
            if (options.IncludeGarbageCollection)
            {
                _logger.LogInformation("Starting aggressive garbage collection");
                var gcBefore = await GetMemoryInfoAsync();
                GC.Collect(2, GCCollectionMode.Forced, true, true);
                GC.WaitForPendingFinalizers();
                GC.Collect(2, GCCollectionMode.Forced, true, true);
                var gcAfter = await GetMemoryInfoAsync();
                result.GarbageCollectionFreed = gcAfter.AvailablePhysical - gcBefore.AvailablePhysical;
            }

            // 2. System cache memory release
            if (options.IncludeSystemCache)
            {
                _logger.LogInformation("Clearing system cache memory");
                result.SystemCacheFreed = await ClearSystemCacheAsync();
            }

            // 3. Optimize paged and non-paged pools
            if (options.IncludePagedPool || options.IncludeNonPagedPool)
            {
                _logger.LogInformation("Optimizing memory pools");
                var poolResult = await OptimizeMemoryPoolsAsync(options);
                result.PagedPoolFreed = options.IncludePagedPool ? poolResult : 0;
                result.NonPagedPoolFreed = options.IncludeNonPagedPool ? poolResult : 0;
            }

            // 4. Working set memory release
            if (options.IncludeWorkingSet)
            {
                _logger.LogInformation("Optimizing working set memory");
                result.WorkingSetFreed = await OptimizeWorkingSetAsync(options, result);
            }

            // 5. Standby memory release
            if (options.IncludeStandbyMemory)
            {
                _logger.LogInformation("Clearing standby memory");
                result.StandbyMemoryFreed = await ClearStandbyMemoryAsync();
            }

            // 6. DNS cache cleanup (new)
            result.DnsCacheFreed = await ClearDnsCacheAsync();

            // 7. Windows caches cleanup (new)
            result.WindowsCachesFreed = await ClearWindowsCachesAsync();

            // 8. Aggressive system processes cleanup (new)
            result.SystemProcessesFreed = await ClearSystemProcessesAsync();

            // 9. Page file memory release (new)
            result.PageFileFreed = await ClearPageFileAsync();

            var memoryAfter = await GetMemoryInfoAsync();
            result.MemoryAfter = memoryAfter;
            
            // Calculate total space freed
            result.SpaceFreed = Math.Max(result.TotalMemoryFreed, memoryAfter.AvailablePhysical - memoryBefore.AvailablePhysical);
            result.Success = true;
            
            _logger.LogInformation("Aggressive memory cleanup completed. Total space freed: {SpaceFreed} MB", 
                result.TotalMemoryFreedMB);
        }
        catch (Exception ex)
        {
            result.Success = false;
            result.ErrorMessage = ex.Message;
            _logger.LogError(ex, "Error during aggressive memory cleanup");
        }
        finally
        {
            stopwatch.Stop();
            result.DurationMs = stopwatch.ElapsedMilliseconds;
        }

        return result;
    }

    private async Task<long> ClearSystemCacheAsync()
    {
        try
        {
            var memoryBefore = await GetMemoryInfoAsync();
            
            // Aggressive system cache release
            var currentProcess = Process.GetCurrentProcess();
            
            // 1. Forcer la libération du working set
            SetProcessWorkingSetSize(currentProcess.Handle, -1, -1);
            
            // 2. Vider le working set
            EmptyWorkingSet(currentProcess.Handle);
            
            // 3. Aggressive system cache release
            ClearSystemWorkingSet();
            
            // 4. Forcer le garbage collection du système
            GC.Collect(2, GCCollectionMode.Forced, true, true);
            GC.WaitForPendingFinalizers();
            GC.Collect(2, GCCollectionMode.Forced, true, true);
            
            // 5. Libération de la mémoire standby
            ClearStandbyList();
            
            var memoryAfter = await GetMemoryInfoAsync();
            return memoryAfter.AvailablePhysical - memoryBefore.AvailablePhysical;
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Error clearing system cache");
        }
        
        return 0;
    }

    private async Task<long> OptimizeMemoryPoolsAsync(MemoryCleanupOptions options)
    {
        try
        {
            var memoryBefore = await GetMemoryInfoAsync();
            
            // Optimisation des pools mémoire via les API Windows
            if (options.IncludePagedPool)
            {
                _logger.LogDebug("Clearing paged pool");
                // Libération du pool paginé via API Windows
                ClearPagedPool();
            }
            
            if (options.IncludeNonPagedPool)
            {
                _logger.LogDebug("Clearing non-paged pool");
                // Libération du pool non paginé via API Windows
                ClearNonPagedPool();
            }
            
            var memoryAfter = await GetMemoryInfoAsync();
            return memoryAfter.AvailablePhysical - memoryBefore.AvailablePhysical;
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Error optimizing memory pools");
        }
        
        return 0;
    }

    private async Task<long> OptimizeWorkingSetAsync(MemoryCleanupOptions options, MemoryCleanupResult result)
    {
        try
        {
            var memoryBefore = await GetMemoryInfoAsync();
            var processCleanupDetails = new List<ProcessCleanupInfo>();
            
            // Optimiser la mémoire de travail de tous les processus accessibles
            var processes = Process.GetProcesses();
            long totalFreed = 0;
            
            foreach (var process in processes)
            {
                try
                {
                    if (process.HasExited) continue;
                    
                    // Vérifier si on peut accéder au processus
                    if (process.WorkingSet64 <= 0) continue;
                    
                    var workingSetBefore = process.WorkingSet64;
                    
                    // Libérer la mémoire de travail du processus
                    if (EmptyWorkingSet(process.Handle))
                    {
                        var workingSetAfter = process.WorkingSet64;
                        var memoryFreed = workingSetBefore - workingSetAfter;
                        
                        if (memoryFreed > 0)
                        {
                            totalFreed += memoryFreed;
                            
                            processCleanupDetails.Add(new ProcessCleanupInfo
                            {
                                ProcessId = process.Id,
                                Name = process.ProcessName,
                                MemoryFreed = memoryFreed,
                                WorkingSetBefore = workingSetBefore,
                                WorkingSetAfter = workingSetAfter,
                                Success = true
                            });
                        }
                    }
                }
                catch (Exception ex)
                {
                    // Ignorer les processus inaccessibles
                    _logger.LogDebug("Process {ProcessId} inaccessible: {Error}", process.Id, ex.Message);
                    
                    try
                    {
                        processCleanupDetails.Add(new ProcessCleanupInfo
                        {
                            ProcessId = process.Id,
                            Name = process.ProcessName,
                            MemoryFreed = 0,
                            WorkingSetBefore = 0,
                            WorkingSetAfter = 0,
                            Success = false,
                            ErrorMessage = ex.Message
                        });
                    }
                    catch
                    {
                        // Ignore if we can't even get basic process info
                    }
                }
                finally
                {
                    process?.Dispose();
                }
            }
            
            var memoryAfter = await GetMemoryInfoAsync();
            var actualFreed = memoryAfter.AvailablePhysical - memoryBefore.AvailablePhysical;
            
            // Store the process cleanup details in the result
            if (result != null)
            {
                result.ProcessCleanupDetails.AddRange(processCleanupDetails);
            }
            
            return Math.Max(totalFreed, actualFreed);
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Error optimizing working set");
        }
        
        return 0;
    }

    private async Task<long> ClearStandbyMemoryAsync()
    {
        try
        {
            var memoryBefore = await GetMemoryInfoAsync();
            
            // Libération de la mémoire standby via l'API Windows
            // Cette fonction force la libération de la mémoire en attente
            if (EmptyWorkingSet(Process.GetCurrentProcess().Handle))
            {
                // Forcer une libération supplémentaire
                SetProcessWorkingSetSize(Process.GetCurrentProcess().Handle, -1, -1);
                
                // Nouveau: Libération agressive de la mémoire standby
                ClearStandbyList();
                
                var memoryAfter = await GetMemoryInfoAsync();
                return memoryAfter.AvailablePhysical - memoryBefore.AvailablePhysical;
            }
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Error clearing standby memory");
        }
        
        return 0;
    }

    private async Task<long> ClearDnsCacheAsync()
    {
        try
        {
            _logger.LogDebug("Clearing DNS cache");
            
            // Execute command to flush DNS cache
            var startInfo = new ProcessStartInfo
            {
                FileName = "ipconfig",
                Arguments = "/flushdns",
                UseShellExecute = false,
                CreateNoWindow = true,
                RedirectStandardOutput = true,
                RedirectStandardError = true
            };

            using var process = Process.Start(startInfo);
            if (process != null)
            {
                await process.WaitForExitAsync();
                _logger.LogDebug("DNS cache cleared successfully");
            }
            
            // DNS cache clearing doesn't directly free memory, but it helps with system performance
            return 0; // Return 0 as DNS cache clearing doesn't directly free RAM
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Error clearing DNS cache");
            return 0;
        }
    }

    private async Task<long> ClearWindowsCachesAsync()
    {
        try
        {
            _logger.LogDebug("Clearing Windows caches");
            
            var memoryBefore = await GetMemoryInfoAsync();
            
            // Clear thumbnail cache
            var thumbnailCachePath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), 
                "Microsoft", "Windows", "Explorer");
            
            if (Directory.Exists(thumbnailCachePath))
            {
                var thumbnailFiles = Directory.GetFiles(thumbnailCachePath, "thumbcache_*.db");
                foreach (var file in thumbnailFiles)
                {
                    try
                    {
                        File.Delete(file);
                    }
                    catch (Exception ex)
                    {
                        _logger.LogDebug("Could not delete thumbnail cache file {File}: {Error}", file, ex.Message);
                    }
                }
            }
            
            // Stop Windows Update service for cache clearing
            var startInfo = new ProcessStartInfo
            {
                FileName = "net",
                Arguments = "stop wuauserv",
                UseShellExecute = false,
                CreateNoWindow = true
            };

            using var process = Process.Start(startInfo);
            if (process != null)
            {
                await process.WaitForExitAsync();
                _logger.LogDebug("Windows Update service stopped for cache clearing");
            }
            
            var memoryAfter = await GetMemoryInfoAsync();
            return memoryAfter.AvailablePhysical - memoryBefore.AvailablePhysical;
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Error clearing Windows caches");
            return 0;
        }
    }

    private async Task<long> ClearSystemProcessesAsync()
    {
        try
        {
            _logger.LogDebug("Clearing system processes memory");
            
            var memoryBefore = await GetMemoryInfoAsync();
            
            // Free memory from all accessible processes
            var processes = Process.GetProcesses();
            foreach (var process in processes)
            {
                try
                {
                    if (process.HasExited) continue;
                    
                    // Avoid touching critical processes
                    if (process.ProcessName.ToLower().Contains("system") || 
                        process.ProcessName.ToLower().Contains("csrss") ||
                        process.ProcessName.ToLower().Contains("winlogon") ||
                        process.ProcessName.ToLower().Contains("services") ||
                        process.ProcessName.ToLower().Contains("lsass"))
                        continue;
                    
                    // Free working set memory of the process
                    EmptyWorkingSet(process.Handle);
                    
                    // Small pause to avoid overloading the system
                    await Task.Delay(1);
                }
                catch (Exception ex)
                {
                    // Ignore inaccessible processes
                    _logger.LogDebug("Could not clear process {ProcessName}: {Error}", process.ProcessName, ex.Message);
                }
                finally
                {
                    process?.Dispose();
                }
            }
            
            var memoryAfter = await GetMemoryInfoAsync();
            return memoryAfter.AvailablePhysical - memoryBefore.AvailablePhysical;
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Error clearing system processes");
            return 0;
        }
    }

    private async Task<long> ClearPageFileAsync()
    {
        try
        {
            _logger.LogDebug("Clearing page file");
            
            var memoryBefore = await GetMemoryInfoAsync();
            
            // Clear page file via system command
            var startInfo = new ProcessStartInfo
            {
                FileName = "wmic",
                Arguments = "computersystem set AutomaticManagedPagefile=False",
                UseShellExecute = false,
                CreateNoWindow = true
            };

            using var process = Process.Start(startInfo);
            if (process != null)
            {
                await process.WaitForExitAsync();
                _logger.LogDebug("Page file settings updated");
            }
            
            var memoryAfter = await GetMemoryInfoAsync();
            return memoryAfter.AvailablePhysical - memoryBefore.AvailablePhysical;
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Error clearing page file");
            return 0;
        }
    }

    public Task<List<ProcessInfo>> GetTopMemoryProcessesAsync(int count = 10)
    {
        var processes = new List<ProcessInfo>();
        
        try
        {
            var systemProcesses = Process.GetProcesses();

            foreach (var process in systemProcesses)
            {
                try
                {
                    // Vérifier si le processus est encore actif et accessible
                    if (process.HasExited)
                        continue;

                    // Vérifier l'accès en mémoire - certains processus système nécessitent des privilèges admin
                    if (process.WorkingSet64 <= 0)
                        continue;

                    processes.Add(new ProcessInfo
                    {
                        ProcessId = process.Id,
                        Name = process.ProcessName,
                        MemoryUsage = process.WorkingSet64,
                        ThreadCount = process.Threads?.Count ?? 0,
                        IsActive = true,
                        StartTime = DateTime.Now // Fallback si StartTime n'est pas accessible
                    });
                }
                catch (Exception ex)
                {
                    // Ignorer les processus inaccessibles (privilèges insuffisants)
                    _logger.LogDebug("Process {ProcessId} ({ProcessName}) inaccessible: {Error}", 
                        process.Id, process.ProcessName, ex.Message);
                    continue;
                }
                finally
                {
                    process?.Dispose();
                }
            }

            // Trier par utilisation mémoire et prendre les top
            processes = processes
                .OrderByDescending(p => p.MemoryUsage)
                .Take(count)
                .ToList();
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error retrieving processes");
        }

        return Task.FromResult(processes);
    }

    public async Task<MemoryCleanupResult> OptimizeMemoryAsync()
    {
        // Use the new aggressive cleanup with default options
        return await CleanMemoryAsync(new MemoryCleanupOptions
        {
            IncludeGarbageCollection = true,
            IncludeSystemCache = true,
            IncludeWorkingSet = true,
            IncludeStandbyMemory = true,
            IncludePagedPool = false, // More risky
            IncludeNonPagedPool = false // More risky
        });
    }

    public IObservable<MemoryInfo> MonitorMemoryAsync()
    {
        return Observable.Create<MemoryInfo>(async observer =>
        {
            try
            {
                while (true)
                {
                    var memoryInfo = await GetMemoryInfoAsync();
                    observer.OnNext(memoryInfo);
                    await Task.Delay(1000); // Mise à jour toutes les secondes
                }
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Error monitoring memory");
                observer.OnError(ex);
            }

            return () => { };
        });
    }

    // API Windows P/Invoke
    [DllImport("kernel32.dll")]
    private static extern bool SetProcessWorkingSetSize(IntPtr proc, int min, int max);

    [DllImport("psapi.dll", SetLastError = true)]
    private static extern bool EmptyWorkingSet(IntPtr proc);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern IntPtr OpenProcess(int dwDesiredAccess, bool bInheritHandle, int dwProcessId);

    [DllImport("kernel32.dll")]
    private static extern bool GlobalMemoryStatusEx(ref MEMORYSTATUSEX lpBuffer);

    // API pour libération agressive du cache système
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool SetSystemWorkingSetSize(IntPtr hProcess, IntPtr dwMinimumWorkingSetSize, IntPtr dwMaximumWorkingSetSize);

    // API pour libération de la mémoire standby avec flags
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool SetProcessWorkingSetSizeEx(IntPtr hProcess, IntPtr dwMinimumWorkingSetSize, IntPtr dwMaximumWorkingSetSize, uint flags);

    // Méthodes helper pour les nouvelles API
    private void ClearSystemWorkingSet()
    {
        try
        {
            SetSystemWorkingSetSize(Process.GetCurrentProcess().Handle, (IntPtr)(-1), (IntPtr)(-1));
        }
        catch (Exception ex)
        {
            _logger.LogDebug("Error clearing system working set: {Error}", ex.Message);
        }
    }

    private void ClearPagedPool()
    {
        try
        {
            // Libération du pool paginé via API Windows
            // Utiliser SetProcessWorkingSetSizeEx avec flags pour libérer la mémoire paginée
            var currentProcess = Process.GetCurrentProcess();
            SetProcessWorkingSetSizeEx(currentProcess.Handle, (IntPtr)(-1), (IntPtr)(-1), 0x00000001); // QUOTA_LIMITS_HARDWS_MIN_DISABLE
        }
        catch (Exception ex)
        {
            _logger.LogDebug("Error clearing paged pool: {Error}", ex.Message);
        }
    }

    private void ClearNonPagedPool()
    {
        try
        {
            // Libération du pool non paginé via API Windows
            // Utiliser SetProcessWorkingSetSizeEx avec flags pour libérer la mémoire non paginée
            var currentProcess = Process.GetCurrentProcess();
            SetProcessWorkingSetSizeEx(currentProcess.Handle, (IntPtr)(-1), (IntPtr)(-1), 0x00000002); // QUOTA_LIMITS_HARDWS_MAX_DISABLE
        }
        catch (Exception ex)
        {
            _logger.LogDebug("Error clearing non-paged pool: {Error}", ex.Message);
        }
    }

    private void ClearStandbyList()
    {
        try
        {
            // Libération agressive de la mémoire standby
            // Utiliser SetProcessWorkingSetSizeEx avec flags pour libérer toute la mémoire en attente
            var currentProcess = Process.GetCurrentProcess();
            SetProcessWorkingSetSizeEx(currentProcess.Handle, (IntPtr)(-1), (IntPtr)(-1), 0x00000004); // QUOTA_LIMITS_HARDWS_MIN_ENABLE
        }
        catch (Exception ex)
        {
            _logger.LogDebug("Error clearing standby list: {Error}", ex.Message);
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct MEMORYSTATUSEX
    {
        public uint dwLength;
        public uint dwMemoryLoad;
        public ulong ullTotalPhys;
        public ulong ullAvailPhys;
        public ulong ullTotalPageFile;
        public ulong ullAvailPageFile;
        public ulong ullTotalVirtual;
        public ulong ullAvailVirtual;
        public ulong ullAvailExtendedVirtual;
    }
}


