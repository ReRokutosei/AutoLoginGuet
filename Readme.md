# 自动登录校园网（哆点网络）

[![Python 3.12+](https://img.shields.io/badge/python-3.12%2B-blue)](https://www.python.org/)
[![Rust](https://img.shields.io/badge/rust-1.86+-orange)](https://www.rust-lang.org/)
[![Yaml](https://img.shields.io/badge/yaml-blue)](https://yaml.org/)
[![Windows](https://img.shields.io/badge/platform-Windows-green)](https://www.microsoft.com/zh-cn/windows)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](LICENSE)

## 介绍
理论上该项目适用于供应商为[哆点网络](https://doctorcom.com/)的校园网自动登录。这里只面向GUET的用户，其他学校用户请自行修改配置参数。

**网络请求这部分代码逻辑来自于桂林理工大学的大佬[HWinZnieJ](https://www.bilibili.com/opus/646733491161006112#reply258018351937)。**

对于Python我增加了日志输出，并将配置抽取到YAML文件中。由于Python打包的程序太大，所以使用Rust重写并构建了分发包。

## 功能特性
- ✨ 校园网自动登录
- 📝 支持自定义日志记录
- 🔄 一键配置开机自启动
- ⚙️ YAML 格式配置文件
- 🚀 提供Python和Rust两种实现版本

## 项目结构
```
AutoLogin
├── 程序文件
│   ├── AutoLogin.exe        # Rust编译版本
│   └── AutoLogin.pyw        # Python源码版本
├── 配置文件
│   └── config.yaml         # 配置参数文件
└── 辅助脚本
    ├── 注册表方式          # 推荐，无需管理员权限
    └── 计划任务方式        # 需要管理员权限
```

## 快速开始

### 配置说明
1. `config.yaml` 需与程序放在同目录
2. 配置文件中只需修改 `sign_parameter` 参数
3. `sign_parameter`参数值获取方法：

    - 打开浏览器，按下F12打开DveTool，选择`网络(network)`，勾选保留日志
    
    - 然后在浏览器地址栏输入校园网登录地址`http://10.0.1.5`，登录你的账号，如果已经登录先退出再重新登录
    
    - 然后在DveTool中查看请求包，找到名称开头为`login?callback=dr1003&DDDDD`的请求包
    
    - `请求网站`的值就是我们要的`sign_parameter`。

![sign参数获取示意图](image.png)

### Rust版本使用说明
1. 下载文件
   - [AutoLogin.exe](https://github.com/ReRokutosei/AutoLoginGuet/releases/download/v0.9/AutoLogin.exe)
   - [config.yaml](https://github.com/ReRokutosei/AutoLoginGuet/releases/download/v0.9/config.yaml)

2. 基础使用
   - 将下载的文件放在同一目录
   - 配置 `config.yaml`
   - 运行 `AutoLogin.exe`

3. 开机自启设置
   - 下载并使用 [Rust-设置开机自启-注册表.bat](https://github.com/ReRokutosei/AutoLoginGuet/releases/download/v0.9/Rust_Set_Starup.bat)
   - 如需取消，使用 [Rust-取消开机自启-注册表.bat](https://github.com/ReRokutosei/AutoLoginGuet/releases/download/v0.9/Rust_Cancel_Starup.bat)

### Python版本使用说明
1. 克隆项目
   ```shell
   git clone https://github.com/GUET-HZY/AutoLogin.git
   cd AutoLogin
   ```

2. 配置
   ```shell
   python -m pip install -r requirements.txt
   xcopy config.yaml .\Python\ /y
   # 接下来按照上面的步骤修改config.yaml
   ```
3. 测试
   ```shell
   python AutoLogin.pyw
   ```


4. 开机自启设置
   ```sh
   cd Bat脚本
   ./Py-设置开机自启-注册表.bat
   ./Py-取消开机自启-注册表.bat # 如需取消
   ```

## Rust构建说明
   >必要文件都在仓库了，自己编译即可。~~不想写了~~

## 注意事项

- 仅在Windows 10测试通过，其他系统请自行调试
- 所有脚本文件需与程序放在同目录
- 推荐使用注册表方式设置开机自启
- 请勿将你的配置文件`config.yaml`上传到公共仓库
- 为避免版权问题，不提供icon图标，请自行准备
- Python版本的bat脚本会自动检测安装路径，无需手动配置
- 如果不再使用，取消开机自启后删除配置文件`config.yaml`、程序文件`AutoLogin`和日志文件`AutoLogin.log`即可
