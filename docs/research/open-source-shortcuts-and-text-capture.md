# 开源优先调研：快捷键序列与取词

日期：2026-06-15

## 结论

本项目必须优先复用或参考成熟开源项目来实现两类高风险能力：

- 快捷键序列：普通全局快捷键、连续按键序列、双击修饰键、`Ctrl+C+C` 这类触发模式。
- 取词/取选中文本：跨应用选区读取、剪贴板取词、OCR fallback、鼠标附近词语扫描。

这两块不允许直接从零设计完整方案。首版原型也必须先对照开源项目验证行为，再决定 Rust/Win32 封装边界。

## 快捷键序列

### 首选实现路径

1. 普通全局快捷键优先使用 Windows `RegisterHotKey` 模型。
2. Rust 侧优先评估 `tauri-apps/global-hotkey` 或类似轻量开源 crate，只使用其底层全局快捷键注册能力，不引入 Tauri 运行时。
3. 需要连续按键序列时，再引入低级键盘监听。候选开源项目：
   - `Narsil/rdev`：跨平台键鼠全局监听/发送，MIT 许可；可参考 Windows hook 封装方式。
   - AutoHotkey：重点参考其热键、双击键、修饰键状态和超时语义；除非许可和体积可接受，否则不直接嵌入运行时。
   - GoldenDict / GoldenDict-ng：重点参考 `Ctrl+C+C`、scan popup、取词触发的用户体验。

### 约束

- 能用 `RegisterHotKey` 的固定组合键，不使用低级 hook。
- 只有以下能力需要低级 hook：
  - 双击 `Ctrl` / 双击 `Alt`。
  - `Ctrl+C+C`、`Ctrl+Ctrl` 这类序列。
  - 未来可能的鼠标选区后自动触发。
- hook 回调只做事件入队，不做翻译、不做 IO、不做复杂状态机。
- 快捷键解析必须做成可测试状态机，输入为按下/释放事件流，输出为触发命令。
- 原型必须覆盖这些用例：
  - `Ctrl+Alt+E` 固定快捷键。
  - `Ctrl+C+C` 在时间窗口内触发。
  - 双击 `Ctrl` 触发。
  - 长按修饰键不误触发。
  - 系统已有快捷键冲突时给出可理解的错误。

## 取词/取选中文本

### 首选实现路径

取词必须采用多策略 fallback，按侵入性从低到高排序：

1. UI Automation `TextPattern`：尝试读取当前焦点控件或鼠标所在控件的选中文本。
2. 剪贴板复制策略：保存当前剪贴板，发送复制命令，读取文本，尽力恢复剪贴板。
3. 剪贴板监听策略：用户复制后自动翻译，作为明确可开关能力。
4. OCR fallback：对无法选择或无法暴露文本的区域，使用截图 OCR。

### 必须参考的开源项目

- GoldenDict-ng：参考 scan popup、Track Selection Change、剪贴板扫描、取词失败场景。
- Crow Translate：参考“选中文本后快捷键翻译”的交互，以及 Windows 下全局快捷键失效问题。
- Pot Desktop：参考选择翻译、剪贴板监听、截图 OCR 的产品组合方式。
- Capture2Text：参考 Windows OCR 热键、区域选择、识别后写入剪贴板/调用外部程序。
- Text Grab：参考 Windows 本地 OCR 工具如何把不可选文本转成可复制文本。

### 约束

- 不承诺“任何窗口都能无损取选中文本”。UI Automation 依赖目标应用暴露 `TextPattern`，不支持时必须 fallback。
- 默认不静默破坏用户剪贴板。复制取词必须保存并恢复剪贴板；如果恢复失败，要记录日志并避免覆盖用户数据。
- OCR 是 fallback，不是默认路径。默认翻译选中文本应优先走 UIA 或剪贴板。
- 需要给用户设置项：
  - 是否允许临时改写剪贴板取词。
  - 是否启用复制监听。
  - 是否启用 OCR fallback。
  - 取词失败时是否显示手动输入窗口。

## 推荐原型顺序

1. 快捷键最小原型：
   - `RegisterHotKey` 注册 `Ctrl+Alt+E`。
   - 低级键盘 hook 只捕获 `Ctrl+C+C` 和双击 `Ctrl`。
   - 输出事件到日志，不接翻译接口。
2. 取词最小原型：
   - 先实现 UIA 读取选区。
   - 再实现保存/恢复剪贴板的复制取词。
   - 记录每次取词使用了哪种策略。
3. OCR fallback 原型：
   - 优先调研 Capture2Text / Text Grab 的可复用边界。
   - 不在首个原型里自研 OCR 框选和识别管线。

## 内置 Google 非官方免 Key 翻译

首版需要内置一个默认翻译提供方，让用户不配置 API Key 也能开箱使用。该提供方命名为 `Google Translate Free` 或类似名称，技术上参考开源项目对 Google Translate 非官方网页端点的使用方式。

必须明确：

- 它不是 Google Cloud Translation 官方 API。
- 它不需要 API Key，但稳定性不由 Google 官方承诺。
- 它可能遇到限流、403/429、返回格式变化、网络区域差异或服务策略变化。
- 失败时应提示用户切换到 OpenAI 兼容接口。
- 日志只记录 provider、状态码、错误类型和文本长度，不记录完整原文。

必须参考的开源项目：

- `translate-shell`：命令行翻译工具，支持 Google Translate，可参考请求和错误处理行为。
- `py-googletrans`：Python Google Translate 非官方库，明确使用 Google Translate Ajax/Web API。
- `deep-translator`：提供 `GoogleTranslator`，可参考免 Key 翻译提供方抽象。

## 待确认问题

- 项目是否必须保持宽松许可证。如果主程序未来不想被 GPL 传染，不能直接拷贝 GPL 项目代码，只能参考行为或以进程集成方式调用。
- 是否允许把 Capture2Text 这类外部工具作为可选集成，而不是内置依赖。
- 是否接受剪贴板短暂闪烁或剪贴板历史出现临时条目。如果不接受，复制取词策略的优先级要降低。

## 资料来源

- Microsoft Learn：`RegisterHotKey` 会在匹配热键时投递 `WM_HOTKEY` 消息。
  https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerhotkey
- Microsoft Learn：`SetWindowsHookEx` 支持 `WH_KEYBOARD_LL` / `WH_MOUSE_LL` 等低级 hook。
  https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexa
- Microsoft Learn：低级键盘 hook 要求安装 hook 的线程有消息循环。
  https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc
- Microsoft Learn：UI Automation `TextPattern` 用于访问文本控件内容和选区。
  https://learn.microsoft.com/en-us/dotnet/framework/ui-automation/ui-automation-textpattern-overview
- Microsoft Learn：`AddClipboardFormatListener` 可监听 `WM_CLIPBOARDUPDATE`。
  https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-addclipboardformatlistener
- `global-hotkey` crate：提供桌面应用全局快捷键注册。
  https://crates.io/crates/global-hotkey
- `Narsil/rdev`：全局监听/发送键盘鼠标事件的开源 Rust 库。
  https://github.com/Narsil/rdev
- GoldenDict-ng：开源词典软件，支持 Windows/Linux/macOS。
  https://github.com/xiaoyifang/goldendict-ng
- GoldenDict scan popup 相关历史 issue。
  https://github.com/goldendict/goldendict/issues/383
- GoldenDict-ng 与 Capture2Text OCR 集成文档。
  https://xiaoyifang.github.io/goldendict-ng/howto/ocr/
- Crow Translate：开源翻译工具，支持选区/屏幕翻译和可配置快捷键。
  https://invent.kde.org/office/crow-translate
- Crow Translate Windows 全局快捷键失效案例。
  https://github.com/crow-translate/crow-translate/issues/597
- Pot Desktop：开源选择翻译、剪贴板监听、截图 OCR 工具。
  https://github.com/pot-app/pot-desktop/blob/master/README_EN.md
- Capture2Text：Windows OCR 热键、识别结果写入剪贴板。
  https://capture2text.sourceforge.net/
- Text Grab：Windows 本地 OCR 开源工具。
  https://github.com/TheJoeFin/Text-Grab
- translate-shell：命令行翻译工具，支持 Google Translate。
  https://github.com/soimort/translate-shell
- py-googletrans：Google Translate 非官方 Python 库。
  https://github.com/ssut/py-googletrans
- deep-translator：提供免 Key GoogleTranslator 的 Python 翻译库。
  https://github.com/nidhaloff/deep-translator
- Google Cloud Translation 官方文档：正式 Cloud Translation 需要按官方方式配置项目和凭据。
  https://cloud.google.com/translate/docs/setup
