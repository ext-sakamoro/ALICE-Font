"""Test fonts for alice-sfnt (tests/fonts/), made here from drawn shapes: no third-party font data.

    uv run -q --no-project --with fonttools python3 crates/alice-sfnt/tests/fonts/make_test_fonts.py

quad.ttf   TrueType: A (triangle, lines), O (four quadratic arcs), space (empty), comp (composite: A moved and
           scaled by 1/2, plus O), cmap formats 4 + 12 (U+1F600 -> O) + 14 (A+VS1 default, O+VS1 -> comp)
cubic.otf  CFF: A (lines), O (four cubic arcs), space
cubic2.otf CFF2: the same glyphs as cubic.otf
pair.ttc   a collection of quad.ttf and cubic.otf

The outlines the tests expect are the shapes drawn below, written out in tests/oracle.rs by hand.
"""
import io
from pathlib import Path

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.t2CharStringPen import T2CharStringPen
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import TTCollection, TTFont

OUT = Path(__file__).resolve().parent
NAMES = {"familyName": "AliceSfntTest", "styleName": "Regular"}
METRICS = {".notdef": (500, 0), "A": (600, 20), "O": (700, 40), "space": (250, 0), "comp": (800, 0)}


def draw_a(pen):
    pen.moveTo((20, 0))
    pen.lineTo((300, 700))
    pen.lineTo((580, 0))
    pen.closePath()


def draw_o_quad(pen):
    pen.moveTo((350, 0))
    pen.qCurveTo((660, 0), (660, 350))
    pen.qCurveTo((660, 700), (350, 700))
    pen.qCurveTo((40, 700), (40, 350))
    pen.qCurveTo((40, 0), (350, 0))
    pen.closePath()


def draw_o_cubic(pen):
    pen.moveTo((350, 0))
    pen.curveTo((520, 0), (660, 160), (660, 350))
    pen.curveTo((660, 540), (520, 700), (350, 700))
    pen.curveTo((180, 700), (40, 540), (40, 350))
    pen.curveTo((40, 160), (180, 0), (350, 0))
    pen.closePath()


def common(fb, order, cmap, uvs=None):
    fb.setupCharacterMap(cmap, uvs=uvs)
    fb.setupHorizontalMetrics({g: METRICS[g] for g in order})
    fb.setupHorizontalHeader(ascent=800, descent=-200)
    fb.setupNameTable(NAMES)
    fb.setupOS2()
    fb.setupPost()


def quad() -> TTFont:
    order = [".notdef", "A", "O", "space", "comp"]
    fb = FontBuilder(1000, isTTF=True)
    fb.setupGlyphOrder(order)
    glyphs = {}
    for name, draw in ((".notdef", draw_a), ("A", draw_a), ("O", draw_o_quad), ("space", None)):
        pen = TTGlyphPen(None)
        if draw:
            draw(pen)
        glyphs[name] = pen.glyph()
    pen = TTGlyphPen(glyphs)
    pen.addComponent("A", (0.5, 0, 0, 0.5, 100, 50))
    pen.addComponent("O", (1, 0, 0, 1, 0, 0))
    glyphs["comp"] = pen.glyph()
    fb.setupGlyf(glyphs)
    cmap = {ord("A"): "A", ord("O"): "O", ord(" "): "space", 0x1F600: "O", ord("C"): "comp"}
    common(fb, order, cmap, uvs=[(ord("A"), 0xFE00, None), (ord("O"), 0xFE00, "comp")])
    return fb.font


def cff(cff2: bool) -> TTFont:
    order = [".notdef", "A", "O", "space"]
    fb = FontBuilder(1000, isTTF=False)
    fb.setupGlyphOrder(order)
    strings = {}
    for name, draw in ((".notdef", draw_a), ("A", draw_a), ("O", draw_o_cubic), ("space", None)):
        pen = T2CharStringPen(None if cff2 else METRICS[name][0], None, CFF2=cff2)
        if draw:
            draw(pen)
        strings[name] = pen.getCharString()
    if cff2:
        fb.setupCFF2(strings)
    else:
        fb.setupCFF("AliceSfntTest-Regular", {"FullName": "AliceSfntTest Regular"}, strings, {})
    common(fb, order, {ord("A"): "A", ord("O"): "O", ord(" "): "space"})
    return fb.font


def save(font, name) -> bytes:
    buf = io.BytesIO()
    font.save(buf)
    (OUT / name).write_bytes(buf.getvalue())
    return buf.getvalue()


def main():
    q = save(quad(), "quad.ttf")
    c = save(cff(False), "cubic.otf")
    save(cff(True), "cubic2.otf")
    ttc = TTCollection()
    ttc.fonts = [TTFont(io.BytesIO(q)), TTFont(io.BytesIO(c))]
    buf = io.BytesIO()
    ttc.save(buf)
    (OUT / "pair.ttc").write_bytes(buf.getvalue())
    for p in sorted(OUT.glob("*.tt[fc]")) + sorted(OUT.glob("*.otf")):
        print(p.name, p.stat().st_size)


if __name__ == "__main__":
    main()
