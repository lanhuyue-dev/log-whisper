# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

LogWhisper is a desktop log analysis tool built with **Electron + Rust** architecture. It provides powerful log parsing capabilities with a plugin system for different log formats.

### Architecture
- **Frontend**: Electron desktop application with HTML/CSS/JavaScript
- **Backend**: Rust HTTP API server (Axum framework)
- **Communication**: HTTP API calls between frontend and backend
- **Plugins**: Rust-based log parsing plugins for different formats

## Development Commands

### Application Development
```bash
# Start full application (recommended)
npm start
# or
start-electron-app.bat

# Development mode with hot reload
npm run dev
# Starts both Electron and Rust API server concurrently

# Start individual components
npm run dev:rust      # Start only Rust API server (port 3030)
npm run dev:electron  # Start only Electron application
```

### Build Commands
```bash
# Build for production
npm run build

# Build CSS styles
npm run build:css:prod

# Build Rust backend only
cd src-rust && cargo build --release --bin api-server

# Package desktop application
npm run package
npm run dist:win    # Windows installer
npm run dist:mac    # macOS package
npm run dist:linux  # Linux package
```

### Testing
```bash
# Run Rust tests
npm run test
cd src-rust && cargo test

# Integration testing
test-integration.bat
```

### Utilities
```bash
# Start API server only
npm run start:api
start-api.bat

# Clean build artifacts
npm run clean

# Quick testing (various modes)
quick-test.bat           # Browser mode for testing
start-dev.bat           # Full development startup
```

## Project Structure

```
log-whisper/
├── electron/                # Electron main process
│   ├── main.js             # Main entry point and window management
│   └── preload.js          # Secure API bridge
├── src/                    # Frontend web interface
│   ├── index.html          # Main application UI
│   ├── main.js             # Frontend JavaScript logic
│   └── style.css           # Tailwind CSS styles
├── src-rust/               # Rust API server
│   ├── src/
│   │   ├── main.rs         # HTTP API server entry point
│   │   ├── config/         # Configuration management
│   │   ├── plugins/        # Plugin system
│   │   └── examples/       # Example usage
│   └── Cargo.toml          # Rust dependencies
├── doc/                    # Documentation
├── logs/                   # Application logs
├── config.db               # Configuration database
└── package.json            # Node.js configuration
```

## Key Architecture Components

### 1. Rust API Server (`src-rust/src/main.rs`)
- HTTP API server running on port 3030
- Handles log parsing requests with plugin system
- Supports both file path and content-based parsing
- Chunked processing for large files
- Configuration management endpoints

### 2. Plugin System (`src-rust/src/plugins/`)
- Enhanced plugin manager for different log formats
- Built-in plugins: auto, mybatis, docker_json, raw
- Support for custom log parsing plugins
- Format auto-detection

### 3. Electron Main Process (`electron/main.js`)
- Window management and lifecycle
- Auto-starts Rust API server
- IPC handling for window controls
- Safe process termination

### 4. Frontend Application (`src/`)
- Modern web interface with Tailwind CSS
- Drag-and-drop file handling
- Real-time log parsing and display
- Theme configuration support

## API Endpoints

### Core APIs
- `GET /health` - Health check
- `GET /api/plugins` - List available plugins
- `POST /api/parse` - Parse log content (supports chunking)
- `POST /api/test` - Test parsing endpoint

### Configuration APIs
- `GET/POST /api/config/theme` - Theme configuration
- `GET /api/config/parse` - Parsing configuration
- `GET /api/config/plugin` - Plugin configuration
- `GET /api/config/window` - Window configuration
- `GET /api/config/all` - All configurations

## Supported Log Formats

- **Auto**: Automatic format detection
- **MyBatis**: SQL log parsing with parameters
- **Docker JSON**: Container log format (JSON with stream/time/log fields)
- **Raw**: Plain text logs
- **SpringBoot**: Java application logs with stack traces

## Development Workflow

1. **Initial Setup**: Run `npm install` and ensure Rust is installed
2. **Development**: Use `npm run dev` for hot reload of both frontend and backend
3. **Testing**: Test individual components with `npm run dev:rust` or `npm run dev:electron`
4. **Building**: Use `npm run build` before packaging with `npm run dist`
5. **Configuration**: Modify settings via API endpoints or config.db

## Important Notes

- The application **must** run in Electron environment, not directly in browser
- Rust API server starts automatically when Electron launches
- Port 3030 must be available for API communication
- Configuration is stored in SQLite database (`config.db`)
- Log files are processed in chunks for large files (>1000 lines)
- Plugin system supports custom parsers via Rust traits

## Migration Context

This project migrated from Tauri to Electron + Rust architecture for improved:
- Environment detection reliability
- Development experience and debugging
- Clear separation of concerns
- Stable HTTP-based communication

The migration maintained all core functionality while improving stability and developer experience.