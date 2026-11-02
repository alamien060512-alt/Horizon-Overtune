// Once a tick: keeps vanilla's music quiet and plays what the player plans.
fn overtune_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::overtune as m;
    use horizon_client::sound_engine::{PlayResult, SoundEngine, SoundInstance, Sounds};
    if !game.native_window() || !game.native_on(m::ID) {
        return;
    }
    let enabled = m::with(|k| game.native_setting(m::ID, k), |p| p.enabled);
    if enabled {
        // Vanilla's music manager stays quiet: stop what it starts, hide its toast.
        if game.music.current_instance().is_some() {
            game.music.stop_playing(None, &mut game.audio);
        }
        game.music.toast_requests.clear();
    }
    let old = m::with(|k| game.native_setting(m::ID, k), |p| p.handle);
    let active = old.is_some_and(|h| game.audio.is_active(h));
    let (plan, gain) = m::with(|k| game.native_setting(m::ID, k), |p| (p.plan(active), p.take_gain()));
    if let Some(h) = old
        && (plan.stop || !active)
    {
        if active {
            game.audio.stop_instance(h);
        }
        game.audio.release(h);
    }
    if let Some(gain) = gain {
        game.audio.update_category_volume(horizon_client::sound_engine::MUSIC, gain);
    }
    if let Some(i) = plan.play {
        let instance = SoundInstance::for_music(m::CATALOG[i].event, SoundEngine::unseeded());
        let (id, result) = game.audio.play_instance(instance, true);
        let started = !matches!(result, PlayResult::NotStarted);
        if !started {
            game.audio.release(id);
        }
        m::with(|k| game.native_setting(m::ID, k), |p| p.started(started.then_some(id)));
    }
}
