# 开发者指南 / Developer Guide

## 中文 / Chinese

### 开发环境设置 / Development Environment Setup

```bash
# 克隆项目
git clone https://github.com/gh503/tuiworker
cd tuiworker

# 安装 Rust (如果没有)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 运行开发版本
cargo run

# 运行测试
cargo test

# 运行 linter
cargo clippy

# 格式化代码
cargo fmt
```

### 项目结构 / Project Structure

```
tuiworker/
├── crates/                  # Rust crate 工作空间
│   ├── core/               # 核心库 (Module trait, 事件系统)
│   ├── storage/            # 数据存储 (sled 封装)
│   ├── logging/            # 日志模块
│   ├── config_manager/    # 配置管理
│   ├── ui/                # UI 组件
│   └── modules/            # 功能模块
│       ├── filebrowser/   # 文件浏览器
│       ├── todo/          # 待办事项
│       ├── note/          # 笔记
│       ├── diary/         # 日记
│       ├── music/         # 音乐播放
│       ├── terminal/      # 终端模拟
│       ├── git/           # Git 操作
│       ├── project/       # 项目跟踪
│       └── mail/          # 邮件
├── src/                    # 二进制入口
├── docs/                   # 文档
└── .github/               # GitHub Actions
```

### 添加新模块 / Adding a New Module

1. 在 `crates/modules/` 下创建新目录
2. 创建 `Cargo.toml` 和 `src/lib.rs`
3. 实现 `Module` trait
4. 在工作空间 `Cargo.toml` 中注册
5. 在主应用中注册模块

示例 / Example:

```rust
use core::module::Module;
use core::event::Action;
use crossterm::event::Event as CrosstermEvent;
use ratatui::{layout::Rect, Frame};

pub struct MyModule {
    name: String,
}

impl Module for MyModule {
    fn name(&self) -> &str {
        &self.name
    }

    fn title(&self) -> &str {
        "My Module"
    }

    fn update(&mut self, event: CrosstermEvent) -> Action {
        Action::None
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        // 绘制 UI
    }

    fn save(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn load(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
```

### 提交规范 / Commit Convention

```
<type>: <subject>

<body>

<footer>
```

**Type 类型**:
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建过程或工具变更

---

## English

### Development Environment Setup

```bash
# Clone the project
git clone https://github.com/gh503/tuiworker
cd tuiworker

# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Run development version
cargo run

# Run tests
cargo test

# Run linter
cargo clippy

# Format code
cargo fmt
```

### Project Structure

```
tuiworker/
├── crates/                  # Rust crate workspace
│   ├── core/               # Core library (Module trait, event system)
│   ├── storage/            # Data storage (sled wrapper)
│   ├── logging/            # Logging module
│   ├── config_manager/    # Configuration management
│   ├── ui/                # UI components
│   └── modules/            # Feature modules
│       ├── filebrowser/   # File browser
│       ├── todo/          # Todo list
│       ├── note/          # Notes
│       ├── diary/         # Diary
│       ├── music/         # Music player
│       ├── terminal/      # Terminal emulator
│       ├── git/           # Git operations
│       ├── project/       # Project tracking
│       └── mail/          # Email
├── src/                    # Binary entry point
├── docs/                   # Documentation
└── .github/               # GitHub Actions
```

### Adding a New Module

1. Create a new directory under `crates/modules/`
2. Create `Cargo.toml` and `src/lib.rs`
3. Implement the `Module` trait
4. Register in workspace `Cargo.toml`
5. Register in the main application

Example:

```rust
use core::module::Module;
use core::event::Action;
use crossterm::event::Event as CrosstermEvent;
use ratatui::{layout::Rect, Frame};

pub struct MyModule {
    name: String,
}

impl Module for MyModule {
    fn name(&self) -> &str {
        &self.name
    }

    fn title(&self) -> &str {
        "My Module"
    }

    fn update(&mut self, event: CrosstermEvent) -> Action {
        Action::None
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        // Draw UI
    }

    fn save(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn load(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
```

### Commit Convention

```
<type>: <subject>

<body>

<footer>
```

**Type**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation update
- `style`: Code formatting
- `refactor`: Refactoring
- `perf`: Performance optimization
- `test`: Test related
- `chore`: Build process or tooling changes
