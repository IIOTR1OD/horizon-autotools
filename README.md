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

Horizon resolves the latest release, checks the `.hnmod`, builds it into your instance and
selects the new build. Update later with `horizon-launcher mods update-github`.

### From the `.hnmod` by hand

```sh
unzip autotools-1.2.2.hnmod -d autotools
horizon-launcher native-build ./autotools --trust-native-code
```

`native-build` takes an unpacked `.hnmod` — unpacking *is* installing. The launcher brings its
own Rust toolchain, so you do not need `cargo` on your machine.

### From a checkout

```sh
git clone https://github.com/IIOTR1OD/horizon-autotools
cd horizon-autotools
horizon-launcher native-build . --profile dev --trust-native-code
```

`--profile dev` rebuilds fastest and runs slowest — the right trade while you are iterating.
For playing, drop the flag (or pass `--profile performance`).

Either way the first build compiles the game's client crate, so expect ten minutes or so. Later
builds reuse the cache and are quicker.

## Use

| | |
|---|---|
| Toggle | `` ` `` — or whatever you bind in **Options → Mods → AutoTools** |
| Where | While mining: hold attack, the slot switches by itself |
| What it picks | The matching tool with the best material: netherite > diamond > iron > stone > wood, gold above netherite because vanilla mines faster with it |
| What it never does | Move items, use items, hit things for you, read the server |

Nothing happens while you are not attacking, and the moment your crosshair leaves a block the
mod stops caring: a block seen more than half a second ago is not acted on.

## How it works

AutoTools is a **native mod**: source that Horizon compiles into the game, not a `.jar`. It adds
one module beside the game's code and hooks seven points:

| hook | what it does |
|---|---|
| `native_block_highlight` (API 86) | every frame the crosshair is on a block: note what tool it wants |
| `native_tick` | once a tick while attacking: pick the slot and select it |
| `native_mouse_button_post` | remember whether the attack button is held |
| `native_key` | the toggle key |
| `native_hud` | draw the two-second `on/off` line |
| `native_mods` / `native_settings` / `native_settings_click` | the row, the page and the button in **Options → Mods** |

The block's name comes from `native_state_block(state)`, so the mod does not have to guess at
the game's internals.

## Compatibility

The `.hnmod` is pinned to one Horizon commit (`6475f62f…`, the build it was written and played
on). Horizon's API moves fast; a mod built for another commit is refused with *"made for another
Horizon version"*. Two ways out:

- build with `--allow-version-mismatch` (usually works — the hooks this mod uses are stable);
- re-pin `horizon_revision` in `mixins/autotools/horizon-mixin.toml` and `horizon` in
  `horizon-native.json` to your commit, then build.

The mod is not on Horizon's registry, so the launcher labels it *"Not made or endorsed by the
Horizon project"*. That is accurate.

## Known limitations

- **Tool choice is a heuristic.** Horizon exposes no way to ask "how fast does this item break
  this block", so AutoTools classifies blocks by id (`_log` → axe, `_ore` → pickaxe, `dirt` →
  shovel, …) and tools by their suffix. Vanilla blocks are almost always right; modded blocks
  fall back to "pickaxe".
- **Switching mid-dig resets the dig.** Exactly as pressing a hotbar key would. That is what
  makes it feel instant, and it is also why it can feel jumpy on a laggy server.
- **One internal field.** Selecting a slot is the only thing Horizon has no API for yet, so the
  mod writes `Game::selected` — the same field the hotbar keys write. When a `select_slot`
  helper lands, this becomes a pure hook mod and the pin can go.
- **Two machines tested.** Not a compatibility matrix.
- **No server-side story.** It is client-side by design: nothing is sent to the server except
  the slot change the game itself already sends.

## Layout

```
horizon-native.json          bundle manifest (an unpacked .hnmod is a checkout)
mixins/autotools/
  horizon-mixin.toml         the seven injections
  runtime.rs                 state, scoring, the one field write
  highlight.rs               target block → tool kind
  tick.rs                    the switch
  key.rs, mouse.rs           input
  hud.rs                     the on/off line
  list.rs, settings.rs, click.rs   Options > Mods
package.py                   builds target/autotools-<version>.hnmod
```

## Sharing

`python3 package.py` writes `target/autotools-<version>.hnmod`: a zip of the manifest and the
`.rs`/`.toml` sources, nothing compiled. Send that file; the other side builds it for their own
machine and their own Horizon. A compiled binary cannot be passed around — native mods are
compiled into the game, not loaded beside it.

## License

[MIT](LICENSE). Not an official Minecraft product, not approved by or associated with Mojang or
Microsoft, and not affiliated with the Horizon project.
