using GameBooster.Common.Models;

namespace GameBooster.Common.Interfaces;

/// <summary>
/// Interface for network management and QoS control
/// </summary>
public interface INetworkService
{
    /// <summary>
    /// Get network processes information
    /// </summary>
    Task<List<NetworkProcessInfo>> GetNetworkProcessesAsync();

    /// <summary>
    /// Set process speed limit using QoS
    /// </summary>
    Task<bool> SetProcessSpeedLimitAsync(int processId, int limitKbps);

    /// <summary>
    /// Remove process speed limit
    /// </summary>
    Task<bool> RemoveProcessLimitAsync(int processId);

    /// <summary>
    /// Get all QoS policies
    /// </summary>
    Task<List<QosPolicyInfo>> GetQosPoliciesAsync();

    /// <summary>
    /// Create QoS policy
    /// </summary>
    Task<bool> CreateQosPolicyAsync(string policyName, string appName, long throttleBits);

    /// <summary>
    /// Remove QoS policy
    /// </summary>
    Task<bool> RemoveQosPolicyAsync(string policyName);

    /// <summary>
    /// Clear all network limits
    /// </summary>
    Task<bool> ClearAllLimitsAsync();

    /// <summary>
    /// Get network statistics
    /// </summary>
    Task<NetworkStats> GetNetworkStatsAsync();

    /// <summary>
    /// Monitor network in real-time
    /// </summary>
    Task StartNetworkMonitoringAsync();

    /// <summary>
    /// Stop network monitoring
    /// </summary>
    Task StopNetworkMonitoringAsync();

    /// <summary>
    /// Get current network usage for a process
    /// </summary>
    Task<NetworkProcessInfo?> GetProcessNetworkUsageAsync(int processId);
}
