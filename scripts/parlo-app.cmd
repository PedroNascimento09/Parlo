@echo off
cd /d "%~dp0.."
if not exist "target\release\parlo-app.exe" (
    echo Building Parlo, this takes a few minutes the first time...
    cargo build --release -p parlo-cli -p parlo-app
    if errorlevel 1 (
        echo The build failed. Run cargo build --release -p parlo-cli -p parlo-app to see why.
        pause
        exit /b 1
    )
)
start "" "target\release\parlo-app.exe"
