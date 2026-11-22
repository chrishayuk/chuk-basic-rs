# Subroutines: GOSUB and RETURN

A **subroutine** is a piece of program that can be called from several
places. `GOSUB 100` jumps to line 100 and remembers where it came from.
`RETURN` jumps back to just after the `GOSUB`.

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

```output
CHAPTER ONE
-----------
THE END
-------
ONE
TWO
THREE
```

- Subroutines share all of the program's variables. Line 10 passes the title
  in by setting `T$` first.
- The `END` on line 60 matters. Without it the program would run on into
  line 100 and reach a `RETURN` that nobody called.
- `ON D GOSUB 200, 300, 400` picks the first, second or third line number
  according to the value of `D`.
- `REM` is a remark for the human reader. An apostrophe does the same job.

## Passing values in and out

A subroutine has no parameters and no private variables. The convention is
to pick some variables as its inputs and some as its outputs, and to say so
in a remark.

```basic
10 A = 12: B = 18: GOSUB 1000
20 PRINT "THE GCD OF 12 AND 18 IS"; G
30 A = 100: B = 75: GOSUB 1000
40 PRINT "THE GCD OF 100 AND 75 IS"; G
50 END
1000 REM GREATEST COMMON DIVISOR OF A AND B, RETURNED IN G
1010 REM USES X, Y AND T
1020 X = A: Y = B
1030 IF Y = 0 THEN G = X: RETURN
1040 T = X - INT(X / Y) * Y
1050 X = Y: Y = T
1060 GOTO 1030
```

```output
THE GCD OF 12 AND 18 IS 6 
THE GCD OF 100 AND 75 IS 25 
```

The subroutine copies A and B into X and Y so that it does not change the
caller's values. It does change X, Y and T, and any other part of the
program that uses those names will be surprised. Most of the hard-to-find
mistakes in BASIC programs come from two parts of a program using the same
variable for different things.

## Subroutines that call subroutines

A subroutine can call another. Each `RETURN` goes back to the most recent
`GOSUB` that has not yet returned.

```basic
10 W = 20
20 GOSUB 100
30 T$ = "REPORT": GOSUB 200
40 GOSUB 100
50 END
100 REM A RULE W CHARACTERS WIDE
110 FOR I = 1 TO W: PRINT "=";: NEXT I: PRINT
120 RETURN
200 REM T$ IN THE MIDDLE OF A LINE W WIDE, WITH A RULE BELOW
210 PRINT TAB((W - LEN(T$)) / 2); T$
220 GOSUB 100
230 RETURN
```

```output
====================
       REPORT
====================
====================
```

## Recursion

A subroutine can call itself. Since all variables are shared, anything that
must survive the inner call has to be saved by hand. This counts down and
back up using only the depth of the calls to remember where it is:

```basic
10 N = 3
20 GOSUB 100
30 PRINT "DONE"
40 END
100 IF N = 0 THEN PRINT "BOTTOM": RETURN
110 PRINT "DOWN"; N
120 N = N - 1
130 GOSUB 100
140 N = N + 1
150 PRINT "UP"; N
160 RETURN
```

```output
DOWN 3 
DOWN 2 
DOWN 1 
BOTTOM
UP 1 
UP 2 
UP 3 
DONE
```

Line 140 undoes line 120, so N is the same after the inner call as before.
`samples/classic/hanoi.bas` solves the Towers of Hanoi this way, keeping the
values it needs in arrays used as a stack.

## ON ... GOSUB

`ON N GOSUB` picks a subroutine by number, and every one of them returns to
the same place. It suits a menu:

```basic
10 PRINT "1 SQUARE   2 CUBE   3 QUIT"
20 INPUT "CHOICE"; C
30 IF C = 3 THEN PRINT "GOODBYE": END
40 INPUT "NUMBER"; X
50 ON C GOSUB 100, 200
60 GOTO 20
100 PRINT X; "SQUARED IS"; X * X: RETURN
200 PRINT X; "CUBED IS"; X * X * X: RETURN
```

```input
1
7
2
3
3
```

```output
1 SQUARE   2 CUBE   3 QUIT
CHOICE? 1
NUMBER? 7
 7 SQUARED IS 49 
CHOICE? 2
NUMBER? 3
 3 CUBED IS 27 
CHOICE? 3
GOODBYE
```

## Where to put them

The convention is to put the main program first, end it with `END` or
`STOP`, and put subroutines after it at round line numbers: 1000, 2000,
3000. A remark on the first line of each says what it does and which
variables it uses.

`STOP` ends the program wherever it is. `END` does the same, and
traditionally marks the last line.

## Two errors

Running into a subroutine without a `GOSUB`:

```basic
10 PRINT "MAIN PROGRAM"
100 PRINT "SUBROUTINE"
110 RETURN
```

```output
MAIN PROGRAM
SUBROUTINE
Runtime error in line 110: RETURN without GOSUB
```

The cure is an `END` before line 100. And calling a line that is not there:

```basic
10 GOSUB 500
20 END
```

```output
Runtime error in line 10: Undefined line 500
```

The same error is reported for a `GOTO` to a missing line.

**Try it:** write a subroutine that prints the string `T$` inside a box of
asterisks, and call it with three different strings.
