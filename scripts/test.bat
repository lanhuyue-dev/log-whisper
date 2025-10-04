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

REM Check if Node.js is installed
where node >nul 2>nul
if %errorlevel% neq 0 (
    echo Node.js is not installed. Please install Node.js first.
    echo Visit https://nodejs.org/ to install Node.js
    pause
    exit /b 1
)

REM Run Rust API tests
echo Running Rust API tests...
cd src-rust
cargo test
if %errorlevel% neq 0 (
    echo Rust API tests failed
    pause
    exit /b 1
)
cd ..

REM Run Node.js tests (if any)
echo Running Node.js tests...
npm test
if %errorlevel% neq 0 (
    echo Node.js tests failed
    pause
    exit /b 1
)

echo All tests passed successfully!
pause
