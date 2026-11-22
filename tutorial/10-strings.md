# Strings

A string is a piece of text. BASIC's string functions take strings apart and
put them together, and they are what every text game uses to understand what
was typed.

**`samples/tutorial/10_strings.bas`**

```basic
10 A$ = "DARTMOUTH BASIC"
20 PRINT LEN(A$)
30 PRINT LEFT$(A$, 9)
40 PRINT RIGHT$(A$, 5)
50 PRINT MID$(A$, 6, 5)
60 PRINT ASC("A"); CHR$(66)
70 PRINT VAL("1964") + 1; STR$(64) + "!"
80 FOR I = 1 TO LEN(A$)
90   IF MID$(A$, I, 1) = " " THEN P = I
100 NEXT I
110 PRINT "FIRST WORD: "; LEFT$(A$, P - 1)
120 PRINT "LAST WORD: "; MID$(A$, P + 1)
130 IF "APPLE" < "BANANA" THEN PRINT "APPLE SORTS BEFORE BANANA"
```

```output
 15 
DARTMOUTH
BASIC
OUTH 
 65 B
 1965  64!
FIRST WORD: DARTMOUTH
LAST WORD: BASIC
APPLE SORTS BEFORE BANANA
```

| Function | Gives |
| --- | --- |
| `LEN(A$)` | the number of characters |
| `LEFT$(A$, n)` | the first `n` characters |
| `RIGHT$(A$, n)` | the last `n` characters |
| `MID$(A$, p, n)` | `n` characters starting at position `p`; the first is 1 |
| `MID$(A$, p)` | everything from position `p` on |
| `INSTR(A$, B$)` | where `B$` first appears in `A$`, or 0 |
| `ASC(A$)` | the character code of the first character |
| `CHR$(n)` | the character with code `n` |
| `VAL(A$)` | the number written in a string |
| `STR$(n)` | a number as a string |
| `SPACE$(n)` | `n` spaces |
| `STRING$(n, A$)` | the first character of `A$`, `n` times |

A function whose name ends in `$` gives a string. The others give a number.

## Taking strings apart

Positions count from 1. Asking for more characters than there are gives as
many as there are, and never an error.

```basic
10 A$ = "PROGRAM"
20 PRINT LEFT$(A$, 3)
30 PRINT RIGHT$(A$, 4)
40 PRINT MID$(A$, 4, 2)
50 PRINT MID$(A$, 4)
60 PRINT LEFT$(A$, 99)
70 PRINT "["; MID$(A$, 20, 2); "]"
80 PRINT LEN(A$); LEN(""); LEN("A B")
```

```output
PRO
GRAM
GR
GRAM
PROGRAM
[]
 7  0  3 
```

A loop over `MID$(A$, I, 1)` visits one character at a time. This reverses a
string:

```basic
10 A$ = "DARTMOUTH"
20 R$ = ""
30 FOR I = LEN(A$) TO 1 STEP -1
40   R$ = R$ + MID$(A$, I, 1)
50 NEXT I
60 PRINT R$
```

```output
HTUOMTRAD
```

And this counts the vowels in a sentence:

```basic
10 A$ = "THE QUICK BROWN FOX JUMPS OVER THE LAZY DOG"
20 N = 0
30 FOR I = 1 TO LEN(A$)
40   IF INSTR("AEIOU", MID$(A$, I, 1)) > 0 THEN N = N + 1
50 NEXT I
60 PRINT N; "VOWELS"
```

```output
 11 VOWELS
```

## Searching: INSTR

`INSTR(A$, B$)` gives the position of the first place `B$` appears in `A$`,
and 0 if it does not appear. A starting position can be given first:
`INSTR(5, A$, B$)` begins looking at position 5.

```basic
10 A$ = "THE CAT SAT ON THE MAT"
20 PRINT INSTR(A$, "CAT")
30 PRINT INSTR(A$, "DOG")
40 PRINT INSTR(A$, "THE")
50 PRINT INSTR(2, A$, "THE")
```

```output
 5 
 0 
 1 
 16 
```

Splitting a sentence into words is a loop of `INSTR` calls, each starting
after the last space found:

```basic
10 A$ = "TAKE THE BRASS LAMP"
20 P = 1
30 S = INSTR(P, A$, " ")
40 IF S = 0 THEN PRINT MID$(A$, P): END
50 PRINT MID$(A$, P, S - P)
60 P = S + 1
70 GOTO 30
```

```output
TAKE
THE
BRASS
LAMP
```

## Characters and codes

Every character has a number, its ASCII code. `ASC` gives the code of a
character and `CHR$` goes the other way.

```basic
10 PRINT ASC("A"); ASC("B"); ASC("Z"); ASC("a"); ASC("0"); ASC(" ")
20 PRINT CHR$(72); CHR$(73)
30 FOR I = 65 TO 90: PRINT CHR$(I);: NEXT I
40 PRINT
```

```output
 65  66  90  97  48  32 
HI
ABCDEFGHIJKLMNOPQRSTUVWXYZ
```

`CHR$(34)` is the double quote, which cannot be written inside a string:

```basic
10 Q$ = CHR$(34)
20 PRINT "SHE SAID "; Q$; "HELLO"; Q$
```

```output
SHE SAID "HELLO"
```

The capital letters are 65 to 90 and the small letters 97 to 122, a
difference of 32. That is enough to turn small letters into capitals, which
a program should do before comparing what somebody typed:

```basic
10 A$ = "Hello, World"
20 U$ = ""
30 FOR I = 1 TO LEN(A$)
40   C = ASC(MID$(A$, I, 1))
50   IF C >= 97 AND C <= 122 THEN C = C - 32
60   U$ = U$ + CHR$(C)
70 NEXT I
80 PRINT U$
```

```output
HELLO, WORLD
```

Codes also turn a letter into a number for arithmetic. This is a Caesar
cipher that moves each letter three places along the alphabet:

```basic
10 A$ = "ATTACK AT DAWN"
20 FOR I = 1 TO LEN(A$)
30   C = ASC(MID$(A$, I, 1))
40   IF C >= 65 AND C <= 90 THEN C = 65 + (C - 65 + 3) - 26 * INT((C - 65 + 3) / 26)
50   PRINT CHR$(C);
60 NEXT I
70 PRINT
```

```output
DWWDFN DW GDZQ
```

## Numbers and strings

`VAL` reads a number from the start of a string and stops at the first
character that cannot be part of one. A string with no number in it gives 0.

`STR$` writes a number as `PRINT` would, with the leading space for the
sign but without the trailing space.

```basic
10 PRINT VAL("42") + 1
20 PRINT VAL("3.5 METRES")
30 PRINT VAL("ABC")
40 PRINT "["; STR$(42); "]"; "["; STR$(-42); "]"
50 A$ = STR$(1964)
60 PRINT LEN(A$)
70 PRINT "THE YEAR " + MID$(A$, 2) + "."
```

```output
 43 
 3.5 
 0 
[ 42][-42]
 5 
THE YEAR 1964.
```

`MID$(STR$(N), 2)` drops the leading space. It is the usual way to put a
number into a sentence without a gap. `STR$` also lets you get at the digits
of a number:

```basic
10 N = 1964
20 A$ = MID$(STR$(N), 2)
30 S = 0
40 FOR I = 1 TO LEN(A$)
50   S = S + VAL(MID$(A$, I, 1))
60 NEXT I
70 PRINT "THE DIGITS OF"; N; "ADD UP TO"; S
```

```output
THE DIGITS OF 1964 ADD UP TO 20 
```

## Building strings

`SPACE$` and `STRING$` make runs of characters. With `LEN` they pad text to
a fixed width.

```basic
10 PRINT STRING$(20, "-")
20 PRINT "A"; SPACE$(18); "B"
30 PRINT STRING$(20, "-")
40 FOR I = 1 TO 3
50   READ N$, P
60   PRINT N$; STRING$(15 - LEN(N$), "."); P
70 NEXT I
80 DATA TEA, 2, COFFEE, 3, LEMONADE, 4
```

```output
--------------------
A                  B
--------------------
TEA............ 2 
COFFEE......... 3 
LEMONADE....... 4 
```

To right-align a number in a column, pad its string on the left:

```basic
10 FOR I = 1 TO 4
20   READ N
30   A$ = MID$(STR$(N), 2)
40   PRINT SPACE$(8 - LEN(A$)); A$
50 NEXT I
60 DATA 5, 1250, 37, 100000
```

```output
       5
    1250
      37
  100000
```

`PRINT USING`, in chapter 12, does this in one step.

## Comparing

Strings compare by character code, one character at a time, so the order is
alphabetical as long as the case is the same. Digits come before capitals,
and capitals before small letters.

```basic
10 IF "ADA" < "GRACE" THEN PRINT "ADA FIRST"
20 IF "10" < "9" THEN PRINT "AS TEXT, 10 COMES BEFORE 9"
30 IF "ADA" <> "ada" THEN PRINT "CASE MATTERS"
40 IF "ADA" <> "ADA " THEN PRINT "SO DOES A TRAILING SPACE"
```

```output
ADA FIRST
AS TEXT, 10 COMES BEFORE 9
CASE MATTERS
SO DOES A TRAILING SPACE
```

Sorting an array of strings is the bubble sort of chapter 8 with a `$` on
the names:

```basic
10 DIM N$(5)
20 FOR I = 1 TO 5: READ N$(I): NEXT I
30 DATA KURTZ, KEMENY, LOVELACE, HOPPER, BABBAGE
40 F = 0
50 FOR I = 1 TO 4
60   IF N$(I) > N$(I + 1) THEN T$ = N$(I): N$(I) = N$(I + 1): N$(I + 1) = T$: F = 1
70 NEXT I
80 IF F = 1 THEN 40
90 FOR I = 1 TO 5: PRINT N$(I); " ";: NEXT I
100 PRINT
```

```output
BABBAGE HOPPER KEMENY KURTZ LOVELACE 
```

## A palindrome test

Everything together: keep only the letters, then compare the string with
itself reversed.

```basic
10 LINE INPUT "A PHRASE: "; A$
20 L$ = ""
30 FOR I = 1 TO LEN(A$)
40   C$ = MID$(A$, I, 1)
50   IF C$ >= "A" AND C$ <= "Z" THEN L$ = L$ + C$
60 NEXT I
70 R$ = ""
80 FOR I = LEN(L$) TO 1 STEP -1: R$ = R$ + MID$(L$, I, 1): NEXT I
90 IF L$ = R$ THEN PRINT "A PALINDROME" ELSE PRINT "NOT A PALINDROME"
```

```input
A MAN, A PLAN, A CANAL: PANAMA
```

```output
A PHRASE: A MAN, A PLAN, A CANAL: PANAMA
A PALINDROME
```

**Try it:** ask for a full name and print the initials. Then print the name
with the surname first, followed by a comma.
