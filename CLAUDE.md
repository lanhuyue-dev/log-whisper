# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

LogWhisper is a desktop log analysis tool built with **Tauri + Rust** architecture. It provides powerful log parsing capabilities with a plugin system for different log formats.

### Architecture
- **Frontend**: Web interface with HTML/CSS/JavaScript running in Tauri webview
- **Backend**: Rust backend integrated with Tauri framework
- **Communication**: Tauri invoke commands between frontend and backend
- **Plugins**: Rust-based log parsing plugins for different formats

## Development Commands

### Application Development
```bash
# Start development mode (recommended)
npm start
# or
npm run dev
# or
start-tauri-dev.bat

# Build CSS styles only
npm run build:css:prod
# or
build-styles.sh
```

### Build Commands
```bash
# Build for production
npm run build

# Package desktop application
npm run package
npm run dist:win    # Windows installer
npm run dist:mac    # macOS package
npm run dist:linux  # Linux package

# Build Tauri application only
build-tauri.bat
```

### Testing
```bash
# Run Rust tests
npm run test
cd src-tauri && cargo test

# Run specific Rust tests
cd src-tauri && cargo test -- specific_test_name

# Check code formatting and linting
cd src-tauri && cargo fmt --check
cd src-tauri && cargo clippy -- -D warnings
```

### Utilities
```bash
# Clean build artifacts
npm run clean

# Quick start
start-tauri.sh         # Linux/macOS development launcher
```

## Project Structure

```
log-whisper/
├── src/                    # Frontend web interface
│   ├── index.html          # Main application UI
│   ├── main.js             # Frontend JavaScript logic with Tauri invoke calls
│   └── style.css           # Tailwind CSS styles
├── src-tauri/              # Tauri Rust backend
│   ├── src/
│   │   ├── main.rs         # Tauri application entry point with commands
│   │   ├── config/         # Configuration management
│   │   ├── plugins/        # Plugin system
│   │   └── examples/       # Example usage
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
├── dist/                   # Built frontend assets
├── doc/                    # Documentation
├── logs/                   # Application logs
└── package.json            # Node.js configuration
```

## Key Architecture Components

### 1. Tauri Backend (`src-tauri/src/main.rs`)
- Tauri application with invoke command handlers
- Handles log parsing requests with plugin system
- Supports both file path and content-based parsing
- Chunked processing for large files
- Configuration management through commands

### 2. Plugin System (`src-tauri/src/plugins/`)
- Enhanced plugin manager for different log formats
- Built-in plugins: auto, mybatis, docker_json, raw
- Support for custom log parsing plugins
- Format auto-detection

### 3. Tauri Commands (`src-tauri/src/main.rs`)
- Window management and lifecycle through Tauri
- Log parsing commands: `parse_log`, `get_plugins`, `health_check`
- Configuration commands: `get_theme`, `set_theme`, etc.
- Safe process handling through Tauri framework

### 4. Frontend Application (`src/`)
- Modern web interface with Tailwind CSS
- Drag-and-drop file handling
- Real-time log parsing and display using Tauri invoke
- Theme configuration support

## Tauri Commands

### Core Commands
- `health_check()` - Health check
- `get_plugins()` - List available plugins
- `parse_log(request)` - Parse log content (supports chunking)
- `test_parse(request)` - Test parsing endpoint

### Configuration Commands
- `get_theme()` - Get theme configuration
- `set_theme(theme)` - Set theme configuration
- `get_parse_config()` - Get parsing configuration
- `get_plugin_config()` - Get plugin configuration
- `get_window_config()` - Get window configuration
- `get_all_config()` - Get all configurations

## Supported Log Formats

- **Auto**: Automatic format detection
- **MyBatis**: SQL log parsing with parameters
- **Docker JSON**: Container log format (JSON with stream/time/log fields)
- **Raw**: Plain text logs
- **SpringBoot**: Java application logs with stack traces

## Development Workflow

1. **Initial Setup**: Run `npm install` and ensure Rust is installed
2. **Development**: Use `npm run dev` for hot reload of both frontend and backend
3. **Testing**: Test components with `cargo test` in src-tauri directory
4. **Building**: Use `npm run build` before packaging with `npm run dist`
5. **Configuration**: Modify settings through Tauri commands

## Key Dependencies

### Rust Backend (`src-tauri/Cargo.toml`)
- **Tauri Framework**: tauri 1.0 with api-all features
- **Serialization**: serde + serde_json for JSON handling
- **Async Runtime**: tokio with full features
- **Text Processing**: regex + unicode-segmentation + encoding_rs
- **Time/UUID**: chrono + uuid for timestamps and unique IDs
- **Error Handling**: thiserror + anyhow

### Frontend Dependencies
- **CSS Framework**: Tailwind CSS 3.4.0 with forms and typography plugins
- **Build Tools**: @tauri-apps/cli for Tauri development

## Important Notes

- The application runs as a native desktop app using Tauri
- Frontend communicates with backend through Tauri invoke system, not HTTP
- Configuration is managed through Tauri commands
- Log files are processed in chunks for large files (>1000 lines)
- Plugin system supports custom parsers via Rust traits
- Use the provided scripts for development and building
- The project uses Cargo workspace for efficient dependency management

## Architecture Benefits

The Tauri + Rust architecture provides:
- **Better Performance**: Native app with webview technology
- **Smaller Bundle Size**: Compared to Electron applications
- **Secure Communication**: Invoke system instead of HTTP APIs
- **Integrated Development**: Single codebase for frontend and backend
- **Cross-Platform**: Native builds for Windows, macOS, and Linux