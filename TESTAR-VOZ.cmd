@echo off
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\test-voice.ps1" %*
pause
