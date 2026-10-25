#!/usr/bin/env python3
"""ALICE-Law G7: every public function that emits a byte buffer across a boundary is classified.

A Law-bearing value that leaves the crate as bytes must either carry the Law identifier (a stamped
form, `law_stamp`) or be a raw buffer whose only consumer is this crate's own shader of the *same
build* (the std140 UBO writers: the GPU reads them at a fixed layout, with no header). This scan
lists every `pub fn` whose return type is a byte buffer (`Vec<u8>` or `Vec<Vec<u8>>`) under `src/`
and checks each against `scripts/api_boundary_allowlist.toml`, which gives every one a class and a
reason. A new byte emitter that is not listed fails here, so a serializer cannot ship unstamped and
unclassified. Scanning nothing fails too (the grep found no emitter: the patterns drifted).

Run: python3 scripts/api_boundary.py
"""
import os
import re
import sys

try:
    import tomllib  # Python 3.11+
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib  # type: ignore

DEFAULT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# a pub fn returning a byte buffer; the signature may wrap across lines before `->`
SIG = re.compile(
    r"pub\s+fn\s+(\w+)\s*(?:<[^>]*>)?\s*\([^;{]*?\)\s*->\s*(Vec<u8>|Vec<\s*Vec<u8>\s*>)",
    re.S,
)

CLASSES = {"raw-same-build", "stamped"}


def rs_files(src):
    for dirpath, _dirs, files in os.walk(src):
        for f in files:
            if f.endswith(".rs"):
                yield os.path.join(dirpath, f)


def strip_block_comments(text):
    # good enough to avoid matching a signature quoted in a doc comment: blank // lines
    out = []
    for line in text.splitlines():
        if line.lstrip().startswith("//"):
            out.append("")
        else:
            out.append(line)
    return "\n".join(out)


def main(root=None, allowlist=None):
    root = root or DEFAULT_ROOT
    allowlist = allowlist or os.path.join(root, "scripts", "api_boundary_allowlist.toml")
    with open(allowlist, "rb") as fh:
        allow = tomllib.load(fh)
    entries = {e["fn"]: e for e in allow.get("emitter", [])}
    for name, e in entries.items():
        if e.get("class") not in CLASSES:
            print(f"api_boundary: allowlist entry {name!r} has no valid class", file=sys.stderr)
            return 1
        if not e.get("reason"):
            print(f"api_boundary: allowlist entry {name!r} has no reason", file=sys.stderr)
            return 1

    found = {}
    src = os.path.join(root, "src")
    for path in rs_files(src):
        text = strip_block_comments(open(path, encoding="utf-8").read())
        for m in SIG.finditer(text):
            fn = m.group(1)
            found.setdefault(fn, []).append(os.path.relpath(path, root))

    if not found:
        print("api_boundary: no byte-emitting pub fn found (the scan matched nothing)", file=sys.stderr)
        return 1

    problems = 0
    for fn, paths in sorted(found.items()):
        if fn not in entries:
            print(
                f"api_boundary: {fn} (in {', '.join(sorted(set(paths)))}) emits bytes but is not "
                f"classified in scripts/api_boundary_allowlist.toml",
                file=sys.stderr,
            )
            problems += 1
        else:
            print(f"api_boundary: {fn}: {entries[fn]['class']} ({entries[fn]['reason']})")

    for name in entries:
        if name not in found:
            print(
                f"api_boundary: allowlist lists {name!r} but no such byte-emitting pub fn exists",
                file=sys.stderr,
            )
            problems += 1

    print(f"api_boundary: {len(found)} byte-emitting pub fn(s), {problems} problem(s)")
    return 1 if problems else 0


if __name__ == "__main__":
    # optional: api_boundary.py [ROOT [ALLOWLIST]]
    arg_root = sys.argv[1] if len(sys.argv) > 1 else None
    arg_allow = sys.argv[2] if len(sys.argv) > 2 else None
    sys.exit(main(arg_root, arg_allow))
