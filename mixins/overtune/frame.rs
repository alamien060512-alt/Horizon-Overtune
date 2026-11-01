// HEAD hook on `native_frame`: once per frame, advance playback and
// auto-advance to the next song when the current one finishes.
//
// Music playback control itself hooks the game's internal audio methods;
// this hook drives the player state machine.

fn overtune_frame(game: &mut crate::game::Game<'_>, elapsed: std::time::Duration) {
    use crate::game::overtune as m;

    if !game.native_on(m::ID) {
        return;
    }

    let settings = m::settings(|key| game.native_setting(m::ID, key));
    if !settings.enabled {
        return;
    }

    // Advance playback position (roughly 60 frames per second -> ~1 second per 60 ticks).
    let mut player = m::player(|key| game.native_setting(m::ID, key));
    if let Some(song) = player.current.as_ref() {
        if !player.paused {
            let advance = (elapsed.as_secs_f32() * 60.0) as u32;
            if player.tick_position(advance) {
                // Song finished: play the next queued song.
                if let Some(next) = player.next() {
                    player.play(next);
                } else if player.repeat == m::RepeatMode::One {
                    if let Some(current) = player.current.clone() {
                        player.play(current);
                    }
                } else if player.repeat == m::RepeatMode::All {
                    if let Some(next) = player.next() {
                        player.play(next);
                    } else {
                        player.stop();
                    }
                } else {
                    player.stop();
                }
            }
        }
    }
    m::set_player(player);
}
