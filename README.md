# rust_flipit

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**FlipIt 翻页时钟屏幕保护程序 — Rust 重写版**

![效果图](./res/logo.png)

翻页时钟风格的 Windows 屏幕保护程序，支持当前时间显示和世界时间多城市时钟。本项目使用 **Rust + windows-rs** 从零重写，零外部运行时依赖。

## 项目起源

本程序基于 [phaselden/FlipIt](https://github.com/phaselden/FlipIt)（C# / .NET Framework 4.8 / Windows Forms + GDI+）使用 **Rust** 语言完全重写。

### 与原版的差异

| 特性 | 原版 (C#) | 本版 (Rust) |
|------|-----------|-------------|
| 运行时依赖 | .NET Framework 4.8 | **无**（纯原生 PE） |
| 字体颜色 | 固定 `#B7B7B7` | **可自定义**（预设7色 + 透明度滑块） |
| 默认时间制 | 12 小时制 | **24 小时制** |
| 默认字号 | Scale=70 | **Scale=100（最大盒子）** |
| 技术栈 | Windows Forms + GDI+ | **windows-rs + GDI+** |


## 新增功能

- **🎨 字体颜色选择器** — 7 种预设颜色（Default / White / Green / Red / Blue / Cyan / Gray）
- **🔆 透明度控制** — 滑块调节文字透明度（0~255）
- **🖥️ 双缓冲渲染** — 消除屏闪，画面更流畅
- **📦 零依赖部署** — 单 .scr 文件即用，无需安装任何运行时

## 技术栈

| 组件 | 说明 |
|------|------|
| **rust_flipit** | Rust (edition 2021) |
| **windows-rs** (v0.60) | 完整的 Windows API 绑定（Win32 / GDI / GDI+） |
| **chrono** | 时区转换与时间格式化 |
| **winresource** | PE 资源嵌入（图标、版本信息） |

### 核心 Windows API 覆盖

通过 `windows-rs` 调用以下原生 Win32 API：

| 类别 | API |
|------|-----|
| **窗口管理** | `CreateWindowExW` · `RegisterClassW` · `DefWindowProcW` · `PeekMessageW` |
| **GDI+ 渲染** | `GdipCreatePath` · `GdipAddPathArc` · `GdipCreateLineBrushFromRect` · `GdipFillPath` · `GdipDrawString` |
| **GDI 文本** | `CreateFontW` · `DrawTextW` · `SelectObject` · `SetTextColor` |
| **双缓冲** | `CreateCompatibleDC` · `CreateCompatibleBitmap` · `BitBlt` |
| **屏保协议** | `SetParent` · `SetWindowLongPtrW` · `ShowCursor(FALSE)` |

## 安装

1. 从 [Releases](https://github.com/liq45/rust_flipit/releases) 下载 `rust_flipit.scr`
2. 以管理员身份复制到系统目录：
   ```bash
   copy rust_flipit.scr %SystemRoot%\system32\
   ```
3. 桌面右键 → **个性化** → **锁屏界面** → **屏幕保护程序设置**
4. 选择 **rust_flipit** → 点击 **设置** 自定义外观

## 自行编译

```bash
# 编译 Release 版本
cargo build --release

编译要求：
- Rust 1.75+
- Windows 目标平台
- MinGW-w64 或 MSVC 链接器

## 许可协议

本 Rust 重写版采用 **MIT** 许可协议。

Copyright (c) 2025 liq45

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

---

*原版 [phaselden/FlipIt](https://github.com/phaselden/FlipIt) 采用 CC0 1.0 许可协议。*
