@echo off
setlocal EnableDelayedExpansion

schtasks /query /tn "AutoLoginPyw" >nul 2>&1
if %errorlevel% equ 0 (
    schtasks /delete /tn "AutoLoginPyw" /f
    echo Success: Auto-start task has been removed.
) else (
    echo Notice: Auto-start task not found.
)

pause
