# 配置说明 / Configuration

## 中文 / Chinese

### 配置文件位置 / Config File Location

首次运行应用时会自动创建配置文件：

The config file is automatically created on first run:

- **Linux**: `~/.config/tuiworker/config.toml`
- **macOS**: `~/.config/tuiworker/config.toml`
- **Windows**: `%APPDATA%\tuiworker\config.toml`

### 配置示例 / Configuration Example

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

### 配置选项 / Configuration Options

#### general 部分

| 选项 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `log_level` | string | "info" | 日志级别: trace, debug, info, warn, error |
| `log_to_file` | boolean | true | 是否输出日志到文件 |
| `theme` | string | "default" | 主题名称 |

#### shortcuts 部分

自定义快捷键，支持的按键：

- 单字符: `a`, `b`, `c`, ...
- 功能键: `F1` - `F12`
- 方向键: `Up`, `Down`, `Left`, `Right`
- 控制键: `Ctrl+Key`, `Shift+Key`, `Alt+Key`

#### modules 部分

| 选项 | 类型 | 描述 |
|------|------|------|
| `enabled` | array | 启用的模块列表 |

可用模块: `filebrowser`, `todo`, `note`, `diary`, `terminal`, `git`, `music`, `project`, `mail`

---

## English

### Config File Location

The config file is automatically created on first run:

- **Linux**: `~/.config/tuiworker/config.toml`
- **macOS**: `~/.config/tuiworker/config.toml`
- **Windows**: `%APPDATA%\tuiworker\config.toml`

### Configuration Example

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

### Configuration Options

#### general section

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `log_level` | string | "info" | Log level: trace, debug, info, warn, error |
| `log_to_file` | boolean | true | Whether to output logs to file |
| `theme` | string | "default" | Theme name |

#### shortcuts section

Customize shortcuts. Supported keys:

- Single characters: `a`, `b`, `c`, ...
- Function keys: `F1` - `F12`
- Arrow keys: `Up`, `Down`, `Left`, `Right`
- Modifier keys: `Ctrl+Key`, `Shift+Key`, `Alt+Key`

#### modules section

| Option | Type | Description |
|--------|------|-------------|
| `enabled` | array | List of enabled modules |

Available modules: `filebrowser`, `todo`, `note`, `diary`, `terminal`, `git`, `music`, `project`, `mail`
