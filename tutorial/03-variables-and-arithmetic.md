# Variables and arithmetic

A **variable** is a named box that holds a value. `LET W = 12` puts 12 in the
box called `W`.

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

```output
AREA: 60 
PERIMETER: 34 
DIAGONAL: 13 
HELLO, GRACE!
 14  20  1024  3.5 
NEVER SET: 0 []
```

- The word `LET` is optional. Line 30 leaves it out.
- A name ending in `$` holds text (a **string**). Any other name holds a
  number.
- The operators are `+ - * /` and `^` for powers.
- Strings are joined with `+`.
- A variable that has never been set is `0`, or the empty string.

## Names

Names start with a letter and can be any length. They are not
case-sensitive: `total`, `Total` and `TOTAL` are one variable.

```basic
10 SCORE = 10
20 score = score + 5
30 PRINT Score
40 PLAYER1$ = "ADA"
50 PRINT PLAYER1$
```

```output
 15 
ADA
```

The first BASICs allowed only a letter, or a letter and a digit: `A`, `B2`,
`X$`. Most old listings keep to that, which is why they are full of
one-letter names.

A name must not *be* a keyword, so there can be no variable called `PRINT`
or `TO`. It may contain one: `TOTAL` and `SCORE` are fine, although they
contain `TO` and `OR`.

## Assignment is not algebra

`=` in a `LET` means "work out the right-hand side, then put it in the box on
the left". So `N = N + 1` is not nonsense. It adds one to N.

```basic
10 N = 5
20 N = N + 1
30 PRINT N
40 N = N * 2
50 PRINT N
```

```output
 6 
 12 
```

Assigning to a variable replaces what was there. Other variables are not
affected, even if they were worked out from it.

```basic
10 A = 3
20 B = A * 2
30 A = 100
40 PRINT A; B
```

```output
 100  6 
```

## Order of calculation

Powers are done first, then `*` and `/`, then `+` and `-`. Operators of the
same rank are done from left to right. Brackets override all of it.

```basic
10 PRINT 2 + 3 * 4
20 PRINT (2 + 3) * 4
30 PRINT 20 - 5 - 3
40 PRINT 20 - (5 - 3)
50 PRINT 2 * 3 ^ 2
60 PRINT (2 * 3) ^ 2
70 PRINT -3 ^ 2
80 PRINT 2 ^ 3 ^ 2
```

```output
 14 
 20 
 12 
 18 
 18 
 36 
-9 
 64 
```

Line 70 is worth a second look. The power is done before the minus sign, so
`-3 ^ 2` is `-(3 ^ 2)`. Line 80 is done from the left: 8 squared.

When in doubt, add brackets. They cost nothing.

## Division and whole numbers

There is one kind of number, and `/` gives the exact quotient, not a whole
number. `INT` rounds down to a whole number.

```basic
10 PRINT 7 / 2
20 PRINT INT(7 / 2)
30 PRINT 7 - INT(7 / 2) * 2
40 PRINT INT(3.99); INT(-3.01)
```

```output
 3.5 
 3 
 1 
 3 -4 
```

Line 30 is the remainder when 7 is divided by 2. There is no remainder
operator in this BASIC, so the idiom is `A - INT(A / B) * B`.

`INT` always rounds *down*, so `INT(-3.01)` is -4. To round to the nearest
whole number, add a half first: `INT(X + .5)`.

```basic
10 X = 2.5
20 PRINT INT(X + .5)
30 X = 2.4
40 PRINT INT(X + .5)
50 P = 3.14159
60 PRINT INT(P * 100 + .5) / 100
```

```output
 3 
 2 
 3.14 
```

Line 60 rounds to two decimal places.

## Numbers are not exact

Numbers are held in binary, to about sixteen significant digits, and printed
to nine. Most decimal fractions cannot be held exactly.

```basic
10 PRINT .1 + .2
20 IF .1 + .2 = .3 THEN PRINT "EQUAL" ELSE PRINT "NOT EQUAL"
30 IF ABS(.1 + .2 - .3) < .000001 THEN PRINT "CLOSE ENOUGH"
```

```output
 .3 
NOT EQUAL
CLOSE ENOUGH
```

The sum prints as `.3` and still does not equal `.3`. Compare fractions with
a tolerance, as line 30 does, or work in whole numbers: count pennies, not
pounds.

## Strings

A string variable holds any text. `+` joins two strings into one.

```basic
10 F$ = "ADA"
20 L$ = "LOVELACE"
30 N$ = F$ + " " + L$
40 PRINT N$
50 PRINT N$ + "!"
60 E$ = ""
70 PRINT "["; E$; "]"
```

```output
ADA LOVELACE
ADA LOVELACE!
[]
```

A number and a string cannot be mixed. Putting text into a number variable,
or adding a number to a string, stops the program:

```basic
10 A$ = "AGE: "
20 PRINT A$ + 36
```

```output
Runtime error in line 20: Type mismatch
```

Chapter 10 shows how to turn one into the other with `STR$` and `VAL`.

## Several statements on a line

A colon separates statements on one line.

```basic
10 A = 1: B = 2: C = 3
20 PRINT A + B + C: PRINT "DONE"
```

```output
 6 
DONE
```

**Try it:** write a program that sets `C` to a temperature in Celsius and
prints it in Fahrenheit. Multiply by 9, divide by 5 and add 32.
