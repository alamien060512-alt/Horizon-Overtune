// HEAD hook on `native_world_lifecycle`: event-based music.
//
// World open/close events start or stop playback.

fn overtune_lifecycle(
    game: &mut crate::game::Game<'_>,
    context: &crate::game::native_lifecycle::NativeLifecycleContext,
    phase: crate::game::native_lifecycle::WorldPhase,
    error_text: &str,
) {
    use crate::game::overtune as m;

    let _ = context;
    let _ = error_text;

    if !game.native_on(m::ID) {
        return;
    }

    let settings = {
        let game = &*game;
        m::settings(|key| game.native_setting(m::ID, key))
    };
    if !settings.enabled {
        return;
    }

    match phase {
        crate::game::native_lifecycle::WorldPhase::WorldOpenReady => {
            // World opened successfully; ambient event music can start here.
        }
        crate::game::native_lifecycle::WorldPhase::WorldOpenFailed => {
            let _ = game.native_chat("Overtune: world failed to open.");
        }
        crate::game::native_lifecycle::WorldPhase::WorldCloseFinal => {
            // World closed normally; stop playback.
            let mut player = m::player(|key| game.native_setting(m::ID, key));
            player.stop();
            m::set_player(player);
        }
        crate::game::native_lifecycle::WorldPhase::WorldCloseFailed => {
            let mut player = m::player(|key| game.native_setting(m::ID, key));
            player.stop();
            m::set_player(player);
        }
    }
}
