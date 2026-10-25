#!/usr/bin/env python3
"""Tests for scripts/api_boundary.py (ALICE-Law G7 boundary scan)."""
import os
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
SCRIPT = os.path.join(HERE, "api_boundary.py")


def run(root, allowlist):
    p = subprocess.run(
        [sys.executable, SCRIPT, root, allowlist],
        capture_output=True,
        text=True,
    )
    return p.returncode, p.stdout + p.stderr


def fixture(tmp, rs, allow_toml):
    os.makedirs(os.path.join(tmp, "src"))
    with open(os.path.join(tmp, "src", "lib.rs"), "w", encoding="utf-8") as fh:
        fh.write(rs)
    allow = os.path.join(tmp, "allow.toml")
    with open(allow, "w", encoding="utf-8") as fh:
        fh.write(allow_toml)
    return allow


EMITTER = 'pub fn emit() -> Vec<u8> { alloc::vec::Vec::new() }\n'
CLASSIFIED = (
    '[[emitter]]\nfn = "emit"\nclass = "raw-same-build"\nreason = "fixture"\n'
)


class ApiBoundary(unittest.TestCase):
    def test_a_classified_emitter_passes(self):
        with tempfile.TemporaryDirectory() as tmp:
            allow = fixture(tmp, EMITTER, CLASSIFIED)
            code, out = run(tmp, allow)
            self.assertEqual(code, 0, out)
            self.assertIn("0 problem(s)", out)

    def test_an_unlisted_emitter_fails(self):
        with tempfile.TemporaryDirectory() as tmp:
            allow = fixture(tmp, EMITTER, "")  # empty allowlist
            code, out = run(tmp, allow)
            self.assertEqual(code, 1, out)
            self.assertIn("not classified", out)

    def test_no_emitter_fails(self):
        # the scan matching nothing is a failure (the patterns drifted), not a silent pass
        with tempfile.TemporaryDirectory() as tmp:
            allow = fixture(tmp, "pub fn not_bytes() -> u32 { 0 }\n", CLASSIFIED)
            code, out = run(tmp, allow)
            self.assertEqual(code, 1, out)
            self.assertIn("matched nothing", out)

    def test_an_allowlist_entry_with_no_emitter_fails(self):
        with tempfile.TemporaryDirectory() as tmp:
            allow = fixture(
                tmp,
                EMITTER,
                CLASSIFIED + '\n[[emitter]]\nfn = "ghost"\nclass = "stamped"\nreason = "none"\n',
            )
            code, out = run(tmp, allow)
            self.assertEqual(code, 1, out)
            self.assertIn("no such byte-emitting pub fn", out)

    def test_a_class_that_is_not_known_fails(self):
        with tempfile.TemporaryDirectory() as tmp:
            allow = fixture(
                tmp, EMITTER, '[[emitter]]\nfn = "emit"\nclass = "bogus"\nreason = "x"\n'
            )
            code, out = run(tmp, allow)
            self.assertEqual(code, 1, out)
            self.assertIn("no valid class", out)


if __name__ == "__main__":
    unittest.main()
