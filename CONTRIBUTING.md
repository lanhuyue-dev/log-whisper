# Contributing to LogWhisper

Thank you for your interest in contributing to LogWhisper! This document provides guidelines and information for contributors.

## 🤝 How to Contribute

### Reporting Issues

1. **Search existing issues** before creating a new one
2. **Use the issue templates** when reporting bugs or requesting features
3. **Provide detailed information** including:
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details (OS, LogWhisper version)
   - Sample log files (if applicable)

### Development Workflow

1. **Fork the repository**
   ```bash
   git clone https://github.com/your-username/log-whisper.git
   cd log-whisper
   ```

2. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Set up development environment**
   ```bash
   # Install dependencies
   npm install
   cd src-tauri && cargo build
   cd ..

   # Start development
   npm run dev
   ```

4. **Make your changes**
   - Follow the coding standards below
   - Add tests for new functionality
   - Update documentation as needed

5. **Test your changes**
   ```bash
   # Run tests
   npm run test
   cd src-tauri && cargo test
   ```

6. **Commit your changes**
   ```bash
   git add .
   git commit -m "feat: add new log parser feature"
   ```

7. **Push and create a Pull Request**
   ```bash
   git push origin feature/your-feature-name
   ```

## 📝 Coding Standards

### Rust Code (src-tauri)

- **Formatting**: Use `cargo fmt`
- **Linting**: Use `cargo clippy -- -D warnings`
- **Documentation**: Add `///` doc comments for public APIs
- **Error Handling**: Use `Result` types properly
- **Performance**: Consider performance implications for log parsing

```rust
// Example of well-documented function
/// Parses a Spring Boot log line and extracts structured information
///
/// # Arguments
/// * `line` - The raw log line to parse
///
/// # Returns
/// * `Result<LogEntry, ParseError>` - Parsed log entry or error
///
/// # Examples
/// ```
/// let entry = parse_springboot_line("2024-01-01 10:00:00 [main] INFO App - Started")?;
/// ```
pub fn parse_springboot_line(line: &str) -> Result<LogEntry, ParseError> {
    // Implementation
}
```

### JavaScript/TypeScript Code (src)

- **Formatting**: Use ESLint (configured in the project)
- **Naming**: Use camelCase for variables and functions
- **Documentation**: Add JSDoc comments for functions
- **Error Handling**: Use try-catch blocks appropriately

```javascript
/**
 * Handles file drag and drop events
 * @param {DragEvent} event - The drag event
 * @param {Function} callback - Callback function with file data
 */
function handleFileDrop(event, callback) {
  try {
    const files = event.dataTransfer.files;
    callback(files);
  } catch (error) {
    console.error('Error handling file drop:', error);
  }
}
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(springboot): add intelligent log prefix compaction

Fixes #123
```

```
fix(parser): handle malformed timestamps gracefully

Closes #45
```

## 🧪 Testing

### Running Tests

```bash
# Frontend tests
npm run test

# Rust tests
cd src-tauri && cargo test

# All tests
npm run test:all
```

### Writing Tests

- **Rust**: Write unit tests for log parsers and core functionality
- **JavaScript**: Write tests for UI components and file handling
- **Integration**: Test complete log parsing workflows

### Test Data

- Place test log files in `tests/fixtures/`
- Use realistic but anonymized log data
- Include edge cases and error conditions

## 📋 Development Guidelines

### Log Parser Development

When creating new log parsers:

1. **Implement the `LogParser` trait**
2. **Add comprehensive tests**
3. **Update documentation**
4. **Consider performance for large files**
5. **Handle edge cases gracefully**

```rust
use crate::plugins::{LogParser, ParseResult, ParseRequest};

pub struct CustomParser;

impl LogParser for CustomParser {
    fn name(&self) -> &str {
        "custom"
    }

    fn description(&self) -> &str {
        "Custom log format parser"
    }

    fn can_parse(&self, content: &str, file_path: Option<&str>) -> bool {
        // Implementation
    }

    fn parse(&self, content: &str, request: &ParseRequest) -> Result<ParseResult, String> {
        // Implementation
    }
}
```

### UI Development

- Use responsive design principles
- Follow existing component patterns
- Ensure accessibility (ARIA labels, keyboard navigation)
- Test with different screen sizes

### Performance Considerations

- Log parsing should handle large files efficiently
- Use streaming for very large files when appropriate
- Implement proper error handling without crashing
- Consider memory usage for long-running operations

## 📖 Documentation

### Updating Documentation

- **README.md**: Update for major feature changes
- **CLAUDE.md**: Update for development workflow changes
- **API Documentation**: Use Rust doc comments
- **User Guides**: Update in `docs/` directory

### Documentation Style

- Use clear, concise language
- Include code examples
- Add screenshots for UI changes
- Provide step-by-step instructions

## 🚀 Release Process

### Version Bumping

Update version numbers in:
- `package.json` (frontend)
- `src-tauri/Cargo.toml` (Rust backend)
- `README.md` (documentation)

### Pull Request Requirements

- **Tests pass**: All tests must pass
- **Code reviewed**: At least one maintainer review
- **Documentation updated**: For user-facing changes
- **No breaking changes**: Unless absolutely necessary

## 💬 Getting Help

- **GitHub Issues**: For bug reports and feature requests
- **Discussions**: For general questions and ideas
- **Discord**: [Invite link if available]

## 📄 License

By contributing to LogWhisper, you agree that your contributions will be licensed under the Apache License 2.0.

---

Thank you for contributing to LogWhisper! 🎉