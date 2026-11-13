# ait

[![GitHub Release](https://img.shields.io/github/v/release/aitsuki/ait)](https://github.com/aitsuki/ait/releases/latest)

ait 是一个极致轻量 Windows 划词翻译
- 默认使用 Google 翻译，支持自定义大模型 API
- 支持离线单词词典，需在设置中开启 （感谢 [ECDICT](https://github.com/skywind3000/ECDICT)）
- 支持深色模式

## 关于便携版

> 两种版本的行为由编译特性决定，重命名 EXE 不会切换模式。自行编译便携版使用 `cargo build --release --features portable`，安装包内的程序使用 `cargo build --release`。

`ait-版本号-portable.exe`：便携版，无需安装直接运行。配置和离线词典保存在 EXE 旁边的 `data/` 目录，可将整个文件夹放进同步盘。便携版不记录日志，避免频繁触发网盘同步。
升级便携版时请先退出软件，替换 exe 文件，保留 `data/` 目录。

**注意：便携模式的 API Key 明文保存在 `data/settings.json`，任何能读取此文件的人都能使用密钥。不要分享整个软件目录，请保护同步盘账号和文件访问权限。** 设置窗口也会显示明文保存提示。

## 演示

<img width="422" height="368" alt="image" src="https://github.com/user-attachments/assets/cac714f2-0b25-45f6-832d-0c4dd04503f1" />

---

<img width="915" height="615" alt="image" src="https://github.com/user-attachments/assets/4f43771a-ac7b-4bc6-8607-1bf3070a9b07" />

