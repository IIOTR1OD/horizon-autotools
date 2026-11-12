// Once a tick: while mining, put the best tool for the block under the
// crosshair in the player's hand.
fn autotools_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::autotools as at;
    at::tick();
    if !game.native_on(at::ID) || !game.native_playing() {
        return;
    }
    if !at::enabled(game) || !at::attack_held() {
        return;
    }
    let Some(kind) = at::wanted() else {
        return;
    };
    let inventory = &game.inventory;
    let Some(slot) = at::best_slot(inventory, kind) else {
        return;
    };
    at::select(game, slot);
}
