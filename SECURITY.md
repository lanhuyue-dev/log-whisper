# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 1.0.x   | :white_check_mark: |

## Reporting a Vulnerability

The LogWhisper team takes security vulnerabilities seriously. We appreciate your efforts to responsibly disclose your findings.

### How to Report

**Please do NOT report security vulnerabilities through public GitHub issues.**

Instead, please send an email to: **security@log-whisper.com**

If you do not receive a response within 48 hours, please follow up via the project's maintainers.

### What to Include

Please include the following information in your report:

- **Vulnerability type**: (e.g., buffer overflow, XSS, etc.)
- **Affected versions**: Which versions of LogWhisper are affected
- **Steps to reproduce**: Detailed steps to reproduce the vulnerability
- **Impact**: Description of the potential impact
- **Proof of concept**: If available, a minimal proof of concept

### Response Process

Once you submit a vulnerability report:

1. **Acknowledgment**: We will acknowledge receipt within 48 hours
2. **Assessment**: We will assess the vulnerability and determine its severity
3. **Resolution**: We will work on a fix and coordinate disclosure
4. **Disclosure**: We will disclose the vulnerability once a fix is available

### Security Best Practices

LogWhisper is designed with security in mind:

- **Sandboxed Environment**: Tauri provides a secure sandbox for file operations
- **No Network Access**: The application does not make network requests without user consent
- **Local Processing**: All log parsing happens locally on your machine
- **Memory Safety**: Rust's memory safety features prevent many classes of vulnerabilities

### Security Features

- **File Access Control**: Restricted to user-selected files only
- **No Code Execution**: Log files are parsed as data, never executed
- **Memory Bounds**: Strict bounds checking on all file operations
- **Input Validation**: Comprehensive validation of log file formats

## Security Advisories

We will publish security advisories for resolved vulnerabilities in our [GitHub Security Advisories](https://github.com/lanhuyue-dev/log-whisper/security/advisories).

## Security Questions

If you have questions about security that don't involve reporting a vulnerability, please:

1. Check our existing documentation
2. Search existing GitHub issues and discussions
3. Create a new discussion with the "security" tag

## Threat Model

### Trust Boundaries

- **User Interface**: Trusted - directly controlled by the user
- **Log Files**: Untrusted - treated as potentially malicious input
- **Plugin System**: Semi-trusted - plugins run in the same process but are validated

### Potential Attack Vectors

We consider and protect against:

1. **Malicious Log Files**: Crafted log files designed to crash the parser
2. **Resource Exhaustion**: Extremely large files or pathological log formats
3. **Memory Corruption**: Buffer overflows and similar memory issues
4. **Path Traversal**: Attempts to access files outside user-selected directories

### Mitigations

- **Input Validation**: All inputs are validated before processing
- **Resource Limits**: Configurable limits on file size and processing time
- **Memory Safety**: Rust's ownership system prevents memory corruption
- **Error Handling**: Comprehensive error handling prevents panic conditions

---

Thank you for helping keep LogWhisper and its users safe!