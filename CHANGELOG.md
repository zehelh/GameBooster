# Changelog

## [1.1.0] - 2025-02-07

### Added
- **⏰ Complete Task Scheduler System**
  - Task creation (templates + custom), editing, deletion
  - Multiple scheduling types: Startup, Interval, Daily, Weekly, On Condition
  - Real-time execution (every 30s) with statistics tracking
  - Persistent storage in `scheduled_tasks.json`

- **🗂️ Enhanced Disk Cleaning Options**
  - Granular control: Temp files, Browser cache, Thumbnails, Recycle bin, System cache
  - Size threshold configuration (50-2000 MB)
  - Detailed cleaning reports

- **🚀 Windows Auto-Startup Integration**
  - Auto-start with Windows (Settings tab)
  - Start minimized and auto-start scheduler options
  - Configurable startup delay and registry integration

- **⚙️ UI/UX Improvements**
  - Better task management interface
  - Real-time scheduler status indicators
  - Enhanced settings organization

### Technical
- Added `TaskScheduler` with JSON persistence
- Enhanced `DiskCleaningOptions` with detailed configuration
- Improved application lifecycle and error handling

## [1.0.0-pre-release] - 2025-06-11

### Added
- Initial project structure.
- RAM monitoring and cleaning functionalities.
- Basic UI with `egui`.
- Services management tab with Windows Defender controls.
