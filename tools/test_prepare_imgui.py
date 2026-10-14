#!/usr/bin/env python3
"""Lightweight checks for the repository prepare tool. No network and no Docker."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import shutil
import sys
import tempfile
from pathlib import Path

CANDIDATE = Path(__file__).resolve().parents[1]
TOOL = CANDIDATE / "tools" / "prepare_imgui.py"
PATCH = CANDIDATE / "patches" / "imgui-smash" / "renderer-bb47.patch"
# The historical patch stays in the tree for provenance; the pin no longer applies it.
EXPECTED_PATCH = "4e138e351334732670ca250c859e44b53e1fa54eb59e447f2385281063b2871d"
EXPECTED_ARCHIVE = "8a06be16dbfd50d10bac81ee5bdd2ffa9cd3472773151cc45f1c2101541f8f7c"
EXPECTED_SIZE = 13851556
EXPECTED_REPO = "https://git.syntaxbyexample.com/intpa/imgui-smash"
EXPECTED_COMMIT = "7f0e47c4a1699a714254798bf79041b1f2e49196"


def sha256_of(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_tool():
    spec = importlib.util.spec_from_file_location("repo_prepare_imgui", TOOL)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load prepare tool")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main() -> int:
    digest = sha256_of(PATCH)
    if digest != EXPECTED_PATCH:
        raise SystemExit(f"patch digest mismatch {digest}")
    text = TOOL.read_text(encoding="utf-8")
    for banned in ("renderer-ready/", "source-integration", "/home/intpa", "/tmp", "1000:1000"):
        if banned in text:
            raise SystemExit(f"tool depends on {banned}")
    module = load_tool()
    pin = module.load_pin()
    if pin["status"] != "ready" or pin["sha256"] != EXPECTED_ARCHIVE or pin["size"] != EXPECTED_SIZE:
        raise SystemExit("pin is not the clean archive")
    if module.pinned_patch(pin) is not None:
        raise SystemExit("the fork pin must not apply a patch")
    if pin["source"]["historical_patch_sha256"] != EXPECTED_PATCH:
        raise SystemExit("pin lost the historical patch digest")
    commands = module.clone_commands(module.UPSTREAM_CACHE, pin)
    joined = " ".join(" ".join(argv) for argv in commands)
    if EXPECTED_REPO not in joined:
        raise SystemExit("clone command lost the fork URL")
    if EXPECTED_COMMIT not in joined or "submodule update --init" not in joined:
        raise SystemExit("clone command lost the pinned checkout")
    with tempfile.TemporaryDirectory() as temporary:
        sample = Path(temporary) / "sample.a"
        sample.write_bytes(b"not-an-archive")
        dest = Path(temporary) / "dest"
        try:
            module.main(["--source", str(sample), "--dest", str(dest)])
        except SystemExit as exc:
            if "Nothing was installed" not in str(exc):
                raise
        else:
            raise SystemExit("--source installed a digest that does not match the pin")
        if dest.exists():
            raise SystemExit("--source created a destination")
    user = f"{os.getuid()}:{os.getgid()}"
    if module.docker_user() != user:
        raise SystemExit(f"docker user was not {user}")
    configure = " ".join(module.docker_argv(module.WORK_TREE, False))
    for required in (
        f"--user {user}",
        "--network none",
        "-C /work/deterministic-archive.cmake",
        "/work/lib/cmake-build",
        "1fc388c3a0d34bd2045a6dadcb1020e069d5f876a187fd705de14b4440c00282",
    ):
        if required not in configure:
            raise SystemExit(f"configure command missing {required}")
    cmake = (CANDIDATE / "patches/imgui-smash/deterministic-archive.cmake").read_text(encoding="utf-8")
    if "<CMAKE_AR> qcD <TARGET> <LINK_FLAGS> <OBJECTS>" not in cmake:
        raise SystemExit("create rule mismatch")
    if "<CMAKE_RANLIB> -D <TARGET>" not in cmake:
        raise SystemExit("finish rule mismatch")
    with tempfile.TemporaryDirectory() as temporary:
        upstream = Path(temporary) / "upstream"
        (upstream / "lib" / "cmake-build").mkdir(parents=True)
        (upstream / "lib" / "cmake-build" / "dirty.o").write_bytes(b"dirty-build-output")
        (upstream / "README").write_text("source\n", encoding="utf-8")
        (upstream / ".git").mkdir()
        (upstream / ".git" / "HEAD").write_text("ref\n", encoding="utf-8")
        submodule = upstream / "imgui-smash" / "extern" / "imgui"
        submodule.mkdir(parents=True)
        (submodule / "imgui.cpp").write_text("int x;\n", encoding="utf-8")
        (submodule / ".git").mkdir()
        (submodule / ".git" / "config").write_text("gitdir\n", encoding="utf-8")
        copied = Path(temporary) / "copied"
        shutil.copytree(upstream, copied, ignore=module.ignore_upstream(upstream))
        if (copied / "lib").exists():
            raise SystemExit("dirty lib output was copied")
        if (copied / "README").read_text(encoding="utf-8") != "source\n":
            raise SystemExit("source file was not retained")
        if (copied / "imgui-smash" / "extern" / "imgui" / "imgui.cpp").read_text(encoding="utf-8") != "int x;\n":
            raise SystemExit("submodule source was not retained")
        if (copied / ".git").exists() or (copied / "imgui-smash" / "extern" / "imgui" / ".git").exists():
            raise SystemExit("git metadata was copied")
        try:
            module.reject_upstream_inside_work(module.WORK_TREE / "nested")
        except SystemExit as exc:
            if "work tree" not in str(exc):
                raise
        else:
            raise SystemExit("upstream inside the work tree was accepted")
        archive = b"clean-archive"
        digest = hashlib.sha256(archive).hexdigest()
        ready = json.loads(json.dumps(pin))
        ready["status"] = "ready"
        ready["sha256"] = digest
        ready["size"] = len(archive)
        cached = Path(temporary) / "cached"
        cached.mkdir()
        (cached / "libimgui_smash.a").write_bytes(archive)
        (cached / "libimgui_smash.a.sha256").write_text(f"{digest}  libimgui_smash.a\n", encoding="utf-8")
        if module.build_from_source(Path(temporary) / "missing-upstream", cached, ready, False) != 0:
            raise SystemExit("verified destination did not return before a build")
        if (cached / "provenance.json").exists():
            raise SystemExit("early exit rewrote provenance")
        install_dest = Path(temporary) / "installed"
        module.install_verified(cached / "libimgui_smash.a", install_dest, ready, False, "local-source")
        provenance = json.loads((install_dest / "provenance.json").read_text(encoding="utf-8"))
        if provenance["origin"] != "local-source" or provenance["sha256"] != digest:
            raise SystemExit("provenance origin or digest is wrong")
        if provenance["patch_sha256"] is not None or provenance["imgui_smash_commit"] != EXPECTED_COMMIT:
            raise SystemExit("provenance lost the source pin")
        if provenance["imgui_smash_repo"] != EXPECTED_REPO or "qcD" not in provenance["deterministic_archive"]["cmake_cxx_archive_create"]:
            raise SystemExit("provenance lost the repository or deterministic rule")
        real = cached / "libimgui_smash.a"
        repair = Path(temporary) / "repair"
        repair.mkdir()
        shutil.copyfile(real, repair / "libimgui_smash.a")
        (repair / "libimgui_smash.a.sha256").write_text("deadbeef  libimgui_smash.a\n", encoding="utf-8")
        module.install_verified(real, repair, ready, False, "local-source")
        if not module.destination_verified(repair, ready):
            raise SystemExit("wrong sidecar was not repaired")
        provenance_bytes = (repair / "provenance.json").read_bytes()
        if module.build_from_source(None, repair, ready, False) != 0:
            raise SystemExit("valid cache did not return before configure")
        if (repair / "provenance.json").read_bytes() != provenance_bytes:
            raise SystemExit("cache return rewrote provenance")
        link = Path(temporary) / "work-link"
        target = Path(temporary) / "work-target"
        target.mkdir()
        (target / "keep").write_text("stay\n", encoding="utf-8")
        link.symlink_to(target, target_is_directory=True)
        saved = module.WORK_TREE
        module.WORK_TREE = link
        try:
            try:
                module.remove_work_tree()
            except SystemExit as exc:
                if "symlink" not in str(exc):
                    raise
            else:
                raise SystemExit("symlink work tree was removed")
            if (target / "keep").read_text(encoding="utf-8") != "stay\n":
                raise SystemExit("symlink removal deleted the target")
        finally:
            module.WORK_TREE = saved
    print("repo candidate checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
