#!/usr/bin/env python3
"""Verify a debug bundle and resolve a plugin-relative or absolute crash PC."""
import argparse
import hashlib
import json
import shutil
import subprocess
from pathlib import Path

def number(value):
    return int(value, 0)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("bundle", type=Path, help="dist/<NRO hash prefix>")
    parser.add_argument("pc", nargs="+", type=number, help="0x-prefixed PC(s)")
    parser.add_argument("--base", type=number, default=0,
                        help="plugin base from crash module map; omit for module offsets")
    parser.add_argument("--module-id", help="optional crash module ID (checked against NRO)")
    args = parser.parse_args()
    manifest = json.loads((args.bundle / "manifest.json").read_text())
    for kind in ("nro", "elf"):
        path = args.bundle / manifest[kind]
        if hashlib.sha256(path.read_bytes()).hexdigest() != manifest[kind + "_sha256"]:
            parser.error(f"{kind} hash mismatch; wrong or modified debug bundle")
    if args.module_id and args.module_id.lower() != manifest["module_id"]:
        parser.error("crash module ID does not match this bundle")
    offsets = [pc - args.base for pc in args.pc]
    nro_size = manifest["nro_size"]
    if any(offset < 0 or offset >= nro_size for offset in offsets):
        parser.error("PC outside NRO; check the plugin base/module from the crash report")
    tool = shutil.which("llvm-addr2line") or shutil.which("addr2line")
    if not tool:
        parser.error("install LLVM/binutils addr2line")
    print(f"build={manifest['build_id']} module={manifest['module_id']}", flush=True)
    subprocess.run([tool, "-C", "-f", "-i", "-e", str(args.bundle / manifest["elf"])]
                   + [hex(offset) for offset in offsets], check=True)

if __name__ == "__main__":
    main()
