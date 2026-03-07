#[cfg(test)]
mod tests {
    use crate::PlayQueue;
    use crate::ProgressTracker;
    use crate::{Credentials, MusicError, PlaybackMode, PlaybackState, SourceType, Track};
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn test_track_local() {
        let path = PathBuf::from("/test/song.mp3");
        let track = Track::local(path.clone(), "Song".to_string(), "Artist".to_string());

        assert_eq!(track.title, "Song");
        assert_eq!(track.artist, "Artist");
        assert_eq!(track.path, path);
        assert_eq!(track.source_type, SourceType::Local);
        assert!(!track.id.is_empty());
    }

    #[test]
    fn test_source_type_name() {
        assert_eq!(SourceType::Local.name(), "Local");
        assert_eq!(SourceType::QqMusic.name(), "QQ Music");
        assert_eq!(SourceType::NetEaseMusic.name(), "NetEase Music");
        assert_eq!(SourceType::Nas { mount_point: None }.name(), "NAS");
    }

    #[test]
    fn test_playback_state_is_playing() {
        assert!(PlaybackState::Playing.is_playing());
        assert!(!PlaybackState::Paused.is_playing());
        assert!(!PlaybackState::Stopped.is_playing());
    }

    #[test]
    fn test_playback_state_is_stopped() {
        assert!(PlaybackState::Stopped.is_stopped());
        assert!(!PlaybackState::Playing.is_stopped());
    }

    #[test]
    fn test_playback_mode_is_repeat() {
        assert!(PlaybackMode::RepeatOne.is_repeat());
        assert!(PlaybackMode::RepeatAll.is_repeat());
        assert!(!PlaybackMode::Sequential.is_repeat());
        assert!(!PlaybackMode::Random.is_repeat());
    }

    #[test]
    fn test_playback_mode_name() {
        assert_eq!(PlaybackMode::Sequential.name(), "Sequential");
        assert_eq!(PlaybackMode::Random.name(), "Random");
        assert_eq!(PlaybackMode::RepeatOne.name(), "Repeat One");
        assert_eq!(PlaybackMode::RepeatAll.name(), "Repeat All");
    }

    #[test]
    fn test_play_queue_add_track() {
        let queue = PlayQueue::new();
        assert!(queue.is_empty());

        let track = Track::local(
            PathBuf::from("/test.mp3"),
            "Test".to_string(),
            "Artist".to_string(),
        );
        queue.add_track(track);

        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());
    }

    #[test]
    fn test_play_queue_mode() {
        let queue = PlayQueue::new();
        assert_eq!(queue.get_mode(), PlaybackMode::Sequential);

        queue.set_mode(PlaybackMode::Random);
        assert_eq!(queue.get_mode(), PlaybackMode::Random);
    }

    #[test]
    fn test_play_queue_navigation() {
        let queue = PlayQueue::new();

        for i in 0..5 {
            let track = Track::local(
                PathBuf::from(format!("/test{}.mp3", i)),
                format!("Song {}", i),
                format!("Artist {}", i),
            );
            queue.add_track(track);
        }

        assert_eq!(queue.current_index(), None);

        queue.set_current_index(Some(2));
        assert_eq!(queue.current_index(), Some(2));

        let next = queue.next_track_index();
        assert_eq!(next, Some(3));

        let prev = queue.prev_track_index();
        assert_eq!(prev, Some(1));

        queue.set_mode(PlaybackMode::RepeatAll);
        queue.set_current_index(Some(4));
        let wrap = queue.next_track_index();
        assert_eq!(wrap, Some(0));
    }

    #[test]
    fn test_play_queue_remove_track() {
        let queue = PlayQueue::new();

        for i in 0..5 {
            let track = Track::local(
                PathBuf::from(format!("/test{}.mp3", i)),
                format!("Song {}", i),
                format!("Artist {}", i),
            );
            queue.add_track(track);
        }

        queue.set_current_index(Some(2));
        queue.remove_track(2).unwrap();

        assert_eq!(queue.len(), 4);
        assert_eq!(queue.current_index(), None);
    }

    #[test]
    fn test_play_queue_clear() {
        let queue = PlayQueue::new();

        let track = Track::local(
            PathBuf::from("/test.mp3"),
            "Test".to_string(),
            "Artist".to_string(),
        );
        queue.add_track(track);

        queue.clear();

        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_progress_tracker() {
        let tracker = ProgressTracker::new();

        assert!(!tracker.is_playing());

        tracker.start(Some(Duration::from_secs(180)));
        assert!(tracker.is_playing());

        tracker.pause();
        assert!(!tracker.is_playing());

        let position = tracker.get_position();
        assert!(position < Duration::from_secs(1));

        tracker.resume();
        assert!(tracker.is_playing());

        tracker.stop();
        assert!(!tracker.is_playing());
        assert_eq!(tracker.get_position(), Duration::default());
    }

    #[test]
    fn test_music_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let music_err: MusicError = io_err.into();

        assert!(matches!(music_err, MusicError::Io(_)));
    }

    #[test]
    fn test_credentials_default() {
        let creds = Credentials::default();

        assert!(creds.username.is_none());
        assert!(creds.password.is_none());
        assert!(creds.token.is_none());
    }
}
