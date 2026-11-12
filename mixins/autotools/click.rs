// A click on AutoTools' settings page: the single button flips it.
//
// No refresh call: at this Horizon, `native_settings_refresh` takes the index
// of the installed mod, which this hook is not given, and refreshing someone
// else's page would redraw the wrong text. The button's label catches up the
// next time the page is opened.
fn autotools_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::autotools as at;
    if mod_id != at::ID || button != 0 {
        return;
    }
    let game = &*game;
    at::toggle(game);
}
