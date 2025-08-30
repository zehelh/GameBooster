namespace GameBooster.Common.Exceptions;

/// <summary>
/// Exception de base pour l'application GameBooster
/// </summary>
public class GameBoosterException : Exception
{
    /// <summary>
    /// Code d'erreur associé à l'exception
    /// </summary>
    public string? ErrorCode { get; }

    /// <summary>
    /// Initialise une nouvelle instance de GameBoosterException
    /// </summary>
    public GameBoosterException() : base() { }

    /// <summary>
    /// Initialise une nouvelle instance de GameBoosterException avec un message
    /// </summary>
    public GameBoosterException(string message) : base(message) { }

    /// <summary>
    /// Initialise une nouvelle instance de GameBoosterException avec un message et un code d'erreur
    /// </summary>
    public GameBoosterException(string message, string errorCode) : base(message)
    {
        ErrorCode = errorCode;
    }

    /// <summary>
    /// Initialise une nouvelle instance de GameBoosterException avec un message et une exception interne
    /// </summary>
    public GameBoosterException(string message, Exception innerException) : base(message, innerException) { }

    /// <summary>
    /// Initialise une nouvelle instance de GameBoosterException avec un message, un code d'erreur et une exception interne
    /// </summary>
    public GameBoosterException(string message, string errorCode, Exception innerException) : base(message, innerException)
    {
        ErrorCode = errorCode;
    }
}
