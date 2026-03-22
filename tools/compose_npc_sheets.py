#!/usr/bin/env python3
"""Compose NPC sprite sheets from PixelLab animation ZIPs.

Downloads each character ZIP, extracts idle + walk frames for all 4 directions,
and composites them into a single sprite sheet:

    8 columns (4 idle + 4 walk) × 4 rows (south / east / north / west)
    Each frame is 32×32 px → final sheet is 256×128 px.

Usage:
    python3 tools/compose_npc_sheets.py

Requires: Pillow (pip install Pillow)
"""

import io
import json
import sys
import zipfile
from pathlib import Path
from urllib.request import urlopen, Request

try:
    from PIL import Image
except ImportError:
    print("Pillow required: pip install Pillow (or nix-shell -p python3Packages.pillow)")
    sys.exit(1)

# Character ID → output asset filename (without path)
CHARACTERS = {
    "9785c052-d42c-4b28-8832-31ae65dc2aeb": "npc_mordred_sheet.png",
    "f4dbc507-201f-4923-9794-d3f939bdf1be": "npc_drizella_sheet.png",
    "ab498239-9a03-4a09-8a3a-f4dc0e2aa01e": "npc_bigby_sheet.png",
    "9c354dd5-4a0c-48c7-a82c-ff6064b63d54": "npc_gothel_sheet.png",
    "b3869d66-5472-466e-bbee-d2aeb946f287": "npc_morgana_sheet.png",
    "57ab9cbc-3f9c-4563-ac88-b0cb7728b519": "npc_cadwallader_sheet.png",
    "0c9294d4-2575-4220-8220-e27edbab3551": "npc_galen_sheet.png",
}

DIRECTIONS = ["south", "east", "north", "west"]
FRAME_SIZE = 32
IDLE_FRAMES = 4
WALK_FRAMES = 4
COLS = IDLE_FRAMES + WALK_FRAMES  # 8
ROWS = len(DIRECTIONS)            # 4

ASSETS_DIR = Path(__file__).resolve().parent.parent / "assets"
API_BASE = "https://api.pixellab.ai/mcp/characters"


def download_zip(char_id: str) -> zipfile.ZipFile:
    """Download character ZIP from PixelLab API.

    Falls back to rotation-only ZIP on HTTP 423 (animations pending).
    """
    url = f"{API_BASE}/{char_id}/download"
    req = Request(url)
    try:
        resp = urlopen(req)
    except Exception as e:
        if "423" in str(e):
            print("    ZIP locked (423) — building static-only sheet from rotations")
            return _rotation_only_zip(char_id)
        raise
    data = resp.read()
    if len(data) < 1024:
        # Small response is likely a JSON error, not a real ZIP.
        print("    ZIP too small — building static-only sheet from rotations")
        return _rotation_only_zip(char_id)
    return zipfile.ZipFile(io.BytesIO(data))


def _rotation_only_zip(char_id: str) -> zipfile.ZipFile:
    """Create an in-memory ZIP with just the rotation PNGs."""
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w") as zf:
        for direction in DIRECTIONS:
            rot_url = (
                f"https://backblaze.pixellab.ai/file/pixellab-characters/"
                f"d897fc0d-4673-40dc-ba8e-6aab496c7b9c/{char_id}/rotations/{direction}.png"
            )
            try:
                img_data = urlopen(rot_url).read()
                zf.writestr(f"rotations/{direction}.png", img_data)
            except Exception as e:
                print(f"    Could not fetch rotation/{direction}: {e}")
    buf.seek(0)
    return zipfile.ZipFile(buf)


def find_frames(zf: zipfile.ZipFile, anim_name: str, direction: str) -> list[str]:
    """Find animation frame paths in the ZIP, sorted by frame index."""
    prefix = f"animations/{anim_name}/{direction}/"
    frames = [
        name for name in zf.namelist()
        if name.startswith(prefix) and name.endswith(".png")
    ]
    frames.sort()
    return frames


def load_frame(zf: zipfile.ZipFile, path: str) -> Image.Image:
    """Load a single frame from the ZIP."""
    return Image.open(io.BytesIO(zf.read(path)))


# Map direction → fallback direction to mirror from (east↔west via horizontal flip).
MIRROR_MAP = {"west": "east", "east": "west"}


def get_rotation_frame(zf: zipfile.ZipFile, direction: str) -> Image.Image | None:
    """Load the static rotation image for a direction."""
    path = f"rotations/{direction}.png"
    if path in zf.namelist():
        return load_frame(zf, path)
    return None


def get_frames(
    zf: zipfile.ZipFile,
    anim_name: str,
    direction: str,
    count: int,
) -> list[Image.Image]:
    """Get animation frames for a direction, with multiple fallback layers."""
    # 1. Try actual animation frames.
    paths = find_frames(zf, anim_name, direction)
    if paths:
        return [load_frame(zf, p) for p in paths[:count]]

    # 2. Try mirroring the opposite horizontal direction.
    fallback = MIRROR_MAP.get(direction)
    if fallback:
        fb_paths = find_frames(zf, anim_name, fallback)
        if fb_paths:
            print(f"    Mirroring {fallback} → {direction} for {anim_name}")
            return [load_frame(zf, p).transpose(Image.FLIP_LEFT_RIGHT) for p in fb_paths[:count]]

    # 3. Fall back to static rotation image (duplicate across all frames).
    rot = get_rotation_frame(zf, direction)
    if rot is None and fallback:
        rot_fb = get_rotation_frame(zf, fallback)
        if rot_fb:
            rot = rot_fb.transpose(Image.FLIP_LEFT_RIGHT)
    if rot:
        print(f"    Using static rotation for {anim_name}/{direction}")
        return [rot] * count

    print(f"    WARNING: No frames for {anim_name}/{direction}")
    return []


def compose_sheet(zf: zipfile.ZipFile, output_path: Path) -> None:
    """Build the sprite sheet from ZIP contents."""
    sheet = Image.new("RGBA", (COLS * FRAME_SIZE, ROWS * FRAME_SIZE), (0, 0, 0, 0))

    for row, direction in enumerate(DIRECTIONS):
        # Idle frames (columns 0..3)
        idle_imgs = get_frames(zf, "breathing-idle", direction, IDLE_FRAMES)
        for col, img in enumerate(idle_imgs):
            sheet.paste(img, (col * FRAME_SIZE, row * FRAME_SIZE))

        # Walk frames (columns 4..7)
        walk_imgs = get_frames(zf, "walking-4-frames", direction, WALK_FRAMES)
        for i, img in enumerate(walk_imgs):
            col = IDLE_FRAMES + i
            sheet.paste(img, (col * FRAME_SIZE, row * FRAME_SIZE))

    sheet.save(output_path)
    print(f"  Saved: {output_path} ({sheet.size[0]}x{sheet.size[1]})")


def main() -> None:
    ASSETS_DIR.mkdir(parents=True, exist_ok=True)
    failed = []

    for char_id, filename in CHARACTERS.items():
        print(f"Processing {filename} ({char_id})...")
        try:
            zf = download_zip(char_id)
            output = ASSETS_DIR / filename
            compose_sheet(zf, output)
        except Exception as e:
            print(f"  FAILED: {e}")
            failed.append(filename)

    if failed:
        print(f"\nFailed: {', '.join(failed)}")
        print("Some animations may still be processing. Retry in a few minutes.")
        sys.exit(1)
    else:
        print("\nAll sprite sheets created successfully!")


if __name__ == "__main__":
    main()
