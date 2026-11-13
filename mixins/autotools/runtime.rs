// AutoTools 1.1: keep the fastest tool for the block under the crosshair in the
// player's hand while mining.
//
// Not `//!`: horizon-mixins pastes a support module's source into the game and
// refuses file attributes (an inner doc comment is one), so the header is a
// plain comment.
//
// The target block now comes from Horizon's own hook point
// (`native_block_highlight`, API 86) instead of the mod calling the game's
// private raycast (`aim`, `clip_ray`, `world.block`, `res.facts`). What is left
// of the game's internals is one field: `Game::selected`, which has no stable
// API yet.
use std::cell::Cell;

use crate::game::autotools_table as table;

/// The mod's id: its settings folder and its row in Options > Mods.
pub const ID: &str = "autotools";
pub const VERSION: &str = "1.2.2";
pub const DESCRIPTION: &str =
    "Picks the fastest tool in your hotbar for the block you are mining. Toggle key, off by default.";
/// The toggle key's name and its default: `~`/`` ` ``.
pub const KEY: &str = "toggle";
pub const DEFAULT_KEY: &str = "key.keyboard.grave.accent";
/// The setting that remembers whether it is on.
pub const ENABLED: &str = "enabled";

thread_local! {
    /// Cached "on" state, read from the settings once and kept until changed.
    static ON: Cell<Option<bool>> = const { Cell::new(None) };
    /// Whether the attack button is down right now.
    static ATTACK: Cell<bool> = const { Cell::new(false) };
    /// What kind of tool the highlighted block wants, and the tick it was seen
    /// on. The highlight hook runs every frame, so a stale entry means the
    /// crosshair left the block.
    static WANT: Cell<Option<Kind>> = const { Cell::new(None) };
    static WANT_TICK: Cell<u32> = const { Cell::new(0) };
    /// The tick counter, bumped by the tick hook.
    static TICK: Cell<u32> = const { Cell::new(0) };
    /// How many ticks the "on/off" line stays on screen, and what it says.
    static NOTICE: Cell<u32> = const { Cell::new(0) };
    static NOTICE_ON: Cell<bool> = const { Cell::new(false) };
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
        let (kind, material) = if let Some(rest) = item.strip_suffix("_pickaxe") {
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
        Some((kind, tier_of(material)))
    }

    /// Whether this tool fits the block, and how well.
    fn score(self, want: Kind) -> Option<u8> {
        match (want, self) {
            // Shears beat everything on what they are for.
            (Kind::Shears, Kind::Shears) => Some(3),
            // A sword clears plants and cobwebs faster than anything.
            (Kind::Sword, Kind::Sword) => Some(3),
            (w, k) if w == k => Some(2),
            // A wrong tool still swings; the game just mines slower.
            _ => Some(1),
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

/// The tool a block wants, from its id. A heuristic: the game's real answer is
/// in its block tags and item components, which have no native API yet.
pub fn block_kind(block: &str) -> Kind {
    let name = block.trim_start_matches("minecraft:");
    // The game's own answer first: the tags name every block it has, so this is
    // exact for vanilla and for anything a data pack tags.
    if let Some(kind) = table::tagged(name) {
        return kind;
    }
    // Below is only for blocks no tag names: a modded block, or one the game
    // lets you break with anything.
    // Tool-specific blocks, whatever they are made of.
    const SHEARS: &[&str] = &["cobweb", "wool", "vine", "leaves"];
    const SWORD: &[&str] = &["bamboo", "sugar_cane", "kelp"];
    // Blocks that need a pickaxe even though their id carries a shovel, axe or
    // wood word: sandstone is stone, not sand; mud_bricks are bricks, not mud;
    // an iron or copper door is not a wooden one.
    const PICKAXE: &[&str] = &[
        "stone", "ore", "deepslate", "brick", "terracotta", "glass", "ice", "obsidian",
        "netherrack", "basalt", "tuff", "calcite", "amethyst", "copper", "quartz",
        "purpur", "prismarine", "concrete", "magma", "ender", "petrified", "iron",
        "blackstone",
    ];
    // The wood itself: a slab, a stair or a button is wood when its material
    // is, and the game wants an axe for it. Stone and metal ones are caught by
    // the pickaxe list first; the rest fall through to the pickaxe default.
    const WOOD: &[&str] = &[
        "oak", "spruce", "birch", "jungle", "acacia", "mangrove", "cherry", "pale_oak",
        "bamboo", "crimson", "warped", "wooden",
    ];
    // Wooden furniture whose id says nothing about its material.
    const FURNITURE: &[&str] = &[
        "bookshelf", "table", "loom", "note_block", "jukebox", "composter", "lectern",
        "beehive", "bee_nest", "chest", "barrel", "ladder", "sign", "door", "trapdoor",
        "fence", "gate", "planks", "log", "wood",
    ];
    const SHOVEL: &[&str] = &[
        "dirt", "grass_block", "podzol", "mycelium", "sand", "gravel", "clay", "snow",
        "farmland", "soul_soil", "soul_sand", "mud",
    ];
    if SHEARS.iter().any(|k| name.contains(k)) {
        return Kind::Shears;
    }
    if SWORD.iter().any(|k| name.contains(k)) {
        return Kind::Sword;
    }
    if PICKAXE.iter().any(|k| name.contains(k)) {
        return Kind::Pickaxe;
    }
    if WOOD.iter().any(|k| name.contains(k)) {
        return Kind::Axe;
    }
    if FURNITURE.iter().any(|k| name.contains(k)) {
        return Kind::Axe;
    }
    if SHOVEL.iter().any(|k| name.contains(k)) {
        return Kind::Shovel;
    }
    if name.contains("wart_block") {
        return Kind::Hoe;
    }
    // Everything else solid is a pickaxe block: ores, bricks, terracotta,
    // glass, ice and the rest.
    Kind::Pickaxe
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

/// Remember what the highlighted block wants, from the highlight hook.
pub fn remember(kind: Kind) {
    WANT.with(|c| c.set(Some(kind)));
    WANT_TICK.with(|c| c.set(TICK.with(Cell::get)));
}

/// What the crosshair was on, if it was still there a moment ago: the
/// highlight hook stops firing when the player looks at the sky.
pub fn wanted() -> Option<Kind> {
    let kind = WANT.with(Cell::get)?;
    let age = TICK.with(Cell::get).wrapping_sub(WANT_TICK.with(Cell::get));
    // Ten ticks is plenty for a frame hook that runs sixty times a second.
    (age <= 10).then_some(kind)
}

/// Bump the tick counter, from the tick hook.
pub fn tick() {
    TICK.with(|c| c.set(c.get().wrapping_add(1)));
}

/// The hotbar slot holding the best tool for `kind`, if any.
pub fn best_slot(inventory: &[Option<crate::game::ItemStack>], kind: Kind) -> Option<usize> {
    let mut best: Option<(u8, u8, usize)> = None; // score, tier, slot
    for (slot, stack) in inventory.iter().take(9).enumerate() {
        let Some(stack) = stack.as_ref() else { continue };
        let Some((tool, tier)) = Kind::of_item(&stack.item) else { continue };
        let Some(score) = tool.score(kind) else { continue };
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
///
/// The only place the mod touches the game's internals: Horizon has no API to
/// select a hotbar slot (the server-side `native_player_item_held` can only
/// hear or cancel a change), so this writes the field the game's own hotbar
/// keys write.
pub fn select(game: &mut crate::game::Game<'_>, slot: usize) {
    if game.selected == slot {
        return;
    }
    game.selected = slot;
    game.level.selected_slot = slot as i32;
}

/// Show the on/off line for two seconds after a toggle.
pub fn notify(on: bool) {
    NOTICE.with(|c| c.set(40));
    NOTICE_ON.with(|c| c.set(on));
}

/// Remember `on` for the line the HUD draws, and count its ticks down.
pub fn tick_notice() -> Option<bool> {
    let left = NOTICE.with(Cell::get);
    if left == 0 {
        return None;
    }
    NOTICE.with(|c| c.set(left - 1));
    Some(NOTICE_ON.with(Cell::get))
}
