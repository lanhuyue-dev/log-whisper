# Changelog

All notable changes to LogWhisper will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Intelligent log prefix compaction for SpringBoot logs
- Smart formatting based on log content analysis
- Thread name abbreviation (e.g., `nio-8080-exec-1` → `H8080-`)
- Class name shortening for better readability
- Automatic format selection between compact and full display

### Changed
- Improved SpringBoot log parser performance
- Enhanced regex compilation with pre-computed patterns
- Updated log format consistency across plugins
- Streamlined user interface for better log visibility

### Fixed
- Log timestamp ISO 8601 conversion issues
- Stream type determination for different log levels
- Test assertion failures in format validation
- Memory allocation optimizations

## [1.0.0] - 2025-10-14

### Added
- Initial release of LogWhisper desktop application
- Tauri + Rust architecture for high-performance log processing
- Multi-format log parsing support:
  - Auto-detection of log formats
  - MyBatis SQL log parser
  - Docker JSON log parser
  - Raw text log parser
  - SpringBoot application log parser
- Drag-and-drop file interface
- Real-time log parsing with progress indicators
- Log level filtering (ERROR, WARN, INFO, DEBUG)
- Export functionality for parsed results
- Performance monitoring and statistics
- Plugin system for extensible log parsers
- Cross-platform support (Windows, macOS, Linux)

### Features
- **High Performance**: Optimized parsing engine capable of processing large log files
- **Smart Detection**: Automatic identification of log formats
- **User-Friendly Interface**: Intuitive drag-and-drop design
- **Flexible Filtering**: Filter logs by level, time range, and keywords
- **Export Options**: Save parsed results in multiple formats
- **Plugin Architecture**: Easy extension with custom parsers

### Technical Highlights
- **Backend**: Rust-based processing engine with Tauri integration
- **Frontend**: Modern web interface with Tailwind CSS
- **Performance**: Optimized for handling files up to 100MB+
- **Memory Efficiency**: Streaming parser for large files
- **Security**: Sandboxed environment for safe log processing

### Supported Log Formats
- **SpringBoot**: Java application logs with stack trace support
- **Docker JSON**: Container logs with stream metadata
- **MyBatis**: SQL execution logs with parameter binding
- **Raw Text**: Generic text-based log files
- **Auto Detection**: Intelligent format identification

## [0.9.0] - 2025-09-30 (Development Phase)

### Added
- Initial project scaffolding
- Basic Tauri application structure
- Core plugin system architecture
- Development environment setup

### Changed
- Migrated from Electron to Tauri for better performance
- Implemented Rust-based log parsing engine
- Established plugin development framework

---

## Version History Summary

### Major Changes
- **v1.0.0**: Production-ready release with comprehensive log parsing capabilities
- **v0.9.0**: Development phase with core architecture establishment

### Key Improvements Over Time
1. **Performance**: From basic parsing to optimized high-throughput processing
2. **Usability**: From command-line tool to intuitive desktop application
3. **Extensibility**: From hardcoded parsers to flexible plugin system
4. **Compatibility**: From limited formats to comprehensive log format support

### Technology Evolution
- Started with basic Electron + Node.js
- Evolved to Tauri + Rust for better performance
- Implemented sophisticated parsing algorithms
- Added intelligent format detection and processing

---

## Support

For information about previous versions or support, please:
- Check the [GitHub Releases](https://github.com/lanhuyue-dev/log-whisper/releases)
- Review the [Documentation](./README.md)
- Open an [Issue](https://github.com/lanhuyue-dev/log-whisper/issues) for questions