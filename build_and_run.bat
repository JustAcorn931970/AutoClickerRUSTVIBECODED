@echo off
setlocal EnableExtensions
cd /d "%~dp0"
title OP Clicker 4.0 - Build and Run

echo.
echo  ======================================
echo       OP CLICKER 4.0 - RUST EDITION
echo  ======================================
echo.

where cargo >nul 2>nul
if errorlevel 1 (
    echo Rust was not found. Installing the official Rust stable toolchain...
    echo This requires an internet connection.
    powershell -NoProfile -ExecutionPolicy Bypass -Command "$ErrorActionPreference='Stop'; $installer=Join-Path $env:TEMP 'rustup-init.exe'; Invoke-WebRequest 'https://win.rustup.rs/x86_64' -OutFile $installer; Start-Process -FilePath $installer -ArgumentList '-y --profile minimal --default-toolchain stable' -Wait"
    if errorlevel 1 (
        echo.
        echo Rust installation did not complete. Install it from https://rustup.rs/ and run this file again.
        pause
        exit /b 1
    )
)

set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
where cargo >nul 2>nul
if errorlevel 1 (
    echo Cargo is still unavailable. Close this window, reopen it, and try again.
    pause
    exit /b 1
)

echo.
echo Building OP Clicker 4.0 in release mode...
echo The first build downloads Rust dependencies and can take a while.
echo.
cargo build --release -j 2
if errorlevel 1 (
    echo.
    echo BUILD FAILED.
    echo If the error mentions link.exe or the MSVC linker, install Visual Studio Build Tools
    echo with the "Desktop development with C++" workload, then run this file again.
    echo See README.md for details.
    pause
    exit /b 1
)

echo.
echo Build successful. Starting OP Clicker 4.0...
start "" "%~dp0target\release\op_clicker_4.exe"
exit /b 0
