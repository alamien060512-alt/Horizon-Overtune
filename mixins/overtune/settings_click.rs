// HEAD hook on `native_settings_click`: handle clicks on Overtune's
// settings page buttons.
//
// Button layout (set by settings.rs):
//   0 = Enabled toggle
//   1 = Volume
//   2 = Now playing (display only, no-op)
//   3..=8 = Event song assignments (one per EventType)

fn overtune_settings_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::overtune as m;

    if mod_id != m::ID {
        return;
    }

    let mut settings = m::settings(|key| game.native_setting(m::ID, key));

    match button {
        0 => {
            settings.enabled = !settings.enabled;
        }
        1 => {
            settings.volume = settings.volume.saturating_add(10).clamp(0, 100);
        }
        2 => {
            // Now playing display: nothing to change.
        }
        3..=8 => {
            let event_idx = button - 3;
            if event_idx < m::EventType::ALL.len() {
                let event = m::EventType::ALL[event_idx];
                let lib = m::vanilla_library();
                let current = settings.event_assignments.get(&event);
                // Cycle: none -> first song -> second song -> ... -> none
                let next_id = current.and_then(|id| {
                    lib.iter().position(|s| s.id == *id).and_then(|i| lib.get(i + 1).map(|s| s.id.clone()))
                }).or_else(|| lib.first().map(|s| s.id.clone()));
                settings.event_assignments.insert(event, next_id.unwrap_or_default());
            }
        }
        _ => {}
    }

    m::set_settings(settings);
    for (key, value) in settings.stored() {
        game.native_set_setting(m::ID, key, &value);
    }
}
