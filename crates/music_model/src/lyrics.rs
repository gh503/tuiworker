//! Lyrics parser and synchronization for music playback

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

/// Lyrics line with timestamp and text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricLine {
    /// Time when this lyric should be displayed (in seconds)
    pub time: f32,
    /// Lyric text content
    pub text: String,
    /// Translation (if available)
    pub translation: Option<String>,
    /// Pinyin (if available)
    pub pinyin: Option<String>,
}

impl LyricLine {
    pub fn new(time: f32, text: String) -> Self {
        Self {
            time,
            text,
            translation: None,
            pinyin: None,
        }
    }
}

/// Parsed lyrics with time-synchronized lines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lyrics {
    /// All synchronized lyric lines
    pub lines: Vec<LyricLine>,
    /// Metadata (if available from lyrics)
    pub metadata: HashMap<String, String>,
}

impl Lyrics {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Find the lyric line that should be displayed at current playback time
    pub fn find_current_line(&self, position: Duration) -> Option<usize> {
        let position_secs = position.as_secs_f32();

        for (i, line) in self.lines.iter().enumerate() {
            if line.time > position_secs {
                if i == 0 {
                    return None;
                }
                return Some(i - 1);
            }
        }

        if !self.lines.is_empty() {
            Some(self.lines.len() - 1)
        } else {
            None
        }
    }

    /// Get the lyric line at specific index
    pub fn get_line(&self, index: usize) -> Option<&LyricLine> {
        self.lines.get(index)
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

impl Default for Lyrics {
    fn default() -> Self {
        Self::new()
    }
}

/// LRC format parser
pub struct LrcParser;

impl LrcParser {
    /// Parse LRC format lyrics string
    ///
    /// LRC Format:
    /// [mm:ss.ms] Lyric text
    /// [mm:ss.ms] Translation text (if tagged with tlyric)
    ///
    /// Example (LRC format):
    /// ```text
    /// [00:00.00] First line
    /// [00:01.50] Second line
    /// ```
    pub fn parse(lrc_text: &str) -> Result<Lyrics, LrcParseError> {
        let mut lyrics = Lyrics::new();

        for line in lrc_text.lines() {
            if line.trim().is_empty() || !line.starts_with('[') {
                continue;
            }

            if let Some((time_str, text)) = Self::parse_line(line) {
                let time = Self::parse_time(&time_str)?;
                let lyric_line = LyricLine::new(time, text);
                lyrics.lines.push(lyric_line);
            }
        }

        if lyrics.lines.is_empty() {
            return Err(LrcParseError::EmptyLyrics);
        }

        lyrics
            .lines
            .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

        Ok(lyrics)
    }

    /// Parse a single LRC line
    fn parse_line(line: &str) -> Option<(String, String)> {
        let first_bracket = line.find('[')?;
        let last_bracket = line.rfind(']')?;

        let time_tags = line[first_bracket + 1..last_bracket].to_string();
        let text = line[last_bracket + 1..].trim().to_string();

        Some((time_tags, text))
    }

    /// Parse time from LRC format "[mm:ss.ms]"
    fn parse_time(time_str: &str) -> Result<f32, LrcParseError> {
        let clean_time = time_str.trim();

        let parts: Vec<&str> = clean_time.split(':').collect();
        if parts.len() != 2 {
            return Err(LrcParseError::InvalidTimeFormat(clean_time.to_string()));
        }

        let minutes: f32 = parts[0]
            .parse()
            .map_err(|_| LrcParseError::InvalidMinutes(parts[0].to_string()))?;

        let seconds_parts: Vec<&str> = parts[1].split('.').collect();
        let seconds: f32 = seconds_parts[0]
            .parse()
            .map_err(|_| LrcParseError::InvalidSeconds(seconds_parts[0].to_string()))?;

        let milliseconds: f32 = if seconds_parts.len() > 1 {
            let ms_str = format!("0.{}", seconds_parts[1]);
            ms_str.parse().unwrap_or(0.0)
        } else {
            0.0
        };

        let total_seconds = minutes * 60.0 + seconds + milliseconds;
        Ok(total_seconds)
    }
}

/// LRC parsing errors
#[derive(Debug)]
pub enum LrcParseError {
    EmptyLyrics,
    InvalidTimeFormat(String),
    InvalidMinutes(String),
    InvalidSeconds(String),
}

impl fmt::Display for LrcParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LrcParseError::EmptyLyrics => write!(f, "No lyrics found"),
            LrcParseError::InvalidTimeFormat(s) => write!(f, "Invalid time format: {}", s),
            LrcParseError::InvalidMinutes(s) => write!(f, "Invalid minutes: {}", s),
            LrcParseError::InvalidSeconds(s) => write!(f, "Invalid seconds: {}", s),
        }
    }
}

impl std::error::Error for LrcParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time() {
        assert_eq!(LrcParser::parse_time("00:00.00").unwrap(), 0.0);
        assert_eq!(LrcParser::parse_time("00:01.50").unwrap(), 1.5);
        assert_eq!(LrcParser::parse_time("01:30.00").unwrap(), 90.0);
        assert_eq!(LrcParser::parse_time("03:20.500").unwrap(), 200.5);
    }

    #[test]
    fn test_parse_lrc() {
        let lrc = "[00:00.00] First line\n[00:01.50] Second line\n[00:03.00] Third line";
        let lyrics = LrcParser::parse(lrc).unwrap();

        assert_eq!(lyrics.lines.len(), 3);
        assert_eq!(lyrics.lines[0].time, 0.0);
        assert_eq!(lyrics.lines[0].text, "First line");
        assert_eq!(lyrics.lines[1].time, 1.5);
        assert_eq!(lyrics.lines[1].text, "Second line");
        assert_eq!(lyrics.lines[2].time, 3.0);
        assert_eq!(lyrics.lines[2].text, "Third line");
    }

    #[test]
    fn test_find_current_line() {
        let lrc = "[00:00.00] Line 1\n[00:02.00] Line 2\n[00:04.00] Line 3";
        let lyrics = LrcParser::parse(lrc).unwrap();

        assert_eq!(
            lyrics.find_current_line(Duration::from_secs_f32(0.0)),
            Some(0)
        );
        assert_eq!(
            lyrics.find_current_line(Duration::from_secs_f32(1.0)),
            Some(0)
        );
        assert_eq!(
            lyrics.find_current_line(Duration::from_secs_f32(2.0)),
            Some(1)
        );
        assert_eq!(
            lyrics.find_current_line(Duration::from_secs_f32(3.0)),
            Some(1)
        );
        assert_eq!(
            lyrics.find_current_line(Duration::from_secs_f32(4.0)),
            Some(2)
        );
        assert_eq!(
            lyrics.find_current_line(Duration::from_secs_f32(5.0)),
            Some(2)
        );
    }
}
