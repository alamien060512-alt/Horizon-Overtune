// HEAD hook on `native_settings`: Overtune's settings page buttons.
//
// Buttons: volume, enabled toggle, event song assignments, import folder.

fn overtune_settings(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::overtune as m;

    if mod_id != m::ID {
        return;
    }

    let game = &*game;
    let settings = m::settings(|key| game.native_setting(m::ID, key));
    let player = m::player(|key| game.native_setting(m::ID, key));

    buttons.push(format!("Enabled: {}", if settings.enabled { "ON" } else { "OFF" }));
    buttons.push(format!("Volume: {}%", settings.volume));
    buttons.push(format!("Now playing: {}", player.current.as_ref().map_or("—".to_owned(), |s| s.title.clone())));

    for event in m::EventType::ALL {
        let assigned = settings.event_assignments.get(&event);
        buttons.push(format!("{}: {}", event.label(), assigned.map_or("none".to_owned(), |id| id.clone())));
    }

    if let Some(dir) = &settings.import_dir {
        buttons.push(format!("Import folder: {}", dir.display()));
    } else {
        buttons.push("Import folder: (none)".to_owned());
    }
}
