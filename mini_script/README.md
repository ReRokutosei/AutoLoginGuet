# mini_script

本目录脚本包含

1. `only-login`：
   - 仅登录功能
   - 适合**有线网**且运营商为**中国移动、联通、电信、广电**的用户

~~2. `only-flow`：~~
   ~~- 仅流量查询功能~~
   ~~- 适合**无线网**且运营商为**校园网**的用户~~
  ~~脚本有问题，不想修了~~
3. `login_and_flow`：
   - 合并登录与流量查询功能
   - 适合适合**有线网**且运营商为**校园网**的用户

> [!CAUTION]
> 三个脚本任选其一即可，所有脚本仅适用于有相关经验的人群

> [!TIP]
> 自购宽带是无限流量，所以没必要使用流量查询功能

## 使用方法

### Windows (PowerShell 7)

> **暂不支持古老的pwsh5**

1. 安装`BurntToast`以支持通知功能：
   - 在pwsh输入命令`Install-Module -Name BurntToast -Scope CurrentUser`

2. 根据自身情况选择脚本文件：
   - [only-login.ps1](mini_script/win/only-login.ps1)
   - ~~[only-flow.ps1](mini_script/win/only-flow.ps1)~~
   - [login_and_flow.ps1](mini_script/win/login_and_flow.ps1)

3. 填写并保存配置

4. 测试：在pwsh执行`./script-name.ps1`，记得将`script-name`改为你选择的脚本

5. 开机自启设置
   - 双击运行对应的 autostart bat 脚本：
     - [autostart_only-login.bat](mini_script/win/autostart_only-login.bat)
     - [autostart_only-flow.bat](mini_script/win/autostart_only-flow.bat) 
     - [autostart_login_and_flow.bat](mini_script/win/autostart_login_and_flow.bat)

   > 脚本会创建一个vbs来隐藏窗口，并设置启动项到注册表中

### MacOS/Linux (Shell)

1. Linux 需要安装 `libnotify-bin` 以支持通知功能：
  ```bash
  # Ubuntu/Debian
  sudo apt install libnotify-bin
  
  # CentOS/RHEL/Fedora
  sudo yum install libnotify
  # 或
  sudo dnf install libnotify
  ```

2. 根据自身情况选择脚本文件：
   - [only-login.sh](mini_script/unix/only-login.sh)
   - [only-flow.sh](mini_script/unix/only-flow.sh)
   - [login_and_flow.sh](mini_script/unix/login_and_flow.sh)

3. 填写并保存配置

4. 添加执行权限并运行：
   ```bash
   # 将`script-name`改为你选择的脚本
   chmod +x script-name.sh
   ./script-name.sh
   ```

5. 开机自启设置

   - 运行相应的 autostart 脚本：
     - [autostart_only-login.sh](mini_script/unix/autostart_only-login.sh) 
     - [autostart_only-flow.sh](mini_script/unix/autostart_only-flow.sh)
     - [autostart_login_and_flow.sh](mini_script/unix/autostart_login_and_flow.sh)

   > 脚本会根据操作系统自动创建并加载 plist 文件 / systemd 服务

