@echo off
setlocal
cd /d "%~dp0"
if not exist "target\release\op_clicker_4.exe" (
    echo No built application was found. Run build_and_run.bat first.
    pause
    exit /b 1
)
start "" "%~dp0target\release\op_clicker_4.exe"
