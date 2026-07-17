# Changelog

本项目的重要变更记录在此文件中。版本号遵循语义化版本。

## [0.3.1] - 2026-07-17

### 新增

- 设置窗口支持直接测试尚未保存的大模型配置。
- 测试成功后显示完整请求的响应耗时（毫秒）。

### 改进

- 全局快捷键注册失败时继续启动，并提示用户前往设置修改快捷键。
- 设置窗口和翻译窗口统一使用应用图标。
- 修复自动发布生成的 Release 说明未包含对应版本 Changelog 的问题。

## [0.3.0] - 2026-07-17

### 新增

- 建立统一的 UI 主题、颜色、尺寸与 DPI 缩放系统。
- 为按钮、输入框、下拉框、列表框和复选框补充 hover、focus、pressed、selected 与 disabled 状态。
- 支持 Tab / Shift+Tab 导航、翻译窗口 Ctrl+Enter，以及设置窗口 Enter / Escape。
- 嵌入 Common Controls v6 和 DPI-aware Windows manifest。
- Release 自动生成 SHA256 校验文件和基于提交记录的变更说明。

### 改进

- 统一输入框、按钮、下拉框和列表项的尺寸与视觉层级。
- 修复输入框圆角边框不连续和单行文字未垂直居中的问题。
- 单行输入框支持三连击全选，多行输入框保留三连击选择段落。
- 修复下拉框选中区域使用错误字体、文字小于下拉菜单的问题。
- 删除按钮改为危险操作样式。
- 改进高 DPI 下的窗口、控件、圆角和内容间距。

### 发布流程

- 以 `Cargo.toml` 的 `package.version` 作为唯一版本号来源。
- tag 发布时强制校验 tag 与包版本一致。
- README 不再写死具体附件版本号。
- Release 页面自动包含实际变更记录，而不是固定模板。

[0.3.1]: https://github.com/aitsuki/ait/releases/tag/v0.3.1
[0.3.0]: https://github.com/aitsuki/ait/releases/tag/v0.3.0
