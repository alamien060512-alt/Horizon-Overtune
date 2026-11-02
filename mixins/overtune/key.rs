// Overtune's keys: play/pause, skip, shuffle, like and volume.
fn overtune_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    use crate::game::overtune as m;
    if !pressed || !game.native_playing() || !game.native_on(m::ID) {
        return;
    }
    let Some([name, _, _]) = m::keys()
        .into_iter()
        .find(|[name, _, default]| game.native_mod_key(m::ID, name, default) == key)
    else {
        return;
    };
    let stored = m::with(|k| game.native_setting(m::ID, k), |p| {
        m::press(p, &name);
        p.stored()
    });
    for (key, value) in stored {
        game.native_set_setting(m::ID, key, &value);
    }
}
