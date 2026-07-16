# UI 组件系统改造实现计划

## 目标

将当前按控件 ID 零散自绘的 Win32 UI，整理成一致、可复用且具备完整交互反馈的组件系统。此次改造不改变翻译业务流程，重点解决点击命中、键盘导航、焦点反馈、尺寸密度和高 DPI 下的一致性。

## 范围

1. 建立共享主题与尺寸 token，统一颜色、圆角、间距和标准控件尺寸。
2. 补齐 Button、Edit、ComboBox、ListBox、Checkbox 的 hover、focus、pressed、disabled 状态。
3. 为可交互控件加入 `WS_TABSTOP`，在主窗口消息循环接入对话框键盘导航。
4. 为翻译窗口和设置窗口补充 Enter / Escape 行为。
5. 保留输入框父绘圆角边框所需的独立绘制空间，统一输入框、下拉框、按钮高度。
6. 为组件提供语义化样式入口，逐步解除绘制逻辑与控件 ID 的耦合，同时保留兼容映射供现有测试和调用使用。
7. 增加 DPI 缩放工具和 DPI-aware manifest，为后续按窗口 DPI 布局提供基础。
8. 更新组件和布局测试，并完成全量回归。

## 实现步骤

### 1. Theme 与 Metrics

- 新增 `src/ui/theme.rs`。
- 提供共享 `RgbColor`、颜色 token、圆角、间距和组件高度。
- 各组件复用主题定义，不再重复维护颜色结构和常量。

### 2. 组件交互状态

- Button：绘制键盘焦点环，保留 hover / pressed / disabled。
- Edit：增加 hover 边框状态，保持边框与原生 EDIT 内容区域分离，并让单行内容垂直居中。
- ComboBox：增加 hover 状态，统一 34px 可见高度和 36px 下拉项高度。
- ListBox：增加 hover 行和焦点边框，统一 36px 行高。
- Checkbox：焦点环覆盖完整可点击行，勾选后仍能看出焦点。

### 3. 键盘操作

- 所有交互控件加入 `WS_TABSTOP`。
- 主消息循环调用 `IsDialogMessageW` 处理 Tab / Shift+Tab。
- 翻译窗口：Ctrl+Enter 触发翻译，Enter 保留多行换行语义，Escape 隐藏。
- 设置窗口：Enter 保存，Escape 取消。

### 4. DPI 基础

- 新增 DPI 工具，提供 96 DPI 基准缩放。
- 嵌入 DPI-aware manifest。
- 组件尺寸 token、窗口和布局按系统 DPI 缩放；后续可继续演进到逐显示器 DPI。

### 5. 验证

- 更新纯函数单元测试。
- 增加焦点、尺寸、样式和 DPI 测试。
- 执行 `cargo fmt`、`cargo test`、`cargo clippy`。

## 实施结果

- [x] 新增共享主题、尺寸和 DPI token。
- [x] Button、Edit、ComboBox、ListBox、Checkbox 补齐交互状态。
- [x] 输入框保留 4px frame gutter，避免原生 EDIT 覆盖圆角边框并维持单行垂直居中。
- [x] 可交互控件加入 `WS_TABSTOP` 和对话框键盘导航。
- [x] 翻译窗口支持 Ctrl+Enter，设置窗口支持 Enter / Escape。
- [x] 统一 34px 控件高度、36px 主按钮和 36px 列表项。
- [x] 删除按钮采用 Danger 语义样式。
- [x] 嵌入 Common Controls v6 与系统 DPI-aware manifest。
- [x] 全量测试和 Release 构建通过。

## 验收标准

- Tab / Shift+Tab 可以遍历所有可操作控件。
- 每个组件都能明显区分 normal、hover、focus、pressed/selected、disabled。
- 按钮获得键盘焦点时有清晰但不过度抢眼的焦点环。
- 输入框圆角边框连续，单行文字垂直居中。
- 标准输入框、下拉框和按钮采用统一高度体系。
- 125% 和 150% 系统缩放下文字不被裁切，窗口与控件保持一致比例。
- 全量测试通过，无翻译和设置功能回归。
