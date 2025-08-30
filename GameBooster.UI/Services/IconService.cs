using System;
using System.Collections.Generic;
using System.IO;
using System.Reflection;
using System.Text.RegularExpressions;
using Microsoft.Extensions.Logging;

namespace GameBooster.UI.Services;

/// <summary>
/// Service for managing SVG icons and mapping emojis to Heroicons
/// </summary>
public static class IconService
{
    private static readonly Dictionary<string, string> EmojiToIconMapping = new()
    {
        // Memory and Performance
        {"🧠", "cpu-chip"},
        {"⚡", "bolt"},
        {"📊", "chart-bar"},
        {"🔄", "arrow-path"},
        {"💾", "computer-desktop"},
        
        // Disk and Storage
        {"🗂️", "folder"},
        {"🌐", "globe-alt"},
        {"🗑️", "trash"},
        {"🖼️", "photo"},
        {"📁", "folder-open"},
        
        // System and Settings
        {"⚙️", "cog-6-tooth"},
        {"🔧", "wrench"},
        {"🛠️", "wrench-screwdriver"},
        {"📋", "clipboard-document-list"},
        {"📝", "pencil"},
        
        // Time and Scheduling
        {"⏰", "clock"},
        {"📅", "calendar-days"},
        {"⏸️", "pause"},
        {"▶️", "play"},
        {"⏹️", "stop"},
        
        // Status and Actions
        {"✅", "check-circle"},
        {"❌", "x-circle"},
        {"⚠️", "exclamation-triangle"},
        {"ℹ️", "information-circle"},
        {"❓", "question-mark-circle"},
        
        // Navigation and UI
        {"➕", "plus"},
        {"➖", "minus"},
        {"✏️", "pencil-square"},
        {"🔍", "magnifying-glass"},
        {"📤", "arrow-up-tray"},
        {"📥", "arrow-down-tray"},
        
        // Network and Communication
        {"📡", "signal"},
        {"🌍", "globe-americas"},
        {"🔗", "link"},
        {"📞", "phone"},
        {"✉️", "envelope"},
        
        // Security and Access
        {"🔒", "lock-closed"},
        {"🔓", "lock-open"},
        {"🛡️", "shield-check"},
        {"🔑", "key"},
        {"👤", "user"},
        
        // Data and Analytics
        {"📈", "chart-bar-square"},
        {"📉", "arrow-trending-down"},
        {"📊", "presentation-chart-bar"},
        {"🔢", "calculator"},
        {"📋", "clipboard"},
        
        // Default fallback
        {"", "question-mark-circle"}
    };

    private static readonly ILogger? Logger = null; // Static service, no DI available

    /// <summary>
    /// Gets the Heroicon name for a given emoji
    /// </summary>
    public static string GetIconNameForEmoji(string emoji)
    {
        if (string.IsNullOrEmpty(emoji))
            return "question-mark-circle";

        return EmojiToIconMapping.TryGetValue(emoji, out var iconName) 
            ? iconName 
            : "question-mark-circle";
    }

    /// <summary>
    /// Gets the SVG path data for a given icon name
    /// </summary>
    public static string GetIconPathData(string iconName)
    {
        try
        {
            var pathData = LoadSvgPathData(iconName);
            return pathData ?? "M12 6v6h4.5m4.5 0a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"; // Default clock icon
        }
        catch (Exception ex)
        {
            Logger?.LogError(ex, "Error loading icon path data for {IconName}", iconName);
            return "M12 6v6h4.5m4.5 0a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"; // Default clock icon
        }
    }

    /// <summary>
    /// Gets the SVG path data for a given emoji
    /// </summary>
    public static string GetIconPathDataForEmoji(string emoji)
    {
        var iconName = GetIconNameForEmoji(emoji);
        return GetIconPathData(iconName);
    }

    /// <summary>
    /// Loads SVG content and extracts path data
    /// </summary>
    private static string? LoadSvgPathData(string iconName)
    {
        // First try to load from file system
        var filePath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Assets", "HeroIcon", $"{iconName}.svg");
        
        if (File.Exists(filePath))
        {
            var svgContent = File.ReadAllText(filePath);
            return ExtractPathDataFromSvg(svgContent);
        }

        // Then try embedded resources
        var assembly = Assembly.GetExecutingAssembly();
        var resourceName = $"GameBooster.UI.Assets.HeroIcon.{iconName}.svg";
        
        try
        {
            using var stream = assembly.GetManifestResourceStream(resourceName);
            if (stream != null)
            {
                using var reader = new StreamReader(stream);
                var svgContent = reader.ReadToEnd();
                return ExtractPathDataFromSvg(svgContent);
            }
        }
        catch (Exception ex)
        {
            Logger?.LogWarning(ex, "Could not load embedded resource {ResourceName}", resourceName);
        }

        return null;
    }

    /// <summary>
    /// Extracts path data from SVG content
    /// </summary>
    private static string? ExtractPathDataFromSvg(string svgContent)
    {
        try
        {
            // Simple regex to extract path data from SVG
            var pathMatch = Regex.Match(svgContent, @"<path[^>]*d=""([^""]*)""[^>]*>");
            if (pathMatch.Success)
            {
                return pathMatch.Groups[1].Value;
            }

            // Try alternative pattern
            pathMatch = Regex.Match(svgContent, @"d=""([^""]*)""");
            if (pathMatch.Success)
            {
                return pathMatch.Groups[1].Value;
            }

            return null;
        }
        catch (Exception ex)
        {
            Logger?.LogError(ex, "Error extracting path data from SVG content");
            return null;
        }
    }

    /// <summary>
    /// Gets all available icon names
    /// </summary>
    public static List<string> GetAvailableIconNames()
    {
        var iconNames = new List<string>();
        
        try
        {
            var iconDirectory = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Assets", "HeroIcon");
            if (Directory.Exists(iconDirectory))
            {
                var svgFiles = Directory.GetFiles(iconDirectory, "*.svg");
                foreach (var file in svgFiles)
                {
                    var fileName = Path.GetFileNameWithoutExtension(file);
                    iconNames.Add(fileName);
                }
            }
        }
        catch (Exception ex)
        {
            Logger?.LogError(ex, "Error getting available icon names");
        }

        return iconNames;
    }

    /// <summary>
    /// Checks if an icon exists
    /// </summary>
    public static bool IconExists(string iconName)
    {
        if (string.IsNullOrEmpty(iconName))
            return false;

        var filePath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Assets", "HeroIcon", $"{iconName}.svg");
        return File.Exists(filePath);
    }
}
