// AutoTools' row in Options > Mods, with its toggle key.
fn autotools_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::autotools as at;
    mods.push(crate::game::native_hooks::NativeMod {
        id: at::ID.into(),
        title: format!("AutoTools {}", at::VERSION),
        description: at::DESCRIPTION.into(),
        keys: vec![[
            at::KEY.into(),
            "Toggle key".into(),
            at::DEFAULT_KEY.into(),
        ]],
    });
}
