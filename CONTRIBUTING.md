# 贡献指南

感谢你对 TUI Workstation 项目的兴趣！我们欢迎所有形式的贡献。

## 行为准则

本项目遵循贡献者公约。请联系 maintainer 获得详细版本。

## 如何贡献

### 报告 Bug

在提交 Bug 报告之前，请：

1. 搜索现有的 Issues，确保 Bug 未被报告
2. 使用 Bug 报告模板（如果可用）
3. 提供复现步骤、预期行为和实际行为
4. 包含环境信息（操作系统、终端类型、Rust 版本）

### 提交 Feature Request

1. 搜索现有的 Issues，确保请求未被提出
2. 描述 Feature 的用例和预期行为
3. 解释为什么这个 Feature 对项目有价值

### 代码贡献

#### 开发环境设置

```bash
# 克隆仓库
git clone https://github.com/gh503/tuiworker
cd tuiworker

# 安装开发工具
cargo install cargo-watch
cargo install cargo-edit
```

#### 代码风格

本项目使用以下工具：

- `rustfmt` - 代码格式化
- `clippy` - Linter

在提交代码前运行：

```bash
# 格式化代码
cargo fmt

# 运行 linter
cargo clippy -- -D warnings

# 运行测试
cargo test

# 运行工作空间所有测试
cargo test --workspace
```

#### 提交规范

提交信息格式：

```
<type>: <subject>

<body>

<footer>
```

**Type**:
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建过程或工具变更

**示例**:
```
feat(filebrowser): 添加按文件大小排序功能

实现了文件浏览器中按文件大小排序的功能，
支持升序和降序排列。

Closes #123
```

#### Pull Request 流程

1. Fork 项目
2. 创建功能分支（`git checkout -b feature/AmazingFeature`）
3. 提交更改（`git commit -m 'feat: Add some AmazingFeature'`）
4. 推送到分支（`git push origin feature/AmazingFeature`）
5. 开启 Pull Request

PR 描述应包含：
- 变更概述
- 相关 Issue 编号
- 测试方法
- 截图（如果适用）

#### 代码审查

Maintainer 会审查所有 PR。可能被要求进行修改。请积极响应反馈。

## 开发指南

### 项目结构

```
tuiworkstation/
├── crates/
│   ├── core/           # 核心库（Module trait, 事件系统）
│   ├── storage/        # 存储层（sled 封装）
│   ├── logging/        # 日志模块
│   └── modules/        # 功能模块
│       ├── filebrowser/
│       ├── todo/
│       └── ...
├── src/                # 二进制入口
└── docs/               # 文档
```

### 添加新模块

1. 在 `crates/modules/` 下创建新目录
2. 创建基础的 `Cargo.toml` 和 `src/lib.rs`
3. 实现 `Module` trait
4. 在工作空间 `Cargo.toml` 中注册
5. 在主应用中注册模块

### 测试

```bash
# 运行所有测试
cargo test

# 运行特定 crate 的测试
cargo test -p core

# 运行特定模块的测试
cargo test -p modules::todo

# 运行单个测试
cargo test test_name

# 显示测试输出
cargo test -- --nocapture
```

### 文档

```bash
# 生成文档
cargo doc --open

# 检查文档完整性
cargo doc --no-deps
```

## 分支策略

- `main` - 主分支，稳定代码
- `develop` - 开发分支
- `feature/*` - 功能分支
- `fix/*` - Bug 修复分支
- `hotfix/*` - 紧急修复分支

## 版本发布

版本号遵循 [语义化版本](https://semver.org/)。

### 发布流程

1. 更新 `CHANGELOG.md`
2. 更新 `Cargo.toml` 版本号
3. 创建 Git tag
4. 构建发布包
5. 发布到 GitHub Releases

## 获取帮助

- 查看 [API 文档](docs/api.md)
- 查看 [实现规划](docs/implementation-plan.md)
- 在 Issues 中提问
- 加入我们的 [Discord/Slack 社区](#) (待建立)

## 认可

贡献者会在 README 的 Contributors 部分被列出。

## 性能考虑

在编写代码时，请注意：

- 避免 UI 渲染阻塞主线程
- 使用异步 I/O 处理长时间操作
- 缓存重复计算结果
- 对于大文件/大目录，使用懒加载
- 监控内存使用

## 安全注意事项

- **密钥存储**: 邮件密码等敏感信息应使用 keyring
- **输入验证**: 所有用户输入需要验证
- **路径遍历**: 防止目录遍历攻击
- **SQL 注入**: sled 是键值存储，但仍需注意（如果未来改用其他 DB）

## 国际化 (i18n)

当前项目仅支持英语。未来计划支持多语言（中文、日文等）。

## 可访问性

我们致力于让所有人都能使用本应用，包括：
- 屏幕阅读器支持
- 键盘导航
- 高对比度主题

---

有任何问题？随时提问！
