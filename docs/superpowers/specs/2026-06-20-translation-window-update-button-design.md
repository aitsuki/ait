# 翻译面板新版本提示按钮设计

## 背景

`ait` 启动后会后台检查 GitHub 最新 Release。当前行为是在发现新版本时直接弹出系统弹框。这个弹框会打断启动体验，尤其是软件随系统启动或用户只想临时划词翻译时。

本次目标是保留自动检查更新，但取消启动时自动弹框。发现新版本后，在翻译面板顶部显示一个明确的“有新版本”按钮。用户点击按钮时，再复用现有更新弹框展示版本信息和 Release 链接。

## 范围

包含：

- 启动后台检查更新。
- 启动检查发现新版本时不自动弹框。
- 翻译面板顶部显示“有新版本”按钮。
- 点击“有新版本”按钮后显示现有更新弹框文案。
- 未发现新版本或检查失败时不显示按钮。

不包含：

- 自动下载或自动安装更新。
- 改造托盘“打开最新版本页面”功能。
- 在状态行显示更新提示。
- 持久化忽略版本或稍后提醒状态。

## 用户体验

翻译面板默认不显示更新按钮。启动后如果检查到新版本，翻译面板顶部出现一个按钮：

```text
有新版本
```

按钮放在顶部控制区，靠近配置下拉框，但不挤占底部状态行。这样翻译状态仍只负责显示“正在取词”“正在翻译”“翻译完成”和错误摘要。

用户点击按钮时弹出与当前更新提示一致的系统弹框，例如：

```text
发现新版本 v0.1.5，当前版本 v0.1.4。打开最新 Release：https://github.com/aitsuki/ait/releases/latest
```

弹框只由用户点击触发，不在启动检查完成时自动出现。

## 架构

继续使用现有更新模块：

- `src/update.rs` 负责检查版本、比较版本、生成更新提示文案。
- `src/app.rs` 负责启动更新检查线程、接收 `WM_UPDATE_CHECK_FINISHED`。
- `src/ui/translate_window.rs` 负责翻译面板 UI 和布局。

主流程调整为：

1. 启动应用后调用 `spawn_update_check_task`。
2. 收到 `WM_UPDATE_CHECK_FINISHED`。
3. 如果结果是 `UpToDate`，保持现有静默行为。
4. 如果结果是 `UpdateAvailable`，不调用 `show_runtime_message`，而是把更新状态传给 `TranslationWindow`。
5. `TranslationWindow` 显示顶部“有新版本”按钮，并保存用于弹框的更新文案或更新状态。
6. 用户点击按钮时，复用 `update_status_message` 和 `show_runtime_message` 展示详情。

## 翻译窗口组件

`TranslationWindow` 新增一个更新按钮控件，默认隐藏。

新增状态建议：

```rust
update_status: Option<UpdateStatus>
```

保存 `UpdateStatus`，因为它保留结构化版本信息，便于测试和未来扩展。按钮点击时再生成文案。

新增方法建议：

```rust
pub fn show_update_available(&mut self, status: UpdateStatus) -> Result<()>
pub fn update_status(&self) -> Option<&UpdateStatus>
```

Windows UI 中新增按钮 ID，例如 `ID_UPDATE_BUTTON`。按钮点击通过当前窗口过程处理，并发送或调用应用层逻辑展示弹框。

## 布局

当前翻译窗口顶部包括：

- 左侧“原文”标签。
- 右侧 profile 下拉框。

新增“有新版本”按钮放在顶部，推荐位于 profile 下拉框左侧。按钮宽度固定，例如 86px，高度与下拉框一致。没有更新时隐藏。

布局函数需要为按钮预留矩形：

```rust
pub update_button: ControlRect
```

当按钮隐藏时，profile 下拉框仍保持现有位置。按钮显示时，放在下拉框左侧并留出间距。最小窗口宽度下，按钮宽度可以压缩到合理下限，避免和“原文”标签重叠。

## 错误处理

- 更新检查失败：记录日志；启动检查模式下不弹框、不显示按钮。
- 更新状态写入翻译窗口失败：记录日志，不影响翻译功能。
- 点击按钮时如果没有保存的更新状态：忽略点击。
- 弹框失败：沿用当前 `MessageBoxW` 的容错方式，不中断主循环。

## 测试策略

使用 TDD。先写失败测试，再实现。

重点测试：

- 翻译窗口布局包含更新按钮矩形。
- 更新按钮矩形位于顶部，并且不与 profile 下拉框重叠。
- 翻译窗口状态默认没有更新提示。
- 设置更新状态后，状态可读取，按钮应显示。
- `WM_UPDATE_CHECK_FINISHED` 收到 `UpdateAvailable` 时不自动弹框，而是进入翻译窗口更新提示路径。
- 已有 release/update 测试保持通过。

可运行测试：

```powershell
cargo test --test workflow_tests
cargo test --test release_tests
cargo test --test settings_window_tests
cargo test
```

## 验收标准

- 启动软件后，即使发现新版本，也不会自动弹出更新弹框。
- 打开翻译面板后，顶部能看到“有新版本”按钮。
- 点击“有新版本”按钮后，弹出更新详情弹框。
- 翻译状态行不显示更新提示，也不需要承载更新逻辑。
- 原有托盘菜单、设置窗口、翻译流程不回退。
