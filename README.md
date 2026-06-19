# ait

ait 是一个 Windows 划词翻译小工具。

选中文字，按下快捷键，就能弹出翻译结果。它会安静地待在系统托盘里，不会一直占着屏幕。

## 下载

从 GitHub Releases 页面下载最新版：

https://github.com/aitsuki/ait/releases/latest

推荐安装器：

```text
ait-v0.1.3-setup.exe
```

这是推荐的安装包，双击后按提示安装即可。

便携版：

```text
ait-v0.1.3-windows.exe
```

它是单文件版本，下载后直接双击运行，不需要解压。

- 当前还没有代码签名。
- 只从 GitHub Releases 官方页面下载。
- 以附件名和 SHA256 校验值确认下载内容。

## 怎么用

1. 启动 ait。
2. 在任意软件里选中一段文字。
3. 按下 `Ctrl+Alt+E`。
4. 等待翻译窗口弹出。

ait 启动后会显示在 Windows 系统托盘里。需要设置或退出时，可以在托盘图标上操作。

## 翻译服务

ait 默认带一个免配置的 Google 翻译接口，可以直接试用。

如果你有自己的 OpenAI-compatible API，也可以在设置里添加：

- API 地址
- 模型名称
- API key

API key 会使用 Windows DPAPI 保存在本机，不会明文写进配置文件。

## 常见问题

### Windows 提示“未知发布者”怎么办？

这是因为 ait 目前还没有代码签名证书。第一次运行时，Windows 可能会弹出安全提醒。

如果你是从本仓库的 GitHub Releases 下载的，可以选择继续运行。

### 为什么有时候默认翻译会失败？

默认的 Google 翻译接口不是 Google Cloud 官方付费接口，可能会被限制、变慢，或者临时不可用。

如果你希望更稳定，可以在设置里配置自己的 OpenAI-compatible API。

### 遇到问题时怎么反馈？

可以先在托盘菜单点击 `打开日志目录`，找到最近的日志文件。

### 支持 macOS 或 Linux 吗？

目前只支持 Windows。

## 开发者说明

本项目使用 Rust 编写。

构建：

```powershell
cargo build
```

运行：

```powershell
cargo run
```

测试：

```powershell
cargo test
```

发布正式版：

1. 打开 GitHub 仓库的 Actions 页面。
2. 选择 `Release` workflow。
3. 点击 `Run workflow`。
4. 输入版本号，例如 `v0.1.3`。

也可以推送 tag 发布：

```powershell
git tag v0.1.3
git push origin v0.1.3
```
