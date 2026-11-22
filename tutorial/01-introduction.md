# Learning BASIC

BASIC was written at Dartmouth College in 1964 by John Kemeny and Thomas
Kurtz so that anybody, not only engineers, could use a computer. A program is
a list of numbered lines. The computer starts at the lowest number and works
upwards. That is nearly all there is to it, which is why a generation learned
to program this way.

Three ideas explain most of what you will see in a BASIC program:

- **Every line has a number.** Lines run in numerical order, and `GOTO 100`
  means "carry on from line 100".
- **Every statement starts with a word.** `PRINT` shows something, `INPUT`
  asks for something, `LET` puts a value in a variable.
- **There are two kinds of value.** Numbers, and pieces of text called
  strings. A variable whose name ends in `$` holds a string.

This tutorial teaches the BASIC of the 1970s microcomputers, the one that
the listings in books and magazines were written in. Chapter 17 goes back to
the first manual of 1964 and runs the programs printed in it.

## Running a BASIC program

Build the interpreter with Cargo, from the top folder of this repository:

```bash
cargo build --release
alias basic=target/release/chuk-basic
```

Give it the name of a file containing a BASIC program:

```bash
basic samples/tutorial/01_hello.bas
```

With no file name it starts the interactive prompt, where you type numbered
lines and then `RUN`. Chapter 18 describes it.

## A first program

**`samples/tutorial/01_hello.bas`**

```basic
10 PRINT "HELLO, WORLD!"
20 PRINT "THIS IS CHUK-BASIC."
30 END
```

```output
HELLO, WORLD!
THIS IS CHUK-BASIC.
```

- Every line starts with a **line number**. Lines run in numerical order, no
  matter what order you type them in. Counting in tens leaves room to slot a
  line 15 in later.
- `PRINT` shows something on the screen. Text goes inside double quotes.
- `END` stops the program. It is optional at the very end.

Keywords can be written in capitals or lower case: `print` is `PRINT`. The
old machines had capitals only, which is why the listings shout.

**Try it:** add a line 25 that prints your name. You do not need to move
anything. Give the new line the number 25 and put it anywhere in the file.

```basic
10 PRINT "HELLO, WORLD!"
30 END
25 PRINT "MY NAME IS ADA."
20 PRINT "THIS IS CHUK-BASIC."
```

```output
HELLO, WORLD!
THIS IS CHUK-BASIC.
MY NAME IS ADA.
```

## How the interpreter works

The whole program is read and checked before any of it runs. A line that
makes no sense is reported as a syntax error and nothing is run. Then the
interpreter starts at the lowest line number and carries out one statement at
a time. The order only changes when a statement says so: `GOTO`, `IF`, `FOR`
and `GOSUB` are the ones that do, and later chapters describe each.

## How to read this tutorial

Every program is followed by what it prints:

```basic
10 PRINT 2 + 2
```

```output
 4 
```

A program that asks questions is shown with what was typed at the keyboard,
one answer to a line, and then the whole conversation:

```basic
10 INPUT "HOW OLD ARE YOU"; A
20 PRINT "NEXT YEAR YOU WILL BE"; A + 1
```

```input
41
```

```output
HOW OLD ARE YOU? 41
NEXT YEAR YOU WILL BE 42 
```

Every program in this tutorial has been run, and the output shown is what the
interpreter printed. `python3 tutorial/check_tutorial.py` runs them all again
and compares.

## The examples

The `samples` folder has about 150 programs:

- `samples/tutorial` holds the main programs of this tutorial.
- `samples/classic` has twenty or so short programs: primes, sorting, the
  Towers of Hanoi, the Mandelbrot set, the Game of Life.
- `samples/games` holds the games from *BASIC Computer Games*, as published
  in the 1970s: Hammurabi, Lunar Lander, Super Star Trek and a hundred more.
- `samples/dartmouth` has the programs printed in the 1964 manual.

Each one has a `.out` file showing what it prints, and a `.in` file if it
needs input.
