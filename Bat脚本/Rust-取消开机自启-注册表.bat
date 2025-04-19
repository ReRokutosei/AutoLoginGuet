@echo off
setlocal EnableDelayedExpansion

reg query "HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run" /v "AutoLogin" >nul 2>&1
if %errorlevel% equ 0 (
    reg delete "HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run" /v "AutoLogin" /f
    echo Success: Auto-start registry entry has been removed.
) else (
    echo Notice: Auto-start registry entry not found.
)

pause
