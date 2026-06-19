# 自动发布 GitHub Release 设计

## 背景

`ait` 是一个 Windows-only 的 Rust 划词翻译托盘应用。当前仓库没有 `.github` 工作流，也没有发布脚本。手动发布需要本地测试、release 编译、复制文件、创建 GitHub Release、上传附件，对不熟悉 GitHub 的维护者来说步骤太多，容易漏掉。

本设计的目标是把发布流程封装到 GitHub Actions：维护者只需要在 GitHub 页面输入版本号，或以后推送版本 tag，GitHub 就能自动测试、编译、制作安装器并上传下载文件。

## 目标

- 支持 GitHub Actions 页面手动发布正式版。
- 支持推送 `vX.Y.Z` 格式的 git tag 自动发布正式版。
- 每次发布都先运行 `cargo test`，测试失败时不发布。
- 每次发布都运行 `cargo build --release`，编译失败时不发布。
- Release 附件同时提供单文件 exe 和安装器 exe。
- 安装器支持安装、开始菜单快捷方式、卸载、安装完成后运行应用。
- 第一版发布流程保持简单、可读、可维护。

## 非目标

- 不做代码签名。
- 不做自动更新。
- 不做多平台构建。
- 不做开机自启。
- 不做桌面快捷方式。
- 不做复杂安装选项。
- 不做 zip 发布包。
- 不自动修改 `Cargo.toml` 版本号。

## 发布入口

工作流命名为 `Release`，放在 `.github/workflows/release.yml`。

触发方式有两种：

1. `workflow_dispatch`
   - 在 GitHub Actions 页面手动点击 `Run workflow`。
   - 输入 `version`，例如 `v0.1.0`。

2. `push` tag
   - 当推送 `v*.*.*` 格式的 tag 时触发。
   - 版本号来自 tag 名称，例如 `v0.1.0`。

两个入口共用同一套构建、打包、发布逻辑。

## 版本规则

发布版本必须使用 `vX.Y.Z` 格式，例如：

- `v0.1.0`
- `v0.2.0`
- `v1.0.0`

工作流会从输入或 tag 中得到版本号，并生成对应文件名：

- `ait-v0.1.0-windows.exe`
- `ait-v0.1.0-setup.exe`

如果版本号不符合 `vX.Y.Z`，工作流直接失败，不创建 Release。

## 构建流程

工作流运行在 `windows-latest`。

主要步骤：

1. Checkout 仓库。
2. 安装 Rust stable toolchain。
3. 解析并校验版本号。
4. 运行 `cargo test`。
5. 运行 `cargo build --release`。
6. 把 `target/release/ait.exe` 复制为 `dist/ait-vX.Y.Z-windows.exe`。
7. 安装 Inno Setup。
8. 使用仓库内的 Inno Setup 脚本生成 `dist/ait-vX.Y.Z-setup.exe`。
9. 创建 GitHub Release。
10. 上传两个附件。

## 安装器设计

安装器使用 Inno Setup。脚本放在 `installer/ait.iss`。

安装器行为：

- 应用名：`ait`
- 安装位置：用户程序目录下的 `ait`
- 安装文件：release 构建产物 `ait.exe`
- 开始菜单：创建 `ait` 快捷方式
- 卸载：创建 Windows 标准卸载入口
- 安装完成页：提供运行 `ait` 的选项

安装器第一版不创建桌面快捷方式，不设置开机自启。

## Release 内容

Release 标题使用版本号：

```text
ait v0.1.0
```

Release 说明使用固定模板：

```markdown
# ait v0.1.0

Windows-only lightweight selection translator.

## Download

- `ait-v0.1.0-setup.exe`: recommended for most users.
- `ait-v0.1.0-windows.exe`: portable single-file executable.

## Notes

- Windows only.
- The built-in no-key Google translation provider may be rate-limited or break.
- OpenAI-compatible providers can be configured in settings.
```

## 失败处理

- 测试失败：停止发布。
- 编译失败：停止发布。
- 安装器构建失败：停止发布。
- 版本号格式错误：停止发布。
- Release 已存在：停止发布，避免覆盖已有正式版。

失败时不做自动重试。维护者查看 GitHub Actions 日志后修复问题，再重新触发发布。

## 安全与权限

工作流使用 GitHub 提供的 `GITHUB_TOKEN` 创建 Release 和上传附件。

工作流权限设置为：

```yaml
permissions:
  contents: write
```

不引入第三方密钥，不在仓库中保存任何发布凭证。

## 测试策略

实现后需要验证：

- 本地 `cargo test` 通过。
- 本地 `cargo build --release` 通过。
- Inno Setup 脚本结构合理，能引用 release exe。
- GitHub Actions YAML 语法合理。
- 手动触发入口可以输入版本号。
- tag 触发入口只匹配 `v*.*.*`。

由于 GitHub Release 创建需要远端环境，最终端到端验证以一次真实的 `v0.1.0` 或测试版本发布为准。

## 用户体验

普通用户进入 GitHub Release 后看到两个下载项：

- 推荐下载 `ait-vX.Y.Z-setup.exe`
- 高级用户可以下载 `ait-vX.Y.Z-windows.exe`

用户不需要解压文件。

维护者发布时只需要选择一种方式：

- 在 GitHub 页面点击 `Run workflow` 并输入版本号。
- 或推送 `vX.Y.Z` tag。
