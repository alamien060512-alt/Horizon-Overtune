// Overtune runtime: testable music player logic shared by the Cargo library
// and the injected `crate::game::overtune` module.
//
// Helper files (`frame.rs`, `hud.rs`, `key.rs`, `mods.rs`, `settings.rs`,
// `settings_click.rs`, `lifecycle.rs`) are compiled inside the game crate
// and call into the public functions below.

/// The mod's id (its settings folder), name and version.
pub const ID: &str = "overtune";
pub const NAME: &str = "Overtune";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "A clean in-game music player for Minecraft's soundtrack.";

/// Default volume (0..=100).
pub const DEFAULT_VOLUME: u8 = 80;

/// The HUD box colour (translucent dark).
pub const BACKGROUND: u32 = 0x7800_0000;
/// The HUD text colour (white).
pub const INK: u32 = 0xFF_FFFF;

use std::collections::HashMap;
use std::path::PathBuf;

/// A music event that can trigger a song.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventType {
    Rain,
    Danger,
    BossFight,
    FirstNether,
    FirstElytra,
    Credits,
}

impl EventType {
    /// All events, in display order.
    pub const ALL: [EventType; 6] = [
        EventType::Rain,
        EventType::Danger,
        EventType::BossFight,
        EventType::FirstNether,
        EventType::FirstElytra,
        EventType::Credits,
    ];

    /// The label shown on the settings page.
    pub fn label(self) -> &'static str {
        match self {
            EventType::Rain => "Rain",
            EventType::Danger => "Danger",
            EventType::BossFight => "Boss fight",
            EventType::FirstNether => "First Nether entry",
            EventType::FirstElytra => "First Elytra flight",
            EventType::Credits => "Credits",
        }
    }

    /// The settings key used to persist the assigned song id.
    pub fn settings_key(self) -> &'static str {
        match self {
            EventType::Rain => "event_rain",
            EventType::Danger => "event_danger",
            EventType::BossFight => "event_boss_fight",
            EventType::FirstNether => "event_first_nether",
            EventType::FirstElytra => "event_first_elytra",
            EventType::Credits => "event_credits",
        }
    }
}

/// A song in the library.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Song {
    /// Stable id (`minecraft:music.…` for vanilla, `import:…` for imported).
    pub id: String,
    /// Display title.
    pub title: String,
    /// Artist (C418, Lena Raine, …).
    pub artist: String,
    /// Album / soundtrack.
    pub album: String,
    /// Duration in seconds.
    pub duration: u32,
    /// Filesystem path for imported songs, empty for vanilla tracks.
    pub path: PathBuf,
    /// True for vanilla soundtrack tracks.
    pub is_vanilla: bool,
}

impl Song {
    pub fn vanilla(id: &str, title: &str, artist: &str, album: &str, duration: u32) -> Self {
        Self {
            id: id.to_owned(),
            title: title.to_owned(),
            artist: artist.to_owned(),
            album: album.to_owned(),
            duration,
            path: PathBuf::new(),
            is_vanilla: true,
        }
    }

    pub fn imported(id: &str, title: &str, artist: &str, path: PathBuf) -> Self {
        Self {
            id: id.to_owned(),
            title: title.to_owned(),
            artist: artist.to_owned(),
            album: "Imported".to_owned(),
            duration: 0,
            path,
            is_vanilla: false,
        }
    }
}

/// How the player queues songs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RepeatMode {
    #[default]
    None,
    One,
    All,
}

/// The music player's state.
#[derive(Clone, Debug)]
pub struct MusicPlayer {
    /// Whether Overtune is enabled.
    pub enabled: bool,
    /// Volume, 0..=100.
    pub volume: u8,
    /// The current song, if any.
    pub current: Option<Song>,
    /// Whether playback is paused.
    pub paused: bool,
    /// Playback position in seconds.
    pub position: u32,
    /// The queue of upcoming songs.
    pub queue: Vec<Song>,
    /// Repeat mode.
    pub repeat: RepeatMode,
    /// Whether shuffle is on.
    pub shuffle: bool,
    /// Songs the player has liked.
    pub liked: Vec<String>,
    /// Custom playlists.
    pub playlists: Vec<Playlist>,
    /// Event -> song id assignments.
    pub events: HashMap<EventType, String>,
    /// Folder scanned for imported songs.
    pub import_dir: Option<PathBuf>,
    /// Whether a song change is pending (set by skip/next).
    pub pending_change: bool,
}

impl Default for MusicPlayer {
    fn default() -> Self {
        Self {
            enabled: true,
            volume: DEFAULT_VOLUME,
            current: None,
            paused: false,
            position: 0,
            queue: Vec::new(),
            repeat: RepeatMode::None,
            shuffle: false,
            liked: Vec::new(),
            playlists: Vec::new(),
            events: HashMap::new(),
            import_dir: None,
            pending_change: false,
        }
    }
}

impl MusicPlayer {
    /// Start or resume playback of `song`, clearing the queue.
    pub fn play(&mut self, song: Song) {
        self.current = Some(song);
        self.paused = false;
        self.position = 0;
        self.queue.clear();
        self.pending_change = false;
    }

    /// Queue `song` to play after the current one.
    pub fn queue(&mut self, song: Song) {
        self.queue.push(song);
    }

    /// Pause or resume playback.
    pub fn toggle_pause(&mut self) {
        if self.current.is_some() {
            self.paused = !self.paused;
        }
    }

    /// Stop playback and clear the queue.
    pub fn stop(&mut self) {
        self.current = None;
        self.paused = false;
        self.position = 0;
        self.queue.clear();
        self.pending_change = false;
    }

    /// Skip to the next song in the queue, or stop if none.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Song> {
        self.queue.pop() // LIFO: most recently queued plays next
    }

    /// Advance the playback position by `seconds`. Returns true when the
    /// current song has finished.
    pub fn tick_position(&mut self, seconds: u32) -> bool {
        if let Some(song) = self.current.as_ref() {
            self.position = self.position.saturating_add(seconds);
            if self.position >= song.duration && song.duration > 0 {
                self.position = song.duration;
                return true;
            }
        }
        false
    }

    /// Whether the player likes the current song.
    pub fn is_liked(&self) -> bool {
        self.current
            .as_ref()
            .is_some_and(|s| self.liked.contains(&s.id))
    }

    /// Toggle the like state of the current song.
    pub fn toggle_like(&mut self) {
        if let Some(song) = self.current.as_ref() {
            if self.liked.contains(&song.id) {
                self.liked.retain(|id| id != &song.id);
            } else {
                self.liked.push(song.id.clone());
            }
        }
    }

    /// The song assigned to an event, if any.
    pub fn event_song<'a>(&self, event: EventType, library: &'a [Song]) -> Option<&'a Song> {
        let id = self.events.get(&event)?;
        library.iter().find(|s| s.id == *id)
    }

    /// Volume as a fraction (0.0..=1.0).
    pub fn volume_fraction(&self) -> f32 {
        self.volume as f32 / 100.0
    }
}

/// A custom playlist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Playlist {
    pub name: String,
    pub songs: Vec<String>, // song ids
}

impl Playlist {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            songs: Vec::new(),
        }
    }
}

/// Settings persisted per key in the mod's data directory.
#[derive(Clone, Debug)]
pub struct Settings {
    pub enabled: bool,
    pub volume: u8,
    pub event_assignments: HashMap<EventType, String>,
    pub import_dir: Option<PathBuf>,
}

impl Settings {
    pub fn read(get: impl Fn(&str) -> Option<String>) -> Self {
        let enabled = get("enabled").is_none_or(|v| v.trim() == "true");
        let volume = get("volume")
            .and_then(|v| v.trim().parse::<u8>().ok())
            .unwrap_or(DEFAULT_VOLUME)
            .clamp(0, 100);
        let mut event_assignments = HashMap::new();
        for event in EventType::ALL {
            if let Some(id) = get(event.settings_key())
                && !id.is_empty()
            {
                event_assignments.insert(event, id);
            }
        }
        let import_dir = get("import_dir")
            .filter(|v| !v.trim().is_empty())
            .map(PathBuf::from);
        Self {
            enabled,
            volume,
            event_assignments,
            import_dir,
        }
    }

    /// The key/value pairs to persist.
    pub fn stored(&self) -> Vec<(&'static str, String)> {
        let mut pairs = vec![
            ("enabled", self.enabled.to_string()),
            ("volume", self.volume.to_string()),
        ];
        for event in EventType::ALL {
            if let Some(id) = self.event_assignments.get(&event) {
                pairs.push((event.settings_key(), id.clone()));
            }
        }
        if let Some(dir) = &self.import_dir {
            pairs.push(("import_dir", dir.to_string_lossy().to_string()));
        }
        pairs
    }
}

// The settings, read once from files then kept in a thread-local cell.
thread_local! {
    static SETTINGS: std::cell::RefCell<Option<Settings>> = const { std::cell::RefCell::new(None) };
    static PLAYER: std::cell::RefCell<Option<MusicPlayer>> = const { std::cell::RefCell::new(None) };
}

/// Read settings once through `get` (one file per key).
pub fn settings(get: impl Fn(&str) -> Option<String>) -> Settings {
    SETTINGS.with(|s| {
        let mut slot = s.borrow_mut();
        slot.get_or_insert_with(|| Settings::read(get)).clone()
    })
}

/// Replace the settings (the settings page changed them).
pub fn set_settings(settings: Settings) {
    SETTINGS.with(|s| *s.borrow_mut() = Some(settings));
}

/// The music player, initialised from settings.
pub fn player(get: impl Fn(&str) -> Option<String>) -> MusicPlayer {
    PLAYER.with(|p| {
        let mut slot = p.borrow_mut();
        slot.get_or_insert_with(|| {
            let s = Settings::read(&get);
            MusicPlayer {
                enabled: s.enabled,
                volume: s.volume,
                events: s.event_assignments,
                import_dir: s.import_dir,
                ..Default::default()
            }
        })
        .clone()
    })
}

/// Replace the music player state.
pub fn set_player(player: MusicPlayer) {
    PLAYER.with(|p| *p.borrow_mut() = Some(player));
}

/// Scan `dir` for audio files and return them as imported songs.
pub fn scan_import_dir(dir: &PathBuf) -> Vec<Song> {
    let mut songs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && let Some(ext) = path.extension().and_then(|e| e.to_str())
            {
                let lower = ext.to_lowercase();
                if matches!(
                    lower.as_str(),
                    "mp3" | "flac" | "wav" | "m4a" | "aac" | "opus" | "ogg"
                ) && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                {
                    songs.push(Song::imported(
                        &format!("import:{}", stem),
                        stem,
                        "Imported",
                        path.clone(),
                    ));
                }
            }
        }
    }
    songs
}

/// The vanilla soundtrack library (Minecraft C418 / Lena Raine tracks).
pub fn vanilla_library() -> Vec<Song> {
    vec![
        Song::vanilla(
            "minecraft:music.13",
            "13",
            "C418",
            "Minecraft - Volume Alpha",
            166,
        ),
        Song::vanilla(
            "minecraft:music.cat",
            "Cat",
            "C418",
            "Minecraft - Volume Alpha",
            179,
        ),
        Song::vanilla(
            "minecraft:music.blocks",
            "Blocks",
            "C418",
            "Minecraft - Volume Alpha",
            134,
        ),
        Song::vanilla(
            "minecraft:music.chirp",
            "Chirp",
            "C418",
            "Minecraft - Volume Alpha",
            106,
        ),
        Song::vanilla(
            "minecraft:music.far",
            "Far",
            "C418",
            "Minecraft - Volume Alpha",
            166,
        ),
        Song::vanilla(
            "minecraft:music.mall",
            "Mall",
            "C418",
            "Minecraft - Volume Alpha",
            129,
        ),
        Song::vanilla(
            "minecraft:music.mellohi",
            "Mellohi",
            "C418",
            "Minecraft - Volume Alpha",
            67,
        ),
        Song::vanilla(
            "minecraft:music.stal",
            "Stal",
            "C418",
            "Minecraft - Volume Alpha",
            151,
        ),
        Song::vanilla(
            "minecraft:music.strad",
            "Strad",
            "C418",
            "Minecraft - Volume Alpha",
            149,
        ),
        Song::vanilla(
            "minecraft:music.ward",
            "Ward",
            "C418",
            "Minecraft - Volume Alpha",
            120,
        ),
        Song::vanilla(
            "minecraft:music.11",
            "11",
            "C418",
            "Minecraft - Volume Alpha",
            251,
        ),
        Song::vanilla(
            "minecraft:music.wait",
            "Wait",
            "C418",
            "Minecraft - Volume Alpha",
            137,
        ),
        Song::vanilla(
            "minecraft:music.flat",
            "Flat",
            "C418",
            "Minecraft - Volume Alpha",
            106,
        ),
        Song::vanilla(
            "minecraft:music.mice_on_venus",
            "Mice on Venus",
            "C418",
            "Minecraft - Volume Alpha",
            124,
        ),
        Song::vanilla(
            "minecraft:music.pigstep",
            "Pigstep",
            "Lena Raine",
            "Minecraft - Volume Beta",
            151,
        ),
        Song::vanilla(
            "minecraft:music.vibes",
            "Vibes",
            "Lena Raine",
            "Minecraft - Volume Beta",
            124,
        ),
    ]
}

/// Format a duration in seconds as `M:SS`.
pub fn format_duration(seconds: u32) -> String {
    let m = seconds / 60;
    let s = seconds % 60;
    format!("{m}:{s:02}")
}

/// Format the current time as `M:SS`.
pub fn format_position(seconds: u32) -> String {
    format_duration(seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stored<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |key| pairs.iter().find(|p| p.0 == key).map(|p| p.1.to_owned())
    }

    #[test]
    fn settings_read_defaults() {
        let s = Settings::read(stored(&[]));
        assert!(s.enabled);
        assert_eq!(s.volume, DEFAULT_VOLUME);
        assert!(s.event_assignments.is_empty());
        assert!(s.import_dir.is_none());
    }

    #[test]
    fn settings_read_custom() {
        let s = Settings::read(stored(&[
            ("enabled", "false"),
            ("volume", "42"),
            ("event_rain", "minecraft:music.pigstep"),
            ("import_dir", "/songs/"),
        ]));
        assert!(!s.enabled);
        assert_eq!(s.volume, 42);
        assert_eq!(
            s.event_assignments.get(&EventType::Rain),
            Some(&"minecraft:music.pigstep".to_owned())
        );
        assert_eq!(
            s.import_dir.as_deref(),
            Some(std::path::Path::new("/songs/"))
        );
    }

    #[test]
    fn settings_volume_clamps() {
        let s = Settings::read(stored(&[("volume", "200")]));
        assert_eq!(s.volume, 100);
        let s = Settings::read(stored(&[("volume", "255")]));
        assert_eq!(s.volume, 100);
    }

    #[test]
    fn settings_stored_roundtrip() {
        let mut events = HashMap::new();
        events.insert(EventType::Rain, "minecraft:music.pigstep".to_owned());
        events.insert(EventType::Credits, "minecraft:music.11".to_owned());
        let s = Settings {
            enabled: false,
            volume: 50,
            event_assignments: events,
            import_dir: Some(PathBuf::from("/songs")),
        };
        let stored: HashMap<&str, String> = s.stored().into_iter().collect();
        assert_eq!(stored.get("enabled"), Some(&"false".to_owned()));
        assert_eq!(stored.get("volume"), Some(&"50".to_owned()));
        assert_eq!(
            stored.get("event_rain"),
            Some(&"minecraft:music.pigstep".to_owned())
        );
        assert_eq!(
            stored.get("event_credits"),
            Some(&"minecraft:music.11".to_owned())
        );
        assert_eq!(stored.get("import_dir"), Some(&"/songs".to_owned()));
    }

    #[test]
    fn player_play_pause_stop() {
        let mut p = MusicPlayer::default();
        let song = Song::vanilla("minecraft:music.cat", "Cat", "C418", "Vol Alpha", 179);
        p.play(song.clone());
        assert!(p.current.is_some());
        assert!(!p.paused);
        assert_eq!(p.position, 0);

        p.toggle_pause();
        assert!(p.paused);
        p.toggle_pause();
        assert!(!p.paused);

        p.stop();
        assert!(p.current.is_none());
        assert!(!p.paused);
        assert_eq!(p.position, 0);
        assert!(p.queue.is_empty());
    }

    #[test]
    fn player_next_from_queue() {
        let mut p = MusicPlayer::default();
        p.play(Song::vanilla(
            "minecraft:music.cat",
            "Cat",
            "C418",
            "Vol Alpha",
            179,
        ));
        p.queue(Song::vanilla(
            "minecraft:music.pigstep",
            "Pigstep",
            "Lena Raine",
            "Vol Beta",
            151,
        ));
        let next = p.next();
        assert!(next.is_some());
        assert_eq!(next.as_ref().unwrap().id, "minecraft:music.pigstep");
    }

    #[test]
    fn player_tick_finishes_song() {
        let mut p = MusicPlayer::default();
        p.play(Song::vanilla(
            "minecraft:music.mellohi",
            "Mellohi",
            "C418",
            "Vol Alpha",
            67,
        ));
        assert!(!p.tick_position(30));
        assert_eq!(p.position, 30);
        assert!(p.tick_position(40));
        assert_eq!(p.position, 67);
    }

    #[test]
    fn player_like_toggle() {
        let mut p = MusicPlayer::default();
        p.play(Song::vanilla(
            "minecraft:music.cat",
            "Cat",
            "C418",
            "Vol Alpha",
            179,
        ));
        assert!(!p.is_liked());
        p.toggle_like();
        assert!(p.is_liked());
        p.toggle_like();
        assert!(!p.is_liked());
    }

    #[test]
    fn event_type_labels_and_keys() {
        assert_eq!(EventType::Rain.label(), "Rain");
        assert_eq!(EventType::FirstNether.label(), "First Nether entry");
        assert_eq!(EventType::Rain.settings_key(), "event_rain");
        assert_eq!(EventType::Credits.settings_key(), "event_credits");
        assert_eq!(EventType::ALL.len(), 6);
    }

    #[test]
    fn format_duration() {
        assert_eq!(super::format_duration(0), "0:00");
        assert_eq!(super::format_duration(5), "0:05");
        assert_eq!(super::format_duration(65), "1:05");
        assert_eq!(super::format_duration(3661), "61:01");
    }

    #[test]
    fn vanilla_library_not_empty() {
        let lib = vanilla_library();
        assert!(!lib.is_empty());
        assert!(lib.iter().any(|s| s.id == "minecraft:music.pigstep"));
        assert!(lib.iter().all(|s| s.is_vanilla));
    }

    #[test]
    fn import_dir_scans_audio_extensions() {
        use std::fs;
        let dir = tempfile_dir();
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("song.mp3"), b"").unwrap();
        fs::write(dir.join("song.flac"), b"").unwrap();
        fs::write(dir.join("song.txt"), b"").unwrap(); // not audio
        let songs = scan_import_dir(&dir);
        let ids: Vec<&str> = songs.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"import:song"));
        assert!(!ids.iter().any(|i| i.contains("txt")));
        let _ = fs::remove_dir_all(dir);
    }

    fn tempfile_dir() -> PathBuf {
        PathBuf::from(std::env::temp_dir()).join(format!("overtune-test-{}", std::process::id()))
    }
}
