# Music Source Implementation Roadmap

## Current Status (Updated: 2026-03-07)

### ✅ Completed
- **Local Source**: Fully implemented with rodio audio playback
- **UI Architecture**: Complete support for displaying all four sources (Local, QQ Music, NetEase Music, NAS)
- **Source Switching**: Working keyboard controls (s key to cycle sources)
- **NAS Research**: Complete - identified smb-rs (SMB2/3) and reqwest_dav (WebDAV) libraries
- **Architecture**: All stub implementations ready with MusicSource trait conformance
- **Build & Deploy**: Binary compiled (8.7MB) and pushed to GitHub (commit ae059c5)
- **Documentation**: Comprehensive implementation roadmap created

### ⏳ In Progress
- **QQ Music API Research**: Background research tasks running (session ses_337be5d22ffeEa73j2uqiWmd56)
- **NetEase Music API Research**: Background research tasks running (session ses_337bded3fffeKEvcjDSzB6Y21F)

### ⏳ Pending (Blocked on API Research)
- **QQ Music Integration**: Awaiting endpoint URLs, authentication method, API format
- **NetEase Music Integration**: Awaiting reverse-engineered API details (known to use encrypted parameters)
- **NAS SMB Integration**: Library identified, implementation straightforward once prioritized
- **NAS WebDAV Integration**: Library identified, implementation straightforward once prioritized
- **Testing**: Integration testing requires API implementations

- ✅ Local Source: Fully implemented with rodio audio playback
- ⏳ QQ Music: Stub implemented, awaiting API research
- ⏳ NetEase Music: Stub implemented, awaiting API research
- ⏳ NAS: Stub with fallback to local files, WebDAV/SMB to be integrated

## Remaining Implementation Tasks

### 1. QQ Music Integration (High Priority)

#### Expected API Information Needed
- Official API endpoints or reverse-engineered endpoints
- Authentication method (OAuth, token, cookies)
- Search API endpoint and format
- Playback/streaming API endpoint
- Audio URL retrieval mechanism
- Rust libraries/SDKs if available

#### Implementation Steps
```rust
// Add to crates/music_model/Cargo.toml
[dependencies]
reqwest = { version = "0.12", features = ["json"] }
```

```rust
// In crates/music_model/src/source.rs QqMusicSource
pub struct QqMusicSource {
    client: reqwest::Client,
    access_token: Option<String>,
    base_url: String,
    // ... existing fields
}

impl MusicSource for QqMusicSource {
    fn load(&mut self, track: &Track) -> Result<()> {
        // 1. Get audio URL from QQ Music API
        let audio_url = self.get_stream_url(track).await?;
        // 2. Fetch audio stream
        // 3. Load into rodio sink
    }

    fn search(&self, query: &str) -> Result<Vec<Track>> {
        // Call QQ Music search API
        let response = self.client
            .get(&format!("{}/search", self.base_url))
            .query(&[("q", query)])
            .send()
            .await?;

        // Parse response and return tracks
    }
}
```

### 2. NetEase Music Integration (High Priority)

#### Expected API Information Needed
- No official API exists - need reverse-engineered endpoints
- Authentication mechanism (phone/password → token)
- Search API endpoint and format
- Song detail endpoint (for getting stream URLs)
- Playback/streaming API endpoint
- Rust libraries or reference implementations

#### Implementation Steps
Similar to QQ Music but with NetEase-specific:
- Login endpoint for authentication
- Search format (often requires encrypted parameters)
- Song URL endpoint (with signature/key in URL)

```rust
// Add to crates/music_model/Cargo.toml (same as QQ)
reqwest = { version = "0.12", features = ["json", "cookies"] }
```

#### Known Challenges
- NetEase often uses encrypted parameters (AES, RSA)
- May need to replicate signature generation
- API endpoints change frequently
- May need reverse-engineering from existing clients

### 3. NAS SMB/WebDAV Integration (Medium Priority)

#### Status
- ✅ Optional dependencies added (`nas-smb`, `nas-webdav` features)
- ✅ Documentation added to NasSource
- ⏳ Actual SMB client integration
- ⏳ Actual WebDAV client integration

#### SMB Integration (feature: nas-smb)

Dependencies already added:
```toml
smb = { version = "0.4", optional = true }
```

Implementation:
```rust
#[cfg(feature = "nas-smb")]
impl NasSource {
    async fn connect_smb(&mut self) -> Result<()> {
        use smb::{Client, ClientConfig, UncPath};
        use std::str::FromStr;

        let config = self.nas_config.as_ref().ok_or_else(|| {
            MusicError::Config("NAS configuration not set".to_string())
        })?;

        let client = Client::new(ClientConfig::default());
        let unc_path = UncPath::from_str(&format!(
            "\\\\{}\\{}",
            config.address,
            config.share_path
        ))?;

        let username = config.username.as_deref().unwrap_or("");
        let password = config.password.as_deref().unwrap_or("");

        client.share_connect(&unc_path, username, password.to_string()).await?;

        // Store SMB client for later file operations
        Ok(())
    }

    async fn read_file_smb(&self, path: &Path) -> Result<Vec<u8>> {
        // Use SMB client to read file
        // Convert file data for rodio playback
    }
}
```

#### WebDAV Integration (feature: nas-webdav)

Dependencies already added:
```toml
reqwest = { version = "0.12", optional = true }
```

Implementation:
```rust
#[cfg(feature = "nas-webdav")]
impl NasSource {
    async fn connect_webdav(&mut self) -> Result<()> {
        use reqwest_dav::{ClientBuilder, Auth};

        let config = self.nas_config.as_ref().ok_or_else(|| {
            MusicError::Config("NAS configuration not set".to_string())
        })?;

        let username = config.username.as_deref().unwrap_or("");
        let password = config.password.as_deref().unwrap_or("");

        let client = ClientBuilder::new()
            .set_host(format!("http://{}", config.address))
            .set_auth(Auth::Basic(username.to_owned(), password.to_owned()))
            .build()?;

        // Store WebDAV client for later operations
        Ok(())
    }

    async fn read_file_webdav(&self, path: &Path) -> Result<Vec<u8>> {
        // Use WebDAV GET endpoint to read file
        // Convert file data for rodio playback
    }
}
```

## Testing Plan

### Unit Tests
- Test QqMusicSource stub with mock responses
- Test NetEaseMusicSource stub with mock responses
- Test NasSource with local file fallback

### Integration Tests
- Test actual QQ Music API with real credentials (if available)
- Test actual NetEase Music API with real credentials
- Test SMB connection to test server (docker)
- Test WebDAV connection to test server

### Manual Tests
- Verify playback from each source
- Search functionality
- Error handling (network failures, authentication)
- State transitions (loading → playing → paused)

## Dependencies Summary

### Already in workspace
- tokio (with full features needed)
- rodio (audio playback)
- serde/serde_json (JSON parsing)
- anyhow (error handling)

### Need to add to music_model
```toml
[dependencies]
reqwest = { version = "0.12", features = ["json", "cookies"], optional = true }
smb = { version = "0.4", optional = true }

[features]
default = ["local"]
local = []
qq = ["reqwest"]
netease = ["reqwest"]
nas-smb = ["smb"]
nas-webdav = ["reqwest"]
```

## Build Commands

Enable specific sources:
```bash
# Local only
cargo build --release

# With QQ Music
cargo build --release --features qq

# With NetEase Music
cargo build --release --features netease

# With NAS SMB
cargo build --release --features nas-smb

# All features
cargo build --release --all-features
```

## Next Steps (Continuation Session)

### Immediate Priorities (Next Session)

1. **Collect API Research Results** (High Priority)
   - Check session ses_337be5d22ffeEa73j2uqiWmd56 for QQ Music API details
   - Check session ses_337bded3fffeKEvcjDSzB6Y21F for NetEase Music API details
   - If research incomplete, use librarian agent to find GitHub implementations

2. **Implement QQ Music Integration** (High Priority - Blocks completion)
   - Add reqwest dependency to music_model/Cargo.toml
   - Implement authentication flow (awaiting research for method)
   - Implement search API client
   - Implement playback/streaming URL retrieval
   - Load audio streams into rodio sink
   - Error handling for network failures

3. **Implement NetEase Music Integration** (High Priority - Blocks completion)
   - Add reqwest dependency (may already be added for QQ)
   - Implement phone/password → token authentication flow
   - Implement search API with encrypted parameters
   - Implement song detail API for stream URLs
   - Handle AES/RSA encryption signatures if required
   - Load audio streams into rodio sink

4. **NAS Implementation** (Medium Priority - Can be done later)
   - Enable `nas-smb` feature and integrate smb-rs crate
   - Enable `nas-webdav` feature and implement WebDAV client
   - Implement file listing for music discovery
   - Stream files over network to rodio

### Testing
- Unit tests with mock API responses
- Integration tests with actual credentials (if available)
- Manual testing: search, play, pause, seek, queue management

### Final Steps
- Verify all four sources work
- Update documentation with API endpoints
- Commit and push final implementation

## Session Status (2026-03-07)

### What Was Accomplished This Session
1. **Architecture**: Complete multi-source music player framework
2. **Implementations**: Stub implementations for QQ Music, NetEase Music, NAS
3. **UI**: Full display and switching support for all sources
4. **Dependencies**: Optional feature flags for NAS protocols added
5. **Build**: Successful compilation (8.7MB binary)
6. **Deploy**: Pushed to GitHub (commit ae059c5)
7. **Documentation**: Comprehensive roadmap created

### What Remains for Full Implementation
1. API endpoint details for QQ Music (research in progress)
2. API endpoint details for NetEase Music (research in progress)
3. Actual HTTP client integration
4. Authentication flows for streaming services
5. Network file streaming for NAS

### Research Status Note
API research tasks were launched but did not complete within reasonable time. Two options for continuation:
1. Check sessions ses_337be5d22ffeEa73j2uqiWmd56 and ses_337bded3fffeKEvcjDSzB6Y21F for results
2. If incomplete, use direct web search to find GitHub implementations of QQ/NetEase music APIs

### Known Implementation References (to search)
- Search GitHub for "netease music api rust"
- Search GitHub for "qq music api rust"
- Look for existing Node.js or Python implementations as references
- Check crate registry for music service clients

## References

### QQ Music
- Official API: TBD (research in progress)
- Reverse-engineered examples: TBD

### NetEase Music
- No official API
- Reverse-engineered examples: TBD (research in progress)
- Known challenge: encrypted parameters

### NAS Libraries
- SMB: https://github.com/afiffon/smb-rs (67 stars, pure Rust)
- WebDAV: https://github.com/niuhuan/reqwest_dav (42 stars, reqwest-based)
- WebDAV alt: https://github.com/cradiy/webdav-request (lightweight, reqwest-based)
