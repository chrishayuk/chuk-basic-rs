# Loops

A loop runs the same lines several times. BASIC has `FOR` for counting and
`WHILE` for "keep going until".

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

```output
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
- Indenting the lines inside a loop is only for the reader. BASIC ignores it.

## How FOR works

`FOR I = A TO B` sets I to A. `NEXT I` adds the step to I and, if I has not
gone past B, goes back to the line after the `FOR`. The start, the limit and
the step are worked out once, when the loop begins.

```basic
10 N = 3
20 FOR I = 1 TO N
30   N = 10
40   PRINT I;
50 NEXT I
60 PRINT
70 PRINT "AFTER THE LOOP I IS"; I
```

```output
 1  2  3 
AFTER THE LOOP I IS 4 
```

Changing N inside the loop did not change the number of turns. After the
loop the variable is one step past the limit, which is how you can tell
whether a loop ran to the end or was left early.

## STEP

```basic
10 FOR I = 0 TO 20 STEP 5: PRINT I;: NEXT I
20 PRINT
30 FOR I = 5 TO 1 STEP -1: PRINT I;: NEXT I
40 PRINT
50 FOR X = 0 TO 1 STEP .25: PRINT X;: NEXT X
60 PRINT
70 FOR I = 1 TO 10 STEP 4: PRINT I;: NEXT I
80 PRINT
```

```output
 0  5  10  15  20 
 5  4  3  2  1 
 0  .25  .5  .75  1 
 1  5  9 
```

Line 70 stops at 9, the last value that does not go past 10.

To count downwards the step must be negative. Without it the loop has
nothing to do and is skipped:

```basic
10 FOR I = 5 TO 1
20   PRINT "NEVER PRINTED"
30 NEXT I
40 PRINT "SKIPPED"
```

```output
SKIPPED
```

Some BASICs of the 1970s ran every loop at least once. This one follows
Dartmouth and the ANSI standard, and skips a loop whose range is empty.

## Adding up and counting

A variable set before the loop and changed inside it collects a total.

```basic
10 S = 0
20 C = 0
30 FOR I = 1 TO 20
40   S = S + I
50   IF I / 3 = INT(I / 3) THEN C = C + 1
60 NEXT I
70 PRINT "SUM:"; S
80 PRINT "MULTIPLES OF 3:"; C
```

```output
SUM: 210 
MULTIPLES OF 3: 6 
```

## Loops inside loops

Each `NEXT` closes the nearest open `FOR`. The inner loop runs through all of
its values for every single value of the outer one.

```basic
10 FOR R = 1 TO 4
20   FOR C = 1 TO 6
30     PRINT R * C;
40   NEXT C
50   PRINT
60 NEXT R
```

```output
 1  2  3  4  5  6 
 2  4  6  8  10  12 
 3  6  9  12  15  18 
 4  8  12  16  20  24 
```

The numbers do not line up, because some have one digit and some two.
`TAB` fixes that:

```basic
10 FOR R = 1 TO 4
20   FOR C = 1 TO 6
30     PRINT TAB(C * 4 - 4); R * C;
40   NEXT C
50   PRINT
60 NEXT R
```

```output
 1   2   3   4   5   6 
 2   4   6   8   10  12 
 3   6   9   12  15  18 
 4   8   12  16  20  24 
```

`NEXT` on its own closes the innermost loop, and `NEXT C, R` closes two at
once. Both are common in old listings.

```basic
10 FOR R = 1 TO 2: FOR C = 1 TO 3
20 PRINT R; C,
30 NEXT C, R
40 PRINT
```

```output
 1  1          1  2          1  3          2  1          2  2          2  3         
```

## Leaving a loop early

Jump out with `GOTO` or `IF ... THEN line`. The loop variable keeps the value
it had.

```basic
10 FOR N = 2 TO 100
20   IF N * N > 500 THEN 40
30 NEXT N
40 PRINT "FIRST SQUARE ABOVE 500 IS"; N * N; "FROM"; N
```

```output
FIRST SQUARE ABOVE 500 IS 529 FROM 23 
```

## WHILE and WEND

`WHILE condition` ... `WEND` repeats for as long as the condition is true.
The test is made before each turn, so the loop may not run at all.

```basic
10 N = 27
20 S = 0
30 WHILE N > 1
40   IF N / 2 = INT(N / 2) THEN N = N / 2 ELSE N = 3 * N + 1
50   S = S + 1
60 WEND
70 PRINT "REACHED 1 AFTER"; S; "STEPS"
```

```output
REACHED 1 AFTER 111 STEPS
```

Use `FOR` when you know how many turns there will be, and `WHILE` when you
only know when to stop. `WHILE` came later than the 1970s game listings,
which do the same thing with `IF` and `GOTO`:

```basic
10 N = 27
20 S = 0
30 IF N <= 1 THEN 70
40 IF N / 2 = INT(N / 2) THEN N = N / 2 ELSE N = 3 * N + 1
50 S = S + 1
60 GOTO 30
70 PRINT "REACHED 1 AFTER"; S; "STEPS"
```

```output
REACHED 1 AFTER 111 STEPS
```

A `WHILE` loop is a good way to ask until the answer is acceptable:

```basic
10 N = 0
20 WHILE N < 1 OR N > 6
30   INPUT "A NUMBER FROM 1 TO 6"; N
40 WEND
50 PRINT "THANK YOU FOR THE"; N
```

```input
9
0
4
```

```output
A NUMBER FROM 1 TO 6? 9
A NUMBER FROM 1 TO 6? 0
A NUMBER FROM 1 TO 6? 4
THANK YOU FOR THE 4 
```

**Try it:** change line 130 of the first program to add up only the odd
numbers from 1 to 100. Then print a triangle of stars that is widest at the
top.
