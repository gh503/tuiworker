# TUI Workstation

A terminal-based workstation application built with Rust, providing a unified and efficient command-line working environment for terminal enthusiasts.

## Features

- 📁 **File Browser** - Browse filesystem, text preview, open external files
- ✅ **Todo List** - Complete todo management with tags and priorities
- 📝 **Notes** - Markdown editing, tagging system, quick notes
- 📅 **Diary** - Calendar view, lunar calendar, holiday markers
- 🎵 **Music Player** - Support for lossless formats: FLAC/MP3/AAC
- 📮 **Mail Client** - IMAP/SMTP client, pure Rust implementation
- 📊 **Project Tracker** - Milestones, risks, progress management
- 💻 **Terminal Emulator** - Embedded terminal, multi-tab, split windows
- 🔀 **Git Integration** - Repository status, commits, log viewer
- 🎨 **Customizable** - Custom shortcuts, themes, module configuration

## Tech Stack

| Layer | Technology |
|-------|-----------|
| TUI Framework | ratatui + crossterm |
| Async Runtime | tokio |
| Database | sled |
| Configuration | config + serde |
| Logging | log + fern |
| Syntax Highlighting | syntect |
| Audio Playback | symphonia + rodio |
| Email | imap + lettre + mailparse |
| Git | git2 |
| PTY | portable-pty |

## Quick Start

### Installation

```bash
# Compile from source
git clone https://github.com/yourname/tui-workstation
cd tui-workstation
cargo build --release

# Or download pre-built binaries (not released yet)
```

### Configuration

First run will automatically create configuration file:

```bash
~/.config/tui-workstation/config.toml
```

Edit config file to customize shortcuts, themes, and module settings.

### Running

```bash
cargo run
```

or

```bash
./target/release/tui-workstation
```

## Project Structure

```
tuiworkstation/
├── crates/                  # Rust crate workspace
│   ├── core/               # Core library (Module trait, event system)
│   ├── storage/            # Data storage (sled wrapper)
│   ├── logging/            # Logging module
│   └── modules/            # Feature modules
│       ├── filebrowser/
│       ├── todo/
│       ├── note/
│       ├── diary/
│       ├── music/
│       ├── project/
│       ├── mail/
│       ├── terminal/
│       └── git/
├── docs/                   # Documentation
│   ├── prd.txt            # Product requirements document
│   ├── implementation-plan.md  # Implementation plan
│   └── api.md             # API design
└── src/                   # Binary entry
```

## Development Status

### Current Status
- ✅ **Completed** - All core infrastructure and modules implemented

### Implementation Progress

| Module | Status |
|--------|--------|
| Core Infrastructure | ✅ Complete |
| Logging Module | ✅ Complete |
| Data Storage | ✅ Complete |
| Configuration Manager | ✅ Complete |
| TUI Rendering Base | ✅ Complete |
| FileBrowser | ✅ Complete |
| Todo | ✅ Complete |
| Note | ✅ Complete |
| Diary | ✅ Complete |
| Terminal | ✅ Complete |
| Git | ✅ Complete |
| Music | ✅ Complete |
| Project | ✅ Complete |
| Mail | ✅ Complete |
| Shortcuts System | ✅ Complete |
| Theme Support | ✅ Complete |

### Release Plan

- **v0.1.0** - MVP (8 core modules)
- **v0.2.0** - + Enhanced terminal features
- **v0.3.0** - + Advanced music features
- **v0.4.0** - + Enhanced project management
- **v0.5.0** - + Advanced mail features
- **v1.0.0** - Full-featured stable release

## Keyboard Shortcuts

### Global Shortcuts

| Shortcut | Function |
|----------|----------|
| `q` | Quit application |
| `Ctrl + Tab` | Switch to next module |
| `Ctrl + Shift + Tab` | Switch to previous module |
| `?` | Show help |

### Module Shortcuts

Each module has its own shortcut set. Press `?` key to view help for the current module.

## Configuration Example

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

## Design Principles

- **Pure Rust Implementation** - All functionality modules written in Rust, no external program dependencies
- **Modular** - Each functional feature is an independent module for easy development and extension
- **Event-Driven** - Event loop handles user input and async tasks
- **Minimal Dependencies** - Core functionality uses pure Rust libraries
- **User Customizable** - Supports custom shortcuts, themes, and module toggles
- **Observable** - Built-in logging system for easy development and debugging

## Data Storage

- **Database**: `sled` embedded key-value storage
- **Location**: `~/.local/share/tui-workstation/db`
- **Notes**: `~/notes` (Markdown files)
- **Diary**: `~/diary` (Markdown files)
- **Logs**: `~/.local/share/tui-workstation/logs/`

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Development Workflow

```bash
# Clone the project
git clone https://github.com/yourname/tui-workstation
cd tui-workstation

# Run tests
cargo test

# Run linter
cargo clippy

# Format code
cargo fmt

# Build
cargo build --release
```

## Documentation

- [Product Requirements Document](docs/prd.txt) - System top-level design
- [Implementation Plan](docs/implementation-plan.md) - Detailed implementation stages and tasks
- [API Documentation](docs/api.md) - Core interface definitions

## License

MIT License - see [LICENSE](LICENSE) file for details

## Acknowledgments

Thanks to these excellent Rust libraries:

- [ratatui](https://github.com/ratatui-org/ratatui) - TUI framework
- [sled](https://github.com/spacejam/sled) - Embedded database
- [tokio](https://tokio.rs/) - Async runtime
- [symphonia](https://github.com/pdeljanov/Symphonia) - Audio decoding

## Contact

- Issue Tracker: [GitHub Issues](https://github.com/yourname/tui-workstation/issues)
- Email: your.email@example.com

---

**Current Version**: 0.1.0-alpha
**Last Updated**: 2025-03-02
