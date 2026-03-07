# Music Source Implementation Roadmap

## Current Status
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

## Next Steps

1. ✅ Wait for API research results (QQ Music and NetEase Music)
2. Implement QQ Music integration (with reqwest)
3. Implement NetEase Music integration (with reqwest)
4. Implement SMB integration (with smb crate)
5. Implement WebDAV integration (with reqwest_dav)
6. Write tests for each source
7. Manual testing with real services
8. Commit changes
9. Push to repository

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
