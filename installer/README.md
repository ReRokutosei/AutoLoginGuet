# AutoLoginGUET 安装程序

这个目录包含了用于创建 AutoLoginGUET Windows 安装程序的 Inno Setup 脚本。

## 文件说明
- [ChineseSimplified.isl](installer\ChineseSimplified.isl)：[项目Inno-Setup-Chinese-Simplified-Translation](https://github.com/kira-96/Inno-Setup-Chinese-Simplified-Translation)制作的简体中文版翻译文件
- [InnoSetup.iss](InnoSetup_ci.iss): Inno Setup 脚本文件
- [README.md](README.md): 本说明文件

## 构建过程

安装程序在 GitHub Actions 工作流中自动构建。构建过程包括：

1. 编译 Rust 项目生成可执行文件
2. 下载并安装 Inno Setup
3. 使用 Inno Setup 脚本创建安装程序
4. 将安装程序和预编译可执行文件一起发布

## 本地构建

如果你想在本地构建安装程序，需要：

1. 安装 Inno Setup 6
2. 编译 Rust 项目: `cargo build --release`
3. 运行 ISCC.exe 编译脚本: `ISCC.exe InnoSetup.iss`

编译后的安装程序将位于当前目录下。