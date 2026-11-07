//! AutoTools: while the attack button is held, keep the fastest tool for the
//! block under the crosshair in the player's hand.
//!
//! This file is the module the bundle adds to `crates/horizon-client/src/game.rs`
//! as `crate::game::autotools`, so it sees the game's private fields and helpers
//! (`inventory`, `selected`, `world`, `res`, `aim`, `clip_ray`, `reach`).
//!
//! The mod does two things:
//!   * `native_key` toggles it, on the key the player bound in Options > Mods;
//!   * `native_tick` looks at the crosshair target once a tick and switches to
//!     the hotbar slot whose tool suits that block best.
use std::cell::Cell;

/// The mod's id: its settings folder and its row in Options > Mods.
pub const ID: &str = "autotools";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str =
    "Picks the fastest tool in your hotbar for the block you are mining. Toggle key, off by default.";
/// The toggle key's name and its default: `~`/`` ` ``.
pub const KEY: &str = "toggle";
pub const DEFAULT_KEY: &str = "key.keyboard.grave";
/// The setting that remembers whether it is on.
pub const ENABLED: &str = "enabled";

thread_local! {
    /// Cached "on" state, read from the settings once and kept until changed.
    static ON: Cell<Option<bool>> = const { Cell::new(None) };
    /// Whether the attack button is down right now.
    static ATTACK: Cell<bool> = const { Cell::new(false) };
    /// The slot this mod chose last, so it can tell its own switch from the
    /// player's.
    static CHOSEN: Cell<i32> = const { Cell::new(-1) };
}

/// What kind of tool a block wants.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Pickaxe,
    Axe,
    Shovel,
    Hoe,
    Shears,
    Sword,
}

impl Kind {
    /// The tool item kinds, by the suffix of their id.
    fn of_item(item: &str) -> Option<(Kind, u8)> {
        let (kind, tiers) = if let Some(rest) = item.strip_suffix("_pickaxe") {
            (Kind::Pickaxe, rest)
        } else if let Some(rest) = item.strip_suffix("_axe") {
            (Kind::Axe, rest)
        } else if let Some(rest) = item.strip_suffix("_shovel") {
            (Kind::Shovel, rest)
        } else if let Some(rest) = item.strip_suffix("_hoe") {
            (Kind::Hoe, rest)
        } else if let Some(rest) = item.strip_suffix("_sword") {
            (Kind::Sword, rest)
        } else if item.ends_with("shears") {
            return Some((Kind::Shears, 1));
        } else {
            return None;
        };
        Some((kind, tier_of(tiers)))
    }

    /// Whether this tool is the right one for the block, and how well.
    fn score(self, want: Option<Kind>) -> Option<u8> {
        match (want, self) {
            // Shears beat everything on what they are for.
            (Some(Kind::Shears), Kind::Shears) => Some(3),
            // A sword clears plants and cobwebs faster than anything.
            (Some(Kind::Sword), Kind::Sword) => Some(3),
            (Some(w), k) if w == k => Some(2),
            // No particular tool wanted (dirt under a pickaxe still breaks):
            // any tool is better than a bare hand.
            (None, _) => Some(1),
            _ => None,
        }
    }
}

/// The speed rank of a tool's material. Gold is faster than iron in vanilla
/// even though it is the weakest, so it ranks high on purpose.
fn tier_of(material: &str) -> u8 {
    match material.trim_start_matches("minecraft:") {
        "wooden" => 1,
        "stone" => 2,
        "iron" => 3,
        "diamond" => 4,
        "netherite" => 5,
        "golden" => 6,
        _ => 0,
    }
}

/// The tool a block wants, from its id. This is a heuristic: the game's real
/// answer lives in its block tags and item components, which this mod does not
/// read yet (see README).
pub fn block_kind(block: &str) -> Option<Kind> {
    let name = block.trim_start_matches("minecraft:");
    const AXE: [&str; 14] = [
        "log", "wood", "planks", "fence", "gate", "chest", "barrel", "bookshelf",
        "crafting_table", "ladder", "sign", "door", "trapdoor", "mushroom_block",
    ];
    const SHOVEL: [&str; 12] = [
        "dirt", "grass_block", "podzol", "mycelium", "sand", "gravel", "clay", "snow",
        "farmland", "soul_soil", "soul_sand", "mud",
    ];
    const SHEARS: [&str; 4] = ["cobweb", "wool", "vine", "leaves"];
    const SWORD: [&str; 3] = ["bamboo", "sugar_cane", "kelp"];
    if SHEARS.iter().any(|k| name.contains(k)) {
        return Some(Kind::Shears);
    }
    if SWORD.iter().any(|k| name.contains(k)) {
        return Some(Kind::Sword);
    }
    if AXE.iter().any(|k| name.contains(k)) {
        return Some(Kind::Axe);
    }
    if SHOVEL.iter().any(|k| name.contains(k)) {
        return Some(Kind::Shovel);
    }
    if name.contains("wart_block") {
        return Some(Kind::Hoe);
    }
    // Everything else solid is a pickaxe block: stone, ores, deepslate,
    // bricks, terracotta, glass, ice and the rest.
    Some(Kind::Pickaxe)
}

/// Whether the mod is on, reading the stored setting the first time.
pub fn enabled(game: &crate::game::Game<'_>) -> bool {
    if let Some(on) = ON.with(Cell::get) {
        return on;
    }
    let on = game
        .native_setting(ID, ENABLED)
        .map(|v| v == "true")
        .unwrap_or(false);
    ON.with(|c| c.set(Some(on)));
    on
}

/// Turn the mod on or off, and remember it.
pub fn set_enabled(game: &crate::game::Game<'_>, on: bool) {
    ON.with(|c| c.set(Some(on)));
    game.native_set_setting(ID, ENABLED, if on { "true" } else { "false" });
}

/// Flip the mod, returning its new state.
pub fn toggle(game: &crate::game::Game<'_>) -> bool {
    let on = !enabled(game);
    set_enabled(game, on);
    on
}

/// Whether the attack button is down, as the mouse hook saw it.
pub fn attack_held() -> bool {
    ATTACK.with(Cell::get)
}

/// Remember the attack button, from the mouse hook.
pub fn set_attack(pressed: bool) {
    ATTACK.with(|c| c.set(pressed));
}

/// The hotbar slot holding the best tool for `block`, if any tool fits or
/// beats the bare hand.
pub fn best_slot(inventory: &[Option<crate::game::ItemStack>], block: &str) -> Option<usize> {
    let want = block_kind(block);
    let mut best: Option<(u8, u8, usize)> = None; // score, tier, slot
    for (slot, stack) in inventory.iter().take(9).enumerate() {
        let Some(stack) = stack.as_ref() else { continue };
        let Some((kind, tier)) = Kind::of_item(&stack.item) else { continue };
        let Some(score) = kind.score(want) else { continue };
        let better = match best {
            None => true,
            Some((s, t, _)) => (score, tier) > (s, t),
        };
        if better {
            best = Some((score, tier, slot));
        }
    }
    best.map(|(_, _, slot)| slot)
}

/// Put the tool in `slot` in the player's hand, if it isn't already.
pub fn select(game: &mut crate::game::Game<'_>, slot: usize) {
    if game.selected == slot {
        return;
    }
    game.selected = slot;
    // The server learns of the change the way it learns of a hotbar key; the
    // field is what the game sends from on its next network tick.
    game.level.selected_slot = slot as i32;
    CHOSEN.with(|c| c.set(slot as i32));
}
