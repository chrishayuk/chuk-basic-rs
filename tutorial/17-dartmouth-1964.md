# The BASIC of 1964

The first BASIC ran at Dartmouth College on 1 May 1964. Its instruction
manual, dated that month, prints a handful of complete programs together
with their results. They are in `samples/dartmouth`, typed in from the
scanned pages in `docs/scans`, and this chapter runs them.

## The first program in print

The manual's preface shows three statements, and page 4 gives them their
line numbers.

```basic
10 LET X = (7+8)/3
20 PRINT X
30 END
```

```output
 5 
```

`LET` was not optional in 1964. Page 4 says, "The last instruction in any
BASIC program is 'END'."

## Example 1: the average of 100 logarithms

```basic
10 PRINT "AVERAGE OF 100 LOGS"
20 LET S = 0
30  FOR N = 1 to 100
40 LET S = S + FNL (N)
50 NEXT N
60 LET S = S/100
70 PRINT S
80 DEF FNL (Z) = LOG(Z)/LOG(10)
99 END
```

```output
AVERAGE OF 100 LOGS
 1.57970004 
```

The manual prints `1.5797`, to six significant digits. This interpreter
prints nine unless it is asked for 1964's ways with `--dialect dartmouth`,
which is the subject of the rest of this chapter; then it prints `1.5797`
too.

Two things in the listing are worth noticing. The function `FNL` is defined
on line 80, after the loop that uses it. That is allowed: definitions are
collected before the program starts. And `LOG` is the natural logarithm, as
it has been in BASIC ever since, so the program divides by `LOG(10)`.

## Example 3: the root of a function

The program looks for the place where a function crosses zero, by halving
the step each time it oversteps. The manual writes powers with an upwards
arrow, the character on the Teletype's keyboard. It is accepted here as
another way to write `^`.

```basic
10 PRINT "ROOT OF FUNCTION."
20 PRINT "FUNCTION IS - AT  0 AND + AT 1"
30 DEF FNF(Z) = Z↑5 +Z↑3 -1
40 LET X = 0
50 LET D = .5
60 LET Y = FNF(X)
70 IF Y = 0 THEN 200
80 IF Y > 0 THEN 150
90 LET X = X + D
100 GOTO 160
150 LET X = X - D
160 IF D <= .0001 THEN 200
170 LET D = D/2
180 GO TO 60
200 PRINT "ROOT =" X
999 END
```

```output
ROOT OF FUNCTION.
FUNCTION IS - AT  0 AND + AT 1
ROOT = .837585449 
```

The manual prints `ROOT = .837585`. It then says to change line 30 to find
the root of a different function:

```basic
10 PRINT "ROOT OF FUNCTION."
20 PRINT "FUNCTION IS - AT  0 AND + AT 1"
30 DEF FNF(Z) = EXP(X) -2
40 LET X = 0
50 LET D = .5
60 LET Y = FNF(X)
70 IF Y = 0 THEN 200
80 IF Y > 0 THEN 150
90 LET X = X + D
100 GOTO 160
150 LET X = X - D
160 IF D <= .0001 THEN 200
170 LET D = D/2
180 GO TO 60
200 PRINT "ROOT =" X
999 END
```

```output
ROOT OF FUNCTION.
FUNCTION IS - AT  0 AND + AT 1
ROOT = .69317627 
```

The manual prints `ROOT = .693176`, which is the natural logarithm of 2 to
four places.

`PRINT "ROOT =" X` has no semicolon between the text and the variable. The
1964 manual writes it that way, and BASIC has accepted it ever since.

## Example 2: the quadratic equation

The program solves three equations, given as `DATA`, and says for each
whether it has two roots, one, or a pair of complex roots.

```basic
5 PRINT "QUADRATIC EQUATION"
6 PRINT
10 READ A, B, C
20 LET D = B * B -4 * A * C
30 LET S = - B / (2 * A )
40 LET T = SQR(D) / (2 * A)
50 IF D = 0 THEN 100
60 IF D < 0 THEN 200
70 PRINT A, B, C, "TWO REAL ROOTS"
80 PRINT "ROOTS =" S + T, S - T
90 GOTO 300
100 PRINT A, B, C, "ONE ROOT"
110 PRINT "ROOT =" S
120 GOTO 300
200 PRINT A, B, C, "COMPLEX ROOTS"
210 PRINT "REAL PART=" S, "IMAGINARY PART=" T
300 PRINT
310 GOTO 10
400 DATA 1, -2, -3, 1, -4, 4, 1, 0, 2
999 END
```

```output
QUADRATIC EQUATION

 1            -2            -3            TWO REAL ROOTS
ROOTS = 3     -1 

 1            -4             4            ONE ROOT
ROOT = 2 

Runtime error in line 40: Illegal function call: SQR
```

The first two equations are solved. On the third the program stops at line
40 with an error, and the page in the manual shows no such thing: it prints
`IMAGINARY PART = 1.41421`.

Line 40 takes the square root of `D` before lines 50 and 60 have looked at
its sign. For the third equation `D` is -8.

The manual explains on page 5. Of `SQR` it says "the square-root is always
taken of the absolute value", and of powers that `A ↑ B` "is actually
interpreted as |A|^B". In 1964 the square root of -8 was the square root of
8, and the program relies on it. Every later BASIC treats the square root of
a negative number as an error, and so does this one unless asked otherwise:

```bash
basic --dialect dartmouth samples/dartmouth/example2.bas
```

```basic dialect=dartmouth
5 PRINT "QUADRATIC EQUATION"
6 PRINT
10 READ A, B, C
20 LET D = B * B -4 * A * C
30 LET S = - B / (2 * A )
40 LET T = SQR(D) / (2 * A)
50 IF D = 0 THEN 100
60 IF D < 0 THEN 200
70 PRINT A, B, C, "TWO REAL ROOTS"
80 PRINT "ROOTS =" S + T, S - T
90 GOTO 300
100 PRINT A, B, C, "ONE ROOT"
110 PRINT "ROOT =" S
120 GOTO 300
200 PRINT A, B, C, "COMPLEX ROOTS"
210 PRINT "REAL PART=" S, "IMAGINARY PART=" T
300 PRINT
310 GOTO 10
400 DATA 1, -2, -3, 1, -4, 4, 1, 0, 2
999 END
```

```output
QUADRATIC EQUATION

 1.           -2.           -3.           TWO REAL ROOTS
ROOTS = 3.    -1. 

 1.           -4.            4.           ONE ROOT
ROOT = 2. 

 1.            0.            2.           COMPLEX ROOTS
REAL PART= 0.               IMAGINARY PART= 1.41421 

Runtime error in line 10: Out of DATA
```

Those are the page's figures, digit for digit: `3.` and `-1.` with a point
after a whole number, and `1.41421`. The program then goes back to
line 10 for a fourth equation and stops because the data has run out, which
is how it is written.

## What the dialect changes

`--dialect dartmouth` changes three things. Two are stated in the manual:
`SQR` takes the absolute value of its argument, and so does a power. The
third is copied from the results the manual prints on pages 21 to 23, which
state no rule: a number is printed to six significant digits, and a whole
number is followed by a point.

```basic
10 PRINT SQR(-16)
```

```output
Runtime error in line 10: Illegal function call: SQR
```

```basic dialect=dartmouth
10 PRINT SQR(-16)
20 PRINT (-2) ^ 3
30 PRINT (-8) ^ (1 / 3)
```

```output
 4. 
 8. 
 2. 
```

Under the default rules a negative number keeps its sign when it is raised
to a whole power:

```basic
10 PRINT (-2) ^ 3
20 PRINT (-2) ^ 4
```

```output
-8 
 16 
```

Nothing else about the language changes with the dialect.

## What 1964 BASIC did not have

After `END`, page 4 of the manual says, "There are only 13 other
operations." Much of this tutorial would have been news to its readers:

| 1964 | Later |
| --- | --- |
| Numbers only | Strings, and variables ending in `$` |
| `READ` and `DATA` for all input | `INPUT` |
| One statement to a line | Colons |
| `IF` ... `THEN` line number | `THEN` statement, `ELSE` |
| Names of one letter, or a letter and a digit | Long names |
| `LET` required | `LET` optional |
| `DEF FNA(X)` with one parameter, on one line | Several parameters, several lines |

A program that keeps to the left-hand column runs here unchanged. Programs
1 and 3 above did, and gave the page's answers. Program 2 needed the
arithmetic of its time.
