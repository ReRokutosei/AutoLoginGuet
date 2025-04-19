@echo off
setlocal EnableDelayedExpansion

:: 获取当前目录和用户信息
set "current_dir=%~dp0"
set "exe_name=AutoLogin.exe"

:: 检查文件存在性
if not exist "%current_dir%%exe_name%" (
    echo Error: %exe_name% not found in current directory.
    pause
    exit /b 1
)

:: 获取当前用户名
for /f "tokens=2 delims=\" %%i in ('whoami') do set "username=%%i"

:: 创建临时XML文件
set "xml_file=%temp%\autologin_task.xml"
(
echo ^<?xml version="1.0" encoding="UTF-16"?^>
echo ^<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task"^>
echo ^<RegistrationInfo^>
echo ^<Description^>Campus Network Auto Login^</Description^>
echo ^<URI^>\AutoLogin^</URI^>
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
echo ^<AllowStartOnDemand^>true^</AllowStartOnDemand^>
echo ^<Enabled^>true^</Enabled^>
echo ^<Hidden^>false^</Hidden^>
echo ^<ExecutionTimeLimit^>PT0S^</ExecutionTimeLimit^>
echo ^<Priority^>7^</Priority^>
echo ^</Settings^>
echo ^<Actions Context="Author"^>
echo ^<Exec^>
echo ^<Command^>"%current_dir%%exe_name%"^</Command^>
echo ^</Exec^>
echo ^</Actions^>
echo ^</Task^>
) > "%xml_file%"

:: 删除已存在的任务并创建新任务
schtasks /query /tn "AutoLogin" >nul 2>&1
if %errorlevel% equ 0 (
    schtasks /delete /tn "AutoLogin" /f /IT >nul 2>&1
)

schtasks /create /tn "AutoLogin" /xml "%xml_file%" /IT /F
if %errorlevel% equ 0 (
    echo Success: Auto-start task has been created successfully.
    echo The program will run automatically when you log in.
) else (
    echo Error: Failed to create auto-start task.
)

:: 清理临时文件
del "%xml_file%" >nul 2>&1

pause