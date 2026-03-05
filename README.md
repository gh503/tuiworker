# TUIWorker

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

一个基于 Rust 的终端用户界面（TUI）工作站应用，为终端爱好者提供统一、高效的命令行工作环境。

## ✨ 项目亮点

- **纯 Rust 打造** - 所有功能模块使用 Rust 编写，不依赖外部命令行工具
- **模块化架构** - 插件式设计，按需启用功能模块
- **高性能** - 基于 ratatui 和 tokio，流畅的终端交互体验
- **可定制** - 支持主题、快捷键、模块配置的完全自定义
- **跨平台** - 支持 Linux、macOS 和 Windows
- **本地优先** - 所有数据存储在本地，保护隐私

## 功能特性

- 📁 **文件管理器** - 浏览文件系统，文本预览，打开外部文件，支持删除和重命名
- ✅ **待办事项** - 完整的 TODO 管理，支持标签、优先级
- 📝 **笔记** - Markdown 编辑，标签系统，快速记录
- 📅 **日记** - 日历视图，农历显示，节假日标记
- 🎵 **音乐播放** - 支持 FLAC/MP3/AAC 等无损格式
- 📮 **邮件收发** - IMAP/SMTP 客户端，纯 Rust 实现
- 📊 **项目跟踪** - 里程碑、风险、进度管理
- 💻 **终端模拟** - 嵌入式终端，多标签，分割窗口
- 🔀 **Git 操作** - 仓库状态、提交、日志查看
- 🎨 **可定制** - 自定义快捷键、主题、模块配置

## 技术栈

| 层次 | 技术 |
|------|------|
| TUI 框架 | ratatui + crossterm |
| 异步运行时 | tokio |
| 数据库 | sled |
| 配置 | config + serde |
| 日志 | log + fern |
| 语法高亮 | syntect |
| 音频播放 | symphonia + rodio |
| 邮件 | imap + lettre + mailparse |
| Git | git2 |
| 伪终端 | portable-pty |

## 快速开始

### 安装

**从源码编译**:
```bash
git clone https://github.com/gh503/tuiworker
cd tuiworker
cargo build --release
```

**下载预编译二进制文件**:
- 从 [Releases](https://github.com/gh503/tuiworker/releases) 页面下载

**通过包管理器安装**:

Ubuntu/Debian:
```bash
wget https://github.com/gh503/tuiworker/releases/download/v0.1.0-alpha/tuiworker_0.1.0-alpha_amd64.deb
sudo dpkg -i tuiworker_0.1.0-alpha_amd64.deb
```

Fedora/RHEL:
```bash
wget https://github.com/gh503/tuiworker/releases/download/v0.1.0-alpha/tuiworker.rpm
sudo rpm -i tuiworker.rpm
```

### 配置

首次运行会自动生成配置文件：

```bash
~/.config/tuiworker/config.toml
```

编辑配置文件可以自定义快捷键、主题和模块设置。

### 运行

```bash
cargo run
```

或

```bash
./target/release/tuiworker
```

## 项目结构

```
tuiworker/
├── crates/                  # Rust crate 工作空间
│   ├── core/               # 核心库 (Module trait, 事件系统)
│   ├── storage/            # 数据存储 (sled 封装)
│   ├── logging/            # 日志模块
│   └── modules/            # 功能模块
│       ├── filebrowser/
│       ├── todo/
│       ├── note/
│       ├── diary/
│       ├── music/
│       ├── project/
│       ├── mail/
│       ├── terminal/
│       └── git/
├── docs/                   # 文档
│   ├── prd.txt            # 产品需求文档
│   ├── implementation-plan.md  # 实现规划
│   └── api.md             # API 设计
└── src/                   # 二进制入口
```

## 开发状态

### 当前状态
- 🚧 **开发中** - 核心基础设施基本完成，FileBrowser 模块已实现核心功能

### 实现进度

**当前版本**: v0.1.0-alpha (仅 FileBrowser 模块可用，其他模块开发中)

| 模块 | 状态 | 说明 |
|------|------|------|
| 核心基础设施 | ✅ 完成 | Module trait、事件系统、应用框架 |
| 日志模块 | ✅ 完成 | 文件日志、输出控制、日志轮转 |
| 数据存储 | ✅ 完成 | sled 数据库封装 |
| 配置管理 | ✅ 完成 | XDG 目录支持、TOML 配置 |
| TUI 渲染基础 | ✅ 完成 | ratatui 集成、状态栏、标签栏 |
| FileBrowser | ✅ 可用 | 文件浏览、搜索、状态栏集成 |
| Todo | ⬜ 框架 | 基础结构已实现 |
| Note | ⬜ 框架 | 基础结构已实现 |
| Diary | ⬜ 框架 | 基础结构已实现 |
| Terminal | ⬜ 框架 | 基础结构已实现 |
| Git | ⬜ 框架 | 基础结构已实现 |
| Music | ⬜ 框架 | 基础结构已实现 |
| Project | ⬜ 框架 | 基础结构已实现 |
| Mail | ⬜ 框架 | 基础结构已实现 |
| 快捷键系统 | ✅ 完成 | 模块间快捷键绑定 |
| 主题支持 | 🟡 开发中 | 基础主题系统 |

**注**: v0.1.0-alpha 仅提供 FileBrowser 功能演示，其他模块将在后续版本逐步实现。

### 发布计划

- **v0.1.0-alpha** - 当前版本（仅 FileBrowser 完成，其他模块框架已建立）
- **v0.1.0** - MVP (FileBrowser + Todo + Note + Diary + Terminal)
- **v0.2.0** - + Git 操作模块
- **v0.3.0** - + Music 播放模块
- **v0.4.0** - + Project 跟踪模块
- **v0.5.0** - + Mail 邮件模块
- **v1.0.0** - 完整功能稳定版

## 快捷键

### 全局快捷键

| 快捷键 | 功能 |
|--------|------|
| `q` | 退出应用 |
| `Ctrl + Tab` | 切换到下一个模块 |
| `Ctrl + Shift + Tab` | 切换到上一个模块 |
| `?` | 显示帮助 |

### 模块快捷键

每个模块有自己的一套快捷键，按 `?` 键查看当前模块的帮助。

## 配置示例

```toml
[general]
log_level = "info"
log_to_file = true
theme = "default"

[shortcuts]
global_quit = "q"
switch_tab_next = "Ctrl+Right"
switch_tab_prev = "Ctrl+Left"

[modules]
enabled = ["filebrowser", "todo", "note", "diary", "terminal", "git"]

[filebrowser]
show_hidden = false
sort_by = "name"

[todo]
default_priority = "medium"

[terminal]
default_shell = "bash"
```

## 设计原则

- **纯 Rust 实现** - 所有功能模块使用 Rust 编写，不依赖外部程序
- **模块化** - 每个功能独立为模块，便于开发和扩展
- **事件驱动** - 基于事件循环处理用户输入和异步任务
- **最小依赖** - 核心功能使用纯 Rust 库
- **用户可定制** - 支持自定义快捷键、主题和模块开关
- **可观测性** - 内置日志系统，便于开发和调试

## 数据存储

- **数据库**: `sled` 嵌入式键值存储
- **位置**: `~/.local/share/tuiworker/db`
- **笔记**: `~/notes` (Markdown 文件)
- **日记**: `~/diary` (Markdown 文件)
- **日志**: `~/.local/share/tuiworker/logs/`

## 贡献指南

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

### 开发流程

```bash
# 克隆项目
git clone https://github.com/gh503/tuiworker
cd tuiworker

# 运行测试
cargo test

# 运行 linter
cargo clippy

# 格式化代码
cargo fmt

# 构建
cargo build --release

# 构建 DEB 和 RPM 包
make package-all
```

### 发布流程

**自动化发布**（推荐）:

1. **打 tag**（会自动创建 draft release）:
   ```bash
   git tag v0.1.0-alpha
   git push origin v0.1.0-alpha
   ```

2. **在 GitHub 上发布**:
   - 访问 https://github.com/gh503/tuiworker/releases
   - 查看自动生成的 Release Notes
   - 点击 "Publish release" 按钮

3. **自动构建并上传**:
   - 5 个平台的二进制文件
   - DEB 和 RPM 包
   - 所有文件自动添加到 Release

**Commit Message 规范**（影响自动生成的 Release Notes）:

```
<type>: <subject>

<body>

<footer>
```

**Type 类型**:
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建过程或工具变更

**示例**:
```bash
git commit -m "feat(filebrowser): 添加按文件大小排序功能"
git commit -m "fix: 修复在 macOS 上的显示问题"
git commit -m "docs: 更新 README 安装说明"
```

## 文档

- [产品需求文档](docs/prd.txt) - 系统顶层设计
- [实现规划](docs/implementation-plan.md) - 详细的实现阶段和任务
- [API 文档](docs/api.md) - 核心接口定义

## 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

## 致谢

感谢以下优秀的 Rust 库：

- [ratatui](https://github.com/ratatui-org/ratatui) - TUI 框架
- [sled](https://github.com/spacejam/sled) - 嵌入式数据库
- [tokio](https://tokio.rs/) - 异步运行时
- [symphonia](https://github.com/pdeljanov/Symphonia) - 音频解码
- [cargo-deb](https://github.com/kornelski/cargo-deb) - DEB 包构建工具
- [cargo-generate-rpm](https://github.com/cat-in-135/cargo-generate-rpm) - RPM 包构建工具

## 联系方式

- Issue: [GitHub Issues](https://github.com/gh503/tuiworker/issues)
- Email: angus_robot@163.com

---

**当前版本**: v0.1.0-alpha
**更新时间**: 2026-03-05
**仓库地址**: https://github.com/gh503/tuiworker

**注意**: 当前为开发中版本，仅 FileBrowser 模块可用。其他模块正在逐步开发中。

## 提高项目曝光度

如果您觉得这个项目有用，欢迎帮助推广：

1. **Star 本项目** - 增加 GitHub 搜索排名
2. **分享给朋友** - 推荐给使用终端的开发者
3. **撰写博客** - 如果您使用了本项目，欢迎分享使用体验
4. **提交 Issue** - 报告 Bug 或提出功能建议
5. **贡献代码** - 欢迎提交 PR 参与开发
6. **推广渠道** - 在 Reddit (r/rust, r/linux), Hacker News, V2EX 等平台分享
