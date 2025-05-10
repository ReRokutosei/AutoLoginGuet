@echo off
setlocal EnableDelayedExpansion

reg query "HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run" /v "AutoLogin" >nul 2>&1
if %errorlevel% equ 0 (
    reg delete "HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run" /v "AutoLogin" /f
    echo Success: Auto-start registry entry has been removed.
) else (
    echo Notice: Auto-start registry entry not found.
)

:: 删除生成的 VBS 脚本文件
set "current_dir=%~dp0"
set "vbs_name=AutoLogin.vbs"
set "vbs_path=%current_dir%%vbs_name%"
if exist "%vbs_path%" (
    del /q "%vbs_path%"
    echo Info: VBS script file has been deleted.
) else (
    echo Info: VBS script file not found.
)

pause
