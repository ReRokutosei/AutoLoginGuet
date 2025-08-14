# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

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