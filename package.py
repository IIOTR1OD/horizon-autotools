#!/usr/bin/env python3
"""Package AutoTools as a `.hmod`: the flat Horizon code-mod package.

The archive holds `horizon-mod.toml`, `Cargo.toml`, `src/` and `assets/`, with fixed
dates and modes, so the same sources always give the same bytes and the same SHA-256.
The launcher compiles it against the build it pins; nothing here is built or run.
"""
import hashlib
import re
import tomllib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent
DATE = (2026, 11, 19, 0, 0, 0)
FILES = ["horizon-mod.toml", "Cargo.toml"]


def sources() -> list[Path]:
    out = [ROOT / f for f in FILES]
    for folder in ("src", "assets"):
        out += sorted((ROOT / folder).rglob("*"))
    return [p for p in out if p.is_file()]


def main() -> None:
    manifest = tomllib.loads((ROOT / "horizon-mod.toml").read_text())
    mod_id, version = manifest["id"], manifest["version"]
    if not re.fullmatch(r"[a-z0-9_]{1,64}", mod_id):
        raise SystemExit(f"bad id {mod_id!r}")
    out = ROOT / f"{mod_id}-{version}.hmod"
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as zip_file:
        for path in sources():
            info = zipfile.ZipInfo(str(path.relative_to(ROOT)), DATE)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o644 << 16
            zip_file.writestr(info, path.read_bytes())
    digest = hashlib.sha256(out.read_bytes()).hexdigest()
    print(f"{out.name}: {out.stat().st_size} bytes, sha256 {digest}")


if __name__ == "__main__":
    main()
