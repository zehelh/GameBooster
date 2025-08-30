namespace GameBooster.UI.Services;

/// <summary>
/// Heroicons SVG icon service for modern UI
/// </summary>
public static class IconService
{
    // Memory related icons
    public const string Memory = "M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z";
    public const string CpuChip = "M8 2v2.5a.5.5 0 001 0V2h6v2.5a.5.5 0 001 0V2h2v2h-2.5a.5.5 0 000 1H18v6h-2.5a.5.5 0 000 1H18v2h-2v-2.5a.5.5 0 00-1 0V18H9v-2.5a.5.5 0 00-1 0V18H6v-2h2.5a.5.5 0 000-1H6V9h2.5a.5.5 0 000-1H6V6h2v2.5a.5.5 0 001 0V6h6z";
    
    // Storage related icons
    public const string CircleStack = "M20 12v8a2 2 0 01-2 2H6a2 2 0 01-2-2v-8m16 0V9a2 2 0 00-2-2H6a2 2 0 00-2 2v3m16 0l-8-4-8 4";
    public const string Trash = "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16";
    
    // Network related icons  
    public const string WifiHigh = "M2 7l4.5-4.5L18 14l-6.5 6.5L2 11.5V7z";
    public const string Signal = "M5 12V7a1 1 0 011-1h1m0 0V4a1 1 0 011-1h2a1 1 0 011 1v2m0 0h1a1 1 0 011 1v5m-6 0V9a1 1 0 011-1h2a1 1 0 011 1v3";
    
    // System related icons
    public const string Cog6Tooth = "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z";
    public const string Clock = "M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z";
    
    // Action icons
    public const string ArrowPath = "M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182m0-4.991v4.99";
    public const string Play = "M5.25 5.653c0-.856.917-1.398 1.667-.986l11.54 6.348a1.125 1.125 0 010 1.971l-11.54 6.347a1.125 1.125 0 01-1.667-.985V5.653z";
    public const string Stop = "M5.25 7.5A2.25 2.25 0 017.5 5.25h9a2.25 2.25 0 012.25 2.25v9a2.25 2.25 0 01-2.25 2.25h-9a2.25 2.25 0 01-2.25-2.25v-9z";
    
    // Status icons
    public const string CheckCircle = "M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z";
    public const string ExclamationTriangle = "M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z";
    public const string InformationCircle = "M11.25 11.25l.041-.02a.75.75 0 011.063.852l-.708 2.836a.75.75 0 001.063.853l.041-.021M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-9-3.75h.008v.008H12V8.25z";
}
