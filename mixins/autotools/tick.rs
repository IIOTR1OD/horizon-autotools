// Once a tick: while mining, put the best tool for the block under the
// crosshair in the player's hand.
fn autotools_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::autotools as at;
    if !game.native_on(at::ID) || !game.native_playing() {
        return;
    }
    if !at::enabled(game) || !at::attack_held() {
        return;
    }
    // Where the eyes point and how far a block can be picked, as the game's
    // own crosshair does it.
    let Some((origin, direction)) = game.aim() else {
        return;
    };
    let reach = game.reach();
    let Some(hit) = game.clip_ray(origin, direction, reach) else {
        return;
    };
    let state = game.world.block(hit.pos[0], hit.pos[1], hit.pos[2]);
    let facts = game.res.facts().get(state);
    // Unbreakable blocks (hardness -1) are not worth a switch.
    if facts.hardness < 0.0 {
        return;
    }
    let block = facts.name.clone();
    let Some(slot) = at::best_slot(&game.inventory, &block) else {
        return;
    };
    at::select(game, slot);
}
