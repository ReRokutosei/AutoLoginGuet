@echo off
setlocal EnableDelayedExpansion

:: 获取当前目录
set "current_dir=%~dp0"
set "exe_name=AutoLogin.exe"
set "vbs_name=AutoLogin.vbs"

:: 检查文件存在性
if not exist "%current_dir%%exe_name%" (
    echo Error: %exe_name% not found in current directory.
    pause
    exit /b 1
)

:: 构建 VBS 文件内容
set "vbs_path=%current_dir%%vbs_name%"
(
    echo Set WshShell = CreateObject^("WScript.Shell"^)
    echo WshShell.Run chr^(34^) ^& "%exe_name%" ^& chr^(34^), 0, False
    echo Set WshShell = Nothing
) > "%vbs_path%"

:: 检查 VBS 文件是否生成成功
if not exist "%vbs_path%" (
    echo Error: Failed to create %vbs_name%.
    pause
    exit /b 1
)

:: 构造注册表命令
set "reg_cmd=reg add "HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run" /v "AutoLogin" /t REG_SZ /d "\"wscript.exe\" \"%vbs_path%\"" /f"

:: 执行注册表操作
%reg_cmd%
if %errorlevel% equ 0 (
    echo Success: Auto-start entry has been added to registry.
    echo The program will run automatically when you log in.
) else (
    echo Error: Failed to add registry entry.
)

pause