// The mod's row in Options > Mods, with its keys.
fn overtune_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::overtune as m;
    mods.push(crate::game::native_hooks::NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: m::keys(),
    });
}
