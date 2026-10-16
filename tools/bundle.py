#!/usr/bin/env python3
"""Assemble the two SD-layout release bundles from already-built, hashed inputs.

  <name>-bundle.zip      Deluxe NRO + FX Control NRO (+ README, manifest)
  <name>-bundle-hoc.zip  the same plus Horizon OC (hoc.kip, exosphere.bin,
                         hoc-clk, overlay, config templates) from the upstream
                         release archive and a hekate_ipl.ini snippet

Nothing is built or downloaded here; every input is a file the caller already
verified (package manifest for the NROs, the GitHub release asset for Horizon
OC). The bundles are written outside the repository and never committed; the
manifest.json inside each bundle lists every member with its SHA-256 and the
upstream source of the third-party parts.
"""
import argparse
import hashlib
import io
import json
import time
import zipfile
from pathlib import Path

PLUGINS = "atmosphere/contents/01006A800016E000/romfs/skyline/plugins/"
HOC_SOURCE = "https://github.com/Horizon-OC/Horizon-OC/releases"
HOC_LICENSE = "GPL-2.0 (see the upstream repository)"

README = """SSBU Online Deluxe (rewrite) bundle

Copy the folders to the root of the SD card. Needs Atmosphere with the Skyline
loader for Smash 13.0.5 (exefs subsdk9 + main.npdm, as installed by the usual
Skyline setup); this bundle ships only the plugins:

  atmosphere/contents/01006A800016E000/romfs/skyline/plugins/
    libssbu_online_deluxe_rewrite.nro   Online Deluxe {deluxe_build}
    libssbu_fx_control.nro              FX Control {fx_build}

Both plugins create their INI on first start under sd:/ultimate/.
Window: hold L+R and tap D-pad Down. Docs: {docs}
{hoc_section}
Every file in this archive is listed with its SHA-256 in manifest.json.
"""

HOC_README = """
Horizon OC {hoc_version} is included (third-party, {hoc_license}, from
{hoc_source}). It installs:

  atmosphere/kips/hoc.kip                     replacement Atmosphere loader
  atmosphere/exosphere.bin                    matching secure monitor
  atmosphere/contents/00FF0000636C6BFF/       hoc-clk sysmodule (autostart on)
  switch/.overlays/horizon-oc-overlay.ovl     Tesla/Ultrahand overlay
  config/horizon-oc/                          templates and languages

It only takes effect when hekate loads the two files: add the lines from
bootloader/hekate_ipl.ini.horizon-oc.txt to the Atmosphere entry you boot
(sysMMC or emuMMC). Horizon OC requires the Atmosphere version it was built
for ({hoc_ams}); do not mix it with another Atmosphere release. Deluxe itself
does no overclocking; its Performance view and captures record the measured
clock (Clock row, # clock= header) so OC runs are visible.
"""

HEKATE_SNIPPET = """# Horizon OC: add these two lines to the hekate_ipl.ini entry you boot, e.g.
#
# [Atmosphere FSS0 sysMMC]
# cal0blank=0
# pkg3=atmosphere/package3
# emummc_force_disable=1
kip1=atmosphere/kips/hoc.kip
secmon=atmosphere/exosphere.bin
"""


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def manifest_build(manifest_path: Path) -> dict:
    manifest = json.loads(manifest_path.read_text())
    return {"build_id": manifest.get("build_id"), "nro_sha256": manifest.get("nro_sha256")}


def make_bundle(path: Path, members: dict, readme: str, sources: dict) -> dict:
    listing = {}
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as out:
        for name in sorted(members):
            data = members[name]
            out.writestr(zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0)), data)
            listing[name] = {"size": len(data), "sha256": sha256(data)}
        out.writestr("README.txt", readme)
        manifest = {
            "schema": 1,
            "created": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "files": listing,
            "sources": sources,
        }
        out.writestr("manifest.json", json.dumps(manifest, indent=2) + "\n")
    return {"path": str(path), "size": path.stat().st_size, "sha256": sha256(path.read_bytes()), "files": len(listing) + 2}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--deluxe", required=True, type=Path, help="Deluxe package dir (NRO + manifest.json from package_debug.py)")
    parser.add_argument("--fx", required=True, type=Path, help="FX Control package dir (NRO + manifest.json from its package.py)")
    parser.add_argument("--hoc", type=Path, help="Horizon OC release dist.zip")
    parser.add_argument("--hoc-version", default="", help="Horizon OC release tag, e.g. 2.6.0-rel")
    parser.add_argument("--hoc-ams", default="", help="Atmosphere version that Horizon OC release targets, e.g. 1.12.1")
    parser.add_argument("--name", required=True, help="Bundle base name, e.g. ssbu-online-deluxe-0.5.3-abcdef")
    parser.add_argument("--output", required=True, type=Path, help="Directory for the two zips (outside the repository)")
    parser.add_argument("--docs", default="https://git.syntaxbyexample.com/intpa/ssbu-online-deluxe-rewrite")
    args = parser.parse_args()

    deluxe_nro = args.deluxe / "libssbu_online_deluxe_rewrite.nro"
    fx_nro = args.fx / "libssbu_fx_control.nro"
    deluxe_info = manifest_build(args.deluxe / "manifest.json")
    fx_info = manifest_build(args.fx / "manifest.json")
    for nro, info in ((deluxe_nro, deluxe_info), (fx_nro, fx_info)):
        if sha256(nro.read_bytes()) != info["nro_sha256"]:
            raise SystemExit(f"{nro} does not match its manifest")
    members = {
        PLUGINS + deluxe_nro.name: deluxe_nro.read_bytes(),
        PLUGINS + fx_nro.name: fx_nro.read_bytes(),
    }
    sources = {
        "ssbu-online-deluxe-rewrite": deluxe_info,
        "ssbu-fx-control": fx_info,
    }
    args.output.mkdir(parents=True, exist_ok=True)
    results = []
    plain_readme = README.format(deluxe_build=deluxe_info["build_id"], fx_build=fx_info["build_id"], docs=args.docs, hoc_section="")
    results.append(make_bundle(args.output / f"{args.name}-bundle.zip", members, plain_readme, sources))

    if args.hoc:
        hoc_bytes = args.hoc.read_bytes()
        hoc_members = dict(members)
        with zipfile.ZipFile(io.BytesIO(hoc_bytes)) as dist:
            for info in dist.infolist():
                if info.is_dir():
                    continue
                hoc_members[info.filename] = dist.read(info)
        hoc_members["bootloader/hekate_ipl.ini.horizon-oc.txt"] = HEKATE_SNIPPET.encode()
        hoc_sources = dict(sources)
        hoc_sources["horizon-oc"] = {
            "release": args.hoc_version,
            "asset": args.hoc.name,
            "asset_sha256": sha256(hoc_bytes),
            "source": HOC_SOURCE,
            "license": HOC_LICENSE,
            "atmosphere": args.hoc_ams,
        }
        hoc_section = HOC_README.format(hoc_version=args.hoc_version, hoc_license=HOC_LICENSE, hoc_source=HOC_SOURCE, hoc_ams=args.hoc_ams)
        hoc_readme = README.format(deluxe_build=deluxe_info["build_id"], fx_build=fx_info["build_id"], docs=args.docs, hoc_section=hoc_section)
        results.append(make_bundle(args.output / f"{args.name}-bundle-hoc.zip", hoc_members, hoc_readme, hoc_sources))

    (args.output / f"{args.name}-bundles.json").write_text(json.dumps(results, indent=2) + "\n")
    for result in results:
        print(f"{result['size']:>10} {result['sha256'][:16]} {result['files']:>3} files  {result['path']}")


if __name__ == "__main__":
    main()
