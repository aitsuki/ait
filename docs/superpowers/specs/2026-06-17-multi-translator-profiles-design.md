# 多接口翻译配置设计

## 背景

当前应用只有一个默认提供方和一个 OpenAI 配置。设置面板使用自由文本框编辑 provider、快捷键、Base URL、Model、API Key 和复制等待时间，交互不适合作为后续多接口配置的基础。翻译窗口也没有配置选择能力，运行期只在启动时读取一次配置，保存设置后不能立即影响当前进程。

目标是支持多个翻译接口配置，覆盖 OpenAI、Claude、Gemini、DeepSeek 和自定义接口。界面文案不出现“兼容接口”，只显示用户可理解的供应商或自定义配置名称。

## 目标

- 设置面板支持新增、编辑、删除、保存和设为默认多个翻译配置。
- 默认初始配置包含 Google、OpenAI、Claude、Gemini、DeepSeek。
- Google 是内置配置，不可删除。
- 用户可以创建自定义配置，并自定义名称、Base URL、模型、API Key 和超时时间。
- 翻译面板显示配置下拉框，可选择任意配置。
- 翻译面板切换配置后保存为默认配置，并立即用当前原文重新翻译。
- 快捷键翻译、窗口内翻译按钮、配置切换重译都使用同一套配置解析和翻译器构建路径。
- 设置保存后影响当前运行进程，不要求用户重启应用。
- 旧配置能够迁移到新的配置列表，避免已有 API Key 丢失。

## 非目标

- 不实现 Claude、Gemini 的原生协议适配。
- 不在界面显示“兼容接口”字样。
- 不加入推理模式或思考模式配置。
- 不实现模型列表远程拉取。
- 不实现每个供应商的全部高级参数。
- 不重做快捷键设置交互，除非它影响本需求的数据结构迁移。

## 用户体验

### 设置面板

设置面板改为配置列表式管理。

左侧是翻译配置列表，显示配置名称、默认标记和内置标记。右侧是当前配置详情。

详情字段：

- 名称
- 供应商：Google、OpenAI、Claude、Gemini、DeepSeek、自定义
- Base URL
- 模型
- API Key
- 超时时间

操作：

- 新增：创建一个自定义配置，并选中它。
- 删除：删除当前配置。Google 配置不可删除，选中 Google 时删除按钮禁用。默认配置被删除时，自动选择列表中的第一个可用配置为默认。
- 保存：校验并保存当前配置。
- 设为默认：将当前配置保存为默认配置。

供应商字段用于填充模板默认值和日志分类，不强制限制 Base URL 或模型。用户把供应商选为 Claude、Gemini 或 DeepSeek 时，界面只显示这些名称，不显示协议细节。Google 使用现有免 Key 翻译路径，不要求 Base URL、模型或 API Key。选中 Google 时，Base URL、模型、API Key 和超时时间字段不参与编辑和保存。

### 翻译面板

翻译窗口顶部增加配置下拉框，显示所有配置名称。当前选择等于默认配置。

切换下拉框时：

1. 将选择的配置保存为默认配置。
2. 如果原文为空，只更新选择，不发起翻译。
3. 如果原文非空，立即用当前原文重新翻译。
4. 重译期间显示加载状态。
5. 重译成功时更新译文。
6. 重译失败时保留原文，显示错误，不清空原有原文。

窗口内“翻译”按钮使用当前选中的配置翻译当前原文。

快捷键翻译使用默认配置翻译新捕获的文本。

## 数据模型

新增配置结构：

```rust
pub struct TranslatorProfile {
    pub id: String,
    pub name: String,
    pub provider: TranslatorProvider,
    pub built_in: bool,
    pub base_url: String,
    pub model: String,
    pub encrypted_api_key: Option<String>,
    pub timeout_secs: u64,
}

pub enum TranslatorProvider {
    Google,
    OpenAi,
    Claude,
    Gemini,
    DeepSeek,
    Custom,
}

pub struct AppSettings {
    pub default_profile_id: String,
    pub translator_profiles: Vec<TranslatorProfile>,
    pub hotkey: String,
    pub target_language: String,
    pub clipboard_capture: ClipboardCaptureSettings,
    pub window: WindowSettings,
    pub markdown: MarkdownSettings,
}
```

`TranslatorProvider` 代表 UI 分类、默认模板和日志名。第一版除 Google 外，其他配置都通过同一个 Chat Completions 风格请求实现翻译。Google 继续使用现有免 Key 翻译实现。

默认配置：

- Google：沿用现有免 Key 翻译实现，作为无需配置 API Key 的默认可用配置。
- OpenAI：`https://api.openai.com/v1`，默认模型沿用当前配置。
- Claude：使用 Anthropic 提供的 OpenAI 风格入口默认 Base URL。
- Gemini：使用 Google Gemini 提供的 OpenAI 风格入口默认 Base URL。
- DeepSeek：使用 DeepSeek OpenAI 风格入口默认 Base URL。

具体默认模型可以选择保守值，并允许用户编辑。

## 迁移

加载配置时支持旧结构。

如果旧配置包含 `openai` 字段：

- 始终创建或补齐一个内置 `Google` profile。
- 将旧 `openai` 配置迁移成一个 `OpenAI` profile。
- 如果旧默认提供方是 OpenAI，并且存在 API Key，则默认配置指向这个 profile。
- 如果旧默认提供方是 Google 免费翻译，则默认配置指向 `Google` profile，保持升级前的默认行为。

迁移后保存新结构。无法解析的配置文件仍沿用现有备份机制。

## 翻译器构建

新增 `TranslatorProfile` 到翻译器的构建函数。

构建规则：

- 必须存在选中的 profile。
- Google profile 不需要 API Key，直接使用现有 Google 翻译器。
- 非 Google profile 的 API Key 为空时返回可理解错误。
- 非 Google profile 解密失败时返回可理解错误。
- 非 Google profile 的 Base URL、模型、超时从 profile 读取。
- 翻译请求使用统一提示词：将用户文本翻译为目标语言，只返回译文，不返回解释、分析或其他内容。

日志记录 provider、profile id 或 profile name、文本长度和结果状态，不记录原文、译文或 API Key。

## 应用状态

主循环需要持有可更新的应用状态，而不是只在启动时创建不可变 `settings` 和 `workflow`。

建议引入：

```rust
struct AppRuntimeState {
    settings: AppSettings,
    active_profile_id: String,
}
```

每次翻译时根据当前 profile 构建或获取翻译器。可以先不做缓存，优先保证配置切换后立即生效。

设置面板保存后应通知主循环重新加载 settings，并刷新翻译窗口下拉框。

翻译窗口下拉选择后应通知主循环更新默认 profile，并触发当前原文重译。

## 错误处理

- 无配置：设置面板允许创建配置；翻译时提示需要先添加配置。
- API Key 缺失：提示当前配置缺少 API Key。
- 配置被删除：翻译窗口刷新列表并选择默认配置。
- 尝试删除 Google：UI 禁用删除动作；底层删除逻辑也拒绝删除内置配置。
- 保存失败：设置窗口显示保存失败，不关闭窗口。
- 切换配置重译失败：翻译窗口保留原文并显示错误。
- 当前原文为空：切换配置只保存默认，不显示错误。

## 测试

单元测试：

- 默认设置包含 Google、OpenAI、Claude、Gemini、DeepSeek 配置。
- Google 配置不可删除。
- Google 配置不要求 API Key、Base URL 或模型。
- 旧配置能迁移为新 profile 列表。
- 默认 profile id 能解析到 profile。
- 删除默认配置后能选择新的默认配置。
- 翻译面板选择配置会映射到“保存默认并重译当前原文”的动作。
- 原文为空时切换配置不触发翻译。
- 配置构建翻译器时不会记录 API Key 或原文。

集成或窗口逻辑测试：

- 设置保存后运行状态刷新。
- 翻译窗口下拉列表来自配置列表。
- 切换配置后使用新 profile 翻译当前原文。

## 验收

- 用户可以在设置面板新增、编辑、删除多个翻译配置。
- Google 作为内置配置存在，不能被删除。
- 用户可以将任意配置设为默认。
- 翻译面板可以选择配置。
- 翻译面板切换配置后，默认配置被更新，并在当前原文非空时立即重新翻译。
- Google、OpenAI、Claude、Gemini、DeepSeek 在界面中作为供应商名称出现，不出现“兼容接口”字样。
- 保存设置后无需重启即可用于后续翻译。
- 旧配置不会因为升级丢失 API Key。
