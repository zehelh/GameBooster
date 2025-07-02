# GameBooster 🚀

## Description
GameBooster is a comprehensive PC optimization utility for Windows specifically designed to improve gaming performance. The application provides a modern and intuitive graphical interface to monitor, clean, optimize, and schedule system maintenance tasks in real-time.

## Features

### 💾 Memory Management
- 📊 Real-time RAM monitoring with graphical visualization
- 🔄 Manual and automatic memory cleaning
- 📈 Memory usage tracking and statistics

### 🗂️ Advanced Disk Cleaning
- ✅ Temporary files cleaning (system, user, browsers)
- 🌐 Browser cache clearing (Chrome, Firefox, Edge)
- 🖼️ Windows thumbnails cache removal
- 🗑️ Recycle bin emptying
- ⚙️ System cache cleaning (experimental)
- 📊 Detailed space recovery reports

### ⏰ Complete Task Scheduler
- 🎯 **Task Types**: RAM cleaning, Disk cleaning, Windows Defender toggle
- 📅 **Scheduling Options**:
  - On Startup: Execute when application starts
  - Interval: Every X minutes (5-1440 min)
  - Daily: At specific time each day
  - Weekly: On specific day and time
  - On Condition: When thresholds are met
- ⚙️ **Management**: Create, edit, delete, enable/disable tasks
- 📊 **Statistics**: Success/failure tracking and execution history
- 💾 **Persistence**: Tasks saved automatically and restored on startup

### 🚀 Windows Integration
- 🔄 Auto-start with Windows (configurable in Settings)
- 🔽 Start minimized option
- ⏰ Auto-start scheduler option
- ⏱️ Configurable startup delay
- 🔧 Windows registry integration

### 🛡️ Windows Services Management
- 🛡️ Windows Defender toggle (temporary/permanent with warnings)
- ⚙️ Service status monitoring and control

### 🌐 Network Management (WIP)
- 🚧 Real-time network process monitoring (in development)
- 🚧 Bandwidth limiting for specific processes (in development)
- 🚧 QoS policy management (experimental)

### 🎨 Modern User Interface
- 🌓 Dark/Light theme support
- 📱 Responsive and intuitive design
- 🎛️ Real-time status indicators
- 📈 Visual feedback and progress tracking

## Roadmap

### ✅ Completed Features
- ✅ **Advanced disk cleaning** - Complete with granular options
- ✅ **Advanced Scheduler** - Full CRUD operations with multiple scheduling types
- ✅ **Windows services optimization** - Defender management implemented

### 🔄 In Progress / Planned
- **Network Limiter (WIP):**
  - QoS-based bandwidth control for processes
  - Real-time network process monitoring
  - Process-specific bandwidth limiting
- **CPU Limiter:**
  - Change CPU priorities of processes
  - Impose CPU usage limitations
  - Process affinity management
- **Enhanced Network Features:**
  - Advanced traffic shaping
  - Application-specific network profiles
  - Network usage analytics
- **System Optimization:**
  - GPU memory cleaning
  - Registry optimization
  - Startup programs management
- **Gaming Profiles:**
  - Pre-configured optimization profiles for popular games
  - One-click performance mode activation
- **Advanced Reporting:**
  - System performance analytics
  - Optimization history and trends
  - Detailed system health reports
- **Creation of an installer** with auto-updater

## Prerequisites
The application is developed in Rust. Make sure you have Rust and Cargo installed.

## Installation
1. Clone the repository:
```bash
git clone https://github.com/your-username/GameBooster.git
cd GameBooster
```
2. Compile the project:
```bash
cargo build --release
```

## Usage
Launch the application from the `target/release` folder:
```bash
./gamebooster.exe 
```

### Quick Start Guide
1. **Memory Tab**: Monitor RAM usage and perform manual cleaning
2. **Disk Tab**: Scan and clean disk space with customizable options
3. **Scheduler Tab**: Create automated maintenance tasks
4. **Services Tab**: Manage Windows Defender and other services
5. **Settings Tab**: Configure auto-startup and application preferences

### Administrator Rights
The application requires administrator rights for:
- Windows Defender management
- System file cleaning
- Registry modifications for auto-startup

## Architecture
- **Frontend**: egui-based modern GUI
- **Backend**: Multi-threaded Rust with async operations
- **Persistence**: JSON-based configuration and task storage
- **Integration**: Windows API for system management
- **Network**: WinDivert for traffic management

## Contribution
Contributions are welcome! Feel free to open an issue or a pull request.

### Development Setup
1. Install Rust toolchain
2. Clone the repository
3. Run `cargo build` for development builds
4. Use `cargo test` to run the test suite

## License
This project is licensed under the MIT License. See the `LICENSE` file for more details.

## Changelog
Check the `CHANGELOG.md` file to follow the project's evolution.

## Support
For issues, feature requests, or questions, please open an issue on GitHub.
