# Random numbers

`RND(1)` gives a different number each time it is used, at least 0 and less
than 1. Games are built on it.

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

```output
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

## Whole numbers in a range

`RND(1)` is a fraction. To get a whole number from 1 to N, multiply by N,
round down, and add 1.

| Wanted | Formula |
| --- | --- |
| a fraction from 0 up to 1 | `RND(1)` |
| a whole number from 1 to 6 | `INT(RND(1) * 6) + 1` |
| a whole number from 0 to 9 | `INT(RND(1) * 10)` |
| a whole number from A to B | `INT(RND(1) * (B - A + 1)) + A` |

```basic
10 FOR I = 1 TO 4: PRINT RND(1);: NEXT I
20 PRINT
30 FOR I = 1 TO 10: PRINT INT(RND(1) * 6) + 1;: NEXT I
40 PRINT
50 FOR I = 1 TO 10: PRINT INT(RND(1) * 21) + 10;: NEXT I
60 PRINT
```

```output
 .194105918  .562631827  .486106138  .271105561 
 5  4  2  5  6  5  2  4  3  3 
 10  18  26  26  28  18  14  26  27  15 
```

Line 50 gives numbers from 10 to 30: there are 21 of them, starting at 10.

## The same numbers again

Each run of a program gets different numbers. The outputs in this chapter
were made with `--seed 42`, which makes the "random" numbers the same on
every run:

```bash
basic --seed 42 samples/tutorial/12_random.bas
```

Use a seed to debug a game, or to make a transcript that can be checked.

Inside a program, `RANDOMIZE 7` does the same job as `--seed 7`, and
`RANDOMIZE` on its own picks a fresh seed. Old listings often start with
`RANDOMIZE` because their computers gave the same numbers after every
switch-on. Here it is harmless and unnecessary.

```basic
10 RANDOMIZE 7
20 PRINT RND(1); RND(1)
30 RANDOMIZE 7
40 PRINT RND(1); RND(1)
```

```output
 .991367909  .538104888 
 .991367909  .538104888 
```

The argument of `RND` matters in one way. A positive number gives the next
random number, whatever the number is. `RND(0)` gives the last one again.

```basic
10 A = RND(1)
20 B = RND(0)
30 C = RND(1)
40 IF A = B THEN PRINT "RND(0) REPEATED THE LAST NUMBER"
50 IF C <> A THEN PRINT "RND(1) MOVED ON"
```

```output
RND(0) REPEATED THE LAST NUMBER
RND(1) MOVED ON
```

## Chances

`RND(1) < .25` is true a quarter of the time.

```basic
10 H = 0
20 FOR I = 1 TO 1000
30   IF RND(1) < .25 THEN H = H + 1
40 NEXT I
50 PRINT H; "OUT OF 1000"
```

```output
 241 OUT OF 1000
```

A coin, and a choice from a list:

```basic
10 FOR I = 1 TO 8
20   IF RND(1) < .5 THEN PRINT "H"; ELSE PRINT "T";
30 NEXT I
40 PRINT
50 DIM W$(4)
60 FOR I = 1 TO 4: READ W$(I): NEXT I
70 DATA NORTH, SOUTH, EAST, WEST
80 PRINT "GO "; W$(INT(RND(1) * 4) + 1)
```

```output
HTHHTTHT
GO WEST
```

## Shuffling

To deal cards, put the numbers 1 to N in an array and exchange each one with
another chosen at random.

```basic
10 N = 10
20 DIM D(N)
30 FOR I = 1 TO N: D(I) = I: NEXT I
40 FOR I = N TO 2 STEP -1
50   J = INT(RND(1) * I) + 1
60   T = D(I): D(I) = D(J): D(J) = T
70 NEXT I
80 FOR I = 1 TO N: PRINT D(I);: NEXT I
90 PRINT
```

```output
 1  8  9  7  3  5  10  4  6  2 
```

Every number appears once, which picking ten random numbers would not
guarantee.

## Guess the number

The smallest complete game:

```basic
10 N = INT(RND(1) * 100) + 1
20 T = 0
30 INPUT "YOUR GUESS"; G
40 T = T + 1
50 IF G < N THEN PRINT "TOO LOW": GOTO 30
60 IF G > N THEN PRINT "TOO HIGH": GOTO 30
70 PRINT "RIGHT, IN"; T; "GUESSES"
```

```input
50
25
12
18
21
20
```

```output
YOUR GUESS? 50
TOO HIGH
YOUR GUESS? 25
TOO HIGH
YOUR GUESS? 12
TOO LOW
YOUR GUESS? 18
TOO LOW
YOUR GUESS? 21
TOO HIGH
YOUR GUESS? 20
RIGHT, IN 6 GUESSES
```

## Estimating pi

Throw darts at a square. The share that land inside the quarter circle is
π/4.

```basic
10 H = 0
20 FOR I = 1 TO 20000
30   X = RND(1): Y = RND(1)
40   IF X * X + Y * Y < 1 THEN H = H + 1
50 NEXT I
60 PRINT "PI IS ABOUT"; 4 * H / 20000
```

```output
PI IS ABOUT 3.129 
```

**Try it:** roll two dice 3600 times and count how often each total from 2 to
12 comes up. Seven should be the most common.
