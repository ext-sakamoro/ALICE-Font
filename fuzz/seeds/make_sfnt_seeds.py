"""Seeds for fuzz_sfnt_font (fuzz/seeds/fuzz_sfnt_font/): 4 bytes of little-endian code point, then a font.

The fonts are alice-sfnt's test fonts (crates/alice-sfnt/tests/fonts/, made by make_test_fonts.py from drawn
shapes: no third-party font data). The crash-* files next to the seeds are kept as they are: inputs that reached a
debug_assert in another reader (a collection whose font count runs past the data, and a cmap format 10 whose
glyph count does).

    python3 fuzz/seeds/make_sfnt_seeds.py
"""
import struct
from pathlib import Path

HERE = Path(__file__).resolve().parent
FONTS = HERE.parent.parent / "crates/alice-sfnt/tests/fonts"
OUT = HERE / "fuzz_sfnt_font"

SEEDS = {
    "quad-A": ("quad.ttf", "A"),
    "quad-O": ("quad.ttf", "O"),
    "quad-composite": ("quad.ttf", "C"),
    "quad-supplementary": ("quad.ttf", "\U0001F600"),
    "cubic-O": ("cubic.otf", "O"),
    "cubic2-O": ("cubic2.otf", "O"),
    "pair-A": ("pair.ttc", "A"),
}


def main():
    OUT.mkdir(exist_ok=True)
    for name, (font, ch) in SEEDS.items():
        data = struct.pack("<I", ord(ch)) + (FONTS / font).read_bytes()
        (OUT / name).write_bytes(data)
        print(name, len(data))


if __name__ == "__main__":
    main()
