# When things go wrong

A program can fail in two ways: it can be written wrongly, or it can do
something impossible while it runs.

**`samples/tutorial/14_errors.bas`**

```basic
10 PRINT "ABOUT TO DIVIDE"
20 X = 0
30 PRINT 10 / X
40 PRINT "NEVER PRINTED"
```

```output
ABOUT TO DIVIDE
Runtime error in line 30: Division by zero
```

An error that happens while the program is running names the line it
happened on. Everything printed before the error still appears.

## Syntax errors

A mistake in the way a line is written is a **syntax error**. The whole
program is checked before it starts, so nothing runs until these are fixed.

```basic
10 X = 1
20 IF X PRINT 1
```

```output
Syntax error in line 20: Expected THEN, found PRINT
```

The message says what was expected and what was found instead. Some more:

```basic
10 PRINT "THIS LINE IS FINE"
20 FOR I = 1 10
```

```output
Syntax error in line 20: Expected TO, found number 10
```

```basic
10 PRINT (2 + 3 * 4
```

```output
Syntax error in line 10: Expected ')', found end of line
```

```basic
10 LET 5 = X
```

```output
Syntax error in line 10: Expected a variable, found number 5
```

## Runtime errors

Each of these programs is correct as written and fails when it runs.

A jump to a line that does not exist:

```basic
10 PRINT "START"
20 GOTO 500
```

```output
START
Runtime error in line 20: Undefined line 500
```

Dividing by zero:

```basic
10 A = 5: B = 0
20 PRINT A / B
```

```output
Runtime error in line 20: Division by zero
```

A subscript outside its array:

```basic
10 DIM A(5)
20 FOR I = 1 TO 6: A(I) = I: NEXT I
```

```output
Runtime error in line 20: Subscript out of range
```

Text where a number is needed:

```basic
10 A$ = "TEN"
20 PRINT A$ * 2
```

```output
Runtime error in line 20: Type mismatch
```

More `READ`s than `DATA`:

```basic
10 READ A, B, C
20 DATA 1, 2
```

```output
Runtime error in line 10: Out of DATA
```

A `NEXT` reached by jumping into the middle of a loop:

```basic
10 GOTO 30
20 FOR I = 1 TO 3
30 NEXT I
```

```output
Runtime error in line 30: NEXT without FOR
```

A `RETURN` with no `GOSUB`, usually because an `END` is missing before the
subroutines:

```basic
10 PRINT "MAIN"
20 RETURN
```

```output
MAIN
Runtime error in line 20: RETURN without GOSUB
```

A function given a value it has no answer for:

```basic
10 PRINT SQR(-1)
```

```output
Runtime error in line 10: Illegal function call: SQR
```

A function that has no `DEF`:

```basic
10 PRINT FNA(2)
```

```output
Runtime error in line 10: Undefined function FNA
```

## A program that never stops

A loop with no way out is not an error, so nothing reports it. `--max-steps`
stops a program after a number of operations and says where it was:

```bash
basic --max-steps 100000 loop.bas
```

A program that asks for input when there is none left stops with
`Out of input`. That happens when answers are fed from a file that is too
short.

## Finding the mistake

The hard mistakes are the ones that give no error: the program runs and
prints the wrong answer. Some tools:

- **Print the variables.** An extra `PRINT` of the values you are unsure
  about is the oldest debugging technique there is, and still the best.
- **Fix the random numbers.** `basic --seed 1 game.bas` makes a game play
  out the same way every time, so a fault can be made to happen again.
- **Prepare the answers.** `basic --echo game.bas < answers.txt` replays a
  session exactly.
- **Stop it early.** Put a `STOP` where you want the program to halt, and
  print what you need just before it.
- **Look at how the line was read.** `basic --ast program.bas` shows how
  each line was understood, as JSON. `basic --tokens program.bas` shows the
  stage before that. They settle arguments about what a crunched old line
  really means.

This program is meant to add the numbers from 1 to 10 and prints 0:

```basic
10 T = 0
20 FOR I = 1 TO 10
30   TOTAL = T + I
40 NEXT I
50 PRINT T
```

```output
 0 
```

Nothing is wrong with any single line. A `PRINT I; T; TOTAL` inside the loop
shows T staying at 0 while TOTAL changes, and the mistake on line 30 is then
plain: two names for what should be one variable. A variable that has never
been set is 0, so a misspelt name is never reported.

**Try it:** each of the programs in this chapter has a one-line fix. Make
them all run.
