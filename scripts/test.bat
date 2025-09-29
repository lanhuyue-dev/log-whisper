@echo off
echo Starting LogWhisper tests...

REM Check if Rust is installed
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo Rust is not installed. Please install Rust first.
    echo Visit https://rustup.rs/ to install Rust
    pause
    exit /b 1
)

REM Run unit tests
echo Running unit tests...
cargo test --lib
if %errorlevel% neq 0 (
    echo Unit tests failed
    pause
    exit /b 1
)

REM Run integration tests
echo Running integration tests...
cargo test --test integration
if %errorlevel% neq 0 (
    echo Integration tests failed
    pause
    exit /b 1
)

REM Run documentation tests
echo Running documentation tests...
cargo test --doc
if %errorlevel% neq 0 (
    echo Documentation tests failed
    pause
    exit /b 1
)

echo All tests passed successfully!
pause
