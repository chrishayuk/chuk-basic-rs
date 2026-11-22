#!/usr/bin/env python3
"""Check that every BASIC program in the tutorial prints what the tutorial says.

A ```basic block is a complete program. It must be followed by an ```output
block, optionally with an ```input block (what is typed at the keyboard) in
between. Each program is run through the interpreter and its output compared.
A ```repl block is a session at the interactive prompt: its lines are typed
at `chuk-basic` with no file, and the ```output block after it is everything
the prompt printed.

    python3 check_tutorial.py            check
    python3 check_tutorial.py --write    rewrite the output blocks from real runs
    python3 check_tutorial.py --wasm     also compile each program to WebAssembly
                                         and check that the module prints the same
                                         (needs wasmtime)

Programs run with `--seed 42 --echo`, so that random numbers are repeatable
and typed answers appear in the transcript. Words after the language on the
fence line change that for one program:

    ```basic dialect=dartmouth      run with --dialect dartmouth
    ```basic seed=7                 run with --seed 7

Listings that are not complete programs go in ```text blocks, which are not
run. The interpreter is taken from $CHUK_BASIC, or from ../target.
"""
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
BLOCK = re.compile(r"```(\w+)([^\n]*)\n(.*?)```", re.S)
STEPS = "50000000"


def interpreter():
    if os.environ.get("CHUK_BASIC"):
        return os.environ["CHUK_BASIC"]
    for build in ("release", "debug"):
        path = HERE.parent / "target" / build / "chuk-basic"
        if path.exists():
            return str(path)
    sys.exit("no interpreter found: run `cargo build --release` or set $CHUK_BASIC")


def options(info):
    """The run options named on a fence line, as command-line arguments."""
    settings = dict(word.split("=", 1) for word in info.split() if "=" in word)
    args = ["--seed", settings.get("seed", "42"), "--echo", "--max-steps", STEPS]
    if "dialect" in settings:
        args += ["--dialect", settings["dialect"]]
    return args


def finish(result):
    """What the person at the terminal would have seen."""
    return result.stdout + (result.stderr if result.returncode != 0 else "")


def run(basic, program, stdin="", info=""):
    """Run a program; returns everything it printed, error message included."""
    with tempfile.TemporaryDirectory() as folder:
        source = pathlib.Path(folder) / "program.bas"
        source.write_text(program)
        return finish(subprocess.run([basic, *options(info), str(source)], input=stdin,
                                     capture_output=True, text=True, timeout=60))


def run_wasm(basic, program, stdin="", info=""):
    """Compile a program to WebAssembly and run the module in wasmtime."""
    with tempfile.TemporaryDirectory() as folder:
        source = pathlib.Path(folder) / "program.bas"
        module = pathlib.Path(folder) / "program.wasm"
        source.write_text(program)
        built = subprocess.run([basic, "--compile", "-o", str(module), str(source)],
                               capture_output=True, text=True, timeout=60)
        if built.returncode != 0:
            return built.stdout + built.stderr
        return finish(subprocess.run(["wasmtime", str(module), *options(info)], input=stdin,
                                     capture_output=True, text=True, timeout=120))


def session(basic, typed):
    """Type lines at the interactive prompt; returns everything it printed."""
    return finish(subprocess.run([basic], input=typed, capture_output=True, text=True,
                                 timeout=60))


def same(a, b):
    return a.rstrip("\n") == b.rstrip("\n")


def main():
    write = "--write" in sys.argv
    wasm = "--wasm" in sys.argv
    if wasm and not shutil.which("wasmtime"):
        sys.exit("--wasm needs wasmtime on the PATH")
    basic = interpreter()
    checked = compiled = failed = 0
    for path in sorted(HERE.glob("*.md")):
        text = path.read_text()
        blocks = list(BLOCK.finditer(text))
        edits = []
        for i, block in enumerate(blocks):
            kind, info, body = block.groups()
            if kind not in ("basic", "repl"):
                continue
            stdin, j = "", i + 1
            if kind == "basic" and j < len(blocks) and blocks[j].group(1) == "input":
                stdin, j = blocks[j].group(3), j + 1
            if j >= len(blocks) or blocks[j].group(1) != "output":
                print(f"{path.name}: program has no output block:\n{body}")
                failed += 1
                continue
            expected = blocks[j].group(3)
            actual = session(basic, body) if kind == "repl" else run(basic, body, stdin, info)
            checked += 1
            if not same(actual, expected):
                if write:
                    edits.append((blocks[j].start(3), blocks[j].end(3), actual.rstrip("\n") + "\n"))
                else:
                    failed += 1
                    print(f"{path.name}: output differs for:\n{body}"
                          f"--- tutorial says\n{expected}--- interpreter prints\n{actual}\n")
            if wasm and kind == "basic" and not write:
                module = run_wasm(basic, body, stdin, info)
                compiled += 1
                if not same(module, actual):
                    failed += 1
                    print(f"{path.name}: compiled module differs for:\n{body}"
                          f"--- interpreter prints\n{actual}--- module prints\n{module}\n")
        for start, end, new in reversed(edits):
            text = text[:start] + new + text[end:]
        if edits:
            path.write_text(text)
            print(f"{path.name}: rewrote {len(edits)} output block(s)")
    extra = f", {compiled} also compiled to WebAssembly" if wasm else ""
    print(f"{checked} programs checked{extra}, {failed} failed")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
