#!/usr/bin/env python3
"""Place the imgui-smash archive at target/native/imgui.

The default action clones the pinned public commit, checks out its pinned
submodule, applies the committed patch, and builds with the pinned devkit
image. --upstream uses a local checkout instead of the network. --source
installs a local archive only when it matches the recorded pin.

Patch generation is not this tool. The Docker user is the current uid and gid.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PATCH_REL = Path("patches/imgui-smash/renderer-bb47.patch")
PIN_REL = Path("patches/imgui-smash/archive-pin.json")
PRELOAD_REL = Path("patches/imgui-smash/deterministic-archive.cmake")
DEFAULT_DEST = ROOT / "target" / "native" / "imgui"
UPSTREAM_CACHE = ROOT / "target" / "native" / "imgui-upstream"
WORK_TREE = ROOT / "target" / "native" / "imgui-work"
ARCHIVE_NAME = "libimgui_smash.a"
IMAGE = (
    "devkitpro/devkita64@sha256:"
    "1fc388c3a0d34bd2045a6dadcb1020e069d5f876a187fd705de14b4440c00282"
)


def sha256_of(path: Path) -> str:
    import hashlib

    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_pin() -> dict:
    path = ROOT / PIN_REL
    pin = json.loads(path.read_text(encoding="utf-8"))
    patch = ROOT / PATCH_REL
    recorded = pin["source"]["patch_sha256"]
    actual = sha256_of(patch)
    if actual != recorded:
        raise SystemExit(f"patch digest {actual} does not match recorded {recorded}")
    return pin


def run(argv: list[str], timeout: int, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        argv,
        cwd=cwd,
        check=False,
        capture_output=True,
        text=True,
        timeout=timeout,
    )


def require_identity(upstream: Path, pin: dict) -> None:
    source = pin["source"]
    if not upstream.is_dir():
        raise SystemExit(f"upstream checkout not found: {upstream}")
    head = run(["/usr/bin/git", "-C", str(upstream), "rev-parse", "HEAD"], 30)
    if head.returncode != 0 or head.stdout.strip() != source["commit"]:
        raise SystemExit(f"upstream HEAD is not {source['commit']}")
    status = run(["/usr/bin/git", "-C", str(upstream), "status", "--porcelain"], 30)
    if status.returncode != 0 or status.stdout.strip():
        raise SystemExit("upstream checkout is dirty")
    sub = run(["/usr/bin/git", "-C", str(upstream), "submodule", "status"], 30)
    if sub.returncode != 0:
        raise SystemExit(sub.stderr.strip() or "git submodule status failed")
    expected = f" {source['submodule_commit']} {source['submodule_path']}"
    lines = [line for line in sub.stdout.splitlines() if line.strip()]
    if len(lines) != 1 or not lines[0].startswith(expected):
        raise SystemExit(f"submodule pin mismatch:\n{sub.stdout}")


def clone_commands(cache: Path, pin: dict) -> list[list[str]]:
    source = pin["source"]
    return [
        ["/usr/bin/git", "clone", "--no-checkout", source["repo"], str(cache)],
        ["/usr/bin/git", "-C", str(cache), "checkout", "--detach", source["commit"]],
        ["/usr/bin/git", "-C", str(cache), "submodule", "update", "--init"],
    ]


def ensure_upstream(pin: dict) -> Path:
    if UPSTREAM_CACHE.exists():
        return UPSTREAM_CACHE
    UPSTREAM_CACHE.parent.mkdir(parents=True, exist_ok=True)
    timeouts = (180, 120, 180)
    for argv, timeout in zip(clone_commands(UPSTREAM_CACHE, pin), timeouts, strict=True):
        result = run(argv, timeout)
        if result.returncode != 0:
            raise SystemExit(result.stderr.strip() or f"command failed: {argv[0]}")
    return UPSTREAM_CACHE


def reject_upstream_inside_work(upstream: Path) -> None:
    resolved = upstream.resolve()
    work = WORK_TREE.resolve()
    if resolved == work or work in resolved.parents:
        raise SystemExit(f"refusing --upstream inside the tool work tree: {resolved}")


def remove_work_tree() -> None:
    if WORK_TREE.is_symlink():
        raise SystemExit("refusing to remove a symlink work tree")
    if WORK_TREE.exists():
        shutil.rmtree(WORK_TREE)


def ignore_upstream(upstream: Path):
    root = upstream.resolve()

    def ignore(directory: str, entries: list[str]) -> set[str]:
        skipped = {entry for entry in entries if entry == ".git"}
        if Path(directory).resolve() == root:
            skipped.add("lib")
        return skipped

    return ignore


def prepare_work(upstream: Path) -> Path:
    reject_upstream_inside_work(upstream)
    remove_work_tree()
    shutil.copytree(upstream, WORK_TREE, ignore=ignore_upstream(upstream))
    applied = run(
        ["/usr/bin/git", "apply", "--whitespace=nowarn", str(ROOT / PATCH_REL)],
        60,
        cwd=WORK_TREE,
    )
    if applied.returncode != 0:
        raise SystemExit(applied.stderr.strip() or "git apply failed")
    shutil.copyfile(ROOT / PRELOAD_REL, WORK_TREE / "deterministic-archive.cmake")
    return WORK_TREE


def docker_user() -> str:
    return f"{os.getuid()}:{os.getgid()}"


def docker_argv(work: Path, build: bool) -> list[str]:
    argv = [
        "/usr/bin/docker",
        "run",
        "--rm",
        "--user",
        docker_user(),
        "--network",
        "none",
        "--memory",
        "2g",
        "--cpus",
        "2",
        "--entrypoint",
        "/usr/bin/cmake",
        "-v",
        f"{work.resolve()}:/work",
        "-w",
        "/work",
        IMAGE,
    ]
    if build:
        return argv + ["--build", "/work/lib/cmake-build", "-j", "2"]
    return argv + [
        "-C",
        "/work/deterministic-archive.cmake",
        "-DCMAKE_TOOLCHAIN_FILE=/work/imgui-smash/cmake/toolchain.cmake",
        "-DCMAKE_BUILD_TYPE=RelWithDebInfo",
        "-DCMAKE_TRY_COMPILE_TARGET_TYPE=STATIC_LIBRARY",
        "/work/imgui-smash",
        "-B",
        "/work/lib/cmake-build",
    ]


def destination_verified(dest_dir: Path, pin: dict) -> bool:
    if pin.get("status") != "ready" or not pin.get("sha256") or not pin.get("size"):
        return False
    target = dest_dir / ARCHIVE_NAME
    sidecar = dest_dir / f"{ARCHIVE_NAME}.sha256"
    if not target.is_file() or not sidecar.is_file():
        return False
    recorded = sidecar.read_text(encoding="utf-8").split()
    if not recorded or recorded[0].lower() != str(pin["sha256"]).lower():
        return False
    return target.stat().st_size == int(pin["size"]) and sha256_of(target) == str(pin["sha256"]).lower()


def write_provenance(dest_dir: Path, pin: dict, origin: str, archive_sha: str, archive_size: int) -> None:
    payload = {
        "origin": origin,
        "sha256": archive_sha,
        "size": archive_size,
        "imgui_smash_commit": pin["source"]["commit"],
        "submodule_commit": pin["source"]["submodule_commit"],
        "patch_sha256": pin["source"]["patch_sha256"],
        "image": IMAGE,
        "docker_user": docker_user(),
        "deterministic_archive": pin["deterministic_archive"],
    }
    (dest_dir / "provenance.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def install_verified(staging: Path, dest_dir: Path, pin: dict, force: bool, origin: str) -> None:
    if pin.get("status") != "ready" or not pin.get("sha256") or not pin.get("size"):
        raise SystemExit("archive pin is not ready\nNothing was installed.")
    expected_sha = str(pin["sha256"])
    expected_size = int(pin["size"])
    actual_size = staging.stat().st_size
    actual_sha = sha256_of(staging)
    if actual_size != expected_size or actual_sha != expected_sha:
        raise SystemExit(
            "verification failed\n"
            f"  size   {actual_size} (expected {expected_size})\n"
            f"  sha256 {actual_sha}\n"
            f"  pinned {expected_sha}\n"
            "Nothing was installed."
        )
    if not force and destination_verified(dest_dir, pin):
        print(f"already prepared: {dest_dir / ARCHIVE_NAME}")
        return
    dest_dir.mkdir(parents=True, exist_ok=True)
    target = dest_dir / ARCHIVE_NAME
    sidecar = dest_dir / f"{ARCHIVE_NAME}.sha256"
    shutil.copyfile(staging, target)
    sidecar.write_text(f"{expected_sha}  {ARCHIVE_NAME}\n", encoding="utf-8")
    write_provenance(dest_dir, pin, origin, expected_sha, expected_size)
    print(f"verified and installed {target}")


def build_from_source(upstream: Path | None, dest_dir: Path, pin: dict, force: bool) -> int:
    if not force and destination_verified(dest_dir, pin):
        print(f"already prepared: {dest_dir / ARCHIVE_NAME}")
        return 0
    if pin["status"] != "ready":
        raise SystemExit("refusing to build while the archive pin is not ready")
    source_tree = upstream if upstream is not None else ensure_upstream(pin)
    require_identity(source_tree, pin)
    work = prepare_work(source_tree)
    configure = run(docker_argv(work, False), 600)
    sys.stdout.write(configure.stdout)
    sys.stderr.write(configure.stderr)
    if configure.returncode != 0:
        return configure.returncode
    built = run(docker_argv(work, True), 900)
    sys.stdout.write(built.stdout)
    sys.stderr.write(built.stderr)
    if built.returncode != 0:
        return built.returncode
    archive = work / "lib" / "cmake-build" / ARCHIVE_NAME
    install_verified(archive, dest_dir, pin, force, "built")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, help="local archive to verify against the pin")
    parser.add_argument("--dest", type=Path, default=DEFAULT_DEST, help="install directory")
    parser.add_argument("--force", action="store_true", help="replace an existing verified archive")
    parser.add_argument("--upstream", type=Path, help="offline checkout of the pinned commit and submodule")
    args = parser.parse_args(argv)
    pin = load_pin()
    if args.source:
        source = args.source.resolve()
        if not source.is_file():
            raise SystemExit(f"source not found: {source}")
        install_verified(source, args.dest, pin, args.force, "local-source")
        return 0
    return build_from_source(args.upstream, args.dest, pin, args.force)


if __name__ == "__main__":
    sys.exit(main())
