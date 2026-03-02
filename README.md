# TUI Workstation

一个基于 Rust 的终端用户界面（TUI）工作站应用，为终端爱好者提供统一、高效的命令行工作环境。

## 功能特性

- 📁 **文件管理器** - 浏览文件系统，文本预览，打开外部文件
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

```bash
# 从源码编译
git clone https://github.com/yourname/tui-workstation
cd tui-workstation
cargo build --release

# 或下载预编译二进制文件（暂未发布）
```

### 配置

首次运行会自动生成配置文件：

```bash
~/.config/tui-workstation/config.toml
```

编辑配置文件可以自定义快捷键、主题和模块设置。

### 运行

```bash
cargo run
```

或

```bash
./target/release/tui-workstation
```

## 项目结构

```
tuiworkstation/
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
- 📝 **规划阶段** - 项目架构和 API 设计完成

### 实现进度

| 模块 | 状态 |
|------|------|
| 核心基础设施 | ⬜ 待开始 |
| 日志模块 | ⬜ 待开始 |
| 数据存储 | ⬜ 待开始 |
| 配置管理 | ⬜ 待开始 |
| TUI 渲染基础 | ⬜ 待开始 |
| FileBrowser | ⬜ 待开始 |
| Todo | ⬜ 待开始 |
| Note | ⬜ 待开始 |
| Diary | ⬜ 待开始 |
| Terminal | ⬜ 待开始 |
| Git | ⬜ 待开始 |
| Music | ⬜ 待开始 |
| Project | ⬜ 待开始 |
| Mail | ⬜ 待开始 |
| 快捷键系统 | ⬜ 待开始 |
| 主题支持 | ⬜ 待开始 |

### 发布计划

- **v0.1.0** - MVP (5 核心模块)
- **v0.2.0** - + Git, Terminal 多标签
- **v0.3.0** - + Music 模块
- **v0.4.0** - + Project 模块
- **v0.5.0** - + Mail 模块
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
- **位置**: `~/.local/share/tui-workstation/db`
- **笔记**: `~/notes` (Markdown 文件)
- **日记**: `~/diary` (Markdown 文件)
- **日志**: `~/.local/share/tui-workstation/logs/`

## 贡献指南

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

### 开发流程

```bash
# 克隆项目
git clone https://github.com/yourname/tui-workstation
cd tui-workstation

# 运行测试
cargo test

# 运行 linter
cargo clippy

# 格式化代码
cargo fmt

# 构建
cargo build --release
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

## 联系方式

- Issue: [GitHub Issues](https://github.com/yourname/tui-workstation/issues)
- Email: your.email@example.com

---

**当前版本**: 计划中
**更新时间**: 2025-03-02
