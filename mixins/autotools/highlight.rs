// The crosshair is on a block this frame: note what tool it wants. Horizon
// calls this hook point for the eligible targeted block only, so it stops
// firing the moment the player looks away.
//
// Every parameter is named as its binding is: horizon-mixins matches them by
// name and type, in declaration order.
fn autotools_highlight(
    game: &crate::game::Game<'_>,
    pos: [i32; 3],
    state: horizon_world::registry::BlockStateId,
    partial: f32,
    canceled: &mut bool,
) {
    use crate::game::autotools as at;
    // Only the block matters here; the rest is bound because the hook hands it
    // over and horizon-mixins wants the parameters in order.
    let _ = (pos, partial, canceled);
    if !game.native_on(at::ID) || !game.native_playing() || !at::enabled(game) {
        return;
    }
    if let Some(name) = game.native_state_block(state) {
        at::remember(at::block_kind(name));
    }
}
