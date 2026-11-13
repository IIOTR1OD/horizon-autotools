// The toggle key went down or up: the player's choice of key, kept in
// Options > Mods (the same store the official mods use).
fn autotools_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    use crate::game::autotools as at;
    // Turned off in Options > Mods: nothing at all.
    if !game.native_on(at::ID) {
        return;
    }
    let bound = game.native_mod_key(at::ID, at::KEY, at::DEFAULT_KEY);
    if key != bound || !pressed {
        return;
    }
    // Only while playing, so typing the key into chat doesn't flip it.
    if !game.native_playing() {
        return;
    }
    let on = {
        let game = &*game;
        at::toggle(game)
    };
    let game = &mut *game;
    at::notify(on);
    // A quiet click while the line shows: higher when on, lower when off.
    if let Some(p) = game.native_player() {
        game.native_play_local_sound(
            "minecraft:ui.button.click",
            [p.x, p.y + 1.0, p.z],
            0.4,
            if on { 1.2 } else { 0.8 },
            0,
        );
    }
}
