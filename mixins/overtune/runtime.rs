// Overtune for Horizon: the player's logic. Nothing here touches the game, so
// Cargo tests it alone; the hook adapters in this folder carry it to Horizon.

pub const ID: &str = "overtune";
pub const NAME: &str = "Overtune";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "A Spotify-style music player for Minecraft's soundtrack: play, pause, skip, shuffle, repeat, likes, albums and a now-playing toast.";

/// Ticks the now-playing toast stays up (20 a second).
pub const TOAST_TICKS: u32 = 120;
/// Failed starts in a row before Overtune pauses itself.
const MAX_FAILURES: u32 = 3;

/// A sound event of the game's soundtrack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Track {
    pub event: &'static str,
    pub title: &'static str,
    pub artist: &'static str,
    pub album: &'static str,
}

const fn t(
    event: &'static str,
    title: &'static str,
    artist: &'static str,
    album: &'static str,
) -> Track {
    Track {
        event,
        title,
        artist,
        album,
    }
}

pub const ALBUMS: [&str; 3] = ["Soundtrack", "Nether", "Music Discs"];

/// The library: sound events the game's `sounds.json` names. The sound
/// engine cannot list its registry, so the list is fixed; an event a pack
/// lacks fails to start and is skipped.
pub const CATALOG: &[Track] = &[
    t(
        "minecraft:music.game",
        "Overworld",
        "Minecraft",
        "Soundtrack",
    ),
    t(
        "minecraft:music.creative",
        "Creative",
        "Minecraft",
        "Soundtrack",
    ),
    t(
        "minecraft:music.menu",
        "Main Menu",
        "Minecraft",
        "Soundtrack",
    ),
    t(
        "minecraft:music.under_water",
        "Underwater",
        "Minecraft",
        "Soundtrack",
    ),
    t("minecraft:music.end", "The End", "Minecraft", "Soundtrack"),
    t(
        "minecraft:music.dragon",
        "Ender Dragon",
        "Minecraft",
        "Soundtrack",
    ),
    t(
        "minecraft:music.credits",
        "Credits",
        "Minecraft",
        "Soundtrack",
    ),
    t(
        "minecraft:music.overworld.deep_dark",
        "Deep Dark",
        "Minecraft",
        "Soundtrack",
    ),
    t(
        "minecraft:music.overworld.meadow",
        "Meadow",
        "Minecraft",
        "Soundtrack",
    ),
    t(
        "minecraft:music.overworld.lush_caves",
        "Lush Caves",
        "Minecraft",
        "Soundtrack",
    ),
    t(
        "minecraft:music.nether.nether_wastes",
        "Nether Wastes",
        "Minecraft",
        "Nether",
    ),
    t(
        "minecraft:music.nether.soul_sand_valley",
        "Soul Sand Valley",
        "Minecraft",
        "Nether",
    ),
    t(
        "minecraft:music.nether.crimson_forest",
        "Crimson Forest",
        "Minecraft",
        "Nether",
    ),
    t(
        "minecraft:music.nether.warped_forest",
        "Warped Forest",
        "Minecraft",
        "Nether",
    ),
    t(
        "minecraft:music.nether.basalt_deltas",
        "Basalt Deltas",
        "Minecraft",
        "Nether",
    ),
    t("minecraft:music_disc.13", "13", "C418", "Music Discs"),
    t("minecraft:music_disc.cat", "cat", "C418", "Music Discs"),
    t(
        "minecraft:music_disc.blocks",
        "blocks",
        "C418",
        "Music Discs",
    ),
    t("minecraft:music_disc.chirp", "chirp", "C418", "Music Discs"),
    t("minecraft:music_disc.far", "far", "C418", "Music Discs"),
    t("minecraft:music_disc.mall", "mall", "C418", "Music Discs"),
    t(
        "minecraft:music_disc.mellohi",
        "mellohi",
        "C418",
        "Music Discs",
    ),
    t("minecraft:music_disc.stal", "stal", "C418", "Music Discs"),
    t("minecraft:music_disc.strad", "strad", "C418", "Music Discs"),
    t("minecraft:music_disc.ward", "ward", "C418", "Music Discs"),
    t("minecraft:music_disc.11", "11", "C418", "Music Discs"),
    t("minecraft:music_disc.wait", "wait", "C418", "Music Discs"),
    t(
        "minecraft:music_disc.pigstep",
        "Pigstep",
        "Lena Raine",
        "Music Discs",
    ),
    t(
        "minecraft:music_disc.otherside",
        "otherside",
        "Lena Raine",
        "Music Discs",
    ),
    t("minecraft:music_disc.5", "5", "Samuel Aberg", "Music Discs"),
    t(
        "minecraft:music_disc.relic",
        "Relic",
        "Aaron Cherof",
        "Music Discs",
    ),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Repeat {
    Off,
    All,
    One,
}

impl Repeat {
    fn next(self) -> Self {
        match self {
            Repeat::Off => Repeat::All,
            Repeat::All => Repeat::One,
            Repeat::One => Repeat::Off,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Repeat::Off => "off",
            Repeat::All => "all",
            Repeat::One => "one",
        }
    }
    fn parse(text: &str) -> Self {
        match text {
            "all" => Repeat::All,
            "one" => Repeat::One,
            _ => Repeat::Off,
        }
    }
}

/// Which part of the library the queue plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    All,
    Liked,
    Album(usize),
}

impl Scope {
    pub fn name(self) -> String {
        match self {
            Scope::All => "All songs".into(),
            Scope::Liked => "Liked songs".into(),
            Scope::Album(i) => ALBUMS[i % ALBUMS.len()].into(),
        }
    }
    fn next(self) -> Self {
        match self {
            Scope::All => Scope::Liked,
            Scope::Liked => Scope::Album(0),
            Scope::Album(i) if i + 1 < ALBUMS.len() => Scope::Album(i + 1),
            Scope::Album(_) => Scope::All,
        }
    }
    fn code(self) -> String {
        match self {
            Scope::All => "all".into(),
            Scope::Liked => "liked".into(),
            Scope::Album(i) => format!("album{i}"),
        }
    }
    fn parse(text: &str) -> Self {
        match text {
            "liked" => Scope::Liked,
            _ => match text.strip_prefix("album").and_then(|n| n.parse().ok()) {
                Some(i) if i < ALBUMS.len() => Scope::Album(i),
                _ => Scope::All,
            },
        }
    }
}

/// What the game side must do after `Player::plan`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Plan {
    /// Stop the instance Overtune started.
    pub stop: bool,
    /// Then start this catalog track.
    pub play: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Player {
    pub enabled: bool,
    pub paused: bool,
    pub shuffle: bool,
    pub repeat: Repeat,
    pub scope: Scope,
    pub volume: u8,
    pub toast: bool,
    pub liked: Vec<usize>,
    pub current: Option<usize>,
    /// The sound instance Overtune started, as the game numbers them.
    pub handle: Option<u64>,
    pub toast_left: u32,
    history: Vec<usize>,
    restart: bool,
    failures: u32,
    sent: Option<u8>,
    rng: u64,
}

impl Player {
    pub fn new(seed: u64) -> Self {
        Self {
            enabled: true,
            paused: false,
            shuffle: false,
            repeat: Repeat::All,
            scope: Scope::All,
            volume: 80,
            toast: true,
            liked: Vec::new(),
            current: None,
            handle: None,
            toast_left: 0,
            history: Vec::new(),
            restart: false,
            failures: 0,
            sent: None,
            rng: seed | 1,
        }
    }

    fn random(&mut self) -> u64 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        x
    }

    /// Catalog indices the queue plays, in library order.
    pub fn queue(&self) -> Vec<usize> {
        (0..CATALOG.len())
            .filter(|i| match self.scope {
                Scope::All => true,
                Scope::Liked => self.liked.contains(i),
                Scope::Album(a) => CATALOG[*i].album == ALBUMS[a % ALBUMS.len()],
            })
            .collect()
    }

    pub fn track(&self) -> Option<&'static Track> {
        self.current.map(|i| &CATALOG[i])
    }

    pub fn is_liked(&self) -> bool {
        self.current.is_some_and(|i| self.liked.contains(&i))
    }

    fn shown(&mut self) {
        self.toast_left = if self.toast { TOAST_TICKS } else { 0 };
    }

    /// The track after the current one. `natural` is a track that ended by
    /// itself, where "repeat off" stops at the end of the queue.
    fn pick_next(&mut self, natural: bool) -> Option<usize> {
        let queue = self.queue();
        if queue.is_empty() {
            return None;
        }
        if natural && self.repeat == Repeat::One && self.current.is_some_and(|c| queue.contains(&c))
        {
            return self.current;
        }
        if self.shuffle {
            let others: Vec<usize> = queue
                .iter()
                .copied()
                .filter(|i| Some(*i) != self.current)
                .collect();
            let pool = if others.is_empty() { &queue } else { &others };
            return Some(pool[(self.random() % pool.len() as u64) as usize]);
        }
        let at = self
            .current
            .and_then(|c| queue.iter().position(|q| *q == c));
        match at {
            Some(p) if p + 1 < queue.len() => Some(queue[p + 1]),
            Some(_) if natural && self.repeat == Repeat::Off => None,
            _ => Some(queue[0]),
        }
    }

    fn go(&mut self, track: Option<usize>, remember: bool) {
        if remember && let Some(old) = self.current {
            self.history.push(old);
            if self.history.len() > 64 {
                self.history.remove(0);
            }
        }
        self.current = track;
        self.restart = track.is_some();
        self.paused = track.is_none() && self.paused;
        self.shown();
    }

    /// Play/pause. With nothing chosen yet, starts the queue.
    pub fn toggle_pause(&mut self) {
        self.enabled = true;
        if self.current.is_none() {
            let first = self.pick_next(false);
            self.paused = false;
            self.go(first, false);
        } else if self.paused {
            self.paused = false;
            self.restart = true;
            self.shown();
        } else {
            self.paused = true;
            self.shown();
        }
    }

    pub fn skip(&mut self) {
        self.enabled = true;
        self.paused = false;
        let next = self.pick_next(false);
        self.go(next, true);
    }

    pub fn previous(&mut self) {
        self.enabled = true;
        self.paused = false;
        let back = self.history.pop().or_else(|| {
            let queue = self.queue();
            let at = self
                .current
                .and_then(|c| queue.iter().position(|q| *q == c));
            at.map(|p| queue[(p + queue.len() - 1) % queue.len()])
        });
        self.go(back, false);
    }

    /// Plays one catalog track now.
    pub fn play_track(&mut self, track: usize) {
        if track < CATALOG.len() {
            self.enabled = true;
            self.paused = false;
            self.go(Some(track), true);
        }
    }

    pub fn toggle_like(&mut self) {
        if let Some(c) = self.current {
            match self.liked.iter().position(|l| *l == c) {
                Some(p) => {
                    self.liked.remove(p);
                }
                None => {
                    self.liked.push(c);
                    self.liked.sort_unstable();
                }
            }
            self.shown();
        }
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
        self.shown();
    }

    pub fn cycle_repeat(&mut self) {
        self.repeat = self.repeat.next();
        self.shown();
    }

    pub fn cycle_scope(&mut self) {
        self.scope = self.scope.next();
        self.shown();
    }

    pub fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
        self.shown();
    }

    pub fn toggle_toast(&mut self) {
        self.toast = !self.toast;
        self.shown();
    }

    pub fn set_volume(&mut self, volume: i32) {
        self.volume = volume.clamp(0, 100) as u8;
        self.shown();
    }

    /// Volume in steps of ten, wrapping from 100 to 0 on the settings page.
    pub fn cycle_volume(&mut self) {
        let next = (i32::from(self.volume) / 10 * 10 + 10) % 110;
        self.set_volume(next);
    }

    pub fn gain(&self) -> f32 {
        f32::from(self.volume) / 100.0
    }

    /// The gain to send the sound engine, once for each volume.
    pub fn take_gain(&mut self) -> Option<f32> {
        (self.sent != Some(self.volume)).then(|| {
            self.sent = Some(self.volume);
            self.gain()
        })
    }

    /// Called once a tick with whether Overtune's instance still plays.
    pub fn plan(&mut self, active: bool) -> Plan {
        if self.toast_left > 0 {
            self.toast_left -= 1;
        }
        if !self.enabled || self.paused {
            self.restart = false;
            self.handle = None;
            return Plan {
                stop: active,
                play: None,
            };
        }
        if self.restart {
            self.restart = false;
            self.handle = None;
            return Plan {
                stop: active,
                play: self.current,
            };
        }
        if !active && self.current.is_some() && self.handle.is_some() {
            // The track ended by itself.
            let next = self.pick_next(true);
            self.go(next, true);
            self.restart = false;
            self.handle = None;
            return Plan {
                stop: false,
                play: next,
            };
        }
        Plan::default()
    }

    /// The game answered `plan`'s play: whether the track started.
    pub fn started(&mut self, handle: Option<u64>) {
        self.handle = handle;
        if handle.is_some() {
            self.failures = 0;
            return;
        }
        self.failures += 1;
        if self.failures >= MAX_FAILURES || self.queue().len() <= 1 {
            self.failures = 0;
            self.paused = true;
        } else {
            let next = self.pick_next(false);
            self.go(next, false);
        }
    }

    /// Settings to keep, as (key, value).
    pub fn stored(&self) -> Vec<(&'static str, String)> {
        let liked: Vec<String> = self
            .liked
            .iter()
            .map(|i| CATALOG[*i].event.to_owned())
            .collect();
        vec![
            ("enabled", self.enabled.to_string()),
            ("shuffle", self.shuffle.to_string()),
            ("repeat", self.repeat.name().into()),
            ("scope", self.scope.code()),
            ("volume", self.volume.to_string()),
            ("toast", self.toast.to_string()),
            ("liked", liked.join(",")),
        ]
    }

    pub fn read(seed: u64, get: impl Fn(&str) -> Option<String>) -> Self {
        let mut p = Self::new(seed);
        let flag = |key: &str, fallback: bool| get(key).map_or(fallback, |v| v == "true");
        p.enabled = flag("enabled", true);
        p.shuffle = flag("shuffle", false);
        p.toast = flag("toast", true);
        p.repeat = get("repeat").map_or(Repeat::All, |v| Repeat::parse(&v));
        p.scope = get("scope").map_or(Scope::All, |v| Scope::parse(&v));
        p.volume = get("volume")
            .and_then(|v| v.parse::<u8>().ok())
            .map_or(80, |v| v.min(100));
        if let Some(liked) = get("liked") {
            p.liked = liked
                .split(',')
                .filter_map(|e| CATALOG.iter().position(|t| t.event == e))
                .collect();
            p.liked.sort_unstable();
        }
        p
    }

    /// The labels of the settings page's buttons.
    pub fn labels(&self) -> Vec<String> {
        let on = |b: bool| if b { "On" } else { "Off" };
        vec![
            format!("Overtune: {}", on(self.enabled)),
            format!("Shuffle: {}", on(self.shuffle)),
            format!("Repeat: {}", self.repeat.name()),
            format!("Library: {}", self.scope.name()),
            format!("Volume: {}%", self.volume),
            format!("Now-playing toast: {}", on(self.toast)),
        ]
    }

    pub fn click(&mut self, button: usize) {
        match button {
            0 => self.toggle_enabled(),
            1 => self.toggle_shuffle(),
            2 => self.cycle_repeat(),
            3 => {
                self.cycle_scope();
                // A new library starts from its first song, when playing.
                if self.current.is_some() && !self.queue().contains(&self.current.unwrap_or(0)) {
                    self.skip();
                }
            }
            4 => self.cycle_volume(),
            5 => self.toggle_toast(),
            _ => {}
        }
    }

    /// A chat command (`/overtune ...`), without the slash and the name.
    /// Returns the lines to print.
    pub fn command(&mut self, args: &str) -> Vec<String> {
        let mut words = args.split_whitespace();
        let word = words.next().unwrap_or("status");
        let rest: Vec<&str> = words.collect();
        match word {
            "play" | "pause" | "toggle" => self.toggle_pause(),
            "next" | "skip" => self.skip(),
            "prev" | "previous" | "back" => self.previous(),
            "shuffle" => self.toggle_shuffle(),
            "repeat" => self.cycle_repeat(),
            "like" | "unlike" => self.toggle_like(),
            "toast" => self.toggle_toast(),
            "scope" | "library" => self.cycle_scope(),
            "on" | "off" => self.enabled = word == "on",
            "volume" | "vol" => match rest.first().and_then(|v| v.parse::<i32>().ok()) {
                Some(v) => self.set_volume(v),
                None => return vec!["Usage: /overtune volume <0-100>".into()],
            },
            "list" | "songs" => return self.list(),
            "select" | "song" => {
                let queue = self.queue();
                match rest.first().and_then(|n| n.parse::<usize>().ok()) {
                    Some(n) if (1..=queue.len()).contains(&n) => self.play_track(queue[n - 1]),
                    _ => return vec![format!("Usage: /overtune select <1-{}>", queue.len())],
                }
            }
            "status" => {}
            _ => {
                return vec![
                    "Overtune: play, next, prev, shuffle, repeat, like, volume <0-100>,".into(),
                    "list, select <n>, scope, toast, on, off, status".into(),
                ];
            }
        }
        vec![self.status()]
    }

    pub fn list(&self) -> Vec<String> {
        let queue = self.queue();
        let mut lines = vec![format!("{} ({} songs)", self.scope.name(), queue.len())];
        for (n, i) in queue.iter().enumerate() {
            let t = &CATALOG[*i];
            let heart = if self.liked.contains(i) { " <3" } else { "" };
            let mark = if self.current == Some(*i) { ">" } else { " " };
            lines.push(format!(
                "{mark}{:>2}. {} - {}{heart}",
                n + 1,
                t.title,
                t.artist
            ));
        }
        lines
    }

    pub fn status(&self) -> String {
        if !self.enabled {
            return "Overtune is off".into();
        }
        let state = match (self.track(), self.paused) {
            (None, _) => "Stopped".to_owned(),
            (Some(t), true) => format!("Paused: {} - {}", t.title, t.artist),
            (Some(t), false) => format!("Playing: {} - {}", t.title, t.artist),
        };
        format!(
            "{state} | shuffle {} | repeat {} | {} | volume {}%",
            if self.shuffle { "on" } else { "off" },
            self.repeat.name(),
            self.scope.name(),
            self.volume
        )
    }
}

/// What the toast draws, in GUI pixels from the box's top left.
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    Fill([f32; 4], u32),
    Text(String, [f32; 2], u32, bool),
}

const PAD: f32 = 5.0;
const LINE: f32 = 11.0;

/// The size of the toast box for these lines.
pub fn toast_size(width: &dyn Fn(&str) -> f32, lines: &[String]) -> (f32, f32) {
    let w = lines.iter().map(|l| width(l)).fold(0.0, f32::max);
    (w + PAD * 2.0, LINE * lines.len() as f32 + PAD * 2.0 - 2.0)
}

/// The toast's lines: title, artist and album, then the player's state.
pub fn toast_lines(p: &Player) -> Vec<String> {
    let Some(t) = p.track() else {
        return vec!["Overtune".into(), "Nothing playing".into()];
    };
    let heart = if p.is_liked() { " <3" } else { "" };
    let mut flags = vec![if p.paused { "Paused" } else { "Playing" }.to_owned()];
    if p.shuffle {
        flags.push("Shuffle".into());
    }
    if p.repeat != Repeat::Off {
        flags.push(format!("Repeat {}", p.repeat.name()));
    }
    flags.push(format!("Vol {}%", p.volume));
    vec![
        format!("{}{heart}", t.title),
        format!("{} - {}", t.artist, t.album),
        flags.join(" | "),
    ]
}

pub fn toast_shapes(lines: &[String], size: (f32, f32)) -> Vec<Shape> {
    let mut shapes = vec![Shape::Fill([0.0, 0.0, size.0, size.1], 0x7800_0000)];
    shapes.push(Shape::Fill([0.0, 0.0, 2.0, size.1], 0xFF1D_B954));
    for (i, line) in lines.iter().enumerate() {
        let rgb = if i == 0 { 0xFF_FFFF } else { 0xB3_B3B3 };
        shapes.push(Shape::Text(
            line.clone(),
            [PAD, PAD + LINE * i as f32],
            rgb,
            i == 0,
        ));
    }
    shapes
}

/// The keys Options > Mods lists first: name, label, default key.
pub fn keys() -> Vec<[String; 3]> {
    [
        ("play_pause", "Play / pause", "key.keyboard.f7"),
        ("previous", "Previous song", "key.keyboard.f6"),
        ("next", "Next song", "key.keyboard.f8"),
        ("shuffle", "Toggle shuffle", "key.keyboard.f9"),
        ("like", "Like song", "key.keyboard.f10"),
        ("volume_up", "Volume up", "key.keyboard.page.up"),
        ("volume_down", "Volume down", "key.keyboard.page.down"),
    ]
    .map(|(a, b, c)| [a.to_owned(), b.to_owned(), c.to_owned()])
    .to_vec()
}

/// Applies the key named `name`; false for a name that is not Overtune's.
pub fn press(p: &mut Player, name: &str) -> bool {
    match name {
        "play_pause" => p.toggle_pause(),
        "previous" => p.previous(),
        "next" => p.skip(),
        "shuffle" => p.toggle_shuffle(),
        "like" => p.toggle_like(),
        "volume_up" => p.set_volume(i32::from(p.volume) + 10),
        "volume_down" => p.set_volume(i32::from(p.volume) - 10),
        _ => return false,
    }
    true
}

// The game's tick and its HUD may run on different threads.
static PLAYER: std::sync::Mutex<Option<Player>> = std::sync::Mutex::new(None);

fn seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0x9E37_79B9, |d| d.as_nanos() as u64)
}

/// Runs `f` on the window's player, read from the stored settings the first
/// time.
pub fn with<R>(get: impl Fn(&str) -> Option<String>, f: impl FnOnce(&mut Player) -> R) -> R {
    let mut slot = PLAYER.lock().unwrap_or_else(|e| e.into_inner());
    f(slot.get_or_insert_with(|| Player::read(seed(), get)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player() -> Player {
        Player::new(42)
    }

    #[test]
    fn catalog_is_unique_and_in_known_albums() {
        for (i, a) in CATALOG.iter().enumerate() {
            assert!(ALBUMS.contains(&a.album), "{}", a.event);
            assert!(CATALOG[i + 1..].iter().all(|b| b.event != a.event));
        }
    }

    #[test]
    fn plays_in_order_and_wraps_on_repeat_all() {
        let mut p = player();
        p.toggle_pause();
        assert_eq!(p.current, Some(0));
        p.skip();
        assert_eq!(p.current, Some(1));
        p.current = Some(CATALOG.len() - 1);
        p.skip();
        assert_eq!(p.current, Some(0));
    }

    #[test]
    fn repeat_off_stops_at_the_end_and_repeat_one_repeats() {
        let mut p = player();
        p.repeat = Repeat::Off;
        p.current = Some(CATALOG.len() - 1);
        p.handle = Some(1);
        assert_eq!(p.plan(false).play, None);
        assert_eq!(p.current, None);
        p.repeat = Repeat::One;
        p.current = Some(3);
        p.handle = Some(1);
        assert_eq!(p.plan(false).play, Some(3));
        p.skip();
        assert_eq!(p.current, Some(4));
    }

    #[test]
    fn shuffle_never_repeats_the_current_song() {
        let mut p = player();
        p.shuffle = true;
        p.toggle_pause();
        for _ in 0..200 {
            let before = p.current;
            p.skip();
            assert_ne!(p.current, before);
        }
    }

    #[test]
    fn previous_returns_through_history() {
        let mut p = player();
        p.toggle_pause();
        p.skip();
        p.skip();
        assert_eq!(p.current, Some(2));
        p.previous();
        assert_eq!(p.current, Some(1));
        p.previous();
        assert_eq!(p.current, Some(0));
        p.previous();
        assert_eq!(p.current, Some(CATALOG.len() - 1));
    }

    #[test]
    fn scopes_filter_the_queue() {
        let mut p = player();
        p.scope = Scope::Album(1);
        assert!(p.queue().iter().all(|i| CATALOG[*i].album == "Nether"));
        p.scope = Scope::Liked;
        assert!(p.queue().is_empty());
        p.current = Some(4);
        p.toggle_like();
        assert_eq!(p.queue(), vec![4]);
        p.toggle_like();
        assert!(p.queue().is_empty());
        assert_eq!(p.pick_next(false), None);
    }

    #[test]
    fn plan_starts_stops_and_follows_the_track_end() {
        let mut p = player();
        p.toggle_pause();
        assert_eq!(
            p.plan(false),
            Plan {
                stop: false,
                play: Some(0)
            }
        );
        p.started(Some(7));
        assert_eq!(p.plan(true), Plan::default());
        // Skipping while playing stops the old instance first.
        p.skip();
        assert_eq!(
            p.plan(true),
            Plan {
                stop: true,
                play: Some(1)
            }
        );
        p.started(Some(8));
        // The track ended by itself.
        assert_eq!(p.plan(false).play, Some(2));
        // Pausing stops it; nothing plays while paused.
        p.started(Some(9));
        p.toggle_pause();
        assert_eq!(
            p.plan(true),
            Plan {
                stop: true,
                play: None
            }
        );
        assert_eq!(p.plan(false), Plan::default());
        p.toggle_pause();
        assert_eq!(p.plan(false).play, Some(2));
    }

    #[test]
    fn failed_starts_skip_then_pause() {
        let mut p = player();
        p.toggle_pause();
        for _ in 0..MAX_FAILURES {
            assert!(!p.paused);
            assert!(p.plan(false).play.is_some());
            p.started(None);
        }
        assert!(p.paused);
    }

    #[test]
    fn disabled_overtune_plays_nothing_and_releases_its_track() {
        let mut p = player();
        p.toggle_pause();
        p.plan(false);
        p.started(Some(1));
        p.toggle_enabled();
        assert_eq!(
            p.plan(true),
            Plan {
                stop: true,
                play: None
            }
        );
    }

    #[test]
    fn settings_round_trip() {
        let mut p = player();
        p.shuffle = true;
        p.repeat = Repeat::One;
        p.scope = Scope::Album(2);
        p.volume = 35;
        p.toast = false;
        p.current = Some(16);
        p.toggle_like();
        let stored = p.stored();
        let get = |k: &str| stored.iter().find(|(n, _)| *n == k).map(|(_, v)| v.clone());
        let q = Player::read(1, get);
        assert!(q.shuffle && !q.toast && q.enabled);
        assert_eq!(q.repeat, Repeat::One);
        assert_eq!(q.scope, Scope::Album(2));
        assert_eq!(q.volume, 35);
        assert_eq!(q.liked, vec![16]);
        assert_eq!(Player::read(1, |_| None).volume, 80);
        assert_eq!(Player::read(1, |_| Some("999".into())).volume, 80);
    }

    #[test]
    fn gain_is_sent_once_for_each_volume() {
        let mut p = player();
        assert_eq!(p.take_gain(), Some(0.8));
        assert_eq!(p.take_gain(), None);
        p.set_volume(30);
        assert_eq!(p.take_gain(), Some(0.3));
    }

    #[test]
    fn volume_clamps_and_cycles() {
        let mut p = player();
        p.set_volume(250);
        assert_eq!(p.volume, 100);
        p.set_volume(-5);
        assert_eq!(p.volume, 0);
        p.volume = 95;
        p.cycle_volume();
        assert_eq!(p.volume, 100);
        p.cycle_volume();
        assert_eq!(p.volume, 0);
        assert_eq!(p.gain(), 0.0);
    }

    #[test]
    fn keys_and_buttons_act() {
        let mut p = player();
        assert!(press(&mut p, "play_pause"));
        assert!(press(&mut p, "volume_down"));
        assert_eq!(p.volume, 70);
        assert!(!press(&mut p, "nothing"));
        assert_eq!(keys().len(), 7);
        assert_eq!(p.labels().len(), 6);
        p.click(1);
        assert!(p.shuffle);
        p.click(9);
    }

    #[test]
    fn changing_library_moves_off_a_song_it_lacks() {
        let mut p = player();
        p.play_track(0);
        p.click(3);
        assert_eq!(p.scope, Scope::Liked);
        // Liked is empty, so there is nothing to play.
        assert_eq!(p.current, None);
    }

    #[test]
    fn commands_answer_in_chat() {
        let mut p = player();
        assert!(p.command("play")[0].starts_with("Playing: Overworld"));
        assert!(p.command("volume 20")[0].contains("volume 20%"));
        assert!(p.command("volume x")[0].starts_with("Usage"));
        assert!(p.command("select 99")[0].starts_with("Usage"));
        assert!(p.command("select 2")[0].contains("Creative"));
        assert!(p.command("like")[0].contains("Creative"));
        assert!(p.list().len() > CATALOG.len());
        assert!(p.command("bogus")[0].starts_with("Overtune:"));
        assert_eq!(p.command("off"), vec!["Overtune is off".to_owned()]);
    }

    #[test]
    fn toast_lays_out_inside_its_box() {
        let mut p = player();
        assert_eq!(toast_lines(&p)[1], "Nothing playing");
        p.play_track(15);
        let lines = toast_lines(&p);
        assert_eq!(lines[0], "13");
        assert!(lines[2].starts_with("Playing"));
        let width = |s: &str| s.len() as f32 * 6.0;
        let size = toast_size(&width, &lines);
        let shapes = toast_shapes(&lines, size);
        assert_eq!(shapes.len(), 2 + lines.len());
        for s in &shapes {
            if let Shape::Text(t, [x, y], ..) = s {
                assert!(x + width(t) <= size.0 && *y < size.1);
            }
        }
    }

    #[test]
    fn toast_counts_down_and_can_be_off() {
        let mut p = player();
        p.skip();
        assert_eq!(p.toast_left, TOAST_TICKS);
        p.plan(false);
        assert_eq!(p.toast_left, TOAST_TICKS - 1);
        p.toast = false;
        p.skip();
        assert_eq!(p.toast_left, 0);
    }
}
