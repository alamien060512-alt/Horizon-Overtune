// HEAD hook on `native_mods`: register Overtune in Options > Mods.

fn overtune_mods(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::overtune as m;

    mods.push(crate::game::native_hooks::NativeMod {
        id: m::ID.to_owned(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.to_owned(),
        keys: vec![
            ["play_pause".to_owned(), "Play / Pause".to_owned(), "key.keyboard.p".to_owned()],
            ["next".to_owned(), "Next track".to_owned(), "key.keyboard.n".to_owned()],
            ["like".to_owned(), "Like song".to_owned(), "key.keyboard.l".to_owned()],
        ],
    });
}
