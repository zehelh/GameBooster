namespace GameBooster.Common.Models;

/// <summary>
/// Process priority levels
/// </summary>
public enum ProcessPriority
{
    /// <summary>
    /// Idle priority - lowest
    /// </summary>
    Idle = 64,

    /// <summary>
    /// Below normal priority
    /// </summary>
    BelowNormal = 16384,

    /// <summary>
    /// Normal priority - default
    /// </summary>
    Normal = 32,

    /// <summary>
    /// Above normal priority
    /// </summary>
    AboveNormal = 32768,

    /// <summary>
    /// High priority
    /// </summary>
    High = 128,

    /// <summary>
    /// Real-time priority - highest
    /// </summary>
    RealTime = 256
}
