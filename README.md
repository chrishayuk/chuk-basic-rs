# chuk-basic-rs

BASIC in Rust: an interpreter that runs natively, on any WASI runtime and in
the browser, and a compiler that turns BASIC programs into WebAssembly.

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
- **Want something to run?** There are [150 sample programs](samples/README.md).
- **No terminal handy?** The [browser playground](web/README.md) runs the same
  interpreter in a web page.

## Quick start

```bash
cargo build --release

target/release/chuk-basic samples/classic/mandelbrot.bas
target/release/chuk-basic samples/games/hammurabi.bas
target/release/chuk-basic samples/games/superstartrek.bas
```

Or install it as `chuk-basic` with `cargo install --path .`.

```text
Usage: chuk-basic [OPTIONS] [FILE.bas]

Runs a BASIC program, or with no file starts the interactive prompt.

Options:
      --tokens         Print the token stream instead of running
      --ast            Print the syntax tree as JSON instead of running
      --compile        Compile to WebAssembly instead of running (FILE.wasm)
  -o, --output <FILE>  Where --compile writes its output
      --seed <N>       Seed the random number generator (repeatable runs)
      --max-steps <N>  Stop with an error after N operations
      --echo           Echo INPUT lines to the output (for piped input)
      --dialect <NAME> Arithmetic to follow: microsoft (default) or dartmouth
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

## Interactive mode

Run `chuk-basic` with no file for the prompt that BASIC always had: type
numbered lines to build a program, and commands to work with it.

```text
$ chuk-basic
CHUK-BASIC 0.1.0 - type HELP for help
Ok
10 FOR I = 1 TO 3
20   PRINT I * I;
30 NEXT I
RUN
 1  4  9 
Ok
LIST 20
20   PRINT I * I;
Ok
PRINT SQR(2)
 1.41421356 
Ok
```

| | |
| --- | --- |
| `10 PRINT "HI"` | store line 10; typing it again replaces it, and `10` alone deletes it |
| `LIST`, `LIST 10-50` | show the program, or part of it |
| `RUN` | run it |
| `NEW` | forget it |
| `LOAD "file"`, `SAVE "file"` | read or write a program |
| anything else | run at once, such as `PRINT 2 + 2` |
| `BYE` | leave |

## WebAssembly + WASI

There are three ways to WebAssembly: run the interpreter on a WASI runtime,
run it in a browser, or compile a BASIC program to a module of its own.

### The interpreter on WASI

The same crate builds for `wasm32-wasip1`; the result is a 760 KB module.

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

### In the browser

[`web/`](web/README.md) is a static page - editor, output, an input line -
that runs the WASI build with a small WASI shim written for it. It offers
142 of the sample programs, the games included, and each one that reads no
input is checked against the native binary. No server side, no dependencies:

```bash
web/build.sh && web/serve.sh        # then open http://localhost:8000
```

### Compiling BASIC to WebAssembly

`--compile` turns a program into a WebAssembly module of its own:

```bash
chuk-basic --compile samples/games/superstartrek.bas     # writes superstartrek.wasm
wasmtime samples/games/superstartrek.wasm
```

The module is a self-contained WASI command that needs no access to the
file system. It accepts `--seed`, `--echo` and `--max-steps` like the
interpreter.

The program's arithmetic and control flow become WebAssembly instructions.
Strings, `PRINT` and `INPUT`, arrays and the built-in functions are calls to
a runtime library - the interpreter's own code, built as a module (250 KB)
that the compiler copies into every program it produces. Because both share
that code, a compiled program prints exactly what the interpreter prints;
the test suite compiles all 170 sample and test programs and holds the two
to that, down to the wording and the moment of every error message.

Counting the primes below 200,000 by trial division:

| | Time |
| --- | --- |
| Interpreter, native | 4.6 s |
| Interpreter, on wasmtime | 12.4 s |
| Compiled to WebAssembly, on wasmtime | 1.3 s |

Programs that spend their time in strings or `PRINT` gain less, since that
work is the same runtime code either way.

The runtime library is checked in as `runtime/chuk_basic_rt.wasm`. After
changing `runtime/` or `src/rt/`, rebuild it with `scripts/build-runtime.sh`
and then rebuild `chuk-basic`, which embeds it.

## The language

### Statements

| Statement | Example |
| --- | --- |
| `LET` (the keyword is optional) | `LET X = 5` &nbsp; `N$ = "ADA"` &nbsp; `A(I, J) = 0` |
| `PRINT`, or `?` | `PRINT "TOTAL:"; T, TAB(30); N$` |
| `PRINT USING` | `PRINT USING "$$#,###.##"; 1234.5` prints ` $1,234.50` |
| `INPUT` | `INPUT "YOUR NAME"; N$` &nbsp; `INPUT A, B` |
| `LINE INPUT` | `LINE INPUT "ADDRESS: "; A$` - a whole line, commas and all |
| `IF` ... `THEN` ... `ELSE` | `IF X > 0 THEN PRINT "POS" ELSE PRINT "NOT"` &nbsp; `IF X = 0 THEN 200` |
| `FOR` ... `TO` ... `STEP`, `NEXT` | `FOR I = 10 TO 1 STEP -1` ... `NEXT I` |
| `WHILE` ... `WEND` | `WHILE N > 1` ... `WEND` |
| `GOTO` (or `GO TO`) | `GOTO 100` |
| `GOSUB` (or `GO SUB`), `RETURN` | `GOSUB 500` ... `RETURN` |
| `ON` ... `GOTO` / `GOSUB` | `ON N GOTO 100, 200, 300` |
| `DIM` | `DIM A(20), B$(5), G(8, 8)` |
| `READ`, `DATA`, `RESTORE` | `READ N, N$` &nbsp; `DATA 3, "A, B", C` &nbsp; `RESTORE 500` |
| `DEF` | `DEF FNA(X, Y) = X * Y` |
| `DEF` ... `FNEND` | a function over several lines; assign the result to its name |
| `MAT` | `MAT READ A` &nbsp; `MAT C = A * B` &nbsp; `MAT I = INV(A)` &nbsp; `MAT PRINT C;` |
| `RANDOMIZE` | `RANDOMIZE` &nbsp; `RANDOMIZE 42` |
| `REM`, or `'` | `REM A COMMENT` |
| `END`, `STOP` | stop the program |

Several statements can share a line, separated by colons.

### Functions

| | |
| --- | --- |
| Numeric | `ABS` `INT` `FIX` `SGN` `SQR` `EXP` `LOG` `SIN` `COS` `TAN` `ATN` `RND` |
| String | `LEN` `ASC` `VAL` `INSTR` `CHR$` `STR$` `LEFT$` `RIGHT$` `MID$` `SPACE$` `STRING$` |
| In `PRINT` | `TAB(n)` `SPC(n)` |

### PRINT USING

The picture is made of fields and ordinary text:

| In the picture | Prints |
| --- | --- |
| `#` | a digit position; `.` marks the decimal point |
| `,` before the point | commas between thousands |
| `+` before or after | the sign, always; `-` after gives a trailing minus |
| `**`, `$$`, `**$` | leading asterisks, a dollar sign, or both |
| `^^^^` | exponent form |
| `!`, `\  \`, `&` | the first character of a string, a fixed width, or all of it |
| `_x` | the character `x` itself |

A number too big for its field is printed in full after a `%`. These rules
were checked against PC-BASIC on fifty pictures.

### MAT

The Dartmouth matrix statements work on ordinary arrays, from subscript 1
(row 0 and column 0 are left alone). A one-dimensional array is a column
vector.

| | |
| --- | --- |
| `MAT READ A, B(2, 3)` | fill from `DATA`, row by row; dimensions may be given |
| `MAT INPUT A` | one line of numbers for each row |
| `MAT PRINT A` &nbsp; `MAT PRINT A;` | print in zones, or packed |
| `MAT C = A` &nbsp; `A + B` &nbsp; `A - B` &nbsp; `A * B` &nbsp; `(K) * A` | copy, add, subtract, matrix product, scale |
| `MAT C = ZER` &nbsp; `CON` &nbsp; `IDN` &nbsp; `ZER(2, 3)` | zeros, ones, identity - in C's shape, or the one given |
| `MAT C = TRN(A)` &nbsp; `INV(A)` | transpose, inverse |

The target takes the shape of the result.

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
- **Source with no spaces**: a line that makes no sense as written is read
  again the way the first Microsoft BASICs read everything, finding keywords
  anywhere - so `FORI=1TO10:PRINTI:NEXTI` works. (On such a line `TOTAL`
  would be `TO TAL`, which is why lines are read the spaced way first.)

### Compared with Dartmouth BASIC, 1964

`samples/dartmouth` holds the programs printed in the first BASIC
instruction manual (Dartmouth, May 1964), typed in from the
[scanned pages](docs/scans/README.md). The manual prints their results
too, and the tests compare ours with the page.

| Program | The 1964 page prints | chuk-basic prints |
| --- | --- | --- |
| Preface: `LET X = (7+8)/3` | | `5` |
| Example 1, average of 100 logs | `1.5797` | `1.57970004` |
| Example 3, root of a function | `ROOT = .837585` | `ROOT = .837585449` |
| Example 3 with `EXP(X) - 2` | `ROOT = .693176` | `ROOT = .69317627` |
| Example 2, quadratic equation | `IMAGINARY PART = 1.41421` | stops: `Illegal function call: SQR` |

Example 2 takes the square root of a negative number on line 40 and relies
on a rule the manual states on page 5: "the square-root is always taken of
the absolute value". Every later BASIC treats that as an error, and so does
chuk-basic unless asked otherwise:

```bash
chuk-basic --dialect dartmouth samples/dartmouth/example2.bas   # IMAGINARY PART= 1.41421356
```

`--dialect dartmouth` makes `SQR` and powers take the absolute value, as
the manual describes (`A ↑ B` "is actually interpreted as |A|^B"). The
upwards arrow is accepted for powers in any dialect. Compiled programs take
the same option.

### Compared with GW-BASIC

The interpreter has been run side by side with
[PC-BASIC](https://robhagemans.github.io/pcbasic/), an independent and
careful emulation of Microsoft GW-BASIC. The 22 programs in
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

- Statements from later, machine-specific BASICs: files, graphics and
  sound, `PEEK`/`POKE`, `ON ERROR`, `DEFINT` and the like.
- `MAT` extras from later Dartmouth editions: `DET`, and string matrices.
- In interactive mode, variables do not outlive a `RUN`.
- Compiled programs report a string used as a number (or the reverse) when
  the statement runs, as the interpreter does - not when compiling.

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
renders as JSON; `tokenize` returns the tokens. `wasm::compile(source,
runtime)` returns the bytes of a WebAssembly module, and `repl::repl` runs
the interactive prompt on any reader and writer.

## How it is put together

```text
                                                      +--> interp: execute   --> output
source --lexer--> tokens --parser--> tree --lower--> ops
                                                      +--> wasm: generate code --> module.wasm
```

```text
src/
  lexer/      mod.rs       the scanner (spaced and crunched)   token.rs  token types
  ast/        program.rs   stmt.rs   expr.rs   builtin.rs      json.rs  JSON rendering
  parser/     mod.rs       lines, with the crunched fallback
              stmt.rs      expr.rs   mat.rs    data.rs
  rt/         what a running program needs, shared by interpreter and compiled code:
              console.rs   PRINT's cursor and zones, INPUT
              arrays.rs    data.rs   builtins.rs   using.rs   mat.rs   format.rs   rng.rs
  interp/     lower.rs     flattens the tree into ops with resolved jumps
              exec.rs      the fetch-execute loop   control.rs   eval.rs   functions.rs
              vars.rs      io.rs   mat.rs   builtins.rs
  wasm/       module.rs    reads the runtime module and adds functions to it
              code.rs      instruction encoder      leb.rs
              gen.rs       expr.rs   stmt.rs   func.rs   code generation
  repl.rs     interactive mode
  error.rs    lib.rs       main.rs
runtime/      the runtime library for compiled programs (a thin layer over rt/)
web/          the browser playground
```

Both back ends start from the same lowered form, in which `IF` and `WHILE`
are conditional jumps and `NEXT J, I` is one operation per variable. The
interpreter walks it with a program counter. The compiler turns it into one
WebAssembly function: a loop around a jump table with a block per operation,
so that `GOTO` - which can go anywhere - is "set the counter and go round".

## Tests

```bash
cargo test
```

| Suite | What it checks |
| --- | --- |
| unit tests in `src/` | the lexer, the parser, the runtime pieces, the WebAssembly encoder and linker, interactive mode |
| `tests/language.rs` | every statement, function and error, source in and output out |
| `tests/golden.rs` | saved transcripts: the classic and tutorial programs, eight picture-drawing games, the conformance programs (whose output PC-BASIC agrees with) and the extensions |
| `tests/games.rs` | all 103 games parse and survive a player typing at random; Hi-Lo is beaten by binary search; Lunar Lander's free fall matches the physics worked out by hand |
| `tests/compiled.rs` | every program compiled to WebAssembly prints what the interpreter prints (needs wasmtime) |
| `tests/tutorial.rs` | every listing and output in the tutorial is real |
| `tests/samples.rs` | the original samples, including a full game of Basketball |
| `web/test.mjs` | the playground's WASI shim and session logic, and all 142 of its example programs, in node |

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
