# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),  
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-08-11

### Added
- 移除 Python 版本，统一使用 Rust 2024 重构全栈
- 从命令行工具升级为带 GUI 的图形化应用（Dioxus）
- 拆分为 core 与 gui 模块，基本实现低耦合设计
- 核心功能异步化（tokio），提升响应与稳定性
- 使用 AES 加密用户密码，基于机器标识生成密钥
- 添加开机自启（Windows 注册表）与静默运行模式
- 配置文件从 YAML 迁移至 TOML，支持界面化配置
- 更新日志系统与系统通知
- 将许可证从 MIT 改为 GPLv3

### Changed
- 全面重构项目架构
- 图形界面采用 Dioxus 框架开发
- 配置文件格式从 YAML 改为 TOML

### Removed
- 移除 Python 版本实现
