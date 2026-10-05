#!/usr/bin/env python3
"""Archive the exact NRO and unstripped ELF together; never deploy."""
import argparse
import hashlib
import json
import re
import shutil
import struct
import subprocess
import tarfile
from pathlib import Path

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def package(root):
    build = root / "target/aarch64-skyline-switch/release"
    nro = build / "libssbu_online_deluxe_rewrite.nro"
    elf = build / "libssbu_online_deluxe_rewrite.so"
    binary = nro.read_bytes()
    symbols = elf.read_bytes()
    if binary[0x10:0x14] != b"NRO0" or symbols[:4] != b"\x7fELF":
        raise ValueError("expected a Skyline NRO and its ELF")
    if struct.unpack_from("<H", symbols, 18)[0] != 183:
        raise ValueError("ELF is not AArch64")
    # Verify NRO executable bytes against the ELF's PT_LOAD text segment.
    phoff = struct.unpack_from("<Q", symbols, 32)[0]
    phsize, phnum = struct.unpack_from("<HH", symbols, 54)
    text_offset, text_size = struct.unpack_from("<II", binary, 0x20)
    matched = False
    for index in range(phnum):
        kind, flags, offset, vaddr, _, filesz, _, _ = struct.unpack_from(
            "<IIQQQQQQ", symbols, phoff + index * phsize)
        if kind == 1 and flags & 1:
            # Linkle adds/fills the first 0x80-byte NRO header in .text.
            start = max(vaddr, 0x80)
            end = min(vaddr + filesz, text_offset + text_size)
            if end > start:
                elf_bytes = symbols[offset + start - vaddr:offset + end - vaddr]
                if binary[start:end] != elf_bytes:
                    raise ValueError("NRO and ELF executable bytes differ")
                matched = True
    if not matched:
        raise ValueError("could not validate matching executable segments")
    ids = set(re.findall(rb"\d+\.\d+\.\d+-(?:[0-9a-f]{12}|uncommitted)(?:-dirty)?-\d{10}", symbols))
    if len(ids) != 1:
        raise ValueError(f"expected one embedded build identity, found {len(ids)}")
    digest = sha(nro)
    dest = root / "dist" / digest[:16]
    dest.mkdir(parents=True, exist_ok=True)
    shutil.copy2(nro, dest / nro.name)
    shutil.copy2(elf, dest / "libssbu_online_deluxe_rewrite.elf")
    shutil.copy2(root / "Cargo.lock", dest / "Cargo.lock")
    # Include new/uncommitted source files too: git diff alone omits untracked
    # Rust modules and is insufficient to reconstruct a development build.
    source_archive = dest / "source.tar.gz"
    with tarfile.open(source_archive, "w:gz") as archive:
        for name in ("src", "Cargo.toml", "Cargo.lock", "build.rs", "tools/prepare_imgui.py"):
            archive.add(root / name, arcname=name)
    dependency = root / "target/native/imgui/provenance.json"
    native = []
    if dependency.exists():
        native.append(json.loads(dependency.read_text()))
        shutil.copy2(dependency, dest / "imgui-provenance.json")
    rustup = Path.home() / ".cargo/bin/rustup"
    compiler = subprocess.check_output(
        [str(rustup), "run", "skyline-v3", "rustc", "--version", "--verbose"], text=True)
    manifest = {
        "schema": 1, "build_id": ids.pop().decode(),
        "nro": nro.name, "nro_sha256": digest, "nro_size": len(binary),
        "module_id": binary[0x40:0x60].hex(),
        "elf": "libssbu_online_deluxe_rewrite.elf", "elf_sha256": sha(elf),
        "cargo_lock_sha256": sha(root / "Cargo.lock"),
        "source_archive": source_archive.name,
        "source_archive_sha256": sha(source_archive),
        "native_dependencies": native,
        "compiler": compiler, "game_version": "13.0.5",
        "text_verified": True, "runtime_tested": False,
    }
    (dest / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(dest)
    return dest

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    package(parser.parse_args().root)
