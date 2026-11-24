# Sample programs

150 BASIC programs to run, read and test with.

| Folder | What is in it |
| --- | --- |
| `samples/` | The seven programs the project started with |
| [`samples/tutorial/`](tutorial) | The 14 programs the [tutorial](../tutorial/01-introduction.md) is built around, in order |
| [`samples/dartmouth/`](dartmouth) | The programs printed in the first BASIC manual, May 1964 ([scans](../docs/scans/README.md)) |
| [`samples/classic/`](classic) | 22 short programs that each show off one idea |
| [`samples/games/`](games) | The 103 programs of *BASIC Computer Games* |

Beside many programs you will find:

- `name.in` - what to type, for programs that use `INPUT`
- `name.out` - exactly what the program prints (with the seed 42 where it
  uses random numbers); the test suite checks these
- `name.err` - the error the program ends with, if it is meant to fail

To reproduce a transcript:

```bash
chuk-basic --seed 42 --echo samples/classic/guess.bas < samples/classic/guess.in
```

## Classic

| Program | Shows |
| --- | --- |
| `hello.bas` | The traditional first program |
| `fibonacci.bas` | A loop with three variables |
| `primes.bas` | The Sieve of Eratosthenes, with an array and nested loops |
| `factorial.bas` | A loop checked against a recursive multi-line function |
| `gcd.bas` | Euclid's algorithm, reading pairs from `DATA` |
| `collatz.bas` | `IF ... THEN ... ELSE` and a `GOTO` loop |
| `bubblesort.bas` | Sorting an array; a subroutine to print it |
| `hanoi.bas` | Recursion by hand: `GOSUB` with an explicit stack |
| `timestable.bas` | Laying out a table with `TAB` |
| `temperature.bas` | `DEF FN` and print zones |
| `weekday.bas` | Zeller's congruence on famous dates; string arrays |
| `roman.bas` | Roman numerals from parallel arrays |
| `binary.bas` | Binary and hexadecimal with `CHR$` and `MID$` |
| `strings.bas` | Reversing, palindromes and a Caesar cipher |
| `matrix.bas` | Matrix multiplication with two-dimensional arrays |
| `pi.bas` | Three approximations to pi |
| `mandelbrot.bas` | The Mandelbrot set in ASCII |
| `life.bas` | Conway's Game of Life: a glider |
| `bottles.bas` | Bottles of beer; `STR$` and plurals |
| `guess.bas` | Guess the number (interactive) |
| `average.bas` | Statistics on numbers typed in (interactive) |

## Games

The programs in `samples/games` are the BASIC listings of David H. Ahl's
*BASIC Computer Games* (Creative Computing, 1978) as collected in the
[coding-horror/basic-computer-games](https://github.com/coding-horror/basic-computer-games)
repository, which is released into the public domain under the Unlicense.
They are byte for byte the files in that repository (compared on 22
November 2026, at commit `a81c331`). They are not necessarily the book as
printed: the collection took them from Vintage BASIC's site, where Lyle
Kopnicky "normalized" them and "tested and tweaked each one of them" to run
under Vintage BASIC, and they have not been checked against the book here.

They were written by many people for several dialects of BASIC, so they make
a demanding compatibility test: all 103 load and run here.

Good ones to start with: `hammurabi.bas` (rule a city for ten years),
`lunar.bas` (land on the moon), `superstartrek.bas` (the big one),
`hangman.bas`, `blackjack.bas`, `animal.bas` (it learns from you), and
`3dplot.bas`, `sinewave.bas` and `bunny.bas`, which just draw pictures.

Most games loop until you stop them: press Ctrl-C (or Ctrl-D at a prompt).
A few expect you to follow the instructions they print - type nonsense and a
1970s program may well fall over with `Subscript out of range`.

The full list: `23matches`, `3dplot`, `aceyducey`, `amazing`, `animal`, `awari`, `bagels`, `banner`, `basketball`, `batnum`, `battle`, `blackjack`, `bombardment`, `bombsaway`, `bounce`, `bowling`, `boxing`, `bug`, `bullfight`, `bullseye`, `bunny`, `buzzword`, `calendar`, `change`, `checkers`, `chemist`, `chief`, `chomp`, `civilwar`, `combat`, `craps`, `cube`, `depthcharge`, `diamond`, `dice`, `digits`, `distributions`, `evenwins`, `flipflop`, `football`, `ftball`, `furtrader`, `gameofevenwins`, `golf`, `gomoko`, `guess`, `gunner`, `hammurabi`, `hangman`, `hello`, `hexapawn`, `hi-lo`, `highiq`, `hockey`, `horserace`, `hurkle`, `kinema`, `king`, `lem`, `letter`, `life`, `lifefortwo`, `litquiz`, `love`, `lunar`, `mastermind`, `mathdice`, `mugwump`, `name`, `nicomachus`, `nim`, `number`, `onecheck`, `orbit`, `pizza`, `poetry`, `poker`, `qubit`, `queen`, `reverse`, `rocket`, `rockscissors`, `roulette`, `russianroulette`, `salvo`, `sinewave`, `slalom`, `slots`, `splat`, `stars`, `stockmarket`, `superstartrek`, `superstartrekins`, `synonym`, `target`, `tictactoe1`, `tictactoe2`, `tower`, `train`, `trap`, `war`, `weekday`, `word`.
