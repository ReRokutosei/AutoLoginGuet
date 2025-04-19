@echo off
setlocal EnableDelayedExpansion

set "current_dir=%~dp0"
set "exe_name=AutoLogin.exe"

:: 检查文件存在性
if not exist "%current_dir%%exe_name%" (
    echo Error: %exe_name% not found in current directory.
    pause
    exit /b 1
)

:: 添加到注册表
reg add "HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run" /v "AutoLogin" /t REG_SZ /d "\"%current_dir%%exe_name%\"" /f
if %errorlevel% equ 0 (
    echo Success: Auto-start entry has been added to registry.
    echo The program will run automatically when you log in.
) else (
    echo Error: Failed to add registry entry.
)

pause
