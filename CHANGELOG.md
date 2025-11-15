# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

### [2.1.1](https://github.com/ReRokutosei/AutoLoginGuet/compare/v2.1.0...v2.1.1) (2025-11-15)

## [2.1.0](https://github.com/ReRokutosei/AutoLoginGuet/compare/v2.0.0...v2.1.0) (2025-09-05)


### Features

* **flow:** 添加流量查询功能 ([cd35412](https://github.com/ReRokutosei/AutoLoginGuet/commit/cd35412751a8282af5c170faf67af44f33dc4d42))
* **ISP:** 运营商添加中国广电 ([7fb8208](https://github.com/ReRokutosei/AutoLoginGuet/commit/7fb82086e376a497d09f179f4fd7e57ec9e23fcc))
* **message:** 支持使用占位符自定义消息格式 ([0b77b23](https://github.com/ReRokutosei/AutoLoginGuet/commit/0b77b23b66f0fd1e5a16ca356f25f0267ffeed60))

## [2.0.0](https://github.com/ReRokutosei/AutoLoginGuet/compare/v1.1.1...v2.0.0) (2025-08-26)


### ⚠ BREAKING CHANGES

* **core:** 解耦项目架构，引入服务层和事件驱动机制 ([a5d09cf](https://github.com/ReRokutosei/AutoLoginGuet/commit/a5d09cfe0a713e3f384280bd4612fafa94805a52))

### feat: 架构重构与模块解耦

* **引入服务层架构**: 将业务逻辑从GUI和CLI中抽离，创建统一的AuthService服务层，为前端提供一致的接口
* **实现事件驱动架构**: 通过EventBus和EventHandler实现模块间解耦，提高系统灵活性
* **添加DTO模式**: 引入 GuiConfigDto 分离界面数据和核心配置，提高数据传输的清晰度
* **抽象核心组件**: 通过trait抽象网络、日志、通知等核心组件

### refactor: 代码结构优化

* **重构核心模块**: 将原有的核心功能拆分为config、crypto、network、logging、notification等独立模块
* **统一错误处理**: 引入AppError枚举和AppResult类型别名，提供一致的错误处理机制
* **改进配置管理**: 重新设计配置加载和保存机制，增加配置验证和标准化处理
* **优化网络状态检查**: 扩展网络状态类型，提供更详细的网络连接信息

### feat: 用户体验改进

* **增强输入验证**: 添加用户名和密码格式验证，提供实时反馈
* **改进错误提示**: 提供更加用户友好的错误消息
* **优化界面交互**: 重新设计GUI界面交互逻辑
* **完善Debug模式**: 改进调试信息输出

### perf: 性能与稳定性提升

* **优化日志管理**: 改进日志记录和清理，提高日志处理效率
* **增强网络处理**: 优化网络请求处理，大幅提升稳定性
* **改善内存管理**: 通过合理的Arc和Mutex使用，优化内存分配和释放

### [1.1.1](https://github.com/ReRokutosei/AutoLoginGuet/compare/v1.1.0...v1.1.1) (2025-08-16)


### Features

* **mini_script:** 为高级用户提供多平台轻量级脚本，请在[项目目录](mini_script)查看 ([d1f5018](https://github.com/ReRokutosei/AutoLoginGuet/commit/d1f50189b17a1af7fb26042417ea9edf45a8f29d))


### Bug Fixes

* **gui:** 修复 Debug 模式登录配置的处理逻辑 ([74e6d96](https://github.com/ReRokutosei/AutoLoginGuet/commit/74e6d96f55b215fefa754c31dcaa325f39fff352))
* **gui:** 修复更改开机自启时可能覆盖已保存密码的问题 ([061803f](https://github.com/ReRokutosei/AutoLoginGuet/commit/061803f22fd3463b8e9edbb9f540adeb1fadf8b4))

## [1.1.0](https://github.com/ReRokutosei/AutoLoginGuet/compare/v1.0.1...v1.1.0) (2025-08-14)


### Features

* **core:** 集中配置管理、增加保存防抖 ([b79f339](https://github.com/ReRokutosei/AutoLoginGuet/commit/b79f339d61755601ca68b91b96635c679bc7ef5f))


### Bug Fixes

* **gui:** 修复第二次启动的空密码框导致登录失败问题 ([268397f](https://github.com/ReRokutosei/AutoLoginGuet/commit/268397f4a48ed062e2381aa9ccb3bfac68226780))
* **network:** 修复 url 缺少斜杠导致请求失败的问题 ([bee6068](https://github.com/ReRokutosei/AutoLoginGuet/commit/bee60680e2b9bfdd9bd95c86fa27c58197bae39b))
* **security:** 修复日志记录中的敏感信息明文泄漏问题 ([1dd0df0](https://github.com/ReRokutosei/AutoLoginGuet/commit/1dd0df08113b251c5ad3cf62b17e9579969f8adb))

### [1.0.1](https://github.com/ReRokutosei/AutoLoginGuet/compare/v1.0.0...v1.0.1) (2025-08-13)


### Features

* **ci:** 添加 GitHub Actions 发布工作流 ([cee3387](https://github.com/ReRokutosei/AutoLoginGuet/commit/cee3387f8a7f7c72adebb2c2452dda0e25ab0190))


### Bug Fixes

* **ci:** 修复workflow ([1f8432c](https://github.com/ReRokutosei/AutoLoginGuet/commit/1f8432c4bdc402ec833de79879a00d9dfa045967))
* **config:** 增强配置完整性校验 ([cab5ad3](https://github.com/ReRokutosei/AutoLoginGuet/commit/cab5ad3bde3e5394ebdd715f14b4ab7a39389f5a))
* **crypto:** 为 SHA-256 密钥派生添加固定 salt ([47e4827](https://github.com/ReRokutosei/AutoLoginGuet/commit/47e48279587445c20c21a1173837c273218a8f53))
* **gui:** 修复密码重复加密导致登录失败的问题 ([d818e62](https://github.com/ReRokutosei/AutoLoginGuet/commit/d818e62c60ec31575a8f12933ab83b2b5e6faee4))
* **network:** 将硬编码IP地址改为使用配置中的`login_ip` ([4bafe8c](https://github.com/ReRokutosei/AutoLoginGuet/commit/4bafe8c272ac5de889b1126b51bc9d17b6a8fc33))

### [1.0.0](https://github.com/ReRokutosei/AutoLoginGuet/commit/06f9b44747bc5c1bf99eab548800d356efa7c9c8) (2025-08-11)


### Features

- **project**!: 重构项目
    - 从命令行工具升级为带 GUI 的图形化应用（Dioxus）
    - 拆分为 core 与 gui 模块，基本实现低耦合设计
    - 核心功能异步化（tokio），提升响应与稳定性
    - 使用 AES 加密用户密码，基于机器标识生成密钥
    - 添加开机自启（Windows 注册表）与静默运行模式
    - 更新日志系统与系统通知

### Refactors

- 全面重构项目架构

### BREAKING CHANGES

- 移除 Python 版本，统一使用 Rust 2024 重构全栈
- 配置文件从 YAML 迁移到 TOML
- 许可证从 MIT 变更为 GPLv3