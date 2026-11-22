# Decisions: IF and GOTO

`IF` does something only when a condition holds. `GOTO` carries on from
another line. Between them they make every decision a program takes.

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

```input
7
-4
250
0
```

```output
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
- A **colon** puts several statements on one line. After `THEN`, everything
  up to the end of the line belongs to the `IF`, so on line 30 the `GOTO`
  only happens for even numbers.
- `GOTO 50` jumps to line 50. `IF ... THEN 10` is short for
  `IF ... THEN GOTO 10`. Line 70 uses it to go round again.

## Comparisons

| | |
| --- | --- |
| `=` | equal |
| `<>` | not equal |
| `<` `<=` | less than; less than or equal |
| `>` `>=` | greater than; greater than or equal |

```basic
10 A = 5: B = 8
20 IF A = B THEN PRINT "EQUAL"
30 IF A <> B THEN PRINT "DIFFERENT"
40 IF A < B THEN PRINT "A IS SMALLER"
50 IF A >= 5 THEN PRINT "A IS AT LEAST 5"
```

```output
DIFFERENT
A IS SMALLER
A IS AT LEAST 5
```

Strings are compared in alphabetical order, character by character. Capital
letters come before small letters.

```basic
10 IF "APPLE" < "BANANA" THEN PRINT "APPLE COMES FIRST"
20 IF "CAT" < "CATS" THEN PRINT "SHORTER COMES FIRST"
30 IF "Z" < "a" THEN PRINT "CAPITALS COME FIRST"
40 A$ = "YES"
50 IF A$ = "YES" THEN PRINT "MATCHED"
```

```output
APPLE COMES FIRST
SHORTER COMES FIRST
CAPITALS COME FIRST
MATCHED
```

## AND, OR, NOT

`AND` needs both sides to be true. `OR` needs at least one. `NOT` reverses a
condition.

```basic
10 FOR N = 0 TO 12 STEP 4
20   IF N >= 1 AND N <= 10 THEN PRINT N; "IS FROM 1 TO 10"
30   IF N < 1 OR N > 10 THEN PRINT N; "IS OUTSIDE"
40   IF NOT (N = 4) THEN PRINT N; "IS NOT 4"
50 NEXT N
```

```output
 0 IS OUTSIDE
 0 IS NOT 4
 4 IS FROM 1 TO 10
 8 IS FROM 1 TO 10
 8 IS NOT 4
 12 IS OUTSIDE
 12 IS NOT 4
```

`AND` is done before `OR`. Put brackets round the `OR` if you mean it to go
first:

```basic
10 A = 1: B = 0: C = 0
20 IF A = 1 OR B = 1 AND C = 1 THEN PRINT "WITHOUT BRACKETS: TRUE"
30 IF (A = 1 OR B = 1) AND C = 1 THEN PRINT "NEVER PRINTED"
40 PRINT "DONE"
```

```output
WITHOUT BRACKETS: TRUE
DONE
```

## True and false are numbers

A comparison works out to a number: -1 for true and 0 for false. `IF` treats
any number other than 0 as true.

```basic
10 PRINT 3 < 5
20 PRINT 3 > 5
30 F = (7 = 7)
40 PRINT F
50 IF F THEN PRINT "F IS TRUE"
60 N = 3
70 IF N THEN PRINT "ANY NON-ZERO NUMBER COUNTS AS TRUE"
```

```output
-1 
 0 
-1 
F IS TRUE
ANY NON-ZERO NUMBER COUNTS AS TRUE
```

Old programs use this for arithmetic. `-(A > B)` is 1 when A is bigger and 0
otherwise, so this counts how many of three numbers are positive:

```basic
10 A = 4: B = -2: C = 9
20 PRINT -(A > 0) - (B > 0) - (C > 0)
```

```output
 2 
```

## THEN and ELSE

After `THEN` comes a statement, several statements with colons, or a line
number. `ELSE` takes the same.

```basic
10 FOR N = 1 TO 3
20   IF N = 2 THEN PRINT "TWO": PRINT "(THE MIDDLE)" ELSE PRINT "NOT TWO"
30 NEXT N
```

```output
NOT TWO
TWO
(THE MIDDLE)
NOT TWO
```

There is no way to spread an `IF` over several lines in this BASIC. When
there is too much to fit on one line, jump over it:

```basic
10 N = 15
20 IF N <= 10 THEN 60
30 PRINT "THAT IS MORE THAN TEN."
40 PRINT "TAKING TEN AWAY."
50 N = N - 10
60 PRINT "N IS NOW"; N
```

```output
THAT IS MORE THAN TEN.
TAKING TEN AWAY.
N IS NOW 5 
```

The test on line 20 is the opposite of the one you have in mind: "if it is
*not* more than ten, skip this part".

## Loops with GOTO

Jumping backwards repeats part of the program. An `IF` decides when to stop.

```basic
10 N = 1
20 PRINT N;
30 N = N * 2
40 IF N <= 1000 THEN 20
50 PRINT
60 PRINT "FIRST POWER OF TWO ABOVE 1000:"; N
```

```output
 1  2  4  8  16  32  64  128  256  512 
FIRST POWER OF TWO ABOVE 1000: 1024 
```

A `GOTO` with no `IF` in the loop goes round for ever. Stop a runaway
program with Ctrl-C, or run it with `--max-steps` to have it stopped for you:

```bash
basic --max-steps 100000 loop.bas
```

## ON ... GOTO

`ON N GOTO 100, 200, 300` jumps to the first line number when N is 1, the
second when N is 2, and so on. If N is 0, or more than the number of lines
listed, the program carries on with the next statement.

```basic
10 FOR N = 1 TO 4
20   ON N GOTO 100, 200, 300
30   PRINT "NO SUCH CHOICE:"; N
40   GOTO 400
100  PRINT "ONE": GOTO 400
200  PRINT "TWO": GOTO 400
300  PRINT "THREE"
400 NEXT N
```

```output
ONE
TWO
THREE
NO SUCH CHOICE: 4 
```

It is the usual way to act on a menu choice.

```basic
10 PRINT "1 ADD   2 MULTIPLY"
20 INPUT "CHOICE"; C
30 INPUT "TWO NUMBERS"; A, B
40 ON C GOTO 60, 70
50 PRINT "PLEASE CHOOSE 1 OR 2": GOTO 20
60 PRINT A + B: END
70 PRINT A * B: END
```

```input
2
6, 7
```

```output
1 ADD   2 MULTIPLY
CHOICE? 2
TWO NUMBERS? 6, 7
 42 
```

## Even or odd

`INT(X)` rounds down to a whole number, so `N / 2 = INT(N / 2)` is the
classic test for "N is even". The same idea tests whether any number divides
another.

```basic
10 FOR N = 1 TO 15
20   IF N / 3 = INT(N / 3) THEN PRINT N; "DIVIDES BY 3"
30 NEXT N
```

```output
 3 DIVIDES BY 3
 6 DIVIDES BY 3
 9 DIVIDES BY 3
 12 DIVIDES BY 3
 15 DIVIDES BY 3
```

**Try it:** ask for a year and say whether it is a leap year. A year is a leap
year if it divides by 4, except that years dividing by 100 are not, unless
they also divide by 400.
