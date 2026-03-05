# TUI Workstation 实现规划

## 项目概览
基于 Rust 的终端用户界面（TUI）工作站应用，集成文件管理、待办、笔记、日记、音乐、邮件、项目跟踪、终端模拟、Git 操作等功能。

## 目录结构设计

```
tuiworkstation/
├── Cargo.toml                    # 工作空间配置
├── Cargo.lock
├── README.md
├── docs/
│   ├── prd.txt                   # 产品需求文档（已存在）
│   ├── implementation-plan.md    # 本文档
│   └── api.md                    # API 设计文档
├── crates/
│   ├── core/                     # 核心库
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── app.rs           # 主应用程序
│   │       ├── module.rs        # Module trait 定义
│   │       ├── event.rs         # 事件系统
│   │       ├── render.rs        # 渲染系统
│   │       └── keyboard.rs      # 快捷键系统
│   │
│   ├── storage/                  # 数据存储库
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── database.rs      # sled 封装
│   │       ├── config.rs        # 配置管理
│   │       └── namespaces.rs    # 命名空间管理
│   │
│   ├── logging/                  # 日志库
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── init.rs          # 日志初始化
│   │       ├── file_logger.rs   # 文件日志
│   │       └── console_logger.rs # 终端日志
│   │
│   └── modules/                  # 功能模块集合
│       ├── Cargo.toml           # 模块工作空间
│       ├── filebrowser/
│       │   └── src/lib.rs
│       ├── todo/
│       │   └── src/lib.rs
│       ├── note/
│       │   └── src/lib.rs
│       ├── diary/
│       │   └── src/lib.rs
│       ├── music/
│       │   └── src/lib.rs
│       ├── project/
│       │   └── src/lib.rs
│       ├── mail/
│       │   └── src/lib.rs
│       ├── terminal/
│       │   └── src/lib.rs
│       └── git/
│           └── src/lib.rs
│
├── src/                          # 二进制入口
│   ├── main.rs
│   └── cli.rs
│
├── config/                       # 默认配置模板
│   └── config.toml.example
│
└── tests/                        # 集成测试目录
    └── integration_test.rs
```

## 实现阶段规划

### 阶段 1：基础设施搭建 (Week 1-2) ✅

**目标：** 建立项目结构，实现核心抽象

#### 1.1 项目初始化 ✅
- [x] 创建 Cargo 工作空间配置
- [x] 配置各 crate 依赖关系
- [ ] 设置 CI/CD 基础配置
- [x] 创建 README 和贡献指南

#### 1.2 核心基础设施 ✅
- [x] 实现 `Module` trait
  ```rust
  pub trait Module {
      fn name(&self) -> &str;
      fn update(&mut self, event: Event) -> Action;
      fn draw(&mut self, frame: &mut Frame, area: Rect);
      fn save(&self) -> Result<()>;
      fn load(&mut self) -> Result<()>;
      fn shortcuts(&self) -> Vec<Shortcut>;
      fn get_status(&self) -> String;
  }
  ```

- [x] 实现事件系统
  - 扩展 crossterm event
  - 自定义事件类型（模块间通信）
  - 事件分发机制

- [x] 实现 App 主循环
  - 事件监听循环
  - 模块管理器
  - 全局渲染

### 阶段 2：数据与配置 (Week 3) ✅

**目标：** 实现存储和配置系统

#### 2.1 日志模块 ✅
- [x] 实现日志 facade（基于 log crate）
- [x] 文件日志输出（带滚动）
- [x] 终端彩色日志
- [x] 配置集成

#### 2.2 数据存储层 ✅
- [x] sled 数据库初始化
- [x] 命名空间封装
- [x] 键值操作 API
- [x] 集合操作 API
- [x] 自动持久化

#### 2.3 配置管理 ✅
- [x] TOML 配置加载
- [ ] 配置热重载
- [x] XDG 目录支持
- [x] 默认配置生成

### 阶段 3：TUI 基础 (Week 4) ✅

**目标：** 实现用户界面基础

- [x] ratatui 集成
- [x] 布局系统
- [x] 主题系统（颜色、样式）
- [x] 状态栏（显示当前模块、时间等）
- [x] 标签栏（模块切换）
- [x] 快捷键绑定系统

### 阶段 4：核心模块实现 (Week 5-8)

**优先级顺序：**

#### Phase 4.1: 文件管理 (Week 5) 🟡
**FileBrowser 模块**
- [x] 文件树遍历（ ignore crate）
- [x] 文件导航（上下键）
- [x] 文件预览（syntect 语法高亮）
- [x] 支持打开外部应用（opener）
- [x] 文件搜索（Ctrl+F）
- [x] 内容搜索（Ctrl+F）
- [x] 状态栏集成（显示文件名和行号）
- [ ] 编辑功能（计划中）
- [ ] 行号滚动优化（进行中）

#### Phase 4.2: 待办事项 (Week 6)
**Todo 模块**
- [ ] CRUD 操作
- [ ] 标签系统
- [ ] 优先级（高/中/低）
- [ ] 完成状态切换
- [ ] 持久化到 sled

#### Phase 4.3: 笔记 (Week 6-7)
**Note 模块**
- [ ] Markdown 编辑器
- [ ] 笔记列表
- [ ] 标签系统
- [ ] 搜索功能
- [ ] 文件存储 (~/.local/share/tui-workstation/notes)

#### Phase 4.4: 日记 (Week 7)
**Diary 模块**
- [ ] 日历视图
- [ ] 按日期编辑笔记
- [ ] 农历显示（lunar-rs）
- [ ] 节假日标记（chinese-calendar）
- [ ] 文件存储 (~/.local/share/tui-workstation/diary)

#### Phase 4.5: 终端 (Week 8)
**Terminal 模块**
- [ ] portable-pty 集成
- [ ] 单标签终端
- [ ] 基本 shell 支持
- [ ] 异步 I/O 处理

### 阶段 5：进阶模块 (Week 9-12)

#### Phase 5.1: 项目跟踪 (Week 9)
**Project 模块**
- [ ] 项目 CRUD
- [ ] 里程碑管理
- [ ] 风险跟踪
- [ ] 进度条可视化

#### Phase 5.2: Git 操作 (Week 10)
**Git 模块**
- [ ] git2 静态链接配置
- [ ] 仓库状态显示
- [ ] 提交信息编辑
- [ ] 日志查看
- [ ] 分支切换

#### Phase 5.3: 音乐播放 (Week 11)
**Music 模块**
- [ ] symphonia 音频解码
- [ ] rodio 音频输出
- [ ] 播放列表
- [ ] 元数据显示（lofty）
- [ ] 播放控制（播放/暂停/下一首/上一首）

#### Phase 5.4: 邮件 (Week 12)
**Mail 模块**
- [ ] IMAP 客户端（imap crate）
- [ ] SMTP 客户端（lettre）
- [ ] 邮件解析（mailparse）
- [ ] 密钥存储（keyring）
- [ ] 邮件列表和视图
- [ ] 发送邮件界面

### 阶段 6：优化与增强 (Week 13-14)

- [ ] 主题支持（多个颜色方案）
- [ ] 自定义快捷键配置
- [ ] 模块开关配置
- [ ] 性能优化
- [ ] 内存使用优化
- [ ] 错误处理改进
- [ ] 用户文档

### 阶段 7：发布准备 (Week 15)

- [ ] 跨平台测试（Linux, macOS, Windows）
- [ ] 打包脚本（GitHub Actions）
- [ ] 预编译二进制构建
- [ ] 发布说明
- [ ] 用户手册

## 模块依赖关系

```
core (基础库)
  ├── storage (依赖日志)
  │     └── logging (独立)
  ├── modules/ (依赖 core+storage)
  │     ├── filebrowser
  │     ├── todo
  │     ├── note
  │     ├── diary
  │     ├── project
  │     ├── terminal
  │     ├── git
  │     ├── music
  │     └── mail
  └── binary (依赖 core + 所有模块)
```

## 关键技术要点

### 1. 异步处理
- tokio 作为异步运行时
- 所有 I/O 操作使用异步
- 避免阻塞主事件循环

### 2. 跨平台
- portable-pty 处理跨平台终端
- dirs crate 获取平台特定目录
- opener 调用系统默认应用
- crossterm 处理跨平台终端交互

### 3. 静态链接
- git2 启用静态链接选项
- 减少运行时依赖
- 简化部署

### 4. 性能优化
- sled 写入批量处理
- UI 渲染节流（60 FPS）
- 文件预览缓存
- 渐进式加载大目录

### 5. 错误处理
- 统一的 `Result<T, AppError>`
- 用户友好的错误消息
- 错误日志记录
- 恢复机制（如数据库损坏）

## 配置文件结构

```toml
[general]
log_level = "info"           # trace, debug, info, warn, error
log_to_file = true
log_file = "~/.local/share/tui-workstation/logs/app.log"
theme = "default"

[shortcuts]
# 全局快捷键
global_quit = "q"
switch_tab_next = "Ctrl+Right"
switch_tab_prev = "Ctrl+Left"

[modules]
enabled = ["filebrowser", "todo", "note", "diary", "music", "project", "mail", "terminal", "git"]

[filebrowser]
show_hidden = false
sort_by = "name"            # name, size, modified

[todo]
default_priority = "medium"

[music]
default_volume = 0.8
playlist_dir = "~/music"

[mail]
# 邮件配置（敏感信息使用 keyring）

[terminal]
default_shell = "default"   # default, bash, zsh, powershell

[git]
default_editor = "editor"   # default, vim, nano
```

## 数据库 Schema (sled)

```
todo:
  - "id:{uuid}" -> TodoItem
  - "tags:{tag}" -> Set<UUID>
  - "list" -> List<UUID>

note:
  - "id:{uuid}" -> NoteItem
  - "tags:{tag}" -> Set<UUID>
  - "search:{term}" -> Set<UUID> (索引)

project:
  - "id:{uuid}" -> ProjectItem
  - "milestones:{project_id}" -> List<Milestone>
  - "risks:{project_id}" -> List<Risk>

mail:
  - "folders" -> List<Folder>
  - "headers:{folder}:{uid}" -> MailHeader
  - "bodies:{uid}" -> MailBody
```

## 风险与挑战

### 1. Terminal 模块复杂度
**风险：** PTY 实现复杂，可能存在平台差异
**缓解：** 首先实现单标签，逐步扩展；先在 Linux 测试完善再到其他平台

### 2. 音乐播放兼容性
**风险：** 不同音频格式解码问题
**缓解：** symphonia 已支持主流格式；提供格式转换提示

### 3. 邮件安全
**风险：** 密钥存储安全
**缓解：** 使用 keyring crate；支持 OAuth2；提供加密存储选项

### 4. Git 静态链接
**风险：** 静态链接可能增加二进制大小
**缓解：** 仅在发布时启用；开发时使用系统链接

### 5. 性能（大目录、大文件）
**风险：** 文件浏览和预览可能卡顿
**缓解：** 懒加载、虚拟滚动、后台预处理

## 建议开发流程

1. **每周迭代目标明确**
2. **先跑通最小可用版本（MVP）**
3. **模块并行开发（不同开发者）**
4. **持续集成测试**
5. **早期用户反馈（Alpha 版本）**

## MVP 定义 (最小可用产品)

首次发布应包含：
- ✅ 核心基础设施（事件、渲染、模块系统）
- ✅ 日志模块
- ✅ 数据存储
- ✅ 配置管理
- ✅ FileBrowser 基础功能
- ✅ Todo 完整功能
- ✅ Note 基础功能
- ✅ Diary 日历视图
- ✅ Terminal 单标签

**暂不包含：** Music, Mail, Project, Git 模块（后续版本添加）

## 发布计划

- **v0.1.0:** MVP (5 个核心模块)
- **v0.2.0:** + Git, Terminal 多标签
- **v0.3.0:** + Music 模块
- **v0.4.0:** + Project 模块
- **v0.5.0:** + Mail 模块
- **v1.0.0:** 完整功能，稳定版

---

**文档版本：** 1.1
**更新日期：** 2026-03-05
**维护者：** Sisyphus AI

## 项目宣传建议

提高项目曝光度的策略：

1. **GitHub 优化**
   - 完善项目 badge（许可证、构建状态等）
   - 添加 GIPHY 动图或截图展示功能
   - 优化 README 结构，突出亮点

2. **社区推广**
   - Reddit: r/rust, r/linux, r/commandline
   - Hacker News: Show HN 专题
   - V2EX, 掘金等中文社区
   - Rust Discord 服务器
   - Terminal 工作室、Terminal lovers 等社区

3. **内容营销**
   - 撰写技术博客分享开发过程
   - 制作演示视频（10-15 分钟）
   - 在 YouTube/B 站上传使用教程
   - 写对比文章与其他 TUI 工具对比

4. **技术分享**
   - Rust Meetup 演讲
   - 开源项目推荐平台
   - Awesome Rust 列表提交
   - Terminal apps 的 awesome 列表提交

5. **持续运营**
   - 发版公告（每完成一个重要功能）
   - 定期更新开发日志
   - 回应 Issue 和 PR
   - 维护更新文档

6. **建立品牌**
   - 设计项目 logo
   - 统一的视觉风格
   - 清晰的定位声明
   - 一致的宣传语
