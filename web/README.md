# chuk-basic playground

A static web page that runs BASIC programs in the browser. The interpreter is
the same Rust crate as the command-line tool, compiled to WebAssembly + WASI
(`wasm32-wasip1`); the page supplies the WASI side in about 300 lines of
JavaScript. No build step for the page, no npm packages, nothing from a CDN.

Edit a program, press **Run** (or Ctrl+Enter), and answer its `INPUT`
prompts in the line under the output.

## Build and serve

```bash
rustup target add wasm32-wasip1   # once
web/build.sh                      # builds the wasm and copies it to web/chuk-basic.wasm
web/serve.sh                      # http://localhost:8000  (web/serve.sh 9000 for another port)
```

`serve.sh` is `python3 -m http.server` in this directory; any static file
server will do. Opening `index.html` from `file://` will not work: browsers
refuse to load module scripts and wasm from there.

`chuk-basic.wasm` is **not committed** (it is in `web/.gitignore`). To publish
the playground, run `web/build.sh` and upload the contents of `web/`.
`build.sh` also rebuilds `examples/` from `samples/`.

## Files

| File | What it is |
| --- | --- |
| `index.html`, `style.css`, `app.js` | the page |
| `wasi.js` | a small WASI preview 1 host: arguments, an in-memory file system holding the program, captured stdout/stderr, stdin from a list of lines. No DOM, so it also runs under node |
| `session.js` | `runProgram` (one run, with a status) and `Session` (the re-run trick below). Shared by the page and the tests |
| `worker.js` | runs `runProgram` off the main thread |
| `examples/` | programs for the drop-down and `index.json` listing them, written by `build-examples.mjs` from `samples/` and `featured.json` |
| `test.mjs` | tests, under node |
| `build.sh`, `serve.sh` | as above |

The drop-down offers 142 programs in groups: the four from the 1964 manual,
nine to start with, the tutorial's fourteen, seventeen classics and the rest
of the 103 games. `build-examples.mjs` gathers them: the programs named in
`featured.json` come first, with the titles, manual pages and credits written
there, then everything else in `samples/tutorial`, `samples/classic` and
`samples/games`. Whether a program asks for input is found by running it.
To feature a program, add it to `featured.json` and run `web/build.sh`.

Add `?example=hammurabi` (or `?example=games/animal`) to the address to open
a particular program.

## How INPUT works: run it again

A WASI program reads its input with a blocking call, and a browser cannot
block: there is no way to pause the wasm in the middle of `fd_read`, wait for
the user to type, and carry on. (It can be done with `SharedArrayBuffer` and
`Atomics.wait`, but those need special HTTP headers that a plain static host
does not send.)

So the program is never paused. Instead:

1. The page keeps the list of lines typed so far - at first, none - and one
   random-number seed for the whole session.
2. It runs the program from the start: a fresh instance of the wasm, with
   `--seed` and the lines as standard input.
3. When the program reads past the end of the list, the host's `fd_read`
   throws an exception, which abandons the run. The page shows what was
   printed up to there and enables the input line.
4. When a line is entered it is added to the list, and the program is run
   again from the start.

Because the seed and the input are the same, each run repeats the previous
one exactly and then gets one `INPUT` further, so what you see is one
continuous session. **Run** starts a new session: an empty list and a new
random seed, or the one in the Seed box if it is filled in.

The interpreter is run without `--echo`. The host notes where in the output
each line was read, and the page puts the typed line back there (in colour),
which gives the same text as `--echo` would - the tests check that against
the native binary.

### Limits

- **The program must be deterministic** given its seed and input, which a
  BASIC program here always is: there is no clock, no files, and `RND` and
  `RANDOMIZE` both come from `--seed`.
- **The cost grows with the session.** Line *n* costs a replay of all *n*
  lines, so a session is quadratic overall. In practice this is hard to
  notice: 300 commands of Super Star Trek replay in about 0.1 s.
- **The step limit covers the whole replay**, not just the latest turn. Each
  run gets `--max-steps 20000000` (about a second of work), so a program that
  loops for ever is stopped and reported - but so, eventually, would be a
  very long session of a compute-heavy program. Likewise the output of one run
  is capped at 2 MB (a program printing in an endless loop reaches that
  before the step limit). Both are constants at the top of `session.js`.
- **Editing the program does not affect a session in progress**; the session
  keeps the text it was started with. Press Run to use the new text.
- There is no end-of-input: a program is either finished or waiting.

Programs run in a web worker, so the page stays responsive and **Stop** can
end a run by terminating the worker. If the worker cannot be started, the
page runs programs on the main thread instead; then the page freezes for the
duration of a run (at most about a second, because of the step limit) and
Stop only works between runs.

The **CAPS** box (on by default) upper-cases what you type, because most of
the old games only understand `YES`, not `yes`.

## Tests

```bash
web/build.sh && cargo build --release
node web/test.mjs
```

`test.mjs` uses the page's own `wasi.js` and `session.js` and compares with
the native binary (`target/release/chuk-basic`): programs without input print
the same bytes; `guess.bas` fed a line at a time through `Session` ends with
`YOU GOT IT` and matches native `--echo` output; a runtime error and a syntax
error are reported with their messages; `10 GOTO 10` hits the step limit and
endless printing the output limit; every bundled example starts.

The page itself (`app.js`, `worker.js`, the layout) has no automated test.
