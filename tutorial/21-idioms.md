# Idioms and common mistakes

BASIC is small, so the same few lines turn up in program after program. This
chapter collects them, then the habits of the 1970s listings, then the
mistakes everybody makes.

## Everyday idioms

**Remainder.** There is no remainder operator.

```basic
10 A = 47: B = 5
20 PRINT A - INT(A / B) * B
```

```output
 2 
```

**Divisible by.**

```basic
10 FOR N = 1 TO 20
20   IF N / 4 = INT(N / 4) THEN PRINT N;
30 NEXT N
40 PRINT
```

```output
 4  8  12  16  20 
```

**Round to the nearest whole number, and to two places.**

```basic
10 X = 3.14159
20 PRINT INT(X + .5); INT(X * 100 + .5) / 100
```

```output
 3  3.14 
```

**Exchange two variables.** A third is needed.

```basic
10 A = 1: B = 2
20 T = A: A = B: B = T
30 PRINT A; B
```

```output
 2  1 
```

**Largest of a list.** Start with the first and replace it whenever a bigger
one turns up.

```basic
10 READ N, M
20 FOR I = 2 TO N
30   READ X
40   IF X > M THEN M = X
50 NEXT I
60 PRINT "LARGEST:"; M
70 DATA 5, 12, 7, 31, 9, 24
```

```output
LARGEST: 31 
```

**A random whole number from 1 to N.**

```basic
10 N = 6
20 PRINT INT(RND(1) * N) + 1
```

```output
 2 
```

**Yes or no.** Look at the first letter only.

```basic
10 INPUT "AGAIN"; A$
20 IF LEFT$(A$, 1) = "Y" THEN PRINT "ONCE MORE" ELSE PRINT "THE END"
```

```input
YES PLEASE
```

```output
AGAIN? YES PLEASE
ONCE MORE
```

**Ask until the answer is allowed.**

```basic
10 INPUT "A NUMBER FROM 1 TO 9"; N
20 IF N < 1 OR N > 9 OR N <> INT(N) THEN PRINT "TRY AGAIN.": GOTO 10
30 PRINT "THANK YOU."
```

```input
12
2.5
7
```

```output
A NUMBER FROM 1 TO 9? 12
TRY AGAIN.
A NUMBER FROM 1 TO 9? 2.5
TRY AGAIN.
A NUMBER FROM 1 TO 9? 7
THANK YOU.
```

**A number in a sentence with no gap.**

```basic
10 N = 3
20 PRINT "YOU HAVE " + MID$(STR$(N), 2) + " LIVES LEFT."
```

```output
YOU HAVE 3 LIVES LEFT.
```

**Singular and plural.**

```basic
10 FOR N = 1 TO 2
20   PRINT N; "POINT";
30   IF N <> 1 THEN PRINT "S";
40   PRINT
50 NEXT N
```

```output
 1 POINT
 2 POINTS
```

**A line of something.**

```basic
10 PRINT STRING$(30, "=")
20 FOR I = 1 TO 30: PRINT "-";: NEXT I: PRINT
```

```output
==============================
------------------------------
```

Line 20 is how it was done before `STRING$`.

**A flag.** A variable that is 0 or 1 to record that something happened.

```basic
10 F = 0
20 FOR I = 1 TO 5
30   READ X
40   IF X < 0 THEN F = 1
50 NEXT I
60 IF F = 1 THEN PRINT "THERE WAS A NEGATIVE NUMBER"
70 DATA 4, 8, -3, 9, 1
```

```output
THERE WAS A NEGATIVE NUMBER
```

## Reading old listings

The games in `samples/games` were written when memory was scarce and every
character cost something. They run here unchanged, but they take some
getting used to.

**No spaces.** Keywords are run together with what follows.

```basic
10 FORI=1TO3:PRINTI;:NEXTI
20 PRINT
30 A$="Y":IFA$="Y"THENPRINT"YES"
```

```output
 1  2  3 
YES
```

A line is first read the ordinary way, with spaces separating words. If that
makes no sense it is read again the way the first Microsoft BASICs read
everything, finding keywords wherever they occur.

**THEN followed by a line number, and GOTO spelt as two words.**

```basic
10 X = 5
20 IF X > 3 THEN 50
30 PRINT "SMALL"
40 GO TO 60
50 PRINT "BIG"
60 PRINT "DONE"
```

```output
BIG
DONE
```

**PRINT items with nothing between them.** A semicolon is understood.

```basic
10 X = 7
20 PRINT "X IS" X "AND TWICE X IS" 2 * X
```

```output
X IS 7 AND TWICE X IS 14 
```

**A missing closing quote** at the end of a line.

```basic
10 PRINT "NO CLOSING QUOTE
20 PRINT "DONE"
```

```output
NO CLOSING QUOTE
DONE
```

**Arrays with no DIM.** They get subscripts 0 to 10.

**Truth as a number.** A comparison is -1 or 0 and can be used in
arithmetic. This adds 1 to S only when X is positive:

```basic
10 S = 10: X = 4
20 S = S - (X > 0)
30 PRINT S
```

```output
 11 
```

**Remarks that are not quite `REM`.** Any word beginning with `REM` at the
start of a statement is a remark.

```basic
10 REMARKABLE PROGRAM, THIS
20 PRINT "STILL RUNS" ' AND AN APOSTROPHE STARTS A REMARK TOO
```

```output
STILL RUNS
```

**Loops that only waste time.** `FOR I = 1 TO 2000: NEXT I` was a pause on a
slow machine. Here it takes no time you would notice.

**One huge line.** Everything after a `THEN` belongs to the `IF`, so a long
line is often a whole block.

```basic
10 H = 0
20 FOR T = 1 TO 3
30   IF T = 2 THEN H = H + 1: PRINT "HIT ON TURN"; T: PRINT "THAT MAKES"; H
40 NEXT T
```

```output
HIT ON TURN 2 
THAT MAKES 1 
```

## Common mistakes

**No END before the subroutines.** The program runs on into the first
subroutine and its `RETURN` has nowhere to go.

```basic
10 GOSUB 100
20 PRINT "BACK"
100 PRINT "IN THE SUBROUTINE"
110 RETURN
```

```output
IN THE SUBROUTINE
BACK
IN THE SUBROUTINE
Runtime error in line 110: RETURN without GOSUB
```

**Statements after THEN that were meant to run always.** Everything on the
line after `THEN` is conditional.

```basic
10 X = 0
20 IF X > 0 THEN PRINT "POSITIVE": PRINT "THIS LINE WAS MEANT TO PRINT ANYWAY"
30 PRINT "END"
```

```output
END
```

Put the second `PRINT` on a line of its own.

**Comparing fractions for equality.** A loop that should stop at 1 does not
get there exactly.

```basic
10 X = 0
20 X = X + .1
30 IF X = 1 THEN PRINT "REACHED 1": END
40 IF X < 2 THEN 20
50 PRINT "WENT STRAIGHT PAST 1"
```

```output
WENT STRAIGHT PAST 1
```

Test with `>=`, or count in whole numbers and divide.

**Two uses for one variable.** The subroutine below uses `I`, and so does the
loop that calls it.

```basic
10 FOR I = 1 TO 3
20   GOSUB 100
30 NEXT I
40 END
100 FOR I = 1 TO 5: PRINT "*";: NEXT I
110 PRINT
120 RETURN
```

```output
*****
```

Only one row was printed. The subroutine left I at 6, and the outer loop
thought it had finished.

**A misspelt name.** It is a new variable with the value 0, and nothing
reports it.

```basic
10 COUNT = 10
20 PRINT "COUNT IS"; CUONT
```

```output
COUNT IS 0 
```

**A number where a string is needed.** Use `STR$` to join a number to text,
or print them as separate items.

```basic
10 N = 5
20 PRINT "TOTAL: " + N
```

```output
Runtime error in line 20: Type mismatch
```

**An answer with a comma in it.** When `INPUT` asks for several values, a
comma ends each one. Ask for free text on its own, or with `LINE INPUT`.

```basic
10 INPUT "TOWN AND YEAR"; T$, Y
20 PRINT T$; Y
```

```input
PARIS, FRANCE, 1964
"PARIS, FRANCE", 1964
```

```output
TOWN AND YEAR? PARIS, FRANCE, 1964
?EXTRA IGNORED
?REDO FROM START
TOWN AND YEAR? "PARIS, FRANCE", 1964
PARIS, FRANCE 1964 
```

**Off by one.** `DIM A(10)` has eleven elements, 0 to 10, and
`FOR I = 0 TO 10` runs eleven times.

```basic
10 C = 0
20 FOR I = 0 TO 10: C = C + 1: NEXT I
30 PRINT C
```

```output
 11 
```

**Counting down without STEP.** The loop is skipped.

```basic
10 FOR I = 10 TO 1
20   PRINT I
30 NEXT I
40 PRINT "NOTHING WAS PRINTED ABOVE"
```

```output
NOTHING WAS PRINTED ABOVE
```

**Jumping out of a subroutine with GOTO.** Each `GOSUB` that never returns
is remembered for ever. Always leave by `RETURN`.

**A keyword inside a name on a line with no spaces.** `TOTAL=5` is fine, but
on a crunched line `FORTOTAL=1TO5` cannot be untangled, because `TO` appears
inside `TOTAL`. Keep spaces in your own programs, and use short names when
there are none.
