#!/usr/bin/env python3
"""Place the pinned imgui-smash static archive where build.rs expects it.

The archive is third-party compiled code and is never committed. This script
either copies an exact local file (--source) or fetches the immutable upstream
copy over HTTPS, verifies the pinned SHA-256, and writes the archive plus a
sidecar digest and provenance record into target/native/imgui (git-ignored).
build.rs never downloads anything; it only checks the sidecar and size.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import sys
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

PINNED = {
    "name": "libimgui_smash.a",
    "sha256": "0042ff0592313228b0746ee9214091249f7c70a1797e1826d990c27ca2fe0e45",
    "size": 16798872,
    # Immutable: addressed by the fork commit that carried the archive.
    "url": (
        "https://raw.githubusercontent.com/MSFS-RyZe/ssbu-online-deluxe-quickplay-13.0.5/"
        "35fdf7cf5fea0f8249ca9fbb848c241b590bb483/lib/libimgui_smash.a"
    ),
    "fork_commit": "35fdf7cf5fea0f8249ca9fbb848c241b590bb483",
    "fork_blob": "aff8692e11b881b99d11dd67dbcfacc24f1cd497",
    "imgui_smash_commit": "bb47b205ed0be2fb864269ee2d37fc886b81221e",
    "imgui_smash_repo": "https://github.com/Coolsonickirby/imgui-smash",
}

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_DEST = REPO_ROOT / "target" / "native" / "imgui"


def sha256_of(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def fetch(url: str, destination: Path) -> None:
    if not url.startswith("https://"):
        raise SystemExit(f"refusing non-HTTPS source: {url}")
    request = urllib.request.Request(url, headers={"User-Agent": "ssbu-online-deluxe-rewrite prepare_imgui"})
    with urllib.request.urlopen(request, timeout=120) as response, destination.open("wb") as out:
        shutil.copyfileobj(response, out)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--source", type=Path, help="exact local copy of libimgui_smash.a to use instead of downloading")
    parser.add_argument("--url", default=PINNED["url"], help="HTTPS URL of the immutable upstream copy")
    parser.add_argument("--dest", type=Path, default=DEFAULT_DEST, help="destination directory (default: target/native/imgui)")
    parser.add_argument("--force", action="store_true", help="replace an existing verified archive")
    args = parser.parse_args()

    dest_dir: Path = args.dest
    dest_dir.mkdir(parents=True, exist_ok=True)
    target = dest_dir / PINNED["name"]
    sidecar = dest_dir / f"{PINNED['name']}.sha256"
    provenance = dest_dir / "provenance.json"

    if target.exists() and sidecar.exists() and not args.force:
        if sha256_of(target) == PINNED["sha256"]:
            print(f"already prepared: {target}")
            return 0
        print("existing archive does not match the pinned digest; replacing it")

    staging = dest_dir / (PINNED["name"] + ".partial")
    if args.source:
        source = args.source.resolve()
        if not source.is_file():
            raise SystemExit(f"source not found: {source}")
        shutil.copyfile(source, staging)
        origin = {"kind": "local-copy", "path": str(source)}
    else:
        print(f"fetching {args.url}")
        fetch(args.url, staging)
        origin = {"kind": "https", "url": args.url}

    actual_size = staging.stat().st_size
    actual_sha = sha256_of(staging)
    if actual_size != PINNED["size"] or actual_sha != PINNED["sha256"]:
        staging.unlink(missing_ok=True)
        raise SystemExit(
            "verification failed\n"
            f"  size   {actual_size} (expected {PINNED['size']})\n"
            f"  sha256 {actual_sha}\n"
            f"  pinned {PINNED['sha256']}\n"
            "Nothing was installed. Use the exact fork archive or update the pin deliberately."
        )
    staging.replace(target)
    sidecar.write_text(f"{PINNED['sha256']}  {PINNED['name']}\n", encoding="utf-8")
    provenance.write_text(
        json.dumps(
            {
                "prepared_at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
                "origin": origin,
                "sha256": PINNED["sha256"],
                "size": PINNED["size"],
                "fork_commit": PINNED["fork_commit"],
                "fork_blob": PINNED["fork_blob"],
                "imgui_smash_repo": PINNED["imgui_smash_repo"],
                "imgui_smash_commit": PINNED["imgui_smash_commit"],
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    print(f"verified and installed {target}")
    print(f"sidecar {sidecar}")
    print(f"provenance {provenance}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
