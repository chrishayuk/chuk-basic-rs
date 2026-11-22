# Arrays

An **array** is a row of numbered boxes with one name. `DIM S(5)` makes the
boxes `S(0)` to `S(5)`.

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

```output
TOTAL: 150  AVERAGE: 30 
 1  0  0 
 0  1  0 
 0  0  1 
TWO ONE ZERO 
```

- The number in brackets, the *subscript*, can be any expression. That is
  what makes arrays and `FOR` loops such natural partners.
- Arrays can have two or more dimensions: `G(R, C)` is a grid.
- Arrays of strings have a `$` in their name.

## DIM

`DIM A(20)` makes 21 boxes, numbered 0 to 20. Many programs ignore box 0 and
count from 1. Every box starts at 0, or the empty string.

```basic
10 DIM A(20)
20 PRINT A(0); A(20)
30 A(20) = 7
40 PRINT A(20)
50 PRINT A(21)
```

```output
 0  0 
 7 
Runtime error in line 50: Subscript out of range
```

Using a subscript outside the array stops the program. An array used without
a `DIM` gets subscripts 0 to 10:

```basic
10 B(10) = 1
20 PRINT B(10)
30 B(11) = 1
```

```output
 1 
Runtime error in line 30: Subscript out of range
```

Several arrays can be dimensioned in one statement, and the size can be a
variable, so a program can ask how much room it needs.

```basic
10 INPUT "HOW MANY NUMBERS"; N
20 DIM X(N), N$(3), G(2, 2)
30 FOR I = 1 TO N: X(I) = I * I: NEXT I
40 PRINT "THE LAST ONE IS"; X(N)
```

```input
50
```

```output
HOW MANY NUMBERS? 50
THE LAST ONE IS 2500 
```

An array and a plain variable may share a name. `S` and `S(1)` are different
things, and so are `S$` and `S$(1)`.

```basic
10 S = 99
20 S(1) = 5
30 PRINT S; S(1)
```

```output
 99  5 
```

## Filling and searching

Reading values into an array and then working through it is the pattern
behind most programs that handle a list.

```basic
10 DIM T(7)
20 FOR D = 1 TO 7: READ T(D): NEXT D
30 DATA 12, 15, 11, 18, 21, 19, 14
40 H = 1
50 FOR D = 2 TO 7
60   IF T(D) > T(H) THEN H = D
70 NEXT D
80 PRINT "WARMEST DAY WAS DAY"; H; "AT"; T(H); "DEGREES"
90 S = 0
100 FOR D = 1 TO 7: S = S + T(D): NEXT D
110 PRINT "AVERAGE"; S / 7
```

```output
WARMEST DAY WAS DAY 5 AT 21 DEGREES
AVERAGE 15.7142857 
```

`H` holds the *position* of the largest value seen so far, not the value
itself. That way the program can report both.

## Counting with an array

Use a value as a subscript to count how often it turns up.

```basic
10 DIM C(5)
20 FOR I = 1 TO 12
30   READ V
40   C(V) = C(V) + 1
50 NEXT I
60 DATA 3, 1, 4, 1, 5, 2, 2, 5, 3, 5, 1, 5
70 FOR V = 1 TO 5
80   PRINT V; "APPEARS"; C(V); "TIMES"
90 NEXT V
```

```output
 1 APPEARS 3 TIMES
 2 APPEARS 2 TIMES
 3 APPEARS 2 TIMES
 4 APPEARS 1 TIMES
 5 APPEARS 4 TIMES
```

## Sorting

A bubble sort compares neighbours and exchanges any pair that is in the
wrong order, again and again until a whole pass makes no exchange.

```basic
10 DIM A(8)
20 FOR I = 1 TO 8: READ A(I): NEXT I
30 DATA 34, 7, 23, 32, 5, 62, 32, 2
40 F = 0
50 FOR I = 1 TO 7
60   IF A(I) <= A(I + 1) THEN 80
70   T = A(I): A(I) = A(I + 1): A(I + 1) = T: F = 1
80 NEXT I
90 IF F = 1 THEN 40
100 FOR I = 1 TO 8: PRINT A(I);: NEXT I
110 PRINT
```

```output
 2  5  7  23  32  32  34  62 
```

Line 70 exchanges two values through a spare variable `T`. `F` is a flag
that records whether this pass changed anything.

## Two dimensions

`DIM G(3, 4)` is a table with rows 0 to 3 and columns 0 to 4. Two nested
loops visit every box.

```basic
10 DIM M(3, 4)
20 FOR R = 1 TO 3
30   FOR C = 1 TO 4
40     M(R, C) = R * 10 + C
50   NEXT C
60 NEXT R
70 FOR R = 1 TO 3
80   FOR C = 1 TO 4: PRINT M(R, C);: NEXT C
90   PRINT
100 NEXT R
```

```output
 11  12  13  14 
 21  22  23  24 
 31  32  33  34 
```

A board game keeps its board this way. This sets up noughts and crosses and
prints it:

```basic
10 DIM B$(3, 3)
20 FOR R = 1 TO 3: FOR C = 1 TO 3: B$(R, C) = ".": NEXT C: NEXT R
30 B$(2, 2) = "X": B$(1, 3) = "O": B$(3, 1) = "X"
40 FOR R = 1 TO 3
50   FOR C = 1 TO 3: PRINT B$(R, C); " ";: NEXT C
60   PRINT
70 NEXT R
```

```output
. . O 
. X . 
X . . 
```

## Subscripts are rounded

A subscript that is not a whole number is rounded to the nearest one.

```basic
10 DIM A(5)
20 A(2) = 22: A(3) = 33
30 PRINT A(2.4); A(2.6)
```

```output
 22  33 
```

Chapter 14 describes the `MAT` statements, which fill, print, add and
multiply whole arrays in one go.

**Try it:** read ten numbers into an array and print them in reverse order.
Then print only those that are above the average.
