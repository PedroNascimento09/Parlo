@echo off
cd /d "%~dp0.."
if not exist "target\release\parlo.exe" (
    echo Building Parlo, this takes about a minute the first time...
    cargo build --release -p parlo-cli
    if errorlevel 1 (
        echo The build failed. Run cargo build --release -p parlo-cli to see why.
        pause
        exit /b 1
    )
)
"target\release\parlo.exe" app %*
pause
