# mini_script

本目录脚本只包含最简单的登录和通知功能

仅适用于有相关经验的人群

## 使用方法

### Windows (PowerShell 7)

> 暂不支持古老的pwsh5

1. 安装`BurntToast`以支持通知功能：
   - `Install-Module -Name BurntToast -Scope CurrentUser`
2. 打开 [login.ps1](mini_script/win/login.ps1)
3. 填写配置
4. 保存文件
5. 执行：`.\login.ps1`

### macOS/Linux (Shell)

1. Linux 需要安装 `libnotify-bin` 以支持通知功能：
  ```bash
  # Ubuntu/Debian
  sudo apt install libnotify-bin
  
  # CentOS/RHEL/Fedora
  sudo yum install libnotify
  # 或
  sudo dnf install libnotify
  ```

2. 打开 [login.sh](mini_script/unix/login.sh)
3. 填写配置
4. 保存文件
5. 添加执行权限并运行：
   ```bash
   chmod +x login.sh
   ./login.sh
   ```

## 开机自启设置

### Windows

双击运行 [autostart.bat](mini_script/win/autostart.bat)
> 该脚本会创建一个vbs脚本来隐藏窗口，并设置启动项到注册表中

### macOS

1. 参考 项目文件 `com.autologin.guet.plist`
2. 创建 plist 文件（ `~/Library/LaunchAgents/com.autologin.guet.plist`）
3. 加载 plist 文件：
   ```bash
   launchctl load ~/Library/LaunchAgents/com.autologin.guet.plist
   ```

### Linux

1. 参考 项目文件 `autologin-guet.service`
2. 创建 systemd 服务文件 `~/.config/systemd/user/autologin-guet.service`
3. 启用并启动服务：
   ```bash
   systemctl --user daemon-reload
   systemctl --user enable autologin-guet.service
   systemctl --user start autologin-guet.service
   ```