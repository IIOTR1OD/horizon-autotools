# AutoTools

**A native mod for [Horizon](https://playhorizon.dev) that keeps the right tool in your hand.**

Mining stone with a sword, chopping a tree with a pickaxe, swinging at dirt with an axe —
AutoTools watches what your crosshair is on and switches your hotbar to the fastest tool you
carry for it. One key turns it off when you want to choose yourself.

[![latest release](https://img.shields.io/github/v/release/IIOTR1OD/horizon-autotools?label=release)](https://github.com/IIOTR1OD/horizon-autotools/releases/latest)
[![license](https://img.shields.io/github/license/IIOTR1OD/horizon-autotools)](LICENSE)

> **This is vibe-coded, for personal use.** It was written by an AI agent in a few long
> sessions, tested on exactly two machines, and it never went through a review. It works — it
> is running right now — but treat it as a well-meaning script, not as a product with a
> support team. Read the source before you trust it with your game.

---

## What it does

- **Picks the right tool, every block.** While you hold the attack button, AutoTools scores
  the nine hotbar slots against the block under your crosshair and switches to the best one.
- **Never touches anything else.** No inventory edits, no auto-eating, no ESP. It changes one
  thing: which hotbar slot is selected.
- **Off by default.** `AutoTools: off` on the first run. Toggle with `` ` `` (backtick) — rebindable
  in **Options → Mods**.
- **Tells you what it did.** A short `AutoTools: on` / `AutoTools: off` line appears above the
  hotbar for two seconds, with a quiet click pitched differently for each state.
- **Remembers.** The on/off state lives in the mod's own settings, so it survives a restart.

## Install

### From GitHub (recommended)

```sh
horizon-launcher mods add-github https://github.com/IIOTR1OD/horizon-autotools
```

Horizon resolves the latest release, checks the `.hmod`, compiles it against the build you play
and selects the new build. Update later with `horizon-launcher mods update-github`.

### From the `.hmod` by hand

```sh
cp autotools-1.3.0.hmod ~/.local/share/horizon/horizon-native-mods/
printf 'mod.autotools:true\n' >> ~/.local/share/horizon/horizon-mods.txt
horizon-launcher native-build --trust-native-code
```

The launcher brings its own Rust toolchain, so you do not need `cargo` on your machine.

### From a checkout

```sh
git clone https://github.com/IIOTR1OD/horizon-autotools
cd horizon-autotools
horizon-launcher native-build . --trust-native-code
```

A native build compiles only the mods against the prebuilt game: seconds, not minutes. The game
is never rebuilt.

## Use

| | |
|---|---|
| Toggle | `` ` `` — or whatever you pick in **Options → Mods → AutoTools → Config** |
| Where | While mining: hold attack, the slot switches by itself |
| What it picks | The matching tool with the best material: netherite > diamond > iron > stone > wood, gold above netherite because vanilla mines faster with it |
| What it never does | Move items, use items, hit things for you, read the server |

Nothing happens while you are not attacking, and the moment your crosshair leaves a block the
mod stops caring: a block seen more than half a second ago is not acted on.

### Known limitation (1.3.0)

Reading the nine hotbar slots needs data the client API does not expose. AutoTools takes it from
the server side of the same process, so it works in single player and when you host a LAN world.
On a remote server the server does not have the mod, so it cannot see your hotbar and does not
switch anything. The client-only mixin that would fix this fails to compile in launcher 0.2.81
(`E0603` in the generated patch crate); it comes back when that is fixed.

## How it works

AutoTools is a **native mod**: a Rust crate the launcher compiles against the prebuilt game —
never a `.jar`, and the game itself is not rebuilt. It answers five mod-API hooks:

| hook | what it does |
|---|---|
| `block_highlight` | every frame the crosshair is on a block: remember what tool it wants |
| `client_tick_pre` | once a tick while attacking: pick the slot and select it |
| `input_key` | the toggle key |
| `gui_render_post` | draw the two-second `on/off` line above the hotbar |
| `register` | the mod page in **Options → Mods**: `AutoTools on` and the toggle key |

The block's name comes from `block_state_string(state)`, and the hotbar slot is chosen with
`select_hotbar_slot` — no game internals are written, and the mod needs no mixin.

## Compatibility

`horizon-mod.toml` pins one exact Horizon build (`dd67b76…`, Nightly 2026-11-18). The launcher
refuses a build made for another revision; the way forward is to bump `[native] horizon`,
`mixins` and the SDK in `.horizon/sdk/` to the build you play (that is what
`cargo horizon-mod update` does), then rebuild.

## Known limitations

- **Tool choice is a heuristic.** Horizon exposes no way to ask "how fast does this item break
  this block", so AutoTools classifies blocks by id (`_log` → axe, `_ore` → pickaxe, `dirt` →
  shovel, …) and tools by their suffix. Vanilla blocks are almost always right; modded blocks
  fall back to "pickaxe".
- **Switching mid-dig resets the dig.** Exactly as pressing a hotbar key would.
- **Remote servers.** See the limitation above: the hotbar is read on the server side of the
  same process, so single player and LAN hosting work, a remote server does not.
- **No sound yet.** The on/off line is drawn, but the click is not played (the sound helper
  needs a typed `SoundPlay` the mod does not build yet).

## Layout

```
horizon-mod.toml             the manifest: id, version, api, [native] pins
Cargo.toml                   one crate, the SDK by the path the launcher stages
src/lib.rs                   the hooks, the scoring, the settings
src/table.rs                 blocks → tool, generated from the game's block tags
assets/autotools/lang/       en_us and ru_ru labels
package.py                   builds autotools-<version>.hmod
```

## Sharing

`python3 package.py` writes `autotools-<version>.hmod`: a zip of the manifest, the crate and the
assets, nothing compiled, with fixed dates so the bytes are reproducible. Send that file; the
other side's launcher compiles it for their own build. A compiled library cannot be passed
around — native mods are linked to one exact game build.

## License

[MIT](LICENSE). Not an official Minecraft product, not approved by or associated with Mojang or
Microsoft, and not affiliated with the Horizon project.
