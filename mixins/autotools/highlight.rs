// The crosshair is on a block this frame: note what tool it wants. Horizon
// calls this hook point for the eligible targeted block only, so it stops
// firing the moment the player looks away.
fn autotools_highlight(
    game: &crate::game::Game<'_>,
    _pos: [i32; 3],
    state: horizon_world::registry::BlockStateId,
) {
    use crate::game::autotools as at;
    if !game.native_on(at::ID) || !game.native_playing() || !at::enabled(game) {
        return;
    }
    if let Some(name) = game.native_state_block(state) {
        at::remember(at::block_kind(name));
    }
}
