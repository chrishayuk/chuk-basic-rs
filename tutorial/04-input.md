# INPUT

`INPUT` asks the person at the keyboard for a value and waits.

**`samples/tutorial/04_input.bas`**

```basic
10 INPUT "WHAT IS YOUR NAME"; N$
20 INPUT "WHAT YEAR WERE YOU BORN"; Y
30 PRINT "HELLO, "; N$; "."
40 PRINT "IN 2030 YOU WILL TURN"; 2030 - Y
50 INPUT "TWO NUMBERS, SEPARATED BY A COMMA"; A, B
60 PRINT A; "+"; B; "="; A + B
```

```input
ADA
1990
3,4
```

```output
WHAT IS YOUR NAME? ADA
WHAT YEAR WERE YOU BORN? 1990
HELLO, ADA.
IN 2030 YOU WILL TURN 40 
TWO NUMBERS, SEPARATED BY A COMMA? 3,4
 3 + 4 = 7 
```

In these transcripts, the text after each `?` is what was typed.

- The text in quotes is the prompt. BASIC adds the `? ` itself.
- Several variables can be read at once. The answers are typed with commas
  between them.

## With and without a prompt

`INPUT` with no prompt prints only the question mark. A separate `PRINT`
ending in a semicolon does the same job as a prompt.

```basic
10 INPUT N
20 PRINT "TWICE THAT IS"; N * 2
30 PRINT "AND ANOTHER";
40 INPUT M
50 PRINT N + M
```

```input
21
8
```

```output
? 21
TWICE THAT IS 42 
AND ANOTHER? 8
 29 
```

A comma after the prompt, in place of the semicolon, leaves the question
mark out:

```basic
10 INPUT "TYPE A NUMBER: ", N
20 PRINT N
```

```input
5
```

```output
TYPE A NUMBER: 5
 5 
```

## Numbers and strings

A number variable needs a number. If something else is typed, BASIC says
`?REDO FROM START` and asks again. The program does not stop.

```basic
10 INPUT "YOUR AGE"; A
20 PRINT "YOU ARE"; A
```

```input
FORTY
40
```

```output
YOUR AGE? FORTY
?REDO FROM START
YOUR AGE? 40
YOU ARE 40 
```

A string variable on its own takes the whole line, commas included. Spaces
at the start and end are dropped.

```basic
10 INPUT "YOUR NAME"; N$
20 PRINT "HELLO, "; N$; "."
```

```input
   ADA LOVELACE
```

```output
YOUR NAME?    ADA LOVELACE
HELLO, ADA LOVELACE.
```

## Several values

`INPUT A, B` wants two values with a comma between them. If only one is
typed, BASIC asks for the rest with `??`. If too many are typed, the spare
ones are thrown away with a warning.

```basic
10 INPUT "TWO NUMBERS"; A, B
20 PRINT A * B
30 INPUT "TWO MORE"; C, D
40 PRINT C * D
```

```input
6
7
1, 2, 3
```

```output
TWO NUMBERS? 6
?? 7
 42 
TWO MORE? 1, 2, 3
?EXTRA IGNORED
 2 
```

When several values are asked for, a comma separates them, so a string
that contains a comma has to be typed in quotes:

```basic
10 INPUT "TOWN, AND ITS POPULATION IN MILLIONS"; T$, P
20 PRINT T$; " HAS"; P; "MILLION PEOPLE"
```

```input
"PARIS, FRANCE", 2
```

```output
TOWN, AND ITS POPULATION IN MILLIONS? "PARIS, FRANCE", 2
PARIS, FRANCE HAS 2 MILLION PEOPLE
```

## LINE INPUT

`LINE INPUT` reads the whole line into one string variable, commas, quotes
and all. It prints no question mark, so put any punctuation in the prompt.

```basic
10 LINE INPUT "ADDRESS: "; A$
20 PRINT "YOU TYPED: "; A$
```

```input
10 DOWNING STREET, LONDON
```

```output
ADDRESS: 10 DOWNING STREET, LONDON
YOU TYPED: 10 DOWNING STREET, LONDON
```

Use `LINE INPUT` when the person may type anything. Use `INPUT` for
numbers, and for several values on a line.

## Yes or no

A program that asks a yes-or-no question should look at the first letter
only, so that `Y`, `YES` and `YEP` all work.

```basic
10 INPUT "ANOTHER GAME (Y/N)"; A$
20 IF LEFT$(A$, 1) = "Y" THEN PRINT "HERE WE GO": GOTO 10
30 PRINT "GOODBYE"
```

```input
YES
Y
NO
```

```output
ANOTHER GAME (Y/N)? YES
HERE WE GO
ANOTHER GAME (Y/N)? Y
HERE WE GO
ANOTHER GAME (Y/N)? NO
GOODBYE
```

## Running with prepared answers

To run a program with answers from a file, feed the file in:

```bash
basic --echo samples/tutorial/04_input.bas < samples/tutorial/04_input.in
```

`--echo` copies the answers into the output so that it reads like the
transcripts above. Without it only the program's own output appears, as it
would on a screen where the keyboard does the echoing.

If the answers run out before the program has finished asking, it stops:

```basic
10 INPUT "FIRST"; A
20 INPUT "SECOND"; B
30 PRINT A + B
```

```input
1
```

```output
FIRST? 1
SECOND? Runtime error in line 20: Out of input
```

**Try it:** ask for a name and a year of birth, and print how old that person
will be in the year 2064, a hundred years after BASIC.
