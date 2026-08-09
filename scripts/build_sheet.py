#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow>=10"]
# ///
"""Assemble finished pixel-art frames into a sprite sheet the engine can read.

The inverse of gameart-mcp's `slice_sheet`. Frames go in row-major order and are
padded -- never resampled -- into exact cells, so a sheet either matches the grid
the Rust side declares or the build fails loudly.

Layout presets mirror the constants in the engine; see LAYOUTS below. Keep them
in sync -- `--verify` exists to catch drift.

    uv run scripts/build_sheet.py --layout npc  --out assets/sprites/npc/npc_x.webp frames/*.png
    uv run scripts/build_sheet.py --layout loop --cols 6 --cell 32x32 --out out.webp f/*.png
    uv run scripts/build_sheet.py --verify --layout npc assets/sprites/npc/npc_galen_sheet.webp
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from PIL import Image

# (cols, rows, cell_w, cell_h, anchor) -- must match the Rust constants cited.
LAYOUTS: dict[str, tuple[int, int, int, int, str]] = {
    # level/src/npcs.rs, level/src/galen.rs: 8 cols (4 idle + 4 walk) x 4 rows (S,E,N,W)
    "npc": (8, 4, 32, 32, "bottom"),
    # level/src/enemies.rs: same grid as NPCs
    "enemy": (8, 4, 32, 32, "bottom"),
    # player/src/animation.rs: 12 cols (4 idle + 4 walk + 4 run) x 8 rows (S,SW,W,NW,N,NE,E,SE)
    "player": (12, 8, 32, 64, "bottom"),
    # Scenery / effect loop: a single horizontal strip. Pass --cols and --cell.
    "loop": (0, 1, 0, 0, "center"),
}

ANCHORS = ("bottom", "center", "top")


def parse_cell(text: str) -> tuple[int, int]:
    try:
        w, h = text.lower().split("x")
        return int(w), int(h)
    except ValueError:
        raise argparse.ArgumentTypeError(f"--cell wants WxH, got {text!r}") from None


def resolve(args: argparse.Namespace) -> tuple[int, int, int, int, str]:
    cols, rows, cw, ch, anchor = LAYOUTS[args.layout]
    if args.cols:
        cols = args.cols
    if args.rows:
        rows = args.rows
    if args.cell:
        cw, ch = args.cell
    if args.anchor:
        anchor = args.anchor
    if not (cols and rows and cw and ch):
        sys.exit(
            f"layout {args.layout!r} needs --cols/--rows/--cell filled in "
            f"(got cols={cols} rows={rows} cell={cw}x{ch})"
        )
    return cols, rows, cw, ch, anchor


def place(frame: Image.Image, cw: int, ch: int, anchor: str, label: str) -> tuple[int, int]:
    """Offset that centres `frame` horizontally and anchors it vertically."""
    if frame.width > cw or frame.height > ch:
        sys.exit(
            f"{label}: frame is {frame.width}x{frame.height}, larger than the "
            f"{cw}x{ch} cell. Re-run pixelize with a smaller target_px -- this "
            f"script pads but never resamples, because scaling pixel art here "
            f"would undo the palette conform."
        )
    x = (cw - frame.width) // 2
    y = {"top": 0, "center": (ch - frame.height) // 2, "bottom": ch - frame.height}[anchor]
    return x, y


def build(args: argparse.Namespace) -> None:
    cols, rows, cw, ch, anchor = resolve(args)
    frames = [Path(p) for p in args.frames]
    want = cols * rows

    if not frames:
        sys.exit("no frames given")
    if len(frames) != want:
        sys.exit(
            f"layout {args.layout!r} is {cols}x{rows} = {want} cells but {len(frames)} "
            f"frames were given. Row-major order, no gaps -- pad a short animation by "
            f"repeating its last frame rather than leaving a cell empty (an empty cell "
            f"reads as a one-frame flicker in game)."
        )

    sheet = Image.new("RGBA", (cols * cw, rows * ch), (0, 0, 0, 0))
    for i, path in enumerate(frames):
        frame = Image.open(path).convert("RGBA")
        ox, oy = place(frame, cw, ch, anchor, path.name)
        col, row = i % cols, i // cols
        sheet.paste(frame, (col * cw + ox, row * ch + oy), frame)

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    if out.suffix != ".webp":
        sys.exit(f"{out}: assets/ is WebP-only in this repo (see CLAUDE.md)")
    # Lossless: lossy WebP resamples across hard colour edges and reintroduces
    # colours the palette conform removed.
    sheet.save(out, "WEBP", lossless=True, quality=100, method=6)

    print(f"{out}  {sheet.width}x{sheet.height}  {cols}x{rows} cells of {cw}x{ch}  {out.stat().st_size} B")


def verify(args: argparse.Namespace) -> None:
    cols, rows, cw, ch, _ = resolve(args)
    bad = False
    for p in args.frames:
        img = Image.open(p)
        want = (cols * cw, rows * ch)
        ok = img.size == want
        bad |= not ok
        print(f"{'ok  ' if ok else 'FAIL'} {p}  {img.width}x{img.height}  want {want[0]}x{want[1]}")
    sys.exit(1 if bad else 0)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("frames", nargs="+", help="frame images in row-major order (or sheets, with --verify)")
    ap.add_argument("--layout", choices=sorted(LAYOUTS), required=True)
    ap.add_argument("--out", help="destination .webp (required unless --verify)")
    ap.add_argument("--cols", type=int, help="override the preset column count")
    ap.add_argument("--rows", type=int, help="override the preset row count")
    ap.add_argument("--cell", type=parse_cell, help="override the preset cell size, WxH")
    ap.add_argument("--anchor", choices=ANCHORS, help="vertical anchor within the cell")
    ap.add_argument("--verify", action="store_true", help="check existing sheets against the layout")
    args = ap.parse_args()

    if args.verify:
        verify(args)
    elif not args.out:
        ap.error("--out is required unless --verify")
    else:
        build(args)


if __name__ == "__main__":
    main()
