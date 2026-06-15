# 取词核心能力修复设计规格

日期：2026-06-16

## 背景

MVP 验证提出 5 个问题，其中“无法取词”仍未解决。取词是划词翻译的核心能力，不能只依赖简单发送 `Ctrl+C`。项目已有调研文档要求取词必须优先参考成熟开源实现，并采用多策略 fallback；当前实现没有充分落实这一点。

本规格只处理“无法取词”问题，不处理窗口置顶和窗口焦点时移动的问题。

## 约束

- 必须遵守 `docs/adr/0001-no-ocr.md`：不实现 OCR，不集成 OCR 引擎、截图识别、区域识别或 OCR 外部进程作为内置能力。
- 必须遵守 `docs/research/open-source-shortcuts-and-text-capture.md`：取词能力要参考成熟开源项目，不从零凭空设计。
- 当前修复只承诺处理可选中文本，不承诺不可选文本、图片、PDF 扫描件或游戏画面文字。
- 剪贴板 fallback 现阶段只承诺恢复文本剪贴板，不承诺完整恢复图片、文件列表、富文本等格式。
- 日志不能记录完整原文，只能记录策略名、文本长度、错误类型和状态。

## 参考实现

本次修复重点参考 Pot Desktop 的独立 `Selection` crate Windows 实现：

- 先尝试 UI Automation `TextPattern.GetSelection()`。
- UIA 没有文本时 fallback 到剪贴板复制。
- 剪贴板复制前记录 `GetClipboardSequenceNumber()`。
- 发送复制前释放常见修饰键和干扰键，例如 `Ctrl`、`Alt`、`Shift`、`Meta`、`Tab`、`Escape`、`CapsLock`、`C`。
- 发送干净的 `Ctrl+C`。
- 等待剪贴板序列号变化，用变化结果判断复制是否真的发生。
- 复制后读取文本，再恢复旧剪贴板内容。

本项目不会直接拷贝 GPL 或许可不明确项目代码。Pot `Selection` 的行为可作为参考，Rust/Win32 封装由本项目实现。

## 根因判断

当前实现失败的高概率原因是剪贴板复制动作不够“干净”：

- 默认快捷键是 `Ctrl+Alt+E`。
- `WM_HOTKEY` 到达后，目标应用或系统键盘状态里可能仍存在 `Ctrl`、`Alt` 等修饰键按下状态。
- 当前代码立即发送 `Ctrl+C`，目标应用实际收到的可能不是普通复制，而是带额外修饰键的组合。
- 当前代码只轮询剪贴板文本，没有用 `GetClipboardSequenceNumber()` 判断复制是否发生，容易把“没复制”和“复制得到旧内容/空内容”混在一起。
- UIA 当前只读焦点控件，且错误会直接 fallback；它没有足够日志解释真实失败原因。

## 目标行为

用户在常见可选中文本环境中选中文本后按快捷键，应用应按以下顺序取词：

1. `uia_focused_selection`：读取当前焦点元素的 UI Automation `TextPattern` 选区。
2. `clipboard_copy`：如果 UIA 没有取到非空文本，则执行干净的剪贴板复制 fallback。

取词成功时：

- 返回非空 `CapturedText`。
- 日志记录使用的策略和文本长度。
- 翻译流程继续执行。

取词失败时：

- 返回明确的 `CaptureErrorKind::NoText` 或 `ClipboardUnavailable` / `CopyFailed`。
- 日志记录每个策略失败原因。
- 不记录完整原文。

## 剪贴板复制策略

`clipboard_copy` 策略按以下步骤执行：

1. 读取旧文本剪贴板内容。
2. 记录 `GetClipboardSequenceNumber()`。
3. 尝试清空文本剪贴板，减少读到旧文本的概率。
4. 释放常见修饰键和干扰键：
   - `Ctrl`
   - `Alt`
   - `Shift`
   - `Win`
   - `Tab`
   - `Escape`
   - `CapsLock`
   - `C`
5. 发送 `Ctrl` down、`C` down/up、`Ctrl` up。
6. 在配置的等待时间内轮询剪贴板序列号变化。
7. 序列号变化后读取文本剪贴板。
8. 尽力恢复旧文本剪贴板。
9. 如果新文本为空或读取失败，返回结构化错误。

如果复制前旧剪贴板没有文本，当前版本可以清空文本剪贴板后不恢复其他格式；这与已有 MVP 约束一致。未来要完整恢复多格式剪贴板时另开设计。

## UIA 策略

`uia_focused_selection` 保留现有方向，但需要调整错误语义：

- UIA 初始化失败、焦点元素不存在、目标控件不支持 `TextPattern`、选区为空，均不应阻止剪贴板 fallback。
- UIA 成功取得非空文本时直接返回，不触碰剪贴板。
- UIA 失败或为空时记录 debug 日志，然后进入剪贴板策略。

本次不实现鼠标所在元素 `ElementFromPoint`，原因是当前紧急目标是修复快捷键触发后的选区复制失败。鼠标元素读取可以作为后续增强，但不应阻塞本次核心修复。

## 模块设计

### `capture`

新增或调整内部抽象：

- `SelectionBackend`：读取系统文本接口选区。
- `ClipboardBackend`：读写文本剪贴板、发送复制、读取剪贴板序列号。
- `KeyboardCopyBackend` 或等价接口：封装释放按键和发送复制动作，便于测试。

`CaptureService` 负责策略编排：

- 先调用 `SelectionBackend`。
- 若返回非空文本，则返回成功。
- 若返回空或可 fallback 的错误，则调用剪贴板复制。
- 将每个策略的结果写入日志。

### `app`

`WindowsWorkflowCapture` 继续装配 Windows 后端。实现细节留在 `capture` 内，不把 Win32 键盘和剪贴板细节泄漏到 `app`。

## 测试要求

实现前必须先补测试，至少覆盖：

- UIA 返回非空文本时不触碰剪贴板。
- UIA 返回空时进入剪贴板 fallback。
- 剪贴板复制前会释放常见修饰键和 `C`。
- 剪贴板序列号未变化时返回 `CopyFailed` 或 `NoText`，不能误报成功。
- 剪贴板序列号变化且新文本非空时返回新文本。
- 取词后尽力恢复旧文本剪贴板。
- UIA 不支持 `TextPattern` 时不会中断 fallback。

实现完成后必须运行：

- `cargo test`
- 至少一次 Windows 手工验证：Notepad、浏览器输入框、网页正文可选中文本。

## 非目标

- 不实现 OCR。
- 不实现 PDF 扫描件取词。
- 不实现鼠标所在元素 `ElementFromPoint`。
- 不实现剪贴板监听自动翻译。
- 不实现完整多格式剪贴板恢复。
- 不修改翻译窗口置顶和焦点移动行为。

## 验收标准

- MVP issue 中“无法取词”可以勾选完成。
- 使用默认 `Ctrl+Alt+E` 时，常见文本选区能被捕获。
- 日志能看出最终使用了 `uia_focused_selection` 还是 `clipboard_copy`。
- 剪贴板复制策略参考了开源实现的关键行为：释放修饰键、发送干净复制、使用剪贴板序列号判断复制发生。
- 不违反 ADR 0001。
