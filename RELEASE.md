# Release Guide

This document provides instructions for releasing LogWhisper.

## Prerequisites

1. **Version Bumping**: Update version numbers in all files
2. **Testing**: Ensure all tests pass
3. **Documentation**: Update changelog and documentation
4. **Build Verification**: Verify builds on all platforms

## Pre-Release Checklist

### 1. Version Updates

Update version numbers in:
- `package.json` (frontend)
- `src-tauri/Cargo.toml` (Rust backend)
- `src-tauri/tauri.conf.json` (Tauri config)

### 2. Documentation Updates

- Update `CHANGELOG.md` with new features
- Update `README.md` if needed
- Verify all links and documentation

### 3. Testing

```bash
# Run all tests
npm run test
cd src-tauri && cargo test

# Check formatting and linting
cargo fmt --all -- --check
cargo clippy -- -D warnings
```

### 4. Build Verification

```bash
# Build CSS
npm run build:css:prod

# Test build on current platform
npm run dist
```

## Release Process

### 1. Create Release Branch

```bash
git checkout -b release/v1.0.0
```

### 2. Final Testing

- Manual testing of all log formats
- Performance testing with large files
- UI/UX testing on different screen sizes

### 3. Tag Release

```bash
git commit -m "chore: prepare v1.0.0 release"
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0
```

### 4. Build Release Packages

**Windows:**
```bash
npm run dist:win
```

**macOS:**
```bash
npm run dist:mac
```

**Linux:**
```bash
npm run dist:linux
```

### 5. Create GitHub Release

1. Go to [GitHub Releases](https://github.com/lanhuyue-dev/log-whisper/releases)
2. Click "Create a new release"
3. Choose the tag you just created
4. Add release notes from CHANGELOG.md
5. Upload built binaries:
   - `src-tauri/target/release/bundle/` directory
   - Include .exe, .dmg, .deb/.rpm files

### 6. Post-Release

- Update documentation with new download links
- Announce release on GitHub Discussions
- Update website if applicable

## Release Channels

### Stable Releases
- Version format: `v1.0.0`, `v1.1.0`, etc.
- Released from `main` branch
- Full testing and documentation

### Pre-releases (Future)
- Version format: `v1.1.0-alpha.1`, `v1.1.0-beta.1`, etc.
- Released from `develop` branch
- For testing new features

## Troubleshooting

### Build Failures
- Check Rust version: `rustc --version` (should be stable)
- Check Node.js version: `node --version` (should be 18+)
- Clear caches: `cargo clean`, `rm -rf node_modules`

### Platform-Specific Issues
- **Windows**: Install Visual Studio Build Tools
- **macOS**: Install Xcode Command Line Tools
- **Linux**: Install development dependencies

### Performance Issues
- Profile with `cargo build --release`
- Check for memory leaks in large file processing
- Optimize regex patterns if needed

## Release Communication

### What to Include in Release Notes

- New features and improvements
- Bug fixes
- Performance improvements
- Breaking changes (if any)
- Installation/upgrade instructions
- Known issues

### Communication Channels

- GitHub Release (primary)
- GitHub Discussions (announcement)
- Documentation updates

---

Remember: Always test releases thoroughly before making them public!