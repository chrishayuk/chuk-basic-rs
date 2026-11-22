# Learning BASIC with chuk-basic

BASIC was written at Dartmouth College in 1964 so that anybody - not just
engineers - could use a computer. A program is a list of numbered lines; the
computer starts at the lowest number and works upwards. That is nearly all
there is to it, which is why a generation learned to program this way.

This tutorial takes you from `PRINT "HELLO"` to a complete game. Every
program in it is a file in [`samples/tutorial`](../samples/tutorial), and the
output shown is exactly what the interpreter prints - the test suite checks
this page against the real thing.

**Contents**

1. [Your first program](#1-your-first-program)
2. [PRINT](#2-print)
3. [Variables and arithmetic](#3-variables-and-arithmetic)
4. [INPUT](#4-input)
5. [Decisions: IF and GOTO](#5-decisions-if-and-goto)
6. [Loops: FOR and NEXT](#6-loops-for-and-next)
7. [Subroutines: GOSUB and RETURN](#7-subroutines-gosub-and-return)
8. [Arrays](#8-arrays)
9. [READ and DATA](#9-read-and-data)
10. [Strings](#10-strings)
11. [Functions](#11-functions)
12. [Random numbers](#12-random-numbers)
13. [A complete game](#13-a-complete-game)
14. [When things go wrong](#14-when-things-go-wrong)
15. [Where next](#15-where-next)

To follow along, build the interpreter once:

```bash
cargo build --release
alias basic=target/release/chuk-basic
```

## 1. Your first program

**`samples/tutorial/01_hello.bas`**

```basic
10 PRINT "HELLO, WORLD!"
20 PRINT "THIS IS CHUK-BASIC."
30 END
```

```text
HELLO, WORLD!
THIS IS CHUK-BASIC.
```

Run it with `basic samples/tutorial/01_hello.bas`.

- Every line starts with a **line number**. Lines run in numerical order, no
  matter what order you type them in. Counting in tens leaves room to slot a
  line 15 in later.
- `PRINT` shows something on the screen. Text goes inside double quotes.
- `END` stops the program. It is optional at the very end.

Keywords can be written in capitals or lower case: `print` is `PRINT`.

**Try it:** add a line 25 that prints your name. You do not need to move
anything - just give it the number 25.

## 2. PRINT

**`samples/tutorial/02_print.bas`**

```basic
10 PRINT "A SEMICOLON JOINS: "; "LIKE"; "THIS"
20 PRINT "A COMMA", "MOVES TO", "THE NEXT ZONE"
30 PRINT "NUMBERS:"; 42; -7; 3.5
40 PRINT TAB(10); "TEN SPACES IN"
50 PRINT "NO NEW LINE ";
60 PRINT "AFTER A SEMICOLON"
70 PRINT
80 ? "A QUESTION MARK IS SHORT FOR PRINT"
```

```text
A SEMICOLON JOINS: LIKETHIS
A COMMA       MOVES TO      THE NEXT ZONE
NUMBERS: 42 -7  3.5 
          TEN SPACES IN
NO NEW LINE AFTER A SEMICOLON

A QUESTION MARK IS SHORT FOR PRINT
```

- A **semicolon** between items prints them right next to each other.
- A **comma** moves to the next *print zone*. Zones are 14 characters wide,
  which makes quick columns.
- **Numbers** are printed with a space in front (where a minus sign would
  go) and a space after. That is why `-7` and `3.5` do not run together.
- `TAB(n)` moves to column `n`, counting from 0.
- Ending a `PRINT` with a semicolon keeps the cursor on the same line.
- `PRINT` on its own prints a blank line.

## 3. Variables and arithmetic

**`samples/tutorial/03_variables.bas`**

```basic
10 LET W = 12
20 LET H = 5
30 A = W * H
40 PRINT "AREA:"; A
50 PRINT "PERIMETER:"; 2 * (W + H)
60 PRINT "DIAGONAL:"; SQR(W ^ 2 + H ^ 2)
70 N$ = "GRACE"
80 G$ = "HELLO, " + N$ + "!"
90 PRINT G$
100 PRINT 2 + 3 * 4; (2 + 3) * 4; 2 ^ 10; 7 / 2
110 PRINT "NEVER SET:"; Z; "["; Z$; "]"
```

```text
AREA: 60 
PERIMETER: 34 
DIAGONAL: 13 
HELLO, GRACE!
 14  20  1024  3.5 
NEVER SET: 0 []
```

A **variable** is a named box that holds a value. `LET W = 12` puts 12 in the
box called `W`. The word `LET` is optional - line 30 leaves it out.

- Names start with a letter and are not case-sensitive. A name must not
  *be* a keyword, but may contain one: `TOTAL` and `SCORE` are fine.
- A name ending in `$` holds text (a **string**); any other name holds a
  number. Putting text in a number variable is a `Type mismatch`.
- The operators are `+ - * /` and `^` for powers. `*` and `/` are done before
  `+` and `-`, and brackets override that, as line 100 shows.
- Strings are joined with `+`.
- A variable that has never been set is `0`, or the empty string.

## 4. INPUT

`INPUT` asks the person at the keyboard for a value.

**`samples/tutorial/04_input.bas`**

```basic
10 INPUT "WHAT IS YOUR NAME"; N$
20 INPUT "WHAT YEAR WERE YOU BORN"; Y
30 PRINT "HELLO, "; N$; "."
40 PRINT "IN 2030 YOU WILL TURN"; 2030 - Y
50 INPUT "TWO NUMBERS, SEPARATED BY A COMMA"; A, B
60 PRINT A; "+"; B; "="; A + B
```

```text
WHAT IS YOUR NAME? ADA
WHAT YEAR WERE YOU BORN? 1990
HELLO, ADA.
IN 2030 YOU WILL TURN 40 
TWO NUMBERS, SEPARATED BY A COMMA? 3,4
 3 + 4 = 7 
```

(In these transcripts, the text after each `?` is what was typed.)

- The text in quotes is the prompt; BASIC adds the `? ` itself.
- Several variables can be read at once - the answers are typed with commas
  between them.
- If a number is wanted and something else is typed, BASIC says
  `?REDO FROM START` and asks again.

To run a program with prepared answers, feed it a file:
`basic --echo samples/tutorial/04_input.bas < samples/tutorial/04_input.in`.
`--echo` copies the answers into the output so that it reads like the
transcript above.

## 5. Decisions: IF and GOTO

**`samples/tutorial/05_decisions.bas`**

```basic
10 INPUT "ENTER A NUMBER"; N
20 IF N < 0 THEN PRINT "NEGATIVE" ELSE PRINT "NOT NEGATIVE"
30 IF N / 2 = INT(N / 2) THEN PRINT "EVEN": GOTO 50
40 PRINT "ODD"
50 IF N >= 1 AND N <= 10 THEN PRINT "BETWEEN 1 AND 10"
60 IF N = 0 OR N > 100 THEN PRINT "ZERO, OR BIG"
70 IF N <> 0 THEN 10
80 PRINT "GOODBYE"
```

```text
ENTER A NUMBER? 7
NOT NEGATIVE
ODD
BETWEEN 1 AND 10
ENTER A NUMBER? -4
NEGATIVE
EVEN
ENTER A NUMBER? 250
NOT NEGATIVE
EVEN
ZERO, OR BIG
ENTER A NUMBER? 0
NOT NEGATIVE
EVEN
ZERO, OR BIG
GOODBYE
```

- `IF condition THEN statement` does the statement only when the condition
  is true. `ELSE` gives an alternative.
- The comparisons are `=`, `<>` (not equal), `<`, `<=`, `>`, `>=`. They work on
  strings too. Combine them with `AND`, `OR` and `NOT`.
- A **colon** puts several statements on one line. After `THEN`, everything
  up to the end of the line belongs to the `IF` - so on line 30, the `GOTO`
  only happens for even numbers.
- `GOTO 50` jumps to line 50. `IF ... THEN 10` is short for
  `IF ... THEN GOTO 10`; line 70 uses it to go round again.

`INT(X)` rounds down to a whole number, so `N / 2 = INT(N / 2)` is the
classic test for "N is even".

## 6. Loops: FOR and NEXT

**`samples/tutorial/06_loops.bas`**

```basic
10 FOR I = 1 TO 5
20   PRINT I; "SQUARED IS"; I * I
30 NEXT I
40 FOR T = 10 TO 0 STEP -2: PRINT T;: NEXT T
50 PRINT "LIFT OFF!"
60 FOR R = 1 TO 4
70   FOR C = 1 TO R
80     PRINT "*";
90   NEXT C
100   PRINT
110 NEXT R
120 S = 0
130 FOR I = 1 TO 100: S = S + I: NEXT I
140 PRINT "1 + 2 + ... + 100 ="; S
```

```text
 1 SQUARED IS 1 
 2 SQUARED IS 4 
 3 SQUARED IS 9 
 4 SQUARED IS 16 
 5 SQUARED IS 25 
 10  8  6  4  2  0 LIFT OFF!
*
**
***
****
1 + 2 + ... + 100 = 5050 
```

- `FOR I = 1 TO 5` ... `NEXT I` runs the lines in between five times, with
  `I` set to 1, 2, 3, 4, 5.
- `STEP` changes the amount added each time. It can be negative (line 40) or
  a fraction.
- Loops can sit inside loops (lines 60 to 110). The inner loop's limit can
  depend on the outer variable.
- Indenting the lines inside a loop is only for the reader; BASIC ignores it.

**Try it:** change line 130 to add up only the odd numbers.

## 7. Subroutines: GOSUB and RETURN

**`samples/tutorial/07_subroutines.bas`**

```basic
10 T$ = "CHAPTER ONE": GOSUB 100
20 T$ = "THE END": GOSUB 100
30 FOR D = 1 TO 3
40   ON D GOSUB 200, 300, 400
50 NEXT D
60 END
100 REM Print T$ with a line under it
110 PRINT T$
120 FOR I = 1 TO LEN(T$): PRINT "-";: NEXT I
130 PRINT
140 RETURN
200 PRINT "ONE": RETURN
300 PRINT "TWO": RETURN
400 PRINT "THREE": RETURN
```

```text
CHAPTER ONE
-----------
THE END
-------
ONE
TWO
THREE
```

A **subroutine** is a piece of program you can call from several places.
`GOSUB 100` jumps to line 100 and remembers where it came from; `RETURN`
jumps back to just after the `GOSUB`.

- Subroutines share all of the program's variables. Line 10 passes the title
  in by setting `T$` first.
- The `END` on line 60 matters. Without it the program would run on into
  line 100 and reach a `RETURN` that nobody called.
- `ON D GOSUB 200, 300, 400` picks the first, second or third line number
  according to the value of `D`. `ON ... GOTO` works the same way.
- `REM` is a remark for the human reader. An apostrophe does the same job.

## 8. Arrays

**`samples/tutorial/08_arrays.bas`**

```basic
10 DIM S(5)
20 FOR I = 1 TO 5: S(I) = I * 10: NEXT I
30 T = 0
40 FOR I = 1 TO 5: T = T + S(I): NEXT I
50 PRINT "TOTAL:"; T; " AVERAGE:"; T / 5
60 DIM G(3, 3)
70 FOR R = 1 TO 3: G(R, R) = 1: NEXT R
80 FOR R = 1 TO 3
90   FOR C = 1 TO 3: PRINT G(R, C);: NEXT C
100   PRINT
110 NEXT R
120 DIM N$(2)
130 N$(0) = "ZERO": N$(1) = "ONE": N$(2) = "TWO"
140 FOR I = 2 TO 0 STEP -1: PRINT N$(I); " ";: NEXT I
150 PRINT
```

```text
TOTAL: 150  AVERAGE: 30 
 1  0  0 
 0  1  0 
 0  0  1 
TWO ONE ZERO 
```

An **array** is a row of numbered boxes with one name. `DIM S(5)` makes the
boxes `S(0)` to `S(5)`.

- The number in brackets - the *subscript* - can be any expression, which is
  what makes arrays and `FOR` loops such natural partners.
- Arrays can have two or more dimensions: `G(R, C)` is a grid.
- Arrays of strings have a `$` in their name.
- An array used without a `DIM` gets subscripts 0 to 10.
- `S` and `S(1)` are different things: a plain variable and an array may
  share a name.

## 9. READ and DATA

**`samples/tutorial/09_data.bas`**

```basic
10 READ N
20 FOR I = 1 TO N
30   READ P$, D
40   PRINT P$; TAB(10); D; "MILLION KM FROM THE SUN"
50 NEXT I
60 RESTORE
70 READ N, F$
80 PRINT "THE FIRST OF"; N; "IS "; F$
100 DATA 4
110 DATA MERCURY, 58, VENUS, 108
120 DATA EARTH, 150, MARS, 228
```

```text
MERCURY    58 MILLION KM FROM THE SUN
VENUS      108 MILLION KM FROM THE SUN
EARTH      150 MILLION KM FROM THE SUN
MARS       228 MILLION KM FROM THE SUN
THE FIRST OF 4 IS MERCURY
```

`DATA` lines hold values inside the program. Each `READ` takes the next
values in order, wherever the `DATA` lines happen to be - they are usually
gathered at the end.

- Strings in `DATA` only need quotes if they contain a comma.
- `RESTORE` goes back to the first item, so the data can be read again.
- Reading more than there is stops the program with `Out of DATA`.

## 10. Strings

**`samples/tutorial/10_strings.bas`**

```basic
10 A$ = "DARTMOUTH BASIC"
20 PRINT LEN(A$)
30 PRINT LEFT$(A$, 9)
40 PRINT RIGHT$(A$, 5)
50 PRINT MID$(A$, 6, 5)
60 PRINT ASC("A"); CHR$(66)
70 PRINT VAL("1964") + 1; STR$(64) + "!"
80 FOR I = 1 TO LEN(A$)
90   IF MID$(A$, I, 1) = " " THEN P = I
100 NEXT I
110 PRINT "FIRST WORD: "; LEFT$(A$, P - 1)
120 PRINT "LAST WORD: "; MID$(A$, P + 1)
130 IF "APPLE" < "BANANA" THEN PRINT "APPLE SORTS BEFORE BANANA"
```

```text
 15 
DARTMOUTH
BASIC
OUTH 
 65 B
 1965  64!
FIRST WORD: DARTMOUTH
LAST WORD: BASIC
APPLE SORTS BEFORE BANANA
```

| Function | Gives |
| --- | --- |
| `LEN(A$)` | the number of characters |
| `LEFT$(A$, n)` | the first `n` characters |
| `RIGHT$(A$, n)` | the last `n` characters |
| `MID$(A$, p, n)` | `n` characters starting at position `p` (first is 1); leave `n` out for "the rest" |
| `ASC(A$)` | the character code of the first character |
| `CHR$(n)` | the character with code `n` |
| `VAL(A$)` | the number written in a string |
| `STR$(n)` | a number as a string |

Lines 80 to 120 use them to split a sentence at its space - the kind of
thing every text game does to understand what you typed.

## 11. Functions

**`samples/tutorial/11_functions.bas`**

```basic
10 PRINT INT(3.7); INT(-3.7); ABS(-5); SGN(-5); SQR(81)
20 DEF FNR(X) = INT(X * 100 + .5) / 100
30 PRINT FNR(3.14159); FNR(2 / 3)
40 DEF FNH(A, B) = SQR(A * A + B * B)
50 PRINT FNH(3, 4)
60 DEF FNM(A, B)
70   FNM = A
80   IF B > A THEN FNM = B
90 FNEND
100 PRINT FNM(3, 9); FNM(8, 2)
```

```text
 3 -4  5 -1  9 
 3.14  .67 
 5 
 9  8 
```

The built-in number functions are `INT`, `FIX`, `ABS`, `SGN`, `SQR`, `SIN`,
`COS`, `TAN`, `ATN`, `EXP`, `LOG` and `RND`. The trigonometric ones work in
radians.

You can define your own with `DEF`. The name must start with `FN`:

- Line 20 defines `FNR`, which rounds to two decimal places.
- A function may take several arguments (line 40).
- A longer function is written over several lines, ending in `FNEND`
  (lines 60 to 90). The answer is whatever was assigned to the function's
  own name.
- The arguments are private to the function: calling `FNR(3.14159)` does not
  disturb a variable called `X` elsewhere in the program.

## 12. Random numbers

`RND(1)` gives a different number each time, at least 0 and less than 1.
The idiom for a whole number from 1 to 6 is `INT(RND(1) * 6) + 1`.

**`samples/tutorial/12_random.bas`**

```basic
10 PRINT "FIVE ROLLS OF A DIE:";
20 FOR I = 1 TO 5
30   PRINT INT(RND(1) * 6) + 1;
40 NEXT I
50 PRINT
60 DIM C(6)
70 FOR I = 1 TO 6000
80   D = INT(RND(1) * 6) + 1
90   C(D) = C(D) + 1
100 NEXT I
110 FOR D = 1 TO 6
120   PRINT D; TAB(5);
130   FOR J = 1 TO C(D) / 50: PRINT "*";: NEXT J
140   PRINT
150 NEXT D
```

```text
FIVE ROLLS OF A DIE: 2  4  3  2  5 
 1   *******************
 2   ********************
 3   ********************
 4   *******************
 5   *******************
 6   ********************
```

Line 130 draws one star for every 50 times a face came up, so six bars of
about the same length show that the die is fair.

Each run of a program gets different numbers. To get the same "random"
numbers every time - to debug a game, or to make the transcript above
repeatable - give a seed: `basic --seed 42 samples/tutorial/12_random.bas`.

## 13. A complete game

Everything so far, in under forty lines. The computer shows a card and you say
whether the next will be higher or lower.

**`samples/tutorial/13_game.bas`**

```basic
10 REM HIGHER OR LOWER - a complete small game
20 PRINT "HIGHER OR LOWER": PRINT
30 S = 0
40 C = INT(RND(1) * 13) + 1
50 FOR T = 1 TO 5
60   GOSUB 300: PRINT "THE CARD IS "; C$
70   INPUT "WILL THE NEXT BE HIGHER OR LOWER (H/L)"; A$
80   A$ = LEFT$(A$, 1)
90   IF A$ <> "H" AND A$ <> "L" THEN PRINT "PLEASE ANSWER H OR L.": GOTO 70
100   N = INT(RND(1) * 13) + 1
110   IF N = C THEN 100
120   W = 0
130   IF A$ = "H" AND N > C THEN W = 1
140   IF A$ = "L" AND N < C THEN W = 1
150   C = N: GOSUB 300
160   IF W = 1 THEN S = S + 1: PRINT "IT IS "; C$; ". RIGHT!"
170   IF W = 0 THEN PRINT "IT IS "; C$; ". WRONG."
180   PRINT
190 NEXT T
200 PRINT "YOU SCORED"; S; "OUT OF 5."
210 IF S = 5 THEN PRINT "PERFECT!"
220 END
300 REM Set C$ to the name of card C
310 IF C = 1 THEN C$ = "AN ACE": RETURN
320 IF C = 11 THEN C$ = "A JACK": RETURN
330 IF C = 12 THEN C$ = "A QUEEN": RETURN
340 IF C = 13 THEN C$ = "A KING": RETURN
350 IF C = 8 THEN C$ = "AN 8": RETURN
360 C$ = "A" + STR$(C)
370 RETURN
```

```text
HIGHER OR LOWER

THE CARD IS A 3
WILL THE NEXT BE HIGHER OR LOWER (H/L)? H
IT IS AN 8. RIGHT!

THE CARD IS AN 8
WILL THE NEXT BE HIGHER OR LOWER (H/L)? X
PLEASE ANSWER H OR L.
WILL THE NEXT BE HIGHER OR LOWER (H/L)? L
IT IS A 7. RIGHT!

THE CARD IS A 7
WILL THE NEXT BE HIGHER OR LOWER (H/L)? H
IT IS A 4. WRONG.

THE CARD IS A 4
WILL THE NEXT BE HIGHER OR LOWER (H/L)? L
IT IS A JACK. WRONG.

THE CARD IS A JACK
WILL THE NEXT BE HIGHER OR LOWER (H/L)? H
IT IS AN 8. WRONG.

YOU SCORED 2 OUT OF 5.
```

Things to notice:

- The subroutine at line 300 turns a card number into a name. It is called
  from two places (lines 60 and 150).
- Line 90 checks the answer and asks again if it makes no sense. Programs
  that talk to people need a lot of this.
- Line 110 deals again if the new card is the same as the old one, so that
  there is always a right answer.
- `W` is a *flag*: 1 for a win, 0 for a loss, set by lines 130 and 140 and
  used by lines 160 and 170.

**Try it:** keep a running total across several games, or let the player
bet points on each card.

## 14. When things go wrong

**`samples/tutorial/14_errors.bas`**

```basic
10 PRINT "ABOUT TO DIVIDE"
20 X = 0
30 PRINT 10 / X
40 PRINT "NEVER PRINTED"
```

```text
ABOUT TO DIVIDE
Runtime error in line 30: Division by zero
```

An error that happens while the program is running names the line it
happened on. The most common ones:

| Message | Usual cause |
| --- | --- |
| `Division by zero` | a divisor that turned out to be 0 |
| `Subscript out of range` | an array index below 0 or above the `DIM` |
| `Type mismatch` | text where a number was needed, or the reverse |
| `Undefined line 500` | a `GOTO` or `GOSUB` to a line that is not there |
| `NEXT without FOR` | a `NEXT` reached by jumping into the middle of a loop |
| `RETURN without GOSUB` | the program ran on into a subroutine - an `END` is missing |
| `Out of DATA` | more `READ`s than `DATA` items |
| `Out of input` | `INPUT` was reading from a file that ran out |

A mistake in the way a line is written is a **syntax error**. The whole
program is checked before it starts, so nothing runs until these are fixed:

```text
Syntax error in line 20: Expected THEN, found PRINT
```

Some tools for finding out what is going on:

- `basic --seed 1 game.bas` makes a game play out the same way every time.
- `basic --max-steps 100000 loop.bas` stops a program that is stuck in a
  loop, and tells you which line it was on.
- `basic --ast program.bas` shows how each line was understood, as JSON.
  `basic --tokens program.bas` shows the stage before that.
- An extra `PRINT` of the variables you are unsure about is the oldest
  debugging technique there is, and still the best.

## 15. Where next

- [`samples/classic`](../samples/classic) has twenty or so short programs:
  prime sieves, sorting, the Towers of Hanoi, the Mandelbrot set, the Game
  of Life. Read one, guess what it prints, then run it.
- [`samples/games`](../samples/games) holds all of *BASIC Computer Games* -
  Hammurabi, Lunar Lander, Super Star Trek and a hundred more, as published
  in the 1970s. They are the best way to see what people built with nothing
  but the statements in this tutorial. Start with `hammurabi.bas` or
  `lunar.bas`.
- Run `basic` with no file for the **interactive prompt**: type numbered
  lines, then `LIST` and `RUN`, just as on a 1970s terminal. Anything typed
  without a line number runs at once, which makes it a handy calculator:
  `PRINT 355 / 113`.
- There is more to the language than this tutorial covers: `WHILE` ...
  `WEND` loops, `PRINT USING "###.##"; X` for tidy columns of figures,
  `LINE INPUT` for a whole line of text, `INSTR` to search a string, and
  the `MAT` statements for matrices. The [README](../README.md) lists them
  all.
- `basic --compile game.bas` turns a program into a WebAssembly module,
  `game.wasm`, that runs on its own - and several times faster.
- The [browser playground](../web/README.md) runs BASIC in a web page.
