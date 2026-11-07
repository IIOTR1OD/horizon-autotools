// A click on AutoTools' settings page: the single button flips it.
fn autotools_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::autotools as at;
    if mod_id != at::ID || button != 0 {
        return;
    }
    let on = {
        let game = &*game;
        at::toggle(game)
    };
    let _ = on;
    game.native_settings_refresh();
}
