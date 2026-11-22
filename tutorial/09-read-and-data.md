# READ and DATA

`DATA` lines hold values inside the program. `READ` takes the next values in
order.

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

```output
MERCURY    58 MILLION KM FROM THE SUN
VENUS      108 MILLION KM FROM THE SUN
EARTH      150 MILLION KM FROM THE SUN
MARS       228 MILLION KM FROM THE SUN
THE FIRST OF 4 IS MERCURY
```

- Each `READ` takes the next values, wherever the `DATA` lines happen to be.
  They are usually gathered at the end.
- Strings in `DATA` only need quotes if they contain a comma.
- `RESTORE` goes back to the first item, so the data can be read again.

## One long list

All the `DATA` lines in a program make one list, in line-number order. It
does not matter how the values are divided between lines, or where the lines
are. A `DATA` line does nothing when the program runs through it.

```basic
10 DATA 1, 2
20 READ A, B, C
30 PRINT A; B; C
40 DATA 3, 4, 5
50 READ D
60 READ E
70 PRINT D; E
```

```output
 1  2  3 
 4  5 
```

## Numbers and strings

A `READ` into a string variable takes the next item as text. A `READ` into a
number variable needs the next item to be a number.

```basic
10 READ N$, A
20 PRINT N$; " IS"; A
30 READ X
40 DATA ADA, 36, GRACE
```

```output
ADA IS 36 
Runtime error in line 30: Bad number in DATA: GRACE
```

Line 30 found `GRACE` where it wanted a number. Keep the order of the `READ`
variables and the order of the `DATA` items in step.

Quotes are needed round an item that contains a comma, or that starts or
ends with spaces that matter:

```basic
10 FOR I = 1 TO 3
20   READ T$
30   PRINT "["; T$; "]"
40 NEXT I
50 DATA PLAIN TEXT, "LONDON, ENGLAND", "  SPACES KEPT  "
```

```output
[PLAIN TEXT]
[LONDON, ENGLAND]
[  SPACES KEPT  ]
```

## Running out

Reading more than there is stops the program.

```basic
10 FOR I = 1 TO 4
20   READ X
30   PRINT X;
40 NEXT I
50 DATA 10, 20, 30
```

```output
 10  20  30 Runtime error in line 20: Out of DATA
```

There are two ways to avoid counting. Put the number of items first, as the
program at the top of this chapter does, or finish with a value that cannot
be real data and stop on it:

```basic
10 S = 0: N = 0
20 READ X
30 IF X = -1 THEN 70
40 S = S + X: N = N + 1
50 GOTO 20
60 DATA 4, 8, 15, 16, 23, 42, -1
70 PRINT N; "ITEMS, TOTAL"; S
```

```output
 6 ITEMS, TOTAL 108 
```

The -1 is called a *sentinel*. Items can now be added without changing
anything else.

## RESTORE

`RESTORE` moves back to the first item. `RESTORE 200` moves to the first
item on line 200, so a program can keep several tables and choose between
them.

```basic
10 FOR T = 1 TO 2
20   IF T = 1 THEN RESTORE 100
30   IF T = 2 THEN RESTORE 200
40   FOR I = 1 TO 3: READ W$: PRINT W$; " ";: NEXT I
50   PRINT
60 NEXT T
100 DATA ONE, TWO, THREE
200 DATA UN, DEUX, TROIS
```

```output
ONE TWO THREE 
UN DEUX TROIS 
```

## Looking things up

A table in `DATA` and a loop that reads until it finds what it wants:

```basic
10 INPUT "WHICH PLANET"; P$
20 RESTORE
30 FOR I = 1 TO 4
40   READ N$, M
50   IF N$ = P$ THEN PRINT N$; " HAS"; M; "MOONS": END
60 NEXT I
70 PRINT "I DO NOT KNOW "; P$
80 DATA EARTH, 1, MARS, 2, VENUS, 0, MERCURY, 0
```

```input
MARS
```

```output
WHICH PLANET? MARS
MARS HAS 2 MOONS
```

## Data for pictures

The old games drew their title pictures from `DATA`: pairs of numbers saying
how many spaces and how many stars.

```basic
10 FOR R = 1 TO 5
20   READ S, N
30   PRINT TAB(S);
40   FOR I = 1 TO N: PRINT "*";: NEXT I
50   PRINT
60 NEXT R
70 DATA 4, 1, 3, 3, 2, 5, 1, 7, 4, 1
```

```output
    *
   ***
  *****
 *******
    *
```

**Try it:** put the names of the months and the number of days in each into
`DATA`, ask for a month number, and print its name and length.
