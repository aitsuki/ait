# 隐藏复制等待时间设置设计

## 背景

设置窗口顶部目前显示“复制等待毫秒”输入框。该字段对应 `clipboard_capture.copy_wait_ms`，用于剪贴板 fallback 取词：发送复制快捷键后，在配置的毫秒数内等待剪贴板序列号变化并重试读取剪贴板。

该参数属于兼容性和诊断参数，不是普通用户理解或日常调整的设置。保留在设置窗口会增加认知负担。

## 目标

- 设置窗口不再显示“复制等待毫秒”标签和输入框。
- 保存设置时不再从窗口读取该控件。
- 底层配置字段 `clipboard_capture.copy_wait_ms` 保留，默认值继续为 `300`。
- 旧配置文件中的 `copy_wait_ms` 继续可读，运行时仍按配置值传给取词服务。

## 非目标

- 不修改剪贴板取词等待逻辑。
- 不删除 `ClipboardCaptureSettings.copy_wait_ms`。
- 不新增高级设置区域。
- 不迁移或重写现有配置文件。

## 设计

设置页只移除复制等待时间的 UI 暴露。`SettingsViewModel` 和 `SettingsProfileDetailUpdate` 可以继续携带 `copy_wait_ms`，也可以在后续清理中收窄；本次优先做低风险变更，避免扩大影响面。

Windows 设置窗口创建控件时，删除 `ID_COPY_WAIT` 对应的静态文本和编辑框。保存时，`SettingsProfileDetailUpdate.copy_wait_ms` 使用当前内存中的 `settings.clipboard_capture.copy_wait_ms`，而不是读取窗口控件。这样保存接口配置或快捷键时不会重置用户已有等待时间。

布局测试从“顶部包含快捷键和复制等待”改为“顶部只包含快捷键，分隔线位于全局设置之后”。配置测试不变，用于证明字段仍兼容。

## 验收

- 设置窗口不再创建“复制等待毫秒”控件。
- 保存设置后 `clipboard_capture.copy_wait_ms` 保持原值。
- `cargo test settings_window` 通过。
- `cargo test` 通过。
