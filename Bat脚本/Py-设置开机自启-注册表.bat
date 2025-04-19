@echo off
setlocal EnableDelayedExpansion

:: 获取当前目录和Python环境
set "current_dir=%~dp0"

:: 检查目标文件
if not exist "%current_dir%AutoLogin.pyw" (
    echo Error: AutoLogin.pyw not found in current directory.
    pause
    exit /b 1
)

:: 查找Python环境
where pythonw.exe >nul 2>&1
if %errorlevel% equ 0 (
    for /f "delims=" %%p in ('where pythonw.exe') do set "python_path=%%p"
) else (
    if exist "%current_dir%pythonw.exe" (
        set "python_path=%current_dir%pythonw.exe"
    ) else (
        echo Error: pythonw.exe not found.
        pause
        exit /b 1
    )
)

:: 添加到注册表
reg add "HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run" /v "AutoLoginPyw" /t REG_SZ /d "\"%python_path%\" \"%current_dir%AutoLogin.pyw\"" /f
if %errorlevel% equ 0 (
    echo Success: Auto-start entry has been added to registry.
    echo The program will run automatically when you log in.
) else (
    echo Error: Failed to add registry entry.
)

pause
