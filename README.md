# chuk-basic-rs

BASIC in Rust: an interpreter that runs natively, on any WASI runtime and in
the browser, and a compiler that turns BASIC programs into WebAssembly.

It speaks the line-numbered BASIC of the 1960s and 70s - Dartmouth BASIC,
Microsoft (Altair) BASIC and Vintage BASIC - well enough to run all 103
programs of *BASIC Computer Games*, Super Star Trek included, without
changing them. They are the listings as the coding-horror collection has
them, which Lyle Kopnicky normalised to run under Vintage BASIC; they have
not been checked against the printed book.

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

- **New to BASIC?** Start with the [tutorial](tutorial/01-introduction.md): 22
  chapters, and every program in them is run and checked.
- **Want something to run?** There are [150 sample programs](samples/README.md).
- **No terminal handy?** The [playground](https://chrishayuk.com/demos/basic)
  runs the same interpreter in a web page ([how it works](web/README.md)).
- **Writing BASIC?** There is a [VS Code extension](editors/vscode/README.md)
  with highlighting and a language server; see [Install](#install).

## Install

You need [Rust](https://rustup.rs). One command installs the interpreter,
the compiler and the language server, which are all the one program:

```bash
cargo install --git https://github.com/chrishayuk/chuk-basic-rs chuk-basic
```

That puts `chuk-basic` in `~/.cargo/bin`. Then:

```bash
echo '10 PRINT "HELLO, WORLD!"' > hello.bas
chuk-basic hello.bas
```

For Visual Studio Code, download `chuk-basic-0.1.3.vsix` from the
[latest release](https://github.com/chrishayuk/chuk-basic-rs/releases/latest)
and install it:

```bash
code --install-extension chuk-basic-0.1.3.vsix
```

Open any `.bas` file and you get highlighting, problems as you type, hover
help, go to a `GOTO`'s line, renumbering and a run button. The extension
carries its own copy of the interpreter and language server, so it works
without the `cargo install` above. To build it yourself, and for Neovim,
Helix and other editors, see [`editors/README.md`](editors/README.md).

Or try BASIC with nothing installed, in the
[playground](https://chrishayuk.com/demos/basic), beside the pages of the
1964 manual. Its program box is checked by the same language server,
running in the page.

## Build and run

To work on the interpreter itself, clone the repository and:

```bash
cargo build --release

target/release/chuk-basic samples/classic/mandelbrot.bas
target/release/chuk-basic samples/games/hammurabi.bas
target/release/chuk-basic samples/games/superstartrek.bas
```

```text
Usage: chuk-basic [OPTIONS] [FILE.bas]

Runs a BASIC program, or with no file starts the interactive prompt.

Options:
      --tokens         Print the token stream instead of running
      --ast            Print the syntax tree as JSON instead of running
      --compile        Compile to WebAssembly instead of running (FILE.wasm)
      --check          List the mistakes that can be found without running
      --renumber       Print the program renumbered in tens
      --lsp            Be a language server for an editor, on stdin and stdout
  -o, --output <FILE>  Where --compile writes its output
      --seed <N>       Seed the random number generator (repeatable runs)
      --max-steps <N>  Stop with an error after N operations
      --echo           Echo INPUT lines to the output (for piped input)
      --dialect <NAME> BASIC to follow: microsoft (default) or dartmouth
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
CHUK-BASIC 0.1.3 - type HELP for help
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
that runs the WASI build with a small WASI shim written for it. Its
program box is coloured and checked by the language server, in the page. It offers
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
  ` .333333333 `. Whole numbers print in full up to 15 digits (`2^40` is
  ` 1099511627776 `, where GW-BASIC's single precision gives `1.099512E+12`).
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
  unless `--seed` is given; then a `RANDOMIZE` without a number reseeds
  from the generator rather than the clock, so the run repeats exactly.
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
[scanned pages](docs/scans/README.md). Two characters in them are our
readings of a damaged page (see [Readings and corrections](#readings-and-corrections)).
The manual prints their results too, and the tests compare the figures
and labels with the page's; the layout of the page is not reproduced.

| Program | The 1964 page prints | chuk-basic prints | with `--dialect dartmouth` |
| --- | --- | --- | --- |
| Preface: `LET X = (7+8)/3` | | `5` | `5.` |
| Example 1, average of 100 logs | `1.5797` | `1.57970004` | `1.5797` |
| Example 3, root of a function | `ROOT = .837585` | `ROOT = .837585449` | `ROOT = .837585` |
| Example 3 with `EXP(X) - 2` | `ROOT = .693176` | `ROOT = .69317627` | `ROOT = .693176` |
| Example 2, quadratic equation | `ROOTS = 3.  -1.`, `IMAGINARY PART = 1.41421` | stops: `Illegal function call: SQR` | `ROOTS = 3.  -1.`, `IMAGINARY PART= 1.41421` |

Example 2 takes the square root of a negative number on line 40 and relies
on a rule the manual states on page 5: "the square-root is always taken of
the absolute value". The later BASICs tried here (GW-BASIC, as PC-BASIC emulates it) treat that as an error, and so does
chuk-basic unless asked otherwise:

```bash
chuk-basic --dialect dartmouth samples/dartmouth/example2.bas   # IMAGINARY PART= 1.41421
```

`--dialect dartmouth` changes seven things. From the manual:

- `SQR` takes the absolute value of its argument, as page 5 says.
- So does a power: `A ↑ B` "is actually interpreted as |A|^B".
- When `READ` finds no data left, "the computer stops" (page 15): the
  program ends without an error message, as the results on page 22 show.
- `PRINT` shows a number as the results on pages 21 to 23 do: six
  significant digits for a number that is not whole, and a point after a
  whole number (`3.`). The beginners' part of the manual states no rule for
  this; it is copied from those pages. They print no whole number of more
  than one digit, so a long one is printed in full (`1234567.`), which is
  our choice.

The manual leaves `INT` and `RND` to its Part II, which Dartmouth's reprint
does not include, and does not say how a leading minus binds. For these
the dialect follows Thomas Kurtz's 2005 account of
[the original BASIC](https://www.dartmouth.edu/basicfifty/commands.html):

- `INT` is "the integer part (truncating toward 0)": `INT(-1.5)` is -1,
  not -2.
- The argument of `RND` "is ignored": `RND(0)` gives the next number
  rather than repeating the last.
- "-x^2 is understood to be (-x)^2", so `-2^2` is 4. With brackets,
  `-(2^2)`, it is -4 in either dialect.

It does not make the interpreter refuse what later BASICs added (`INPUT`,
strings, `ELSE`). The upwards arrow is accepted for powers in any dialect.
Compiled programs take the same option. Where the 1964 system's behaviour
is documented nowhere - the digits of its 30-bit arithmetic, its random
numbers, its print zones (Kurtz gives 15 columns; this keeps 14) - a 1964
program can still print something different here.

### Readings and corrections

What this repository changes or chooses in reading its sources, by kind:
a *slip* is wrong under any reading of the source; a *conflict* is where
the sources say two things; an *interpretation* is our reading of what was
meant; *ours* is a consequence of choices made in this reconstruction.

| Where | What | Kind |
| --- | --- | --- |
| Manual p. 21, example 2, line 50 | The relation is smudged. It is typed as `IF D = 0`: of the six relations the manual allows (p. 12), only `=` gives the results printed on p. 22. | interpretation |
| Manual p. 23, example 3, line 160 | A damaged glyph that looks like ≤, typed as `<=`. `<` would print the same for both functions. | interpretation |
| Manual p. 22, results | Prints `READ PART`; the program on the same page says `"REAL PART="`. We follow the program. | conflict |
| Manual p. 22, results | Spaces before `=` that the program's strings lack; columns that follow no single zone width; `TIME:` lines. Not reproduced. | ours |
| Manual p. 4 and Kurtz | The manual counts fourteen operations ("13 other" and `END`); Kurtz lists fifteen statements. The 1964 checks use Kurtz's list. | conflict |
| Manual pp. 21-23 | Six significant digits, and a point after a whole number, inferred from the printed results; whole numbers in full. | interpretation; ours for long whole numbers |
| Manual p. 15 | Running out of data ends the program quietly. | interpretation |
| Kurtz, 2005 | `INT`, `RND` and the leading minus, taken as describing the system of May 1964, about which the manual is silent. | interpretation |
| `samples/games/splat.bas` | `GOTO 540`, and no line 540. Present in the coding-horror listing; the printed book has not been checked. | slip (in that listing) |
| `samples/games/diamond.bas` | Draws correctly only with `TAB` counting from 0 and never moving back. Not a defect in the listing: a compatibility choice. | ours |

Nothing else in the transcriptions is changed. Example 3's variant really
does write `DEF FNF(Z) = EXP(X) -2`, using `X` rather than its argument
`Z`; it works because `X` is the variable the program is changing, and it
is typed as printed.

In November 2026 a second model, OpenAI's Codex, reviewed this repository
independently: it built and ran everything and checked twenty places against
the scans. What it found and what was done about each finding is in
[`docs/independent-review.md`](docs/independent-review.md); several rows of
the table above come from it.

### Compared with GW-BASIC

The interpreter has been run side by side with
[PC-BASIC](https://robhagemans.github.io/pcbasic/), an independent and
careful emulation of Microsoft GW-BASIC. The 22 programs in
`tests/conformance` - expressions, loops, strings, `PRINT` layout, `INPUT`,
`READ`/`DATA`, arrays, functions - print the same in both, byte for byte on
standard output, once PC-BASIC's line endings and its closing `Ok` prompt
are normalised (PC-BASIC 2.0.8, 22 November 2026). Error messages and exit
statuses are not compared. Of the 35 classic and tutorial samples, 20 print
exactly the same and one differs only in trailing spaces; the others differ
in spacing (`TAB`), in wrapping at 80 columns, in precision or in random
numbers. `scripts/pcbasic-diff.py` repeats the comparison and says which.

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

## In your editor

`chuk-basic --lsp` is a language server: the same lexer and parser, answering
an editor's questions over the Language Server Protocol. It has no
dependencies either; the JSON is in `src/lsp/json.rs`.

| It gives | |
| --- | --- |
| Mistakes as you type | every line with a syntax error, not only the first; a `GOTO`, `GOSUB`, `THEN` or `ON` that names a line which is not there, marked on the number; a line number used twice; `WEND` without `WHILE` |
| Go to definition | from a jump to its line, and from `FNA(...)` to its `DEF` |
| Find references | from a line, every jump to it; from a variable, every use |
| Hover | what a statement or function does and whether 1964 BASIC had it; on a jump, the line it goes to |
| Outline | the subroutines, named by the `REM` at their head, and the `DEF FN`s |
| Renumber | lines in tens, with every jump changed to match |
| Colours | semantic tokens, so an editor with no BASIC grammar still colours a program; a line's number and the number a jump names are one kind |
| Completion | after a jump, the program's lines; otherwise the words that begin as typed |

Set the dialect to `dartmouth` and it also marks, as warnings, whatever the
BASIC of 1964 did not have: `INPUT`, a missing `LET`, two instructions on a
line, a variable called `TOTAL` or `N$`, a function outside the original
ten, a program that does not end with `END`, an `IF` without a relation or
with a statement after `THEN`, a `DEF` with more than one argument, a number
of more than nine digits. The fifteen statements and ten functions it
allows are the ones Thomas Kurtz lists for
[the original BASIC](https://www.dartmouth.edu/basicfifty/commands.html)
(the manual itself counts fourteen operations; see above);
the manual's own programs in `samples/dartmouth` pass without a mark, and
every one of the 103 listings from 1978 gets at least one. These are checks
of vocabulary and of a few statement forms, not a full grammar of 1964
BASIC, so a program that passes them is not proved to be one the 1964
system would accept. The interpreter itself still runs all of it.

[`editors/vscode`](editors/vscode/README.md) is an extension for VS Code:
syntax highlighting for `.bas` files, the language server, and commands to
run the file, renumber it and switch dialect. Any editor that speaks the
protocol can use the server.

The same checks work from the command line:

```bash
chuk-basic --check program.bas                      # file:line:column: error: ...
chuk-basic --check --dialect dartmouth program.bas  # and what 1964 did not have
chuk-basic --renumber program.bas > renumbered.bas
```

`--check` exits with status 1 if it finds an error. Run over the samples it
finds one: `samples/games/splat.bas` has a `GOTO 540` and no line 540, as
the listing in the coding-horror collection does (the printed book has not
been checked). `--renumber` refuses, with a reason, a program it
cannot be sure of: one that does not parse, or one with a jump written into
a run of letters (`ONXGOTO30`).

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
  analysis/   what an editor asks, answered from the text:
              scan.rs      words and where they are   diagnostics.rs   original.rs  the 1964 rules
              navigate.rs  docs.rs   renumber.rs
  lsp/        mod.rs       the language server        json.rs
  repl.rs     interactive mode
  error.rs    lib.rs       main.rs
runtime/      the runtime library for compiled programs (a thin layer over rt/)
web/          the browser playground
editors/      the VS Code extension
```

Both back ends start from the same lowered form, in which `IF` and `WHILE`
are conditional jumps and `NEXT J, I` is one operation per variable. The
interpreter walks it with a program counter. The compiler turns it into one
WebAssembly function: a loop around a jump table with a block per operation,
so that `GOTO` - which can go anywhere - is "set the counter and go round".

## Tutorial and dataset

[`tutorial/`](tutorial/01-introduction.md) is a 22-chapter tutorial, from
`PRINT "HELLO"` through strings, `PRINT USING`, `MAT`, a complete game, the
1964 Dartmouth dialect, interactive mode and compiling to WebAssembly, to a
reference. Every program in it is followed by its output, and
`python3 tutorial/check_tutorial.py` runs all 232 and compares (`--wasm`
also compiles each one and runs it in wasmtime).

`tutorial/dataset/basic.jsonl` is a small chat-format training set about
this BASIC. `python3 tutorial/build_dataset.py` rebuilds it; every output in
an answer comes from running the program.

## Tests

```bash
cargo test
```

| Suite | What it checks |
| --- | --- |
| unit tests in `src/` | the lexer, the parser, the runtime pieces, the WebAssembly encoder and linker, interactive mode |
| `tests/language.rs` | every statement, function and error, source in and output out |
| `tests/golden.rs` | saved transcripts: the classic and tutorial programs, eight picture-drawing games, the conformance programs (whose standard output PC-BASIC agrees with) and the extensions |
| `tests/games.rs` | all 103 games parse and survive a player typing at random; Hi-Lo is beaten by binary search; Lunar Lander's free fall matches the physics worked out by hand |
| `tests/compiled.rs` | every program compiled to WebAssembly prints what the interpreter prints (needs wasmtime) |
| `tests/tutorial.rs` | every program in the tutorial's 22 chapters prints the output shown under it |
| `tests/samples.rs` | the original samples, including a full game of Basketball |
| `tests/analysis.rs` | the editor's checks against every sample: no false alarms, and every program prints the same after renumbering |
| `tests/lsp.rs` | the language server, driven over its standard input and output as an editor drives it |
| `web/test.mjs` | the playground's WASI shim and session logic, and all 142 of its example programs, in node |

After a deliberate change in behaviour, `UPDATE_GOLDEN=1 cargo test --test
golden` rewrites the transcripts; review the diff before committing it.

## Credits

The games in `samples/games` are from David H. Ahl's *BASIC Computer Games*
(1978), via the public-domain
[coding-horror/basic-computer-games](https://github.com/coding-horror/basic-computer-games)
collection. See [samples/README.md](samples/README.md).

## Licence

Apache License 2.0 - see [LICENSE](LICENSE). The programs in `samples/games` are in the
public domain.
