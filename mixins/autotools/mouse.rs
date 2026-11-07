// The attack button went down or up while playing: the mod only switches
// tools during a dig, so it watches this one bit.
fn autotools_mouse(game: &mut crate::game::Game<'_>, button: i32, pressed: bool) {
    use crate::game::autotools as at;
    // 0 is the left button, the one that mines.
    if button == 0 {
        at::set_attack(pressed);
    }
    let _ = game;
}
