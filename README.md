# AutoTools — native mod for Horizon

Keeps the fastest tool for the block under the crosshair in your hand while you
mine, and switches back as soon as you look at something else. One key toggles
it; the key is rebindable in **Options → Mods**.

## What it does

- While the left (attack) button is held and the crosshair is on a block, the
  mod scores every hotbar slot against that block and moves the selection to the
  best one.
- It only fires while mining: walking around never changes your slot.
- It never moves anything out of the hotbar and never touches the inventory.
- It starts **off** and remembers the state in the mod's own settings
  (`native_setting`/`native_set_setting`, the same store the official mods use).
- Default toggle key: `` ` `` (`key.keyboard.grave`).

## Layout

```
mixins/autotools/
  horizon-mixin.toml      manifest: module + six injections
  horizon-native.json     id, version, the Horizon commit it was written on
  runtime.rs              state, tool scoring, slot selection
  key.rs                  native_key  → toggle
  mouse.rs                native_mouse_button_post → attack held
  tick.rs                 native_tick → pick the tool for the crosshair block
  list.rs                 native_mods → the Options > Mods row and key
  settings.rs             native_settings → the page's one button
  click.rs                native_settings_click → flip it
```

## How it works

`native_tick` (20 times a second, on the client) does what the game's own
crosshair does:

1. `game.aim()` — the eye position and the view vector;
2. `game.clip_ray(origin, direction, game.reach())` — the first block hit;
3. `game.world.block(x, y, z)` → the block state,
   `game.res.facts().get(state)` → its id and hardness;
4. `best_slot()` scores the nine hotbar stacks and `select()` writes
   `game.selected` (and `game.level.selected_slot`, the field the game syncs
   from) when the choice changes.

Scoring is: the right tool kind for the block wins (shears > sword > matching
kind), ties are broken by material rank (netherite > diamond > iron > stone >
wood; gold ranks above netherite because vanilla mines faster with it). Anything
is better than a bare hand, so a shovel still gets picked for dirt even when the
block wanted a pickaxe.

## v1.1: what changed

The first version reached into the game's private raycast: `aim()`, `clip_ray()`,
`world.block()`, `res.facts()`. Horizon's API grew since, so v1.1 takes the block
from the game's own hook point instead:

| need | v1.0 | v1.1 |
|---|---|---|
| block under the crosshair | private raycast | `native_block_highlight(pos, state, …)` (API 86) |
| block name | `res.facts()` | `native_state_block(state)` (API 164) |
| attack held | mouse hook | mouse hook (unchanged) |
| hotbar contents | `Game::inventory` | `Game::inventory` (no client API yet) |
| select a slot | `Game::selected` | `Game::selected` (no API yet) |

The raycast and the block facts are gone; one field write remains, and the mod
is still pinned to the commit it builds on. When Horizon gains a
`select_slot`-style helper, the pin can go.

## Honest limitations

- **Not compiled yet.** The user asked to defer the build (weak laptop), so this
  is source only: it was written against commit `d17e8fa` but no `native-build`
  has run. Expect one or two compile fixes on the first build.
- **Block classification is a heuristic.** `block_kind()` guesses from the
  block id (`_log`, `_ore`, `dirt`, `leaves`, …) instead of reading the game's
  real `mineable/*` block tags and the items' `tool` component. It gets vanilla
  blocks right almost always, and modded blocks fall back to "pickaxe".
- **It cannot know a bare hand is better.** The game's real answer is
  `Item.getDestroySpeed(state)`; this mod approximates it. For a few blocks
  (obsidian with a wooden pickaxe) the "best" tool may still be too weak — the
  game then just mines slowly, nothing breaks.
- **Slot changes mid-dig.** Switching tools part-way through a dig resets the
  break progress, exactly as pressing a hotbar key would. That is what makes it
  feel instant rather than magic.
- **Server rules.** It only touches your own hotbar, so it is client-side; a
  server with a strict "no automation" rule may still frown on it.

## Building (when you decide to)

```sh
# from the folder that holds mixins/autotools
horizon-launcher native-build ./mods/autotools --profile dev --trust-native-code
```

`--profile dev` rebuilds the mod quickest and runs the game slowest — the right
trade for the first compile. Once it builds, `--profile performance` is the
default for playing.

## Sharing it with someone else

`python3 package.py` writes `target/autotools-1.0.0.hnmod`: a zip of
`horizon-native.json` plus `mixins/autotools/*.rs` and `*.toml`. It holds **source
only** — no compiled code — because a native mod is compiled into the game on the
machine that plays it. Send that one file.

Their side:

```sh
# 1. unpack it somewhere the launcher can read
unzip autotools-1.0.0.hnmod -d autotools

# 2. build it into their instance (their own Rust comes with the launcher)
horizon-launcher native-build ./autotools --trust-native-code

# 3. play as usual: the launcher selects the new build for that instance
horizon-launcher play
```

`native-build` takes an unpacked `.hnmod` as its folder, so unpacking first is
the whole install. The launcher fetches its own portable Rust, so they need no
`cargo` on their system.

If their Horizon sits on another commit, the launcher refuses to mix it with
mods pinned elsewhere and offers **Build anyway** (`--allow-version-mismatch`).
That often works; when it does not, the error says the mods were made for other
versions. Re-pinning `horizon_revision` in `horizon-mixin.toml` and
`horizon-native.json` to their commit is the clean fix.

