# Functions

A function takes one or more values and gives one back. BASIC has a set
built in, and `DEF` lets you add your own.

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

```output
 3 -4  5 -1  9 
 3.14  .67 
 5 
 9  8 
```

- Line 20 defines `FNR`, which rounds to two decimal places.
- A function may take several arguments (line 40).
- A longer function is written over several lines, ending in `FNEND`
  (lines 60 to 90).

## The numeric functions

| Function | Gives |
| --- | --- |
| `INT(X)` | the largest whole number not above X |
| `FIX(X)` | X with its fraction cut off |
| `ABS(X)` | X without its sign |
| `SGN(X)` | -1, 0 or 1 according to the sign of X |
| `SQR(X)` | the square root |
| `EXP(X)` | e to the power X |
| `LOG(X)` | the natural logarithm |
| `SIN(X)` `COS(X)` `TAN(X)` | the trigonometric functions, X in radians |
| `ATN(X)` | the arctangent, in radians |
| `RND(X)` | a random number; see chapter 13 |

`INT` and `FIX` differ only for negative numbers:

```basic
10 PRINT INT(2.7); FIX(2.7)
20 PRINT INT(-2.7); FIX(-2.7)
30 PRINT ABS(-8); ABS(8); SGN(-8); SGN(0); SGN(8)
40 PRINT SQR(2); SQR(144)
```

```output
 2  2 
-3 -2 
 8  8 -1  0  1 
 1.41421356  12 
```

`LOG` and `EXP` are each other's opposites. There is no function for
logarithms to base 10, but dividing by `LOG(10)` gives one:

```basic
10 PRINT EXP(1)
20 PRINT LOG(EXP(2))
30 PRINT LOG(1000) / LOG(10)
40 PRINT EXP(LOG(2) * 10)
```

```output
 2.71828183 
 2 
 3 
 1024 
```

## Angles

The trigonometric functions work in radians. There are 2π radians in a
circle, and no constant for π, so programs work it out as four times the
arctangent of 1.

```basic
10 P = 4 * ATN(1)
20 PRINT P
30 PRINT SIN(P / 2); COS(0)
40 D = 30
50 PRINT "SINE OF 30 DEGREES:"; SIN(D * P / 180)
60 PRINT "ARCTANGENT OF 1 IN DEGREES:"; ATN(1) * 180 / P
```

```output
 3.14159265 
 1  1 
SINE OF 30 DEGREES: .5 
ARCTANGENT OF 1 IN DEGREES: 45 
```

A sine wave down the page, the way the first program in *BASIC Computer
Games* draws one:

```basic
10 FOR T = 0 TO 6.3 STEP .7
20   PRINT TAB(INT(15 + 14 * SIN(T))); "*"
30 NEXT T
```

```output
               *
                        *
                            *
                           *
                   *
          *
  *
 *
      *
```

## What a function cannot take

A function stops the program when it is given a value it has no answer for.

```basic
10 PRINT SQR(16)
20 PRINT SQR(-16)
```

```output
 4 
Runtime error in line 20: Illegal function call: SQR
```

Chapter 17 explains why the BASIC of 1964 printed 4 for that.

```basic
10 PRINT LOG(0)
```

```output
Runtime error in line 10: Illegal function call: LOG
```

## DEF: functions of your own

`DEF FNA(X) = formula` defines a function called `FNA`. The name must start
with `FN`. It is then used like any built-in function.

```basic
10 DEF FNC(F) = (F - 32) * 5 / 9
20 DEF FNF(C) = C * 9 / 5 + 32
30 PRINT FNC(212); FNC(32); FNF(37)
40 PRINT FNF(FNC(50))
```

```output
 100  0  98.6 
 50 
```

The name in brackets is a *parameter*. It belongs to the function. Calling
the function does not change a variable of the same name elsewhere in the
program.

```basic
10 X = 100
20 DEF FNS(X) = X * X
30 PRINT FNS(7)
40 PRINT X
```

```output
 49 
 100 
```

A function may use the program's other variables as well as its parameters.
It sees the value they have when it is called, not when it was defined.

```basic
10 DEF FNT(P) = P + P * R / 100
20 R = 10
30 PRINT FNT(200)
40 R = 20
50 PRINT FNT(200)
```

```output
 220 
 240 
```

A function can have several parameters, or none, and can give a string if
its name ends in `$`.

```basic
10 DEF FNM(A, B) = (A + B) / 2
20 DEF FNP = 4 * ATN(1)
30 DEF FNI$(A$) = LEFT$(A$, 1) + "."
40 PRINT FNM(10, 20)
50 PRINT FNP
60 PRINT FNI$("ADA") + FNI$("LOVELACE")
```

```output
 15 
 0 
A.L.
```

A `DEF` can be anywhere in the program. All the definitions are collected
before the program starts, so a function can be used on a line that comes
before its `DEF`. The first BASIC manual does exactly that in one of its
examples, which chapter 17 runs.

```basic
10 PRINT FNA(1)
20 DEF FNA(X) = X + 1
```

```output
 2 
```

Using a function that has no `DEF` at all stops the program:

```basic
10 PRINT FNZ(1)
```

```output
Runtime error in line 10: Undefined function FNZ
```

## Useful one-line functions

A few that turn up in program after program:

```basic
10 DEF FNR(X) = INT(X * 100 + .5) / 100
20 DEF FNM(A, B) = A - INT(A / B) * B
30 DEF FND(N) = INT(RND(1) * N) + 1
40 DEF FNX(A, B) = -(A > B) * A - (A <= B) * B
50 PRINT "ROUNDED:"; FNR(2 / 3)
60 PRINT "17 MOD 5:"; FNM(17, 5)
70 PRINT "A DIE:"; FND(6)
80 PRINT "LARGER OF 3 AND 8:"; FNX(3, 8)
```

```output
ROUNDED: .67 
17 MOD 5: 2 
A DIE: 2 
LARGER OF 3 AND 8: 8 
```

Line 40 uses the fact that a true comparison is -1 and a false one 0, which
was covered in chapter 5.

## Functions of several lines

When one formula is not enough, leave out the `=` and write the function over
several lines, ending with `FNEND`. Give the result by assigning to the
function's own name.

```basic
10 DEF FNF(N)
20   FNF = 1
30   FOR I = 2 TO N
40     FNF = FNF * I
50   NEXT I
60 FNEND
70 FOR K = 1 TO 6: PRINT FNF(K);: NEXT K
80 PRINT
```

```output
 1  2  6  24  120  720 
```

Only the parameters are private. `I` in the function above is the program's
ordinary variable `I`, which is why line 70 uses `K` for its own loop.

A multi-line function can call itself, and each call has its own
parameters:

```basic
10 DEF FNG(A, B)
20   IF B = 0 THEN FNG = A ELSE FNG = FNG(B, A - INT(A / B) * B)
30 FNEND
40 PRINT FNG(48, 18); FNG(17, 5); FNG(1071, 462)
```

```output
 6  1  21 
```

Multi-line functions came from later Dartmouth BASICs. The microcomputer
BASICs had only the one-line form, so the game listings do not use them.

**Try it:** define `FNH(A, B)` to give the longest side of a right-angled
triangle from the other two, and `FNE(N)` to give 1 if N is even and 0 if it
is odd.
