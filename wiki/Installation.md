# 安装指南 / Installation Guide

## 中文 / Chinese

### 从源码编译 / Build from Source

```bash
# 克隆项目
git clone https://github.com/gh503/tuiworker
cd tuiworker

# 编译 Release 版本
cargo build --release

# 运行
./target/release/tuiworker
```

### 下载预编译二进制 / Download Pre-built Binaries

从 [Releases](https://github.com/gh503/tuiworker/releases) 页面下载对应平台的二进制文件：

- **Linux**: `tuiworker-glibc2.35` (Ubuntu 22.04+), `tuiworker-glibc2.39` (Ubuntu 24.04+)
- **macOS**: `tuiworker-macos-x86_64` (Intel), `tuiworker-macos-arm64` (Apple Silicon)
- **Windows**: `tuiworker-windows.exe`

### 通过包管理器安装 / Install via Package Manager

#### Ubuntu / Debian

```bash
wget https://github.com/gh503/tuiworker/releases/download/v0.1.0-alpha/tuiworker_0.1.0-alpha_amd64.deb
sudo dpkg -i tuiworker_0.1.0-alpha_amd64.deb
```

#### Fedora / RHEL / CentOS

```bash
wget https://github.com/gh503/tuiworker/releases/download/v0.1.0-alpha/tuiworker.rpm
sudo rpm -i tuiworker.rpm
```

### 系统要求 / System Requirements

- **Linux**: GLIBC 2.35+ (Ubuntu 22.04+, Debian 12+)
- **macOS**: 10.15+ (Catalina)
- **Windows**: Windows 10+

---

## English

### Build from Source

```bash
# Clone the project
git clone https://github.com/gh503/tuiworker
cd tuiworker

# Build release version
cargo build --release

# Run
./target/release/tuiworker
```

### Download Pre-built Binaries

Download the pre-built binary for your platform from the [Releases](https://github.com/gh503/tuiworker/releases) page:

- **Linux**: `tuiworker-glibc2.35` (Ubuntu 22.04+), `tuiworker-glibc2.39` (Ubuntu 24.04+)
- **macOS**: `tuiworker-macos-x86_64` (Intel), `tuiworker-macos-arm64` (Apple Silicon)
- **Windows**: `tuiworker-windows.exe`

### Install via Package Manager

#### Ubuntu / Debian

```bash
wget https://github.com/gh503/tuiworker/releases/download/v0.1.0-alpha/tuiworker_0.1.0-alpha_amd64.deb
sudo dpkg -i tuiworker_0.1.0-alpha_amd64.deb
```

#### Fedora / RHEL / CentOS

```bash
wget https://github.com/gh503/tuiworker/releases/download/v0.1.0-alpha/tuiworker.rpm
sudo rpm -i tuiworker.rpm
```

### System Requirements

- **Linux**: GLIBC 2.35+ (Ubuntu 22.04+, Debian 12+)
- **macOS**: 10.15+ (Catalina)
- **Windows**: Windows 10+
