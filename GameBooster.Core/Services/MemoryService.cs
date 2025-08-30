using System.Diagnostics;
using System.Reactive.Linq;
using GameBooster.Common.Interfaces;
using GameBooster.Common.Models;
using Microsoft.Extensions.Logging;
using System.Runtime.Versioning;
using System.Runtime.InteropServices;

namespace GameBooster.Core.Services;

/// <summary>
/// Service de gestion de la mémoire système
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

    public async Task<CleanupResult> CleanMemoryAsync()
    {
        var stopwatch = Stopwatch.StartNew();
        var result = new CleanupResult
        {
            Type = CleanupType.Memory,
            Timestamp = DateTime.Now
        };

        try
        {
            // Force le garbage collection
            GC.Collect();
            GC.WaitForPendingFinalizers();
            GC.Collect();

            // Libère la mémoire système
            var memoryBefore = await GetMemoryInfoAsync();
            
            // Appel à la fonction Windows pour libérer la mémoire
            SetProcessWorkingSetSize(Process.GetCurrentProcess().Handle, -1, -1);
            
            var memoryAfter = await GetMemoryInfoAsync();
            
            result.SpaceFreed = memoryAfter.AvailablePhysical - memoryBefore.AvailablePhysical;
            result.Success = true;
            
            _logger.LogInformation("Nettoyage mémoire terminé. Espace libéré: {SpaceFreed} MB", 
                result.SpaceFreedMB);
        }
        catch (Exception ex)
        {
            result.Success = false;
            result.ErrorMessage = ex.Message;
            _logger.LogError(ex, "Erreur lors du nettoyage mémoire");
        }
        finally
        {
            stopwatch.Stop();
            result.DurationMs = stopwatch.ElapsedMilliseconds;
        }

        return result;
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
                    _logger.LogDebug("Processus {ProcessId} ({ProcessName}) inaccessible: {Error}", 
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
            _logger.LogError(ex, "Erreur lors de la récupération des processus");
        }

        return Task.FromResult(processes);
    }

    public async Task<CleanupResult> OptimizeMemoryAsync()
    {
        var stopwatch = Stopwatch.StartNew();
        var result = new CleanupResult
        {
            Type = CleanupType.Memory,
            Timestamp = DateTime.Now
        };

        try
        {
            // Optimisation de la mémoire système
            var memoryBefore = await GetMemoryInfoAsync();
            
            // Libère la mémoire cache
            EmptyWorkingSet(Process.GetCurrentProcess().Handle);
            
            var memoryAfter = await GetMemoryInfoAsync();
            
            result.SpaceFreed = memoryAfter.AvailablePhysical - memoryBefore.AvailablePhysical;
            result.Success = true;
            
            _logger.LogInformation("Optimisation mémoire terminée. Espace libéré: {SpaceFreed} MB", 
                result.SpaceFreedMB);
        }
        catch (Exception ex)
        {
            result.Success = false;
            result.ErrorMessage = ex.Message;
            _logger.LogError(ex, "Erreur lors de l'optimisation mémoire");
        }
        finally
        {
            stopwatch.Stop();
            result.DurationMs = stopwatch.ElapsedMilliseconds;
        }

        return result;
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
                _logger.LogError(ex, "Erreur lors de la surveillance de la mémoire");
                observer.OnError(ex);
            }

            return () => { };
        });
    }

    [System.Runtime.InteropServices.DllImport("kernel32.dll")]
    private static extern bool SetProcessWorkingSetSize(IntPtr proc, int min, int max);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool EmptyWorkingSet(IntPtr proc);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern IntPtr OpenProcess(int dwDesiredAccess, bool bInheritHandle, int dwProcessId);

    [System.Runtime.InteropServices.DllImport("kernel32.dll")]
    private static extern bool GlobalMemoryStatusEx(ref MEMORYSTATUSEX lpBuffer);

    [System.Runtime.InteropServices.StructLayout(System.Runtime.InteropServices.LayoutKind.Sequential)]
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
