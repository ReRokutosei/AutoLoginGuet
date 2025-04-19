@echo off
setlocal EnableDelayedExpansion

:: 获取当前目录和用户信息
set "current_dir=%~dp0"
for /f "tokens=2 delims=\" %%i in ('whoami') do set "username=%%i"

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
    if exist "%current_dir%python.exe" (
        if exist "%current_dir%pythonw.exe" (
            set "python_path=%current_dir%pythonw.exe"
        ) else (
            echo Error: pythonw.exe not found.
            pause
            exit /b 1
        )
    ) else (
        echo Error: Python environment not found.
        pause
        exit /b 1
    )
)

:: 创建任务计划XML
set "xml_file=%temp%\autologin_task.xml"
(
echo ^<?xml version="1.0" encoding="UTF-16"?^>
echo ^<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task"^>
echo ^<RegistrationInfo^>
echo ^<Description^>Campus Network Auto Login^</Description^>
echo ^<URI^>\AutoLoginPyw^</URI^>
echo ^</RegistrationInfo^>
echo ^<Triggers^>
echo ^<LogonTrigger^>
echo ^<Enabled^>true^</Enabled^>
echo ^</LogonTrigger^>
echo ^</Triggers^>
echo ^<Principals^>
echo ^<Principal id="Author"^>
echo ^<LogonType^>InteractiveToken^</LogonType^>
echo ^<RunLevel^>LeastPrivilege^</RunLevel^>
echo ^</Principal^>
echo ^</Principals^>
echo ^<Settings^>
echo ^<MultipleInstancesPolicy^>IgnoreNew^</MultipleInstancesPolicy^>
echo ^<DisallowStartIfOnBatteries^>false^</DisallowStartIfOnBatteries^>
echo ^<StopIfGoingOnBatteries^>false^</StopIfGoingOnBatteries^>
echo ^<AllowHardTerminate^>true^</AllowHardTerminate^>
echo ^<StartWhenAvailable^>true^</StartWhenAvailable^>
echo ^<RunOnlyIfNetworkAvailable^>false^</RunOnlyIfNetworkAvailable^>
echo ^<AllowStartOnDemand^>true^</AllowStartOnDemand^>
echo ^<Enabled^>true^</Enabled^>
echo ^<Hidden^>false^</Hidden^>
echo ^<ExecutionTimeLimit^>PT72H^</ExecutionTimeLimit^>
echo ^</Settings^>
echo ^<Actions Context="Author"^>
echo ^<Exec^>
echo ^<Command^>"%python_path%"^</Command^>
echo ^<Arguments^>"%current_dir%AutoLogin.pyw"^</Arguments^>
echo ^</Exec^>
echo ^</Actions^>
echo ^</Task^>
) > "%xml_file%"

:: 删除已存在的任务并创建新任务
schtasks /query /tn "AutoLoginPyw" >nul 2>&1
if %errorlevel% equ 0 (
    schtasks /delete /tn "AutoLoginPyw" /f /IT >nul 2>&1
)

schtasks /create /tn "AutoLoginPyw" /xml "%xml_file%" /IT /F
if %errorlevel% equ 0 (
    echo Installation successful!
) else (
    echo Installation failed!
)

:: 清理临时文件
del "%xml_file%" >nul 2>&1

pause
