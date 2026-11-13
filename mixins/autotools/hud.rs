// The on/off line: vanilla's "selected item name" spot, two seconds after a
// toggle, then gone. `HudBatch` carries the screen size, so no layout maths of
// our own.
fn autotools_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    use crate::game::autotools as at;
    // Bound because the hook hands it over; the line needs only the batch.
    let _ = corners;
    let Some(on) = at::tick_notice() else {
        return;
    };
    let text = if on { "AutoTools: on" } else { "AutoTools: off" };
    let width = game.native_text_width(text);
    let x = (batch.width - width) / 2.0;
    // The height the game itself uses for the item name above the hotbar.
    let y = batch.height - 59.0;
    let color = if on { 0x55FF55 } else { 0xFF8080 };
    game.native_text(batch, text, [x, y], color, true);
}
