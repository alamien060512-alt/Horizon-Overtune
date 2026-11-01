// HEAD hook on `native_hud`: draw the now-playing overlay in a corner.

fn overtune_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    use crate::game::overtune as m;
    use horizon_client::extras::Corner;

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

    let player = m::player(|key| game.native_setting(m::ID, key));
    let Some(song) = player.current.as_ref() else {
        return;
    };

    let corner = [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomLeft,
        Corner::BottomRight,
    ][0]; // fixed corner for now-playing; settings could choose

    if !game.native_corner_shown(corner) {
        return;
    }

    let status = if player.paused { "⏸" } else { "▶" };
    let text = format!(
        "{} {} — {}  [{}:{}]",
        status,
        song.title,
        song.artist,
        m::format_position(player.position),
        m::format_duration(song.duration)
    );
    let w = game.native_text_width(&text) + 8.0;
    let (x, y) = corners.place(corner, w, 16.0);
    game.native_fill(batch, [x, y, w, 16.0], m::BACKGROUND);
    game.native_text(batch, &text, [x + 4.0, y + 2.0], m::INK, true);
}
