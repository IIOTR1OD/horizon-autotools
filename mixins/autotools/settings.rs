// AutoTools' settings page: one button that says whether it is on.
fn autotools_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::autotools as at;
    if mod_id != at::ID {
        return;
    }
    let game = &*game;
    let state = if at::enabled(game) { "on" } else { "off" };
    buttons.push(format!("AutoTools: {state}"));
}
