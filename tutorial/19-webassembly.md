# WebAssembly

WebAssembly is a portable machine code that runs in browsers and in
standalone runtimes such as wasmtime. WASI is the part that gives a
WebAssembly program a keyboard, a screen and files. There are three ways to
run BASIC on it:

1. Run the interpreter itself as a WebAssembly module.
2. Run that module in a web page.
3. Compile a BASIC program into a module of its own.

Every program in this tutorial prints the same thing whether it is
interpreted or compiled, and the last section shows how that is checked.

## The interpreter on WASI

The interpreter builds for the target `wasm32-wasip1` with no changes.

```bash
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

The result is `target/wasm32-wasip1/release/chuk-basic.wasm`. Run it with a
WASI runtime, giving the same arguments as the native program:

```bash
wasmtime run --dir=. target/wasm32-wasip1/release/chuk-basic.wasm samples/tutorial/01_hello.bas
```

```text
HELLO, WORLD!
THIS IS CHUK-BASIC.
```

A WebAssembly program can see nothing of the machine unless it is told it
may. `--dir=.` lets the interpreter read files in the current folder, which
it needs in order to open the `.bas` file. `PRINT` and `INPUT` use the
runtime's standard output and input, so a game is played in the terminal as
before:

```bash
wasmtime run --dir=. target/wasm32-wasip1/release/chuk-basic.wasm --seed 1 samples/games/hammurabi.bas
```

Options such as `--seed` and `--echo` go after the name of the module.

## In the browser

`web/` is a page with an editor, an output pane and a line for answering
`INPUT`. It runs the same WASI module, with a small piece of JavaScript
standing in for the runtime.

```bash
web/build.sh
web/serve.sh
```

Then open `http://localhost:8000`. Nothing is sent to a server: the program
runs in the page.

## Compiling a program

`--compile` turns a BASIC program into a WebAssembly module.

```basic
10 PRINT "COMPILED BASIC"
20 FOR I = 1 TO 5
30   PRINT I; "CUBED IS"; I ^ 3
40 NEXT I
```

```output
COMPILED BASIC
 1 CUBED IS 1 
 2 CUBED IS 8 
 3 CUBED IS 27 
 4 CUBED IS 64 
 5 CUBED IS 125 
```

Save that as `cubes.bas`, and then:

```bash
basic --compile cubes.bas
wasmtime cubes.wasm
```

```text
COMPILED BASIC
 1 CUBED IS 1 
 2 CUBED IS 8 
 3 CUBED IS 27 
 4 CUBED IS 64 
 5 CUBED IS 125 
```

The first command writes `cubes.wasm` next to the program. `-o` puts it
somewhere else:

```bash
basic --compile -o /tmp/cubes.wasm cubes.bas
```

The module is complete in itself. It does not need the interpreter, the
`.bas` file or access to any folder, and it runs on any WASI runtime.

## Options of a compiled program

A compiled program takes the options that matter while it runs. They go
after the module's name:

```bash
basic --compile samples/tutorial/12_random.bas
wasmtime samples/tutorial/12_random.wasm --seed 42
```

| Option | Effect |
| --- | --- |
| `--seed N` | the same random numbers every time |
| `--echo` | copy answers to `INPUT` into the output |
| `--max-steps N` | stop after N operations |
| `--dialect dartmouth` | the arithmetic and printing of 1964; see chapter 17 |

Input comes from the keyboard, or from a file:

```bash
basic --compile samples/tutorial/13_game.bas
wasmtime samples/tutorial/13_game.wasm --seed 42 --echo < samples/tutorial/13_game.in
```

That prints the transcript shown in chapter 15.

## What the compiler does

The program's arithmetic, its variables and its jumps become WebAssembly
instructions. Strings, `PRINT`, `INPUT`, arrays and the built-in functions
become calls to a runtime library, which is the interpreter's own code built
as a module. The compiler copies that library into every program it
produces.

Because the two share that code, a compiled program prints exactly what the
interpreter prints, including the wording of error messages:

```basic
10 PRINT "ABOUT TO DIVIDE"
20 X = 0
30 PRINT 10 / X
```

```output
ABOUT TO DIVIDE
Runtime error in line 30: Division by zero
```

The compiled module prints the same two lines and stops.

Errors in the way the program is written are reported when it is compiled,
and no module is produced.

## How much faster

Counting the primes below 200,000 by trial division, on one machine:

| | Time |
| --- | --- |
| Interpreter, native | 4.6 s |
| Interpreter, as a WebAssembly module on wasmtime | 12.4 s |
| Program compiled to WebAssembly, on wasmtime | 1.3 s |

Programs that spend their time in arithmetic and loops gain the most.
Programs that spend it printing or handling strings gain less, since that
work is the same runtime code either way.

## Checking it

`python3 tutorial/check_tutorial.py --wasm` compiles every program in this
tutorial, runs the module in wasmtime, and compares what it prints with what
the interpreter prints.

**Try it:** compile `samples/classic/mandelbrot.bas` and run it both ways.
Time them with `time`.
