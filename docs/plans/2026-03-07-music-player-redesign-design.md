# 音乐播放器重新设计文档

## 概述
重新设计tuiworker项目的音乐播放器模块，支持本地文件、QQ音乐、网易云音乐和NAS网络播放。设计原则是底层架构一致、UI区域独立、界面清爽美观、操作简单。

## 设计目标

1. **多源支持**：同时支持本地播放、QQ音乐、网易云音乐和NAS网络播放
2. **统一体验**：所有播放源的UI和操作体验一致
3. **分层架构**：UI层、业务逻辑层、数据源层分离
4. **可靠稳定**：错误处理优雅，播放状态准确
5. **性能优化**：缓存、预加载、异步不阻塞UI

## 架构设计

### 系统架构图

```
┌─────────────────────────────────────────┐
│   MusicModule (UI层)                      │
│   - 播放列表 (Playlist Widget)              │
│   - 播放器控件 (Player Controls)          │
│   - 播放控制 (Keyboard/Mouse)             │
│   - 播放源选择 (Source Selector)          │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│   MusicCore (业务逻辑层)                  │
│   - PlayerController (播放协调)            │
│   - PlayQueue (播放队列管理)               │
│   - ProgressTracker (进度跟踪)            │
│   - StateMachine (状态管理)                 │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│   MusicSource (抽象层)                   │
│   trait MusicSource (+ 方法)               │
│   ├─ LocalSource (本地文件)                │
│   ├─ QqMusicSource (QQ音乐)               │
│   ├─ NetEaseMusicSource (网易云音乐)      │
│   └─ NasSource (NAS网络)                   │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│   后端播放引擎                           │
│   - rodio (本地)                          │
│   - 官方SDK/API (在线)                      │
│   - SMB/WebDAV客户端 (NAS)                 │
└─────────────────────────────────────────┘
```

### 分层职责

**UI层**:
- 显示播放列表和当前播放曲目
- 响应用户输入（键盘/鼠标）
- 显示播放状态和进度
- 提供配置界面

**业务逻辑层**:
- 管理播放队列和播放顺序
- 协调不同音乐源的切换
- 跟踪播放进度
- 处理播放状态转换
- 错误恢复和重试机制

**数据源层**:
- 抽象的播放接口
- 实现具体播放源的加载、播放、控制
- 提供搜索功能
- 管理认证和会话

## 核心组件设计

### 数据结构

```rust
/// 音频轨道元数据
struct Track {
    id: String,
    path: PathBuf,                    // 本地文件路径或在线URL
    title: String,
    artist: String,
    album: String,
    duration: Option<Duration>,
    source_type: SourceType,        // 音乐源类型
    cover_url: Option<String>,        // 封面图URL
}

/// 音乐源类型
enum SourceType {
    Local,
    QqMusic,
    NetEaseMusic,
    Nas { mount_point: Option<PathBuf> }, // 本地挂载点优先
}

/// 播放状态
enum PlaybackState {
    Stopped,
    Loading,
    Playing,
    Paused,
    Buffering,
}

/// 播放模式
enum PlaybackMode {
    Sequential,   // 顺序播放
    Random,       // 随机播放
    RepeatOne,     // 单曲循环
    RepeatAll,     // 列表循环
}

/// 播放队列
struct PlayQueue {
    tracks: Vec<Track>,
    current_index: Option<usize>,
    mode: PlaybackMode,
    history: Vec<usize>,      // 播放历史
    position: usize,           // 当前选择位置（UI焦点）
}
```

### PlayerController 设计

```rust
struct PlayerController {
    current_source: Option<Box<dyn MusicSource>>,
    state: Arc<Mutex<PlaybackState>>,
    play_queue: Arc<Mutex<PlayQueue>>,
    volume: f32,
    position: Arc<Mutex<Duration>>,
    duration: Arc<Mutex<Option<Duration>>>,
    volume: f32,
    listeners: Vec<Box<dyn MusicEventListener>>, // 观察者列表
}

impl PlayerController {
    // 播放控制方法
    pub fn play(&mut self, track: Track) -> anyhow::Result<()>;
    pub fn pause(&mut self) -> anyhow::Result<()>;
    pub fn resume(&mut self) -> anyhow::Result<()>;
    pub fn stop(&mut self) -> anyhow::Result<()>;
    pub fn seek(&mut self, position: Duration) -> anyhow::Result<()>;
    
    // 音量控制
    pub fn set_volume(&mut self, volume: f32);
    pub fn get_volume(&self) -> f32;
    
    // 播放模式切换
    pub fn cycle_playback_mode(&mut self);
    
    // 队列管理
    pub fn add_track(&mut self, track: Track);
    pub fn add_tracks(&mut self, tracks: Vec<Track>);
    pub fn remove_track(&mut self, index: usize) -> anyhow::Result<()>;
    pub fn clear_queue(&mut self);
    
    // 导航控制
    pub fn next_track(&mut self);
    pub fn prev_track(&mut self);
    pub fn goto_track(&mut self, index: usize);
    
    // 源管理
    pub fn set_source(&mut self, source: SourceType) -> anyhow::Result<()>;
    pub fn get_current_source(&self) ->SourceType;
    
    // 观察者模式
    pub fn add_listener(&mut self, listener: Box<dyn MusicEventListener>);
}
```

### MusicSource 抽象接口

```rust
/// 音乐源抽象trait
#[async_trait]
pub trait MusicSource: Send + Sync {
    /// 加载曲目
    async fn load(&mut self, track: &Track) -> anyhow::()>;
    
    /// 播放曲目
    async fn play(&mut self) -> anyhow::()>;
    
    /// 暂停播放
    fn pause(&mut self) -> anyhow::()>;
    
    /// 恢复播放
    fn resume(&mut self) -> anyhow::()>;
    
    /// 停止播放
    fn stop(&mut self) -> anyhow::()>;
    
    /// 跳转到指定位置
    async fn seek(&mut self, position: Duration) -> anyhow::()>;
    
    /// 获取当前播放位置
    async fn get_position(&self) -> Duration;
    
    /// 获取总时长
    async fn get_duration(&self) -> Option<Duration>;
    
    /// 获取播放状态
    async fn get_state(&self) -> PlaybackState;
    
    /// 获取封面图
    async fn get_cover_art(&self, track: &Track) -> Option<Vec<u8>>;
    
    /// 搜索曲目
    async fn search(&self, query: &str) -> anyhow::Result<Vec<Track>>;
    
    /// 获取源类型
    fn get_source_type(&self) -> SourceType;
    
    /// 验证
    async fn authenticate(&mut self, credentials: Option<&Credentials>) -> anyhow::Result<()>;
    
    /// 清理资源
    async fn cleanup(&mut self);
    
    /// 是否支持流媒体
    fn supports_streaming(&self) -> bool;
}
```

## UI设计

### 界面布局

```
┌──────────────────────────────────────────────────────────┐
│ 音乐播放器                             [配置] [退出]        │
├──────────────────────────────────────────────────────────┤
│ ┌───────────────────────────────────────────────────────┐  │
│ │ 播放器控制                                            │  │
│ │  正在播放: 周杰伦 - 稻香        [Loading...]       │  │
│ │   播放中  |  音量: 75%  |  循环: 全部  | 随机: 关 │  │  │
│ │    -:--:--:--:--: 25%                         │  │
│ │                                                 │  │
│ │  [<<]  [▶]  [||]  [▶▶]  [>>]  静音              │  │
│ └───────────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────┐  │
│ │ 播放队列                      搜索 [ ]  播放源 [本地▼] │  │
│ │                                                 │  │
│ │ 本地: ~/Music                      [45首]            │  │
│ │ ┌───────────────────────────────────────────────┐  │
│ │ │► 1. 周杰伦 - 稻香          3:45            │  │
│ │ │  2. 陈奕迅 - 单车          4:12            │  │
│ │ │  3. 林俊杰 - 江南                         │  │
│ │ │  4. 孙燕姿 - 遇光          3:28            │  │
│ │ │  5. 蔡依林 - 没完        [搜索...]  │  │
│ │ │  ...                                     │  │
│ │ └───────────────────────────────────────────────┘  │  │
│ │                                                 │  │
│ └─────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
│ [Help] Space:播放 < >:音量 r:循环 s:随机 n/p:上/下 q:退出      │
└──────────────────────────────────────────────────────────┘
```

### UI组件说明

**播放器控制区**:
- 当前播放信息：显示歌手-歌曲名，loading状态时显示
- 状态指示：播放中/暂停/停止/加载中/缓冲中
- 进度条：可视化显示播放进度，时间显示（当前/总时长）
- 播放按钮：上一首、播放/暂停、停止、下一首
- 音量调节：可滚动调节，显示当前音量百分比
- 播放模式：循环模式、随机模式
- 快进/快退：支持键盘快捷键（可选）

**播放源选择器**:
- 下拉菜单：本地、QQ音乐、网易云音乐、NAS
- 配置按钮：打开各源的配置界面
- 默认显示配置的第一个可用源

**搜索功能**:
- 搜索框：输入关键词后按Enter搜索
- 搜索结果：显示符合条件的歌曲列表
- 点击可添加到当前播放队列
- 不同源使用对应的搜索API

**播放列表**:
- 显示当前队列的所有歌曲
- 支持上下导航（j/k或↑/↓）
- 当前播放高亮显示（►符号）
- 选中项背景高亮
- 右键菜单：删除、收藏、查看详情

## 配置管理

### 配置文件结构

```toml
# 音乐配置
[music]
default_source = "local"  # 默认播放源

# 本地音乐配置
[music.local]
music_directory = "~/Music"
supported_formats = ["mp3", "flac", "ogg", "wav", "m4a"]
auto_refresh = true

# 播放器配置
[music.playback]
volume = 0.75
playback_mode = "sequential"
auto_play_next = true
crossfade_duration = 2000  # 毫秒

# 循环配置
[music.repeat]
mode = "all"  # none, one, all
remember_position = true

# QQ音乐配置
[music.qq_music]
enabled = false
# credentials配置（需要加密存储）

# 网易云音乐配置
[music.netease_music]
enabled = false
# credentials配置（需要加密存储）

# NAS配置
[music.nas]
enabled = false
local_mount_point = "/mnt/nas/music"
smb_url = "smb://192.168.1.100/Music"
webdav_url = "https://192.168.1.100/music/"
username = ""
password = ""
```

### 配置持久化

使用TOML格式存储配置：
- 路径：`~/.config/tuiworker/music.toml`
- 敏感信息（密码、token）需要加密存储
- 配置变更时自动保存

## API设计

### 事件系统

```rust
/// 音乐事件类型
pub enum MusicEvent {
    TrackChanged(Track),
    StateChanged(PlaybackState, Option<PlaybackState>),
    ProgressUpdated { position: Duration, duration: Option<Duration> },
    Error(MusicError),
    SourceChanged(SourceType, SourceType),
    QueueChanged(Vec<Track>),
    VolumeChanged(f32, f32),
    ModeChanged(PlaybackMode, PlaybackMode),
}

/// 事件监听器trait
pub trait MusicEventListener: Send + Sync {
    fn on_event(&self, event: MusicEvent);
}

/// 事件分发器
impl MusicCore {
    listeners: Vec<Box<dyn MusicEventListener>>,
    pub fn add_listener(&mut self, listener: Box<dyn MusicEventListener>);
    pub fn remove_listener(&mut self, index: usize);
    fn dispatch_event(&self, event: MusicEvent);
}
```

### 配置API

```rust
impl MusicCore {
    // 加载配置
    pub fn load_config(&mut self) -> anyhow::Result<()>;
    
    // 保存配置
    pub fn save_config(&self) -> anyhow::Result<()>;
    
    // 更新配置
    pub fn update_config(&mut self, config: MusicConfig) -> anyhow::<()>;
    
    // 获取当前配置
    pub fn get_config(&self) -> &MusicConfig;
}
```

## 音频格式支持

### 本地文件支持
- **格式**：MP3, FLAC, OGG, WAV, M4A
- **容器格式**：支持MP4, MKA等包含音频的文件
- **解码**: 使用symphonia解码多种格式

### 在线流媒体支持
- **QQ音乐**: 使用官方SDK获取播放URL（需要认证）
- **网易云音乐**: 使用官方API获取播放URL（需要认证）
- **流媒体格式**: HTTP/HTTPS流播放

### NAS协议支持
- **SMB/CIFS**: 使用rust-smb客户端访问Windows共享
- **WebDAV**: 使用reqwest + WebDAV规范
- **本地挂载**: 优先使用已挂载的本地路径

## 技术栈

### 依赖库
- `rodio`: 音频播放核心（本地文件）
- `symphonia`: 音频解码器框架
- `rust-smb`: SMB/CIFS协议支持
- `reqwest`: HTTP客户端（WebDAV、在线API）
- `tokio`: 异步运行时
- `serde`: 序列化配置
- `toml`: 配置文件解析

### 性能优化
- **音频预加载**: 提前加载下一首文件
- **缓存策略**: API搜索结果缓存
- **异步加载**: 不阻塞UI主线程
- **进度更新节流**: 限制更新频率避免性能问题
- **内存管理**: 大文件的流式处理

## 错误处理与恢复

### 错误类型
```rust
pub enum MusicError {
    SourceNotAvailable(String),
    AuthenticationFailed(String),
    NetworkError(String),
    FileNotFound(PathBuf),
    PermissionDenied(PathBuf),
    UnsupportedFormat(String),
    PlaybackFailed(String),
    APIError(String),
    ConfigurationError(String),
    Unknown(String),
}

// From实现
impl From<std::io::Error> for MusicError
impl From<rodio::PlayError> for MusicError
impl From<serde_json::Error> for MusicError
```

### 错误恢复机制
- **重试逻辑**: 网络请求失败自动重试3次
- **降级机制**: 主流派失败时尝试备用源
- **状态恢复**: 保存播放位置，重启后可恢复
- **优雅降级**: 某个功能失败不影响其他功能

## 测试策略

### 单元测试
- 各音乐源的加载、播放、控制功能
- 播放队列管理逻辑
- 状态机转换逻辑
- 配置解析和持久化

### 集成测试
- 多源切换流程
- 播放队列操作
- 错误恢复场景
- 端到端播放流程

### 需要mock的部分
- 在线API调用（QQ音乐、网易云）
- NAS网络连接（SMB、WebDAV）

## 实施计划

此设计将转换为详细的实施计划，包括：
1. 目录结构调整
2. 核心trait和结构体实现
3. 各音乐源的具体实现
4. UI组件更新
5. 集成测试
6. 文档更新

## 参考文档
- TUI应用最佳实践
- rodio和symphonia文档
- QQ音乐/网易云音乐API文档
- SMB/WebDAV协议规范
