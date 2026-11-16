"""Verify retained 0.10.8 media and review receipts; never launch or publish.

Run with --evidence pointing at the private deluxe-gamespeed evidence bundle.
Human pixel/authorization reviews remain human assertions; this checker binds
their receipts to the exact files and checks their scope and chronology.
It does not replace Cargo, static hook analysis, runtime or compliance gates.
"""
import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path

from PIL import Image


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8"))


def git(repo, *args):
    return subprocess.check_output(["git", "-C", str(repo), *args])


def paired_speed_throttle_recovery(repo, evidence):
    meta = read_json(evidence / "game-speed-media-0108/ENCODING.json")
    review = read_json(evidence / "game-speed-media-0108/VISUAL-REVIEW.json")
    timeline = read_json(evidence / "match-strip-media-0108/SOURCE-ACTION-TIMELINE.json")
    assert [f["cut"] for f in meta["frames"]] == (
        ["normal100"] * 9 + ["stable92"] * 9 + ["recover92"] * 9
    )
    assert review["accepted"] is True
    assert review["gif_sha256"] == meta["output_sha256"]
    assert [review[k]["observed"] for k in ("first", "middle", "last")] == [
        "online timed battle, Game100%, no cause",
        "same match, Game93% (delay too low)",
        "same match, Game100%, no cause",
    ]
    for frame in meta["frames"]:
        source = Path(frame["source"])
        assert sha(source) == frame["sha256"]
        capture = source.parent.parent / "capture.json"
        assert sha(capture) == frame["capture_json_sha256"]
        recorded = read_json(capture)["frames"][int(source.stem)]
        assert recorded["at_seconds"] == frame["source_at_seconds"]
        assert recorded["sha256"] == frame["sha256"]
    actions = timeline["actions"]
    assert [a["command"]["percent"] for a in actions] == [92, 100, 80, 100]
    assert all(a["exit"] == 0 for a in actions)
    assert (timeline["starts"]["normal100"]["wall_ns"]
            < actions[0]["controller_wall_bracket"][0]
            < timeline["starts"]["stable92"]["wall_ns"]
            < actions[1]["controller_wall_bracket"][0]
            < timeline["starts"]["recover92"]["wall_ns"])
    # Wall brackets support chronology, not a quantified cross-host clock proof.
    assert timeline["wall_metadata_only"]


def verify_game_speed_gif(repo, evidence):
    meta = read_json(evidence / "game-speed-media-0108/ENCODING.json")
    output = repo / "docs/media/game-speed.gif"
    assert sha(output) == meta["output_sha256"]
    delays = []
    with Image.open(output) as gif:
        assert gif.size == (1280, 720)
        assert gif.n_frames == len(meta["frames"]) == 27
        for index in range(gif.n_frames):
            gif.seek(index)
            delays.append(gif.info["duration"])
    assert delays == meta["durations_ms"]
    assert 0 < sum(delays) < 10000
    assert all(delay >= 1000 / 15 for delay in delays)
    elapsed = 0.0
    boundaries = [0]
    for frame in meta["frames"]:
        elapsed += frame["next_at_seconds"] - frame["source_at_seconds"]
        boundaries.append(round(elapsed * 100) * 10)
    assert delays == [b - a for a, b in zip(boundaries, boundaries[1:])]
    assert meta["source_requested_fps_not_achieved"] is True
    assert meta["caption"] == "Eden (untested on hardware)"
    for name, expected in meta["stills"].items():
        assert sha(repo / "docs/media" / name) == expected
    base = evidence / "runtime-counter-fix-1/evidence-a/out/m-match-a"
    for name, cut, index in (
        ("normal", "normal100", 10),
        ("dip", "stable92", 10),
        ("recovered", "recover92", 8),
    ):
        assert sha(repo / f"docs/media/game-speed-{name}.png") == sha(
            base / f"media-{cut}/frames/{index:04d}.png"
        )


def verify_media_refresh_inventory(repo, evidence):
    baseline = "a4411607be20a104bd992e0c2428288aa8b5d633"
    changed = {"match-strip.gif", "match-settings-strip.png", "match-strip-closeup.png"}
    names = git(repo, "ls-tree", "-r", "--name-only", baseline, "--", "docs/media").decode().splitlines()
    protected = 0
    for name in names:
        old = git(repo, "show", f"{baseline}:{name}")
        path = repo / name
        if path.name in changed:
            archive = path.with_name(path.stem + "-v070-offline" + path.suffix)
            assert archive.read_bytes() == old
            assert path.read_bytes() != old
        else:
            assert path.read_bytes() == old
            protected += 1
    assert protected == 44
    manifest = read_json(evidence / "match-strip-media-0108/FINAL-SOURCE-MANIFEST.json")
    for entry in manifest:
        assert sha(repo / entry["path"]) == entry["sha256"]
    checks = read_json(evidence / "match-strip-media-0108/MEDIA-DOC-CHECKS.json")
    assert checks["pass"] is True
    assert not checks["missing_local_paths"] and not checks["broken_local_anchors"]


def verify_media_docs_references(repo, evidence):
    gallery = (repo / "docs/SCREENSHOTS.md").read_text()
    index = (repo / "docs/MEDIA_PROVENANCE.md").read_text()
    current = (
        "game-speed.gif", "game-speed-normal.png", "game-speed-dip.png",
        "game-speed-recovered.png", "match-strip.gif", "match-settings-strip.png",
        "match-strip-closeup.png",
    )
    for name in current:
        assert f"(media/{name})" in gallery and f"(media/{name})" in index
    for text in (gallery, index):
        assert "2026-11-16 UTC" in text
        assert "0.10.8-a4411607be20-1794789531" in text
        assert "Eden (untested on hardware)" in text
    for relative in ("README.md", "docs/GAME_SPEED.md", "docs/RELEASE_NOTES_0_10_8.md"):
        text = (repo / relative).read_text()
        assert "game-speed.gif" in text
        assert "Eden (untested on hardware)" in text
    for relative in ("docs/SCREENSHOTS.md", "docs/MEDIA_PROVENANCE.md"):
        path = repo / relative
        for target in re.findall(r"!?\[[^\]]*\]\(([^\s)]+)", path.read_text()):
            if "://" in target or target.startswith(("#", "/")):
                continue
            assert (path.parent / target.split("#")[0]).exists(), (relative, target)


def verify_design_review_checkpoint(repo, evidence):
    checkpoint = read_json(evidence / "CHECKPOINT.json")
    assert checkpoint["source_head"] == "426184eaa77d39cfc717ea5d40d63522cd5ac3ae"
    assert checkpoint["source_worktree_clean"] is True
    from datetime import datetime
    submitted = datetime.fromisoformat(checkpoint["inbox_utc_get_date"].replace("Z", "+00:00"))
    implemented = int(git(repo, "show", "-s", "--format=%ct", "8484712").decode())
    assert submitted.timestamp() < implemented
    approvals = read_json(evidence / "publish-package/REVIEW-AUTHORIZATION.json")
    assert approvals["authority"] == "primary rulings relayed in this lane"
    assert approvals["design_review"] == "APPROVED before feature implementation"
    assert approvals["implementation_review"] == "GO after counter evidence"
    assert approvals["separate_sd_pair"] == "CANCELLED before Up"
    # These are explicit human review assertions, not inferred from artifact content.
    assert approvals["human_receipt_not_independent_authorization"] is True


def verify_release_authorization_receipts(repo, evidence):
    approvals = read_json(evidence / "publish-package/REVIEW-AUTHORIZATION.json")
    assert approvals["local_commits"] == "authorized"
    assert approvals["publication_boundary"] == "primary package review required"
    assert approvals["held_actions"] == ["push", "tag", "Forgejo release", "card write"]
    refs = git(repo, "for-each-ref", "--format=%(refname) %(objectname)",
               "refs/tags", "refs/remotes").decode()
    assert hashlib.sha256(refs.encode()).hexdigest() == approvals["remote_and_tag_refs_sha256"]
    assert "refs/tags/v0.10.8 " not in refs
    assert "No push, tag, release or card" in (repo / "docs/STATUS.md").read_text()
    # Negative action history is a human receipt; refs are only a cross-check.
    assert approvals["no_external_mutation_recorded"] is True


CHECKS = (
    paired_speed_throttle_recovery, verify_game_speed_gif,
    verify_media_refresh_inventory, verify_media_docs_references,
    verify_design_review_checkpoint, verify_release_authorization_receipts,
)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("repo", type=Path)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    results = []
    for check in CHECKS:
        try:
            check(args.repo.resolve(), args.evidence.resolve())
            results.append({"check": check.__name__, "pass": True})
        except (AssertionError, OSError, KeyError, ValueError, subprocess.SubprocessError) as error:
            results.append({"check": check.__name__, "pass": False,
                            "error": f"{type(error).__name__}: {error}"})
    args.out.write_text(json.dumps(results, indent=2) + "\n")
    print(json.dumps(results, indent=2))
    return int(any(not row["pass"] for row in results))


if __name__ == "__main__":
    raise SystemExit(main())
