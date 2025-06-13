# Changelog

## [1.1.0-pre-release] - 2025-06-13

### Added
- Experimental Linux support: basic UI, RAM monitoring (`sysinfo`), and RAM cleaning (`drop_caches` as root).
- Zig integration for cross-compilation (Linux/Windows) with example config and wrapper script.
- `src/utils.rs` module with `is_elevated()` function for admin/root privilege checking.

### Changed
- Updated `README.md` with Linux compatibility and Zig setup instructions.
- Refactored OS-specific code for better separation.

### Fixed
- Resolved Linux build errors (Windows API usage, linker issues).
- Corrected `utils` module path resolution.
- Cleaned up unused imports and variables.

## [1.0.0-pre-release] - 2025-06-11

### Added
- Initial project structure.
- RAM monitoring and cleaning functionalities.
- Basic UI with `egui`.
- Services management tab with Windows Defender controls.
