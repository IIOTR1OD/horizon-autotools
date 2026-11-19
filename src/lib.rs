//! AutoTools: keep the fastest tool in your hotbar for the block you are mining.
//!
//! The block comes from `block_highlight` (the one the crosshair is on), the held attack button
//! from the vanilla `key.attack` mapping, and the slot is switched with `select_hotbar_slot`.
//! The tool table is generated from the game's own block tags (`src/table.rs`).

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};
use std::sync::Mutex;

use mod_api::{
    self, BlockHighlightEvent, ClientHost, ClientTickPreEvent, Flow, GuiRenderPostEvent, Hooks,
    InputKeyEvent, KeyAction,
};

mod table;

const ID: &str = "autotools";
const VERSION: &str = "1.3.0";
const DESCRIPTION: &str =
    "Picks the fastest tool in your hotbar for the block you are mining. Toggle key, off by default.";

/// The keys the Config page offers for the toggle, with their translation keys.
const TOGGLE_KEYS: [(&str, &str); 14] = [
    ("key.keyboard.grave.accent", "autotools.key.grave"),
    ("key.keyboard.period", "autotools.key.period"),
    ("key.keyboard.b", "autotools.key.b"),
    ("key.keyboard.n", "autotools.key.n"),
    ("key.keyboard.g", "autotools.key.g"),
    ("key.keyboard.h", "autotools.key.h"),
    ("key.keyboard.j", "autotools.key.j"),
    ("key.keyboard.k", "autotools.key.k"),
    ("key.keyboard.f6", "autotools.key.f6"),
    ("key.keyboard.f7", "autotools.key.f7"),
    ("key.keyboard.f8", "autotools.key.f8"),
    ("key.keyboard.f9", "autotools.key.f9"),
    ("key.keyboard.backslash", "autotools.key.backslash"),
    ("key.keyboard.semicolon", "autotools.key.semicolon"),
];

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

    fn score(self, want: Kind) -> Option<u8> {
        match (want, self) {
            (Kind::Shears, Kind::Shears) => Some(3),
            (Kind::Sword, Kind::Sword) => Some(3),
            (w, k) if w == k => Some(2),
            _ => Some(1),
        }
    }

    fn code(self) -> u8 {
        match self {
            Kind::Pickaxe => 1,
            Kind::Axe => 2,
            Kind::Shovel => 3,
            Kind::Hoe => 4,
            Kind::Shears => 5,
            Kind::Sword => 6,
        }
    }

    fn from_code(code: u8) -> Option<Kind> {
        Some(match code {
            1 => Kind::Pickaxe,
            2 => Kind::Axe,
            3 => Kind::Shovel,
            4 => Kind::Hoe,
            5 => Kind::Shears,
            6 => Kind::Sword,
            _ => return None,
        })
    }
}

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

/// The tool a block wants: the game's tags first, then the same heuristics as before.
pub fn block_kind(block: &str) -> Kind {
    let name = block.trim_start_matches("minecraft:");
    if let Some(kind) = table::tagged(name) {
        return kind;
    }
    const SHEARS: &[&str] = &["cobweb", "wool", "vine", "leaves"];
    const SWORD: &[&str] = &["bamboo", "sugar_cane", "kelp"];
    const PICKAXE: &[&str] = &[
        "stone", "ore", "deepslate", "brick", "terracotta", "glass", "ice", "obsidian",
        "netherrack", "basalt", "tuff", "calcite", "amethyst", "copper", "quartz", "purpur",
        "prismarine", "concrete", "magma", "ender", "petrified", "iron", "blackstone",
    ];
    const WOOD: &[&str] = &[
        "oak", "spruce", "birch", "jungle", "acacia", "mangrove", "cherry", "pale_oak", "bamboo",
        "crimson", "warped", "wooden",
    ];
    const FURNITURE: &[&str] = &[
        "bookshelf", "table", "loom", "note_block", "jukebox", "composter", "lectern", "beehive",
        "bee_nest", "chest", "barrel", "ladder", "sign", "door", "trapdoor", "fence", "gate",
        "planks", "log", "wood",
    ];
    const SHOVEL: &[&str] = &[
        "dirt", "grass_block", "podzol", "mycelium", "sand", "gravel", "clay", "snow", "farmland",
        "soul_soil", "soul_sand", "mud",
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
    if WOOD.iter().any(|k| name.contains(k)) || FURNITURE.iter().any(|k| name.contains(k)) {
        return Kind::Axe;
    }
    if SHOVEL.iter().any(|k| name.contains(k)) {
        return Kind::Shovel;
    }
    if name.contains("wart_block") {
        return Kind::Hoe;
    }
    Kind::Pickaxe
}

/// The tool kind the crosshair last saw, the tick it saw it, and the notice.
static WANT: AtomicU8 = AtomicU8::new(0);
static WANT_TICK: AtomicU32 = AtomicU32::new(0);
static TICK: AtomicU32 = AtomicU32::new(0);
static NOTICE: AtomicU32 = AtomicU32::new(0);
static NOTICE_ON: AtomicBool = AtomicBool::new(false);

/// The hotbar's item ids, put here by the `hotbar` mixin every tick (empty: an empty slot).
static HOTBAR: Mutex<[String; 9]> = Mutex::new([
    String::new(), String::new(), String::new(), String::new(), String::new(),
    String::new(), String::new(), String::new(), String::new(),
]);

/// Called by the mixin, `src/mixins/hotbar.rs`, once a tick.
pub fn set_hotbar(ids: [String; 9]) {
    *HOTBAR.lock().unwrap_or_else(|e| e.into_inner()) = ids;
}

fn enabled(host: &dyn ClientHost) -> bool {
    host.setting_bool("enabled").unwrap_or(false)
}

fn toggle_key(host: &dyn ClientHost) -> &'static str {
    let index = host.setting_choice("toggle_key").unwrap_or(0) as usize;
    TOGGLE_KEYS
        .get(index)
        .map_or("key.keyboard.grave.accent", |(key, _)| key)
}

fn notify(on: bool) {
    NOTICE.store(40, Ordering::Relaxed);
    NOTICE_ON.store(on, Ordering::Relaxed);
}

fn wanted() -> Option<Kind> {
    let code = WANT.load(Ordering::Relaxed);
    if code == 0 {
        return None;
    }
    let age = TICK.load(Ordering::Relaxed).wrapping_sub(WANT_TICK.load(Ordering::Relaxed));
    (age <= 10).then(|| Kind::from_code(code)).flatten()
}

#[derive(Default)]
struct AutoTools;

#[mod_api::horizon_mod]
impl Hooks for AutoTools {
    /// The mod's page in Options > Mods: whether it is on, and its toggle key.
    fn register(
        &self,
        registrar: &mut dyn mod_api::Registrar,
        _context: &mod_api::RegistrationContext,
    ) {
        let _ = registrar.register_setting(mod_api::SettingSpec::toggle(
            "enabled",
            "autotools.setting.enabled",
            false,
        ));
        let _ = registrar.register_setting(mod_api::SettingSpec::choice(
            "toggle_key",
            "autotools.setting.toggle_key",
            TOGGLE_KEYS.iter().map(|(_, label)| *label),
            0,
        ));
    }

    /// The toggle key while playing.
    fn input_key(&self, host: &mut dyn ClientHost, event: &mut InputKeyEvent<'_>) {
        if event.action != KeyAction::Press || !host.playing() || event.key != toggle_key(host) {
            return;
        }
        let on = !enabled(host);
        host.set_setting(ID, "enabled", if on { "true" } else { "false" });
        notify(on);

    }

    /// The block under the crosshair: remember the tool it wants.
    fn block_highlight(&self, host: &dyn ClientHost, event: &mut BlockHighlightEvent) -> Flow {
        if !host.playing() || !enabled(host) {
            return Flow::Continue;
        }
        if let Some(name) = host.block_state_string(event.state) {
            WANT.store(block_kind(&name).code(), Ordering::Relaxed);
            WANT_TICK.store(TICK.load(Ordering::Relaxed), Ordering::Relaxed);
        }
        Flow::Continue
    }

    /// The server half: the player's inventory slots arrive here (single player and LAN host),
    /// because the client API has no per-slot inventory. The hotbar is 0..9.
    fn player_inventory_changed(
        &self,
        host: &mut dyn mod_api::ServerHost,
        event: &mut mod_api::PlayerInventoryChangedEvent<'_>,
    ) {
        let mut ids: [String; 9] = std::array::from_fn(|_| String::new());
        for (slot, stack) in event.stacks.iter().take(9).enumerate() {
            if let Some(stack) = stack {
                ids[slot] = stack.item().id(host);
            }
        }
        // A snapshot with an empty hotbar says nothing the mod can use: keep the last one.
        if ids.iter().any(|id| !id.is_empty()) {
            set_hotbar(ids);
        }
    }

    /// Once a tick: while the attack button is held, put the best tool in hand.
    fn client_tick_pre(&self, host: &mut dyn ClientHost, _event: &mut ClientTickPreEvent) {
        TICK.fetch_add(1, Ordering::Relaxed);
        if !host.playing() || !enabled(host) {
            return;
        }
        let attack = host
            .mapping("key.attack")
            .ok()
            .flatten()
            .is_some_and(|key| key.down);
        if !attack {
            return;
        }
        let Some(kind) = wanted() else {
            return;
        };
        let inventory = HOTBAR.lock().unwrap_or_else(|e| e.into_inner()).clone();
        let _ = host;
        let mut best: Option<(u8, u8, u8)> = None;
        for (slot, id) in inventory.iter().enumerate() {
            if id.is_empty() {
                continue;
            }
            let Some((tool, tier)) = Kind::of_item(id) else {
                continue;
            };
            let Some(score) = tool.score(kind) else {
                continue;
            };
            let better = match best {
                None => true,
                Some((s, t, _)) => (score, tier) > (s, t),
            };
            if better {
                best = Some((score, tier, slot as u8));
            }
        }
        if let Some((_, _, slot)) = best {
            let _ = host.select_hotbar_slot(slot);
        }
    }

    /// The on/off line above the hotbar for two seconds after a toggle.
    fn gui_render_post(&self, _host: &dyn ClientHost, event: &mut GuiRenderPostEvent<'_>) {
        let left = NOTICE.load(Ordering::Relaxed);
        if left == 0 {
            return;
        }
        NOTICE.store(left - 1, Ordering::Relaxed);
        let on = NOTICE_ON.load(Ordering::Relaxed);
        let text = if on { "AutoTools: on" } else { "AutoTools: off" };
        let [width, height] = event.batch.size();
        let x = (width - event.batch.text_width(text)) / 2.0;
        let color = if on { 0xFF55FF55 } else { 0xFFFF8080 };
        event.batch.text(text, [x, height - 59.0], color, true);
    }
}

/// The mod's version, for the launcher's list.
pub fn version() -> &'static str {
    VERSION
}

/// The mod's description, for the launcher's list.
pub fn description() -> &'static str {
    DESCRIPTION
}
