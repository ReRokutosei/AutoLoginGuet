@echo off
setlocal EnableDelayedExpansion

set "current_dir=%~dp0"
set "ps1_name=only-login.ps1"
set "vbs_name=AutoLogin.vbs"
set "full_path=%current_dir%%ps1_name%"

if not exist "!full_path!" (
    echo Error: %ps1_name% file not found
    pause
    exit /b 1
)

set "vbs_path=%current_dir%%vbs_name%"
(
    echo Set WshShell = CreateObject^("WScript.Shell"^)
    echo WshShell.Run chr^(34^) ^& "!full_path!" ^& chr^(34^), 0, False
    echo Set WshShell = Nothing
) > "!vbs_path!"

if not exist "%vbs_path%" (
    echo Error: Failed to create %vbs_name%.
    pause
    exit /b 1
)

set "reg_cmd=reg add "HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run" /v "AutoLoginGUET" /t REG_SZ /d "\"wscript.exe\" \"%vbs_path%\"" /f"

%reg_cmd%
if %errorlevel% equ 0 (
    echo Success
) else (
    echo Failed
)

pause