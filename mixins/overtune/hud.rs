// The now-playing toast, in the bottom right corner.
fn overtune_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    use crate::game::overtune as m;
    if !game.native_on(m::ID) {
        return;
    }
    let corner = horizon_client::extras::Corner::BottomRight;
    if !game.native_corner_shown(corner) {
        return;
    }
    let game = &*game;
    let lines = m::with(|key| game.native_setting(m::ID, key), |p| {
        (p.enabled && p.toast_left > 0).then(|| m::toast_lines(p))
    });
    let Some(lines) = lines else {
        return;
    };
    let size = m::toast_size(&|text| game.native_text_width(text), &lines);
    let (x, y) = corners.place(corner, size.0, size.1);
    for shape in m::toast_shapes(&lines, size) {
        match shape {
            m::Shape::Fill([dx, dy, w, h], argb) => game.native_fill(batch, [x + dx, y + dy, w, h], argb),
            m::Shape::Text(text, [dx, dy], rgb, shadow) => {
                game.native_text(batch, &text, [x + dx, y + dy], rgb, shadow)
            }
        }
    }
}
