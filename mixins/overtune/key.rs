// HEAD hook on `native_key`: handle Overtune keybinds.
//
// Keys: play/pause, next track, previous track, toggle like, volume up/down.

fn overtune_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    use crate::game::overtune as m;

    if !pressed {
        return;
    }
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

    let play_pause = game.native_mod_key(m::ID, "play_pause", "key.keyboard.p");
    let next = game.native_mod_key(m::ID, "next", "key.keyboard.n");
    let like = game.native_mod_key(m::ID, "like", "key.keyboard.l");

    let mut player = m::player(|key| game.native_setting(m::ID, key));

    match key {
        k if k == play_pause => {
            player.toggle_pause();
        }
        k if k == next => {
            if let Some(next) = player.next() {
                player.play(next);
            } else {
                player.stop();
            }
        }
        k if k == like => {
            player.toggle_like();
            let _ = game.native_chat(format!("Overtune: liked {}", player.is_liked()));
        }
        _ => {}
    }

    m::set_player(player);
}
