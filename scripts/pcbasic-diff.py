#!/usr/bin/env python3
"""Differential test against PC-BASIC, an emulator of Microsoft GW-BASIC.

Runs each program through chuk-basic and through PC-BASIC, and reports
whether the two print the same thing:

    exact           identical, byte for byte
    trailing-space  identical but for spaces at the ends of lines
    wrapped-80col   identical once PC-BASIC's wrapping at 80 columns is undone
    spacing         same words and numbers, different spacing - in practice
                    TAB, which counts from 1 in GW-BASIC and from 0 here -
                    or numbers that agree to 6 significant digits
    DIFF            something else; the differing lines are shown

Setup (PC-BASIC is only needed for this script, not to build or test):

    python3 -m venv .venv && .venv/bin/pip install pcbasic
    cargo build --release
    .venv/bin/python scripts/pcbasic-diff.py tests/conformance/*.bas

What is compared is standard output. From PC-BASIC's side, line endings are
made LF, the 0xFF bytes it writes are removed, and the "Ok" prompt and blank
lines after the program's last output are dropped; nothing else is changed.
Neither side's error messages on standard error, nor the exit status, are
compared.

A program `name.bas` is given `name.in` as keyboard input if it exists.
Programs that use RND cannot match (the generators differ), and GW-BASIC has
no FNEND and reserves more words (NAME, for one).
"""
import os
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OURS = os.path.join(ROOT, "target", "release", "chuk-basic")
PCBASIC = shutil.which("pcbasic") or os.path.join(os.path.dirname(sys.executable), "pcbasic")
NUMBER = re.compile(r"-?(?:\d+\.?\d*|\.\d+)(?:E[+-]?\d+)?")


def run_ours(path, keys):
    done = subprocess.run([OURS, "--seed", "42", "--echo", path], input=keys.encode(), capture_output=True, timeout=60)
    return done.stdout.decode(errors="replace")


def run_pcbasic(path, keys):
    source = open(path, encoding="latin-1").read().replace("\r\n", "\n")
    with tempfile.TemporaryDirectory() as work:
        with open(os.path.join(work, "P.BAS"), "w", newline="\r\n") as copy:
            copy.write(source)
        # typed lines end in a carriage return; SYSTEM leaves the interpreter
        typed = (keys + "SYSTEM\n").replace("\n", "\r").encode()
        done = subprocess.run([PCBASIC, "-n", "P.BAS"], cwd=work, input=typed, capture_output=True, timeout=120)
    lines = done.stdout.decode("latin-1").replace("\r\n", "\n").replace("\xff", "").split("\n")
    while lines and lines[-1].strip() in ("", "Ok", "SYSTEM"):
        lines.pop()
    return "\n".join(lines) + "\n"


def lines_of(text):
    return [line.rstrip() for line in text.rstrip("\n").split("\n")]


def close(ours, theirs):
    """Same words; numbers equal to about 6 significant digits; any spacing."""
    ours, theirs = " ".join(ours.split()), " ".join(theirs.split())
    words = lambda text: [word.strip() for word in NUMBER.split(text)]
    a, b = NUMBER.findall(ours), NUMBER.findall(theirs)
    if words(ours) != words(theirs) or len(a) != len(b):
        return False
    return all(abs(float(x) - float(y)) <= 2e-6 * max(abs(float(x)), abs(float(y)), 1e-30) for x, y in zip(a, b))


def compare(path):
    keys_file = os.path.splitext(path)[0] + ".in"
    keys = open(keys_file).read() if os.path.exists(keys_file) else ""
    ours_text, theirs_text = run_ours(path, keys), run_pcbasic(path, keys)
    if ours_text == theirs_text:
        return "exact", []
    ours, theirs = lines_of(ours_text), lines_of(theirs_text)
    if ours == theirs:
        return "trailing-space", []
    differing = []
    for number in range(max(len(ours), len(theirs))):
        a = ours[number] if number < len(ours) else "<nothing>"
        b = theirs[number] if number < len(theirs) else "<nothing>"
        if a != b and not close(a, b):
            differing.append((number + 1, a, b))
    if not differing:
        return "spacing", []
    if close(" ".join(ours), " ".join(theirs)):
        return "wrapped-80col", []
    return "DIFF", differing


def main(paths):
    if not paths:
        sys.exit(__doc__)
    for needed in (OURS, PCBASIC):
        if not os.path.exists(needed):
            sys.exit(f"{needed} not found - see the setup notes:\n{__doc__}")
    totals = {}
    for path in paths:
        verdict, differing = compare(path)
        totals[verdict] = totals.get(verdict, 0) + 1
        print(f"{verdict:14} {path}")
        for number, a, b in differing[:8]:
            print(f"    line {number}\n      chuk-basic: {a[:120]!r}\n      PC-BASIC:   {b[:120]!r}")
    print(", ".join(f"{count} {verdict}" for verdict, count in sorted(totals.items())))
    sys.exit(1 if "DIFF" in totals else 0)


if __name__ == "__main__":
    main(sys.argv[1:])
