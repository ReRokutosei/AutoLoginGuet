# AutoLoginGUET 安装程序

这个目录包含了用于创建 AutoLoginGUET Windows 安装程序的 Inno Setup 脚本。

## 文件说明
- [ChineseSimplified.isl](installer\ChineseSimplified.isl)：[项目Inno-Setup-Chinese-Simplified-Translation](https://github.com/kira-96/Inno-Setup-Chinese-Simplified-Translation)制作的简体中文版翻译文件
- [InnoSetup.iss](InnoSetup_ci.iss): Inno Setup 脚本文件
- [README.md](README.md): 本说明文件

## 构建说明

安装程序在 GitHub Actions 工作流中自动构建。

workflow已定义自动捕获版本号，本地构建需要自行更改。

如果你想在本地构建安装程序：

1. 安装 [Inno Setup 6.5.0](https://files.jrsoftware.org/is/6/innosetup-6.5.0.exe)
2. 编译 Rust 项目: `cargo build --release`
3. 修改iss脚本文件的`[Files]`源文件路径注释
    > 我试了定义`#ifdef LOCAL_BUILD`、`#if defined(LOCAL_BUILD)`等都没办法识别
    >
    > 只能自行修改路径了
4. 运行 ISCC.exe 编译脚本: `your_path/ISCC.exe InnoSetup.iss`

编译后的安装程序将位于当前目录下。