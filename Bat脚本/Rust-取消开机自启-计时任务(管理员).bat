@echo off
setlocal EnableDelayedExpansion

schtasks /query /tn "AutoLogin" >nul 2>&1
if %errorlevel% equ 0 (
    schtasks /delete /tn "AutoLogin" /f
    echo Success: Auto-start task has been removed.
) else (
    echo Notice: Auto-start task not found.
)

pause
