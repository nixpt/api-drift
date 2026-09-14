#!/usr/bin/env python3
"""
api-drift Branding Asset Generator

Generates:
  - Vector-accurate raster PNGs from master SVGs using rsvg-convert (or Pillow fallback):
      - assets/branding/api-drift-banner-1200.png (1200 x 420)
      - assets/branding/api-drift-logo.png        (860 x 220)
      - assets/branding/api-drift-icon-512.png    (512 x 512)
      - assets/branding/api-drift-icon-128.png    (128 x 128)
      - assets/branding/api-drift-icon-64.png     (64 x 64)
      - assets/branding/api-drift-icon-32.png     (32 x 32)
      - assets/branding/api-drift-mark.png        (256 x 256)
  - TrueColor 24-bit ANSI terminal banner:
      - assets/branding/api-drift-ansi.txt
"""

import os
import subprocess
import sys
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
BRANDING_DIR = BASE_DIR / "assets" / "branding"


def render_svg_to_png(svg_path: Path, png_path: Path, width: int, height: int):
    """Render an SVG to PNG at specified resolution using rsvg-convert or PIL fallback."""
    if not svg_path.exists():
        print(f"Error: source SVG {svg_path} does not exist", file=sys.stderr)
        return False

    # Prefer rsvg-convert for vector-perfect font and filter rendering
    try:
        cmd = [
            "rsvg-convert",
            "-w", str(width),
            "-h", str(height),
            str(svg_path),
            "-o", str(png_path),
        ]
        res = subprocess.run(cmd, capture_output=True, text=True, check=True)
        print(f"  [rsvg-convert] Created {png_path.relative_to(BASE_DIR)} ({width}x{height})")
        return True
    except (subprocess.SubprocessError, FileNotFoundError) as e:
        print(f"  Warning: rsvg-convert failed ({e}), attempting Pillow resize...", file=sys.stderr)

    # Fallback to Pillow if master PNG exists
    master_png = png_path.with_name(png_path.stem.rsplit("-", 1)[0] + "-512.png")
    if master_png.exists():
        try:
            from PIL import Image
            im = Image.open(master_png)
            resized = im.resize((width, height), Image.Resampling.LANCZOS)
            resized.save(png_path)
            print(f"  [Pillow] Resized {png_path.relative_to(BASE_DIR)} ({width}x{height})")
            return True
        except Exception as e:
            print(f"  Error: Pillow fallback failed: {e}", file=sys.stderr)
            return False

    return False


def generate_ansi_banner(output_path: Path):
    """Generate 24-bit TrueColor ANSI terminal banner."""
    def rgb(r: int, g: int, b: int, text: str) -> str:
        return f"\033[38;2;{r};{g};{b}m{text}\033[0m"

    lines = [
        f"   {rgb(56, 189, 248, '/|')}            {rgb(129, 140, 248, '▲')}             {rgb(244, 63, 94, '|\\')}     {rgb(56, 189, 248, '█████')}  {rgb(56, 189, 248, '████')}   {rgb(56, 189, 248, '█████')}         {rgb(0, 240, 255, '████')}   {rgb(0, 240, 255, '████')}   {rgb(0, 240, 255, '█████')}  {rgb(0, 240, 255, '█████')}  {rgb(0, 240, 255, '█████')}",
        f"  {rgb(56, 189, 248, '/ |')}    {rgb(16, 185, 129, '.......[Δ].......>')}    {rgb(244, 63, 94, '| \\')}    {rgb(56, 189, 248, '█   █')}  {rgb(56, 189, 248, '█   █')}    {rgb(56, 189, 248, '█')}   {rgb(100, 116, 139, '███████')} {rgb(0, 240, 255, '█   █')}  {rgb(0, 240, 255, '█   █')}    {rgb(0, 240, 255, '█')}    {rgb(0, 240, 255, '█')}        {rgb(0, 240, 255, '█')}",
        f" {rgb(56, 189, 248, '<  ●')}   {rgb(245, 158, 11, '------------------>')}    {rgb(244, 63, 94, '|  ★ >')} {rgb(56, 189, 248, '█████')}  {rgb(56, 189, 248, '████')}     {rgb(56, 189, 248, '█')}   {rgb(100, 116, 139, '███████')} {rgb(0, 240, 255, '█   █')}  {rgb(0, 240, 255, '████')}      {rgb(0, 240, 255, '█')}    {rgb(0, 240, 255, '████')}     {rgb(0, 240, 255, '█')}",
        f"  {rgb(56, 189, 248, '\\ |')}   {rgb(244, 63, 94, '=========')}{rgb(56, 189, 248, '=========>')}    {rgb(244, 63, 94, '| /')}    {rgb(56, 189, 248, '█   █')}  {rgb(56, 189, 248, '█')}        {rgb(56, 189, 248, '█')}           {rgb(0, 240, 255, '█   █')}  {rgb(0, 240, 255, '█  █')}     {rgb(0, 240, 255, '█')}    {rgb(0, 240, 255, '█')}        {rgb(0, 240, 255, '█')}",
        f"   {rgb(56, 189, 248, '\\|')}                         {rgb(244, 63, 94, '|/')}     {rgb(56, 189, 248, '█   █')}  {rgb(56, 189, 248, '█')}      {rgb(56, 189, 248, '█████')}         {rgb(0, 240, 255, '████')}   {rgb(0, 240, 255, '█   █')}  {rgb(0, 240, 255, '█████')}  {rgb(0, 240, 255, '█')}        {rgb(0, 240, 255, '█')}",
        "",
        f"   {rgb(56, 189, 248, '[ snapshot ]')} {rgb(100, 116, 139, '──▶')} {rgb(56, 189, 248, '[ diff ]')} {rgb(100, 116, 139, '──▶')} {rgb(245, 158, 11, '[ classify ]')} {rgb(100, 116, 139, '──▶')} {rgb(16, 185, 129, '[ suggest ]')}   {rgb(148, 163, 184, 'v0.1.0 · MSRV 1.74 · zero-dep core')}",
        f"   {rgb(148, 163, 184, 'Contract drift detection, break classification & mechanical patch suggestions')}",
        f"   {rgb(100, 116, 139, 'https://github.com/nixpt/api-drift  ·  crates.io/crates/api-drift')}",
    ]

    content = "\n".join(lines) + "\n"
    output_path.write_text(content, encoding="utf-8")
    print(f"  [ANSI] Created {output_path.relative_to(BASE_DIR)}")


def main():
    BRANDING_DIR.mkdir(parents=True, exist_ok=True)
    print("Generating branding raster assets...")

    # Master Banner
    render_svg_to_png(
        BRANDING_DIR / "api-drift-banner.svg",
        BRANDING_DIR / "api-drift-banner-1200.png",
        width=1200, height=420
    )

    # Master Logo Lockup
    render_svg_to_png(
        BRANDING_DIR / "api-drift-logo.svg",
        BRANDING_DIR / "api-drift-logo.png",
        width=860, height=220
    )

    # Standalone Mark
    render_svg_to_png(
        BRANDING_DIR / "api-drift-mark.svg",
        BRANDING_DIR / "api-drift-mark.png",
        width=256, height=256
    )

    # App Icons in various resolutions
    icon_svg = BRANDING_DIR / "api-drift-icon.svg"
    for size in (512, 128, 64, 32):
        render_svg_to_png(
            icon_svg,
            BRANDING_DIR / f"api-drift-icon-{size}.png",
            width=size, height=size
        )

    # TrueColor ANSI banner
    generate_ansi_banner(BRANDING_DIR / "api-drift-ansi.txt")

    print("\nAll api-drift branding assets generated successfully.")


if __name__ == "__main__":
    main()
