# chuk-basic-rs

A BASIC interpreter written in Rust. It runs natively, and as WebAssembly on
any WASI runtime.

It speaks the line-numbered BASIC of the 1960s and 70s - Dartmouth BASIC,
Microsoft (Altair) BASIC and Vintage BASIC - well enough to run all 103
programs of *BASIC Computer Games* unmodified, Super Star Trek included.

```basic
10 PRINT "HELLO, WORLD!"
20 FOR I = 1 TO 3
30   PRINT I; "SQUARED IS"; I * I
40 NEXT I
```

```text
HELLO, WORLD!
 1 SQUARED IS 1 
 2 SQUARED IS 4 
 3 SQUARED IS 9 
```

This is the Rust port, and completion, of the Python `chuk-basic` project,
which had a tokenizer and a parser. No dependencies outside the standard
library.

- **New to BASIC?** Start with the [tutorial](docs/TUTORIAL.md).
- **Want something to run?** There are [145 sample programs](samples/README.md).

## Quick start

```bash
cargo build --release

target/release/chuk-basic samples/classic/mandelbrot.bas
target/release/chuk-basic samples/games/hammurabi.bas
target/release/chuk-basic samples/games/superstartrek.bas
```

Or install it as `chuk-basic` with `cargo install --path .`.

```text
Usage: chuk-basic [OPTIONS] <FILE.bas>

Options:
      --tokens         Print the token stream instead of running
      --ast            Print the syntax tree as JSON instead of running
      --seed <N>       Seed the random number generator (repeatable runs)
      --max-steps <N>  Stop with an error after N operations
      --echo           Echo INPUT lines to the output (for piped input)
  -h, --help           Show this help
  -V, --version        Show the version
```

`INPUT` reads from standard input, so a program can be played at the
keyboard or driven from a file:

```bash
chuk-basic --seed 42 --echo samples/classic/guess.bas < samples/classic/guess.in
```

The exit status is 0 when the program ends normally, 1 on a syntax or
runtime error (reported on standard error with its line number), and 2 for
bad command-line arguments.

## WebAssembly + WASI

The same crate builds for `wasm32-wasip1`; the result is a 330 KB module.

```bash
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1

wasmtime run --dir=. target/wasm32-wasip1/release/chuk-basic.wasm samples/games/lunar.bas
```

`--dir=.` gives the sandbox access to the current directory so that it can
read the `.bas` file; the program's `PRINT` and `INPUT` use the runtime's
standard output and input. Any WASI preview 1 runtime should work; wasmtime
is the one this is tested with.

Two checks keep the WASI build honest:

```bash
cargo test --target wasm32-wasip1   # the whole test suite, run inside wasmtime
scripts/wasi-check.sh               # every sample: native and WASI output must be identical
```

## The language

### Statements

| Statement | Example |
| --- | --- |
| `LET` (the keyword is optional) | `LET X = 5` &nbsp; `N$ = "ADA"` &nbsp; `A(I, J) = 0` |
| `PRINT`, or `?` | `PRINT "TOTAL:"; T, TAB(30); N$` |
| `INPUT` | `INPUT "YOUR NAME"; N$` &nbsp; `INPUT A, B` |
| `IF` ... `THEN` ... `ELSE` | `IF X > 0 THEN PRINT "POS" ELSE PRINT "NOT"` &nbsp; `IF X = 0 THEN 200` |
| `FOR` ... `TO` ... `STEP`, `NEXT` | `FOR I = 10 TO 1 STEP -1` ... `NEXT I` |
| `GOTO` (or `GO TO`) | `GOTO 100` |
| `GOSUB` (or `GO SUB`), `RETURN` | `GOSUB 500` ... `RETURN` |
| `ON` ... `GOTO` / `GOSUB` | `ON N GOTO 100, 200, 300` |
| `DIM` | `DIM A(20), B$(5), G(8, 8)` |
| `READ`, `DATA`, `RESTORE` | `READ N, N$` &nbsp; `DATA 3, "A, B", C` &nbsp; `RESTORE 500` |
| `DEF` | `DEF FNA(X, Y) = X * Y` |
| `DEF` ... `FNEND` | a function over several lines; assign the result to its name |
| `RANDOMIZE` | `RANDOMIZE` &nbsp; `RANDOMIZE 42` |
| `REM`, or `'` | `REM A COMMENT` |
| `END`, `STOP` | stop the program |

Several statements can share a line, separated by colons.

### Functions

| | |
| --- | --- |
| Numeric | `ABS` `INT` `FIX` `SGN` `SQR` `EXP` `LOG` `SIN` `COS` `TAN` `ATN` `RND` |
| String | `LEN` `ASC` `VAL` `CHR$` `STR$` `LEFT$` `RIGHT$` `MID$` |
| In `PRINT` | `TAB(n)` `SPC(n)` |

### Operators

Highest precedence first: `^`, unary `-`, `*` `/`, `+` `-`,
`=` `<>` `<` `<=` `>` `>=`, `NOT`, `AND`, `OR`. `+` also joins strings, and
the comparisons work on strings.

### The fine print

Where the old dialects disagree, this is what chuk-basic does:

- **Names** are not case-sensitive and may be any length. A name ending in
  `$` is a string; any other is a number. Scalars and arrays have separate
  namespaces (`A` and `A(1)` are unrelated).
- **Numbers** are 64-bit floats. They print with up to 9 significant digits,
  a leading space or minus sign, and a trailing space: `PRINT 1/3` gives
  ` .333333333 `.
- **Print zones** are 14 columns wide; `TAB` counts columns from 0.
- **Truth** is -1 and falsehood 0. `AND`, `OR` and `NOT` work bit by bit on
  integers, as in Microsoft BASIC.
- **`IF`**: when the condition is false, the rest of the line is skipped (up
  to an `ELSE`, if there is one).
- **`FOR`**: a loop whose range is empty is skipped altogether, as in
  Dartmouth and ANSI BASIC. (Microsoft BASIC would run it once.)
- **Arrays** start at subscript 0. Without a `DIM` they go up to 10.
  Subscripts are rounded to the nearest whole number.
- **`RND(x)`**: a positive `x` gives the next random number in [0, 1), 0
  repeats the last one, and a negative `x` reseeds. Each run is different
  unless `--seed` is given.
- **`INPUT`** asks again with `?REDO FROM START` after a bad number, and with
  `??` if more values are needed.
- **Lines** run in line-number order. Lines without numbers are allowed, and
  stay where they are written.
- **Old listings**: keywords glued to a value are understood (`FOR I=1TO8`,
  `IF A$="Y"THEN100`, `X>1ANDX<9`), any word starting with `REM` at the start
  of a statement is a comment (`REMARKABLE PROGRAM`), and a closing quote may
  be left off at the end of a line.

### Compared with GW-BASIC

The interpreter has been run side by side with
[PC-BASIC](https://robhagemans.github.io/pcbasic/), an independent and
careful emulation of Microsoft GW-BASIC. The 18 programs in
`tests/conformance` - expressions, loops, strings, `PRINT` layout, `INPUT`,
`READ`/`DATA`, arrays, functions - print byte-for-byte the same in both, and
so do most of the classic and tutorial samples. `scripts/pcbasic-diff.py`
repeats the comparison.

Where the two differ, it is on purpose. chuk-basic follows the earlier
Microsoft BASICs that the 1970s listings were written for:

| | chuk-basic | GW-BASIC |
| --- | --- | --- |
| `TAB(n)` | columns count from 0; no effect if already past | count from 1; moves to the next line if already past |
| Output width | unlimited | wraps at 80 columns |
| Precision | 64-bit, 9 digits printed (`1/3` is `.333333333`) | 32-bit, 7 digits (`.3333333`) |
| Division by zero | stops the program | prints a warning and carries on with the largest number |
| Extra values typed at `INPUT` | `?EXTRA IGNORED` | asks again |
| `STOP` | ends quietly | prints `Break in 140` |
| `PRINT 1 2 3` | three numbers | spaces inside a number are ignored: `123` |
| `REMARKABLE`, `DEF`...`FNEND`, long names containing keywords | accepted | syntax errors |

The `TAB` rule is not a matter of taste: `samples/games/diamond.bas` draws
its diamonds correctly here and breaks them across lines under GW-BASIC's
rule.

One consequence of 64-bit arithmetic: `.1 + .2 = .3` is false, as in most
modern languages, though the sum prints as `.3`. Compare fractions with a
tolerance, or work in whole numbers.

### Not supported

- Source with no spaces at all (`FORI=1TO10`, `IFA=BTHEN50`). A keyword must
  be a word on its own or glued to a number, a string or a bracket, so that
  names such as `TOTAL` and `SCORE` stay usable.
- The Dartmouth `MAT` matrix statements.
- Statements from later, machine-specific BASICs: `WHILE`/`WEND`, `PRINT
  USING`, `PEEK`/`POKE`, files, graphics and sound.
- An interactive editor (`LIST`, `RUN`, `NEW`): programs are files.

## Using it as a library

```rust
let mut output = Vec::new();
chuk_basic::run("10 PRINT 6 * 7", &mut std::io::empty(), &mut output, None).unwrap();
assert_eq!(output, b" 42 \n");
```

`run` takes any `BufRead` for the keyboard and any `Write` for the screen.
For more control, build an `Interpreter`:

```rust
use chuk_basic::{parse, Interpreter};

let program = parse("10 PRINT \"FOREVER\"\n20 GOTO 10").unwrap();
let (mut input, mut output) = (std::io::empty(), Vec::new());
let mut basic = Interpreter::new(program, &mut input, &mut output, Some(42)).unwrap();
basic.set_step_limit(Some(10_000)); // do not trust it to stop on its own
let error = basic.run().unwrap_err();
assert_eq!(error.to_string(), "Runtime error in line 10: Step limit exceeded");
```

`parse` returns the syntax tree (`chuk_basic::ast`), which `to_json()`
renders as JSON; `tokenize` returns the tokens.

## How it is put together

```text
source text --lexer--> tokens --parser--> syntax tree --compile--> ops --exec--> output
```

```text
src/
  lexer/      mod.rs       the scanner              token.rs  token types
  ast/        program.rs   stmt.rs   expr.rs        builtin.rs  the function table
              json.rs      JSON rendering
  parser/     mod.rs       lines and statement lists
              stmt.rs      one method per statement  expr.rs  precedence climbing
              data.rs      DATA item lists
  interp/     compile.rs   flattens the tree into ops with resolved jumps
              exec.rs      the fetch-execute loop    control.rs  FOR / NEXT / GOSUB
              eval.rs      expressions               builtins.rs  functions.rs (DEF FN)
              vars.rs      variables and arrays      io.rs  PRINT and INPUT
              format.rs    number formatting         rng.rs  value.rs
  error.rs    the error type
  lib.rs      run, parse, tokenize
  main.rs     the command line
```

`IF` is lowered to conditional jumps and each `NEXT J, I` to one op per
variable, so the executor is a flat loop over ops with a program counter;
`GOTO` is a table lookup.

## Tests

```bash
cargo test
```

| Suite | What it checks |
| --- | --- |
| unit tests in `src/` | the lexer, the parser, JSON output, number formatting |
| `tests/language.rs` | every statement, function and error, source in and output out |
| `tests/golden.rs` | the classic and tutorial programs, eight picture-drawing games, and the 18 conformance programs (whose output PC-BASIC agrees with), against saved transcripts |
| `tests/games.rs` | all 103 games parse and survive a player typing at random; Hi-Lo is beaten by binary search; Lunar Lander's free fall matches the physics worked out by hand |
| `tests/tutorial.rs` | every listing and output in the tutorial is real |
| `tests/samples.rs` | the original samples, including a full game of Basketball |

After a deliberate change in behaviour, `UPDATE_GOLDEN=1 cargo test --test
golden` rewrites the transcripts; review the diff before committing it.

## Credits

The games in `samples/games` are from David H. Ahl's *BASIC Computer Games*
(1978), via the public-domain
[coding-horror/basic-computer-games](https://github.com/coding-horror/basic-computer-games)
collection. See [samples/README.md](samples/README.md).

## Licence

MIT - see [LICENSE](LICENSE). The programs in `samples/games` are in the
public domain.
