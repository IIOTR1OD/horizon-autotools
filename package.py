#!/usr/bin/env python3
"""Package AutoTools as a source-only `.hnmod`, the file Horizon's launcher takes.

The archive holds `horizon-native.json` and `mixins/<id>/` (only `.rs` and `.toml`),
with fixed dates and modes, so the same sources always give the same bytes and the
same SHA-256. The launcher compiles it on the pinned Horizon revision; nothing here
is built or run.

Adapted from horizon-native-mod-template/tools/package.py (MIT).
"""
import hashlib
import json
import tomllib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent
GAME = "26.3"
# The Horizon build this mod was written on (the installed `native-6475f62`), and
# the horizon-mixins revision that build used.
COMPAT = {
    "horizon_revision": "6475f62f3c834b7489e1816bf47f47ea5eb2b879",
    "horizon_mixins_revision": "5a64af938ec86b00d53d16a408aabc0038fb3e57",
}


def package():
    manifests = sorted((ROOT / "mixins").glob("*/horizon-mixin.toml"))
    if len(manifests) != 1:
        raise SystemExit("a .hnmod holds one bundle: keep one mixins/<id>/horizon-mixin.toml")
    bundle = manifests[0].parent
    mixin = tomllib.loads(manifests[0].read_text())
    mod = mixin["mod"]
    if bundle.name != mod["id"]:
        raise SystemExit(f"rename mixins/{bundle.name}/ to mixins/{mod['id']}/ (the launcher finds it by id)")
    if mixin.get("compatibility", {}).get("horizon_revision") != COMPAT["horizon_revision"]:
        raise SystemExit("horizon-mixin.toml and this script pin different Horizon revisions")

    manifest = dict(
        schema=1,
        id=mod["id"],
        version=mod["version"],
        game=GAME,
        horizon=COMPAT["horizon_revision"],
        mixins=COMPAT["horizon_mixins_revision"],
    )
    files = {"horizon-native.json": (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode()}
    for path in sorted(bundle.iterdir()):
        if path.is_file() and path.suffix in (".rs", ".toml"):
            files[path.relative_to(ROOT).as_posix()] = path.read_bytes()

    out = ROOT / "target" / f"{mod['id']}-{mod['version']}.hnmod"
    out.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(out, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, content in sorted(files.items()):
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, content)
    print(json.dumps(dict(file=str(out), size=out.stat().st_size,
                          sha256=hashlib.sha256(out.read_bytes()).hexdigest()), indent=2))


if __name__ == "__main__":
    package()
