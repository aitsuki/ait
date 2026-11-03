# ait

[![GitHub Release](https://img.shields.io/github/v/release/aitsuki/ait)](https://github.com/aitsuki/ait/releases/latest)

ait 是一个 Windows 划词翻译小工具。

选中文字，按下快捷键，就能弹出翻译结果。它会安静地待在系统托盘里，不会一直占着屏幕。

## 下载

从 GitHub Releases 页面下载最新版：

https://github.com/aitsuki/ait/releases/latest

版本变更记录见 [CHANGELOG.md](CHANGELOG.md)。

推荐安装器：

```text
ait-vX.Y.Z-setup.exe
```

这是推荐的安装包，双击后按提示安装即可。

便携版：

```text
ait-vX.Y.Z-windows.exe
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

## 本地英汉词典

在设置中点击「下载词典」，然后勾选「优先使用本地英汉词典」并保存。下载过程中按钮禁用并显示进度；完成后可「重新下载」，失败后可「重试」。此功能默认关闭；关闭不会删除已下载的词典。

- 词典直接从 [ECDICT 上游 GitHub](https://github.com/skywind3000/ECDICT) 下载，约 63 MiB，不包含在安装包或本仓库中；首次下载需要能访问 `raw.githubusercontent.com`。
- 下载完成后自动校验 SHA256 并建立本地 SQLite 索引，期间可继续翻译。关闭设置窗口不会中断下载，退出应用会中断，下次可重新下载。
- 开启后，单个英文单词优先离线查询，显示音标、中文释义（含词性）及词库提供的词形原形释义。句子、其他语言、未收录单词或词典不可用时，自动使用当前 Google / 大模型配置。
- 只有中文目标语言使用英汉词典。本地命中不请求翻译服务，不需要 API key。
- 文件保存在 `%LOCALAPPDATA%\aitsu\ait\data\dictionary`，建立索引时需额外磁盘空间。下载失败不会替换已有词典。

固定上游版本为 [`39cfb967`](https://github.com/skywind3000/ECDICT/tree/39cfb9672c0906b63569e0453c2e6abfb9ab182a)，原始 CSV 的 SHA256 为 `d1ecbdf9ee2b920afb6f430ca6b6bb9f559588cf0dff86e04bd255f9390843cc`。上游标注 MIT，许可见 [ECDICT-LICENSE.txt](docs/ECDICT-LICENSE.txt)，下载时也会保存在本地词典目录。释义来自上游汇集数据，可能有缺漏或不准确。

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

发布正式版时，`Cargo.toml` 是唯一的版本号来源：

1. 修改 `Cargo.toml` 中的 `package.version`。
2. 执行 `cargo check`，同步更新 `Cargo.lock`。
3. 提交并推送到 `main`。
4. 在 GitHub Actions 中手动运行 `Release` workflow，无需再次输入版本号。

也可以推送与包版本一致的 tag 触发发布：

```powershell
git tag vX.Y.Z
git push origin vX.Y.Z
```

Release workflow 会校验 tag 与 `Cargo.toml` 是否一致，并自动生成变更说明、安装包、便携版和 SHA256 校验文件。
