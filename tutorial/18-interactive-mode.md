# Interactive mode

Run the interpreter with no file and it gives you the prompt that BASIC
always had. Type numbered lines to build a program, and commands to work
with it.

```bash
basic
```

In this chapter, each session is shown as what was typed, followed by
everything the prompt printed. `Ok` means it is ready for the next line.

```repl
10 PRINT "HELLO"
20 PRINT "WORLD"
RUN
```

```output
CHUK-BASIC 0.1.4 - type HELP for help
Ok
HELLO
WORLD
Ok
```

## Lines and commands

A line that starts with a number is stored. Nothing happens until `RUN`.
Lines are kept in numerical order, whatever order they are typed in.

```repl
30 PRINT "THREE"
10 PRINT "ONE"
20 PRINT "TWO"
LIST
RUN
```

```output
CHUK-BASIC 0.1.4 - type HELP for help
Ok
10 PRINT "ONE"
20 PRINT "TWO"
30 PRINT "THREE"
Ok
ONE
TWO
THREE
Ok
```

`LIST` shows the program. Typing a line number that is already there
replaces the line, and a number on its own deletes it. That is how a program
is edited: type the line again.

```repl
10 PRINT "ONE"
20 PRINT "TOO"
30 PRINT "THREE"
20 PRINT "TWO"
30
LIST
```

```output
CHUK-BASIC 0.1.4 - type HELP for help
Ok
10 PRINT "ONE"
20 PRINT "TWO"
Ok
```

This is why programs are numbered in tens. A forgotten line can be given a
number in between:

```repl
10 INPUT "A NUMBER"; N
20 PRINT N * N
15 PRINT "ITS SQUARE IS";
LIST
```

```output
CHUK-BASIC 0.1.4 - type HELP for help
Ok
10 INPUT "A NUMBER"; N
15 PRINT "ITS SQUARE IS";
20 PRINT N * N
Ok
```

`LIST` can show part of a long program: `LIST 20` for one line, `LIST 10-50`
for a range, `LIST -50` for everything up to line 50 and `LIST 50-` for
everything from it.

```repl
10 A = 1
20 B = 2
30 C = 3
40 PRINT A + B + C
LIST 20
LIST 30-
LIST -20
```

```output
CHUK-BASIC 0.1.4 - type HELP for help
Ok
20 B = 2
Ok
30 C = 3
40 PRINT A + B + C
Ok
10 A = 1
20 B = 2
Ok
```

## Immediate statements

Anything typed without a line number is run at once. That makes the prompt a
calculator.

```repl
PRINT 355 / 113
PRINT SQR(2) * SQR(2)
? LEFT$("DARTMOUTH", 4)
FOR I = 1 TO 5: PRINT I * I;: NEXT I
```

```output
CHUK-BASIC 0.1.4 - type HELP for help
Ok
 3.14159292 
Ok
 2 
Ok
DART
Ok
 1  4  9  16  25 
Ok
```

Variables do not last from one immediate line to the next, or after a `RUN`.
Each one starts afresh.

```repl
X = 5
PRINT X
X = 5: PRINT X
```

```output
CHUK-BASIC 0.1.4 - type HELP for help
Ok
Ok
 0 
Ok
 5 
Ok
```

## A session

Writing a program, running it, finding a mistake and fixing it:

```repl
10 FOR I = 1 TO 5
20 S = S + I
30 NEXT
40 PRINT "TOTAL"; T
RUN
40 PRINT "TOTAL"; S
RUN
```

```output
CHUK-BASIC 0.1.4 - type HELP for help
Ok
TOTAL 0 
Ok
TOTAL 15 
Ok
```

`INPUT` in a program run from the prompt reads from the keyboard as usual.

## Mistakes

A mistake does not end the session. The prompt reports it and waits.

```repl
PRINT 1 / 0
PRINT (2 + 3
10 GOTO 100
RUN
```

```output
CHUK-BASIC 0.1.4 - type HELP for help
Ok
?Runtime error: Division by zero
Ok
?Syntax error at source line 1: Expected ')', found end of line
Ok
?Runtime error in line 10: Undefined line 100
Ok
```

## Saving and loading

`SAVE "name"` writes the program to a file, and `LOAD "name"` reads one in,
replacing the program in memory. The file is ordinary text, the same as the
`.bas` files in `samples`, so a program can be written in an editor and
loaded, or typed at the prompt and saved.

```repl
LOAD "samples/tutorial/01_hello.bas"
LIST
RUN
```

```output
CHUK-BASIC 0.1.4 - type HELP for help
Ok
Ok
10 PRINT "HELLO, WORLD!"
20 PRINT "THIS IS CHUK-BASIC."
30 END
Ok
HELLO, WORLD!
THIS IS CHUK-BASIC.
Ok
```

`NEW` forgets the program. `BYE` leaves.

```repl
10 PRINT "SOON FORGOTTEN"
NEW
LIST
RUN
```

```output
CHUK-BASIC 0.1.4 - type HELP for help
Ok
Ok
Ok
Ok
```

## The commands

| | |
| --- | --- |
| `10 PRINT "HI"` | store line 10, replacing any line 10 already there |
| `10` | delete line 10 |
| `LIST`, `LIST 10-50` | show the program, or part of it |
| `RUN` | run it |
| `NEW` | forget it |
| `LOAD "file"` | read a program from a file |
| `SAVE "file"` | write the program to a file |
| `HELP` | a summary of these commands |
| `BYE` | leave |
| anything else | run at once |

**Try it:** type in the guessing game from chapter 13 at the prompt, play it,
then change it to pick a number from 1 to 1000 and `SAVE` it.
