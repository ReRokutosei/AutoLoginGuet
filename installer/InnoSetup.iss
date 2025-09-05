; AutoLoginGUET 安装脚本

#define MyAppName "AutoLoginGUET"
#define MyAppVersion "2.1.0"
#define MyAppPublisher "ReRokutosei"
#define MyAppURL "https://github.com/ReRokutosei/AutoLoginGUET"
#define MyAppExeName "AutoLoginGUET.exe"

[Setup]
AppId={#MyAppName}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}

; 安装目录设置
DefaultDirName={autopf}\{#MyAppName}
DisableProgramGroupPage=yes
AllowNoIcons=yes

; 文件信息
SourceDir=..
LicenseFile=LICENSE
OutputDir=.
OutputBaseFilename=AutoLoginGUET-{#MyAppVersion}-installer
Compression=lzma
SolidCompression=yes
WizardStyle=modern

; 权限设置 - 使用最低权限
PrivilegesRequired=lowest

; 支持的 Windows 版本
MinVersion=10.0.10240

; 图标设置
SetupIconFile=assets\icon.ico
; WizardImageFile=assets\setup.bmp
; WizardSmallImageFile=assets\setup-small.bmp

; 版本信息
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription=AutoLoginGUET Setup
VersionInfoTextVersion={#MyAppVersion}
VersionInfoVersion={#MyAppVersion}
VersionInfoProductName={#MyAppName}
VersionInfoProductVersion={#MyAppVersion}
VersionInfoCopyright=By ReRokutosei. All rights reserved.

[Languages]
Name: "zh_CN"; MessagesFile: ".\installer\ChineseSimplified.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}";

[Files]
; 主程序文件

; 本地路径
; Source: "target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

; workflow路径
Source: "target\x86_64-pc-windows-msvc\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Registry]

[Run]
; 安装完成后启动程序
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#MyAppName}}"; Flags: nowait postinstall skipifsilent

[Code]
{
  安装时删除旧版本的注册表项
}
procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
  begin
    RegDeleteValue(HKEY_CURRENT_USER,
                   'Software\Microsoft\Windows\CurrentVersion\Run',
                   'AutoLoginGUET.exe');
  end;
end;

{
  卸载时删除残留文件
}
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  WebView2Path, ConfigPath, LogPath: String;
begin
  if CurUninstallStep = usPostUninstall then
  begin
    // 删除 WebView2 用户数据目录
    WebView2Path := ExpandConstant('{app}\AutoLoginGUET.exe.WebView2');
    if DirExists(WebView2Path) then
      DelTree(WebView2Path, True, True, True);

    // 删除配置文件
    ConfigPath := ExpandConstant('{app}\config.toml');
    if FileExists(ConfigPath) then
      DeleteFile(ConfigPath);

    // 删除日志文件
    LogPath := ExpandConstant('{app}\AutoLogin.log');
        if FileExists(LogPath) then
          DeleteFile(LogPath);

    // 清理开机启动注册表值
    RegDeleteValue(HKEY_CURRENT_USER,
                   'Software\Microsoft\Windows\CurrentVersion\Run',
                   'AutoLoginGUET.exe');

    RegDeleteValue(HKEY_CURRENT_USER,
                   'Software\Microsoft\Windows\CurrentVersion\Run',
                   'AutoLoginGUET');
  end;
end;