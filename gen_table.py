#!/usr/bin/env python3
"""Regenerate `mixins/autotools/table.rs` from the game's own block tags.

Horizon writes `block-tags.json` beside the client (for example
`versions/native-*/​.local/horizon-client/26.3/block-tags.json`). Its
`minecraft:mineable/*` tags are the game's own answer to "what breaks this
block", so the table copied from them is exact — no guessing by id.

    python3 gen_table.py ~/.local/share/horizon-launcher/versions/native-*/.local/horizon-client/26.3/block-tags.json
"""
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE / "mixins" / "autotools" / "table.rs"

# The tags that say how fast a tool breaks a block, in the order this mod
# prefers them when a block is in more than one (melon is axe and sword).
TAGS = [
    ("Shears", "minecraft:shears_major_breaking_speed"),
    ("Axe", "minecraft:mineable/axe"),
    ("Hoe", "minecraft:mineable/hoe"),
    ("Pickaxe", "minecraft:mineable/pickaxe"),
    ("Shovel", "minecraft:mineable/shovel"),
    ("Sword", "minecraft:sword_efficient"),
]

HEADER = '''// Generated from the game's own block tags (`block-tags.json` of Horizon 26.3),
// so the mod answers exactly what the game does. Regenerate with `gen_table.py`
// when the tags move.
//
// A block no tag names (a modded one) falls through to the heuristics in
// `runtime.rs`.

use crate::game::autotools::Kind;

/// The tool the game itself wants for a block, or `None` when no tag names it.
pub fn tagged(name: &str) -> Option<Kind> {
    let kind = match name {
'''


def resolve(tags, tag, seen=None):
    """A tag's blocks, with `#other_tag` entries followed."""
    seen = seen or set()
    if tag in seen:
        return set()
    seen.add(tag)
    out = set()
    for value in tags.get(tag, {}).get("values", []):
        if value.startswith("#"):
            out |= resolve(tags, value[1:], seen)
        else:
            out.add(value)
    return out


def main():
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    raw = json.loads(Path(sys.argv[1]).read_text())
    tags = {name: json.loads(body) for name, body in raw.items()}

    table = {}
    for kind, tag in TAGS:
        for block in resolve(tags, tag):
            table.setdefault(block.split(":", 1)[1], kind)

    lines = [HEADER]
    for block in sorted(table):
        lines.append(f'        "{block}" => Kind::{table[block]},\n')
    lines.append("        _ => return None,\n    };\n    Some(kind)\n}\n")
    OUT.write_text("".join(lines))
    print(f"{OUT}: {len(table)} blocks")


if __name__ == "__main__":
    main()
