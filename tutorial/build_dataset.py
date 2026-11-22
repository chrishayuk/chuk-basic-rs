#!/usr/bin/env python3
"""Build dataset/basic.jsonl, a chat-format training set about BASIC.

Every program in an answer is run through the interpreter, and the output
shown in the answer is what it actually printed, so the dataset cannot drift
from the language. Run from this folder after building the interpreter:

    python3 build_dataset.py
"""
import json
import pathlib
import sys

from check_tutorial import interpreter, run

HERE = pathlib.Path(__file__).resolve().parent
SAMPLES = HERE.parent / "samples"
SYSTEM = ("You are a knowledgeable assistant that provides detailed information "
          "about the BASIC programming language.")
BASIC = interpreter()
rows = []


def output(src, stdin="", info="", fails=False):
    """What a program prints. It must succeed unless `fails` says otherwise."""
    text = run(BASIC, src + "\n", stdin, info).rstrip("\n")
    failed = "error in line" in text.splitlines()[-1] if text else False
    if failed != fails:
        sys.exit(f"program {'did not fail' if fails else 'failed'}:\n{src}\n{text}")
    return text


def prog(src, stdin="", info="", fails=False):
    """A program, its input if any, and its real output, as markdown."""
    src = src.strip("\n")
    text = f"```basic\n{src}\n```\n\n"
    if stdin:
        typed = ", then ".join(f"`{line}`" for line in stdin.strip("\n").split("\n"))
        text += f"With {typed} typed at the keyboard, the session is:\n\n"
    elif fails:
        text += "It stops with an error:\n\n"
    else:
        text += "It prints:\n\n"
    return text + f"```\n{output(src, stdin, info, fails)}\n```"


def qa(question, answer):
    rows.append({"messages": [
        {"role": "system", "content": SYSTEM},
        {"role": "user", "content": question},
        {"role": "assistant", "content": answer.strip()},
    ]})


# ---------------------------------------------------------------- the basics

qa("What is the BASIC programming language?",
   "BASIC is a programming language written at Dartmouth College in 1964 by John "
   "Kemeny and Thomas Kurtz so that anybody, not only engineers, could use a "
   "computer. A program is a list of numbered lines that run in numerical order. "
   "Each statement starts with a word such as `PRINT`, `INPUT`, `LET`, `IF`, `FOR` "
   "or `GOTO`. There are two kinds of value, numbers and strings, and a variable "
   "whose name ends in `$` holds a string. Here is a complete program:\n\n"
   + prog('10 PRINT "HELLO, WORLD!"\n20 END'))

qa("How do I print Hello World in BASIC?",
   "Use `PRINT` with the text in double quotes.\n\n" + prog('10 PRINT "HELLO, WORLD!"'))

qa("Why do BASIC programs have line numbers?",
   "The line number says where the line belongs. Lines run in numerical order, "
   "whatever order they were typed in, and statements such as `GOTO 100` use the "
   "number to say where to carry on. Counting in tens leaves room to add a line "
   "later.\n\n" + prog('30 PRINT "THREE"\n10 PRINT "ONE"\n20 PRINT "TWO"'))

qa("How do I end a BASIC program?",
   "`END` stops the program. It is optional after the last line, because a program "
   "also stops when it runs out of lines. It is needed when subroutines follow the "
   "main program, to stop the program running on into them. `STOP` does the same "
   "job anywhere in a program.\n\n"
   + prog('10 PRINT "MAIN PROGRAM"\n20 END\n30 PRINT "NEVER REACHED"'))

qa("Does BASIC care about capital letters?",
   "No. Keywords and variable names can be written in capitals or small letters, "
   "and `print`, `Print` and `PRINT` are the same word. Text inside quotes is "
   "printed exactly as written. Old listings are all in capitals because the "
   "machines had no small letters.\n\n"
   + prog('10 total = 5\n20 print TOTAL; "Mixed Case Text"'))

qa("How do I write a comment in BASIC?",
   "Start a statement with `REM`, for remark. Everything after it on the line is "
   "ignored. An apostrophe does the same and can follow a statement on the same "
   "line.\n\n"
   + prog("10 REM THIS PROGRAM GREETS THE WORLD\n20 PRINT \"HELLO\" ' A REMARK AFTER A STATEMENT"))

qa("How do I put several statements on one line in BASIC?",
   "Separate them with colons.\n\n" + prog('10 A = 1: B = 2: PRINT A + B'))

# ---------------------------------------------------------------- PRINT

qa("What is the difference between a semicolon and a comma in a BASIC PRINT statement?",
   "A semicolon prints the next item immediately after the last. A comma moves to "
   "the start of the next print zone, and zones are 14 columns wide, so commas "
   "make columns.\n\n"
   + prog('10 PRINT "A"; "B"; "C"\n20 PRINT "A", "B", "C"'))

qa("Why does BASIC print spaces around numbers?",
   "A number is printed with a space in front, where a minus sign would go, and "
   "a space after it. That keeps numbers apart when they are joined with "
   "semicolons.\n\n" + prog('10 PRINT 1; 2; -3; 4\n20 PRINT "["; 42; "]"'))

qa("How do I stop PRINT from starting a new line in BASIC?",
   "End the `PRINT` with a semicolon. The next `PRINT` carries on along the same "
   "line.\n\n" + prog('10 PRINT "ONE ";\n20 PRINT "TWO ";\n30 PRINT "THREE"'))

qa("How do I print a blank line in BASIC?",
   "`PRINT` with nothing after it prints an empty line.\n\n"
   + prog('10 PRINT "ABOVE"\n20 PRINT\n30 PRINT "BELOW"'))

qa("What does TAB do in BASIC?",
   "`TAB(n)` in a `PRINT` statement moves to column `n`, counting the first column "
   "as 0. If the line is already past that column it does nothing. `SPC(n)` prints "
   "`n` spaces from wherever the line has got to.\n\n"
   + prog('10 PRINT "0123456789012345678901234"\n20 PRINT TAB(5); "FIVE"; TAB(20); "TWENTY"\n30 PRINT "AB"; SPC(3); "CD"'))

qa("What does the question mark mean in a BASIC program?",
   "At the start of a statement `?` is short for `PRINT`.\n\n" + prog('10 ? "SHORT FOR PRINT"; 2 + 2'))

qa("How does BASIC print very large and very small numbers?",
   "Numbers are printed with up to nine significant digits. Very large and very "
   "small ones are printed in exponent form: `1E+20` is 1 followed by twenty "
   "zeros.\n\n" + prog('10 PRINT 1 / 3\n20 PRINT 2 ^ 40\n30 PRINT 1E20\n40 PRINT 1E-10'))

qa("How do I print a double quote character in BASIC?",
   "A string cannot contain a double quote, because the quote ends it. Use "
   "`CHR$(34)`, the character whose code is 34.\n\n"
   + prog('10 Q$ = CHR$(34)\n20 PRINT "SHE SAID "; Q$; "HELLO"; Q$'))

# ---------------------------------------------------------------- variables

qa("How do variables work in BASIC?",
   "A variable is a named box that holds a value. `LET X = 5` puts 5 in X, and the "
   "word `LET` is optional. A name ending in `$` holds a string. Any other name "
   "holds a number. A variable that has never been set is 0, or the empty "
   "string.\n\n"
   + prog('10 LET W = 12\n20 H = 5\n30 N$ = "AREA"\n40 PRINT N$; W * H\n50 PRINT "NEVER SET:"; Z'))

qa("What does N = N + 1 mean in BASIC?",
   "`=` in an assignment means \"work out the right-hand side, then store it in the "
   "variable on the left\". So `N = N + 1` adds one to N.\n\n"
   + prog('10 N = 5\n20 N = N + 1\n30 PRINT N'))

qa("What is the order of operations in BASIC?",
   "Powers (`^`) are done first, then unary minus, then `*` and `/`, then `+` and "
   "`-`. Operators of the same rank are done from left to right, and brackets "
   "override the order. Note that `-3 ^ 2` is -9, because the power is done before "
   "the minus.\n\n"
   + prog('10 PRINT 2 + 3 * 4\n20 PRINT (2 + 3) * 4\n30 PRINT 2 * 3 ^ 2\n40 PRINT -3 ^ 2'))

qa("How do I get the remainder of a division in BASIC?",
   "Classic BASIC has no remainder operator. Use `A - INT(A / B) * B`.\n\n"
   + prog('10 A = 47: B = 5\n20 PRINT A - INT(A / B) * B'))

qa("How do I do integer division in BASIC?",
   "`/` gives the exact quotient. Wrap it in `INT` to round down to a whole "
   "number.\n\n" + prog('10 PRINT 7 / 2\n20 PRINT INT(7 / 2)'))

qa("How do I round a number in BASIC?",
   "`INT` rounds down, so add a half first to round to the nearest whole number. "
   "To round to two decimal places, multiply by 100, round, and divide by 100.\n\n"
   + prog('10 X = 3.14159\n20 PRINT INT(X + .5)\n30 PRINT INT(X * 100 + .5) / 100'))

qa("What is the difference between INT and FIX in BASIC?",
   "`INT` gives the largest whole number not above its argument, so it rounds "
   "negative numbers away from zero. `FIX` cuts the fraction off. They differ only "
   "for negative numbers.\n\n" + prog('10 PRINT INT(2.7); FIX(2.7)\n20 PRINT INT(-2.7); FIX(-2.7)'))

qa("Why is .1 + .2 not equal to .3 in BASIC?",
   "Numbers are held in binary, and most decimal fractions cannot be held exactly. "
   "The sum prints as `.3` but differs from `.3` in the last binary place. Compare "
   "fractions with a tolerance, or work in whole numbers.\n\n"
   + prog('10 PRINT .1 + .2\n20 IF .1 + .2 = .3 THEN PRINT "EQUAL" ELSE PRINT "NOT EQUAL"\n'
          '30 IF ABS(.1 + .2 - .3) < .000001 THEN PRINT "CLOSE ENOUGH"'))

qa("How do I join strings in BASIC?",
   "Use `+`.\n\n" + prog('10 F$ = "ADA"\n20 L$ = "LOVELACE"\n30 PRINT F$ + " " + L$'))

qa("What does Type mismatch mean in BASIC?",
   "A string was used where a number was needed, or a number where a string was "
   "needed. Adding a number to a string is the usual cause.\n\n"
   + prog('10 N = 5\n20 PRINT "TOTAL: " + N', fails=True)
   + "\n\nUse `STR$` to turn the number into a string, or print the two as separate "
     "items: `PRINT \"TOTAL:\"; N`.")

qa("How do I swap two variables in BASIC?",
   "Use a third variable to hold one of the values.\n\n"
   + prog('10 A = 1: B = 2\n20 T = A: A = B: B = T\n30 PRINT A; B'))

# ---------------------------------------------------------------- INPUT

qa("How do I read input from the keyboard in BASIC?",
   "`INPUT` prints a prompt and a question mark, waits, and stores what is typed "
   "in a variable.\n\n"
   + prog('10 INPUT "WHAT IS YOUR NAME"; N$\n20 PRINT "HELLO, "; N$; "."', "ADA\n"))

qa("How do I read several values with one INPUT in BASIC?",
   "List the variables with commas between them. The person types the values with "
   "commas between them too.\n\n"
   + prog('10 INPUT "TWO NUMBERS"; A, B\n20 PRINT A; "+"; B; "="; A + B', "3, 4\n"))

qa("What does ?REDO FROM START mean in BASIC?",
   "`INPUT` wanted a number and was given something that is not one. BASIC asks "
   "the question again and the program carries on.\n\n"
   + prog('10 INPUT "YOUR AGE"; A\n20 PRINT "YOU ARE"; A', "FORTY\n40\n"))

qa("What is LINE INPUT in BASIC?",
   "`LINE INPUT` reads a whole line into one string variable, commas and quotes "
   "included. It prints no question mark of its own.\n\n"
   + prog('10 LINE INPUT "ADDRESS: "; A$\n20 PRINT "YOU TYPED: "; A$', "10 DOWNING STREET, LONDON\n"))

qa("How do I ask a yes or no question in BASIC?",
   "Read the answer into a string and look at its first letter, so that `Y` and "
   "`YES` both work.\n\n"
   + prog('10 INPUT "PLAY AGAIN"; A$\n20 IF LEFT$(A$, 1) = "Y" THEN PRINT "HERE WE GO" ELSE PRINT "GOODBYE"', "YES\n"))

qa("How do I keep asking until the answer is valid in BASIC?",
   "Check the answer and go back to the `INPUT` if it is not allowed.\n\n"
   + prog('10 INPUT "A NUMBER FROM 1 TO 9"; N\n'
          '20 IF N < 1 OR N > 9 OR N <> INT(N) THEN PRINT "TRY AGAIN.": GOTO 10\n'
          '30 PRINT "THANK YOU."', "12\n2.5\n7\n"))

# ---------------------------------------------------------------- decisions

qa("How does IF work in BASIC?",
   "`IF condition THEN statement` runs the statement only when the condition is "
   "true. `ELSE` gives an alternative. The comparisons are `=`, `<>`, `<`, `<=`, "
   "`>` and `>=`.\n\n"
   + prog('10 FOR N = -1 TO 1\n20   IF N < 0 THEN PRINT N; "IS NEGATIVE" ELSE PRINT N; "IS NOT NEGATIVE"\n30 NEXT N'))

qa("What does IF X > 5 THEN 100 mean in BASIC?",
   "A line number after `THEN` is short for `GOTO`. The program jumps to that line "
   "when the condition is true and carries on with the next line when it is "
   "not.\n\n"
   + prog('10 X = 7\n20 IF X > 5 THEN 50\n30 PRINT "SMALL"\n40 GOTO 60\n50 PRINT "BIG"\n60 PRINT "DONE"'))

qa("How do I use AND, OR and NOT in BASIC?",
   "`AND` needs both conditions to be true, `OR` needs at least one, and `NOT` "
   "reverses a condition. `AND` is done before `OR`.\n\n"
   + prog('10 FOR N = 0 TO 12 STEP 6\n20   IF N >= 1 AND N <= 10 THEN PRINT N; "IS FROM 1 TO 10"\n'
          '30   IF N < 1 OR N > 10 THEN PRINT N; "IS OUTSIDE"\n40   IF NOT (N = 6) THEN PRINT N; "IS NOT 6"\n50 NEXT N'))

qa("What value does a comparison have in BASIC?",
   "A comparison works out to a number: -1 when it is true and 0 when it is false. "
   "`IF` treats any number other than 0 as true. Old programs use this in "
   "arithmetic.\n\n" + prog('10 PRINT 3 < 5; 3 > 5\n20 S = 10: X = 4\n30 S = S - (X > 0)\n40 PRINT S'))

qa("What happens to the statements after THEN on the same line in BASIC?",
   "Everything on the line after `THEN` belongs to the `IF`. When the condition is "
   "false the rest of the line is skipped, including statements after colons.\n\n"
   + prog('10 X = 0\n20 IF X > 0 THEN PRINT "POSITIVE": PRINT "ALSO SKIPPED"\n30 PRINT "NEXT LINE"'))

qa("How do I test whether a number is even in BASIC?",
   "Divide by 2 and see whether the result is a whole number.\n\n"
   + prog('10 FOR N = 1 TO 4\n20   IF N / 2 = INT(N / 2) THEN PRINT N; "IS EVEN" ELSE PRINT N; "IS ODD"\n30 NEXT N'))

qa("What does ON ... GOTO do in BASIC?",
   "`ON N GOTO 100, 200, 300` jumps to the first line number when N is 1, the "
   "second when N is 2, and so on. If N is 0 or larger than the list, the program "
   "carries on with the next statement. `ON N GOSUB` calls a subroutine the same "
   "way.\n\n"
   + prog('10 FOR N = 1 TO 4\n20   ON N GOTO 100, 200, 300\n30   PRINT "NO SUCH CHOICE": GOTO 400\n'
          '100  PRINT "ONE": GOTO 400\n200  PRINT "TWO": GOTO 400\n300  PRINT "THREE"\n400 NEXT N'))

qa("How do strings compare in BASIC?",
   "Strings are compared character by character using the character codes, which "
   "gives alphabetical order when the case is the same. Capitals come before small "
   "letters, and a shorter string comes before a longer one that starts with it.\n\n"
   + prog('10 IF "APPLE" < "BANANA" THEN PRINT "APPLE COMES FIRST"\n'
          '20 IF "CAT" < "CATS" THEN PRINT "SHORTER COMES FIRST"\n30 IF "Z" < "a" THEN PRINT "CAPITALS COME FIRST"'))

# ---------------------------------------------------------------- loops

qa("How does a FOR loop work in BASIC?",
   "`FOR I = 1 TO 5` sets I to 1. `NEXT I` adds 1 to I and goes back to the line "
   "after the `FOR` unless I has gone past 5.\n\n"
   + prog('10 FOR I = 1 TO 5\n20   PRINT I; "SQUARED IS"; I * I\n30 NEXT I'))

qa("How do I count backwards in a BASIC FOR loop?",
   "Give a negative `STEP`. Without it, a loop from a larger number to a smaller "
   "one is skipped.\n\n"
   + prog('10 FOR T = 10 TO 0 STEP -2: PRINT T;: NEXT T\n20 PRINT "LIFT OFF!"'))

qa("What does STEP do in a BASIC FOR loop?",
   "It sets the amount added to the loop variable each time round. It can be "
   "negative or a fraction. The loop ends when the variable goes past the limit.\n\n"
   + prog('10 FOR I = 0 TO 20 STEP 5: PRINT I;: NEXT I\n20 PRINT\n30 FOR X = 0 TO 1 STEP .25: PRINT X;: NEXT X\n40 PRINT'))

qa("What is the value of the loop variable after a FOR loop in BASIC?",
   "One step past the limit. After `FOR I = 1 TO 3` has finished, I is 4.\n\n"
   + prog('10 FOR I = 1 TO 3: NEXT I\n20 PRINT I'))

qa("How do I write nested loops in BASIC?",
   "Put one `FOR` loop inside another. The inner loop runs through all of its "
   "values for each value of the outer loop.\n\n"
   + prog('10 FOR R = 1 TO 4\n20   FOR C = 1 TO R\n30     PRINT "*";\n40   NEXT C\n50   PRINT\n60 NEXT R'))

qa("How do I add up the numbers from 1 to 100 in BASIC?",
   "Set a total to 0 and add to it inside a loop.\n\n"
   + prog('10 S = 0\n20 FOR I = 1 TO 100\n30   S = S + I\n40 NEXT I\n50 PRINT S'))

qa("How do I write a WHILE loop in BASIC?",
   "`WHILE condition` ... `WEND` repeats for as long as the condition is true. The "
   "test is made before each turn.\n\n"
   + prog('10 N = 27: S = 0\n20 WHILE N > 1\n30   IF N / 2 = INT(N / 2) THEN N = N / 2 ELSE N = 3 * N + 1\n'
          '40   S = S + 1\n50 WEND\n60 PRINT "REACHED 1 AFTER"; S; "STEPS"'))

qa("How do I write a loop with GOTO in BASIC?",
   "Jump back to an earlier line, with an `IF` to decide when to stop.\n\n"
   + prog('10 N = 1\n20 PRINT N;\n30 N = N * 2\n40 IF N <= 100 THEN 20\n50 PRINT'))

qa("How do I leave a FOR loop early in BASIC?",
   "Jump out of it with `GOTO` or `IF ... THEN line`. The loop variable keeps its "
   "value.\n\n"
   + prog('10 FOR N = 2 TO 100\n20   IF N * N > 500 THEN 40\n30 NEXT N\n40 PRINT "FIRST SQUARE ABOVE 500 IS"; N * N'))

# ---------------------------------------------------------------- subroutines

qa("What are GOSUB and RETURN in BASIC?",
   "`GOSUB 100` jumps to line 100 and remembers where it came from. `RETURN` jumps "
   "back to the statement after the `GOSUB`. That makes a subroutine: a piece of "
   "program that can be called from several places.\n\n"
   + prog('10 T$ = "CHAPTER ONE": GOSUB 100\n20 T$ = "THE END": GOSUB 100\n30 END\n'
          '100 PRINT T$\n110 FOR I = 1 TO LEN(T$): PRINT "-";: NEXT I\n120 PRINT\n130 RETURN'))

qa("How do I pass a value to a subroutine in BASIC?",
   "A subroutine has no parameters. All variables are shared, so set a variable "
   "before the `GOSUB` and let the subroutine leave its answer in another.\n\n"
   + prog('10 A = 12: B = 18: GOSUB 100\n20 PRINT "GCD IS"; G\n30 END\n'
          '100 REM GCD OF A AND B, RETURNED IN G\n110 X = A: Y = B\n120 IF Y = 0 THEN G = X: RETURN\n'
          '130 T = X - INT(X / Y) * Y: X = Y: Y = T\n140 GOTO 120'))

qa("What does RETURN without GOSUB mean in BASIC?",
   "The program reached a `RETURN` that no `GOSUB` had called. Usually the main "
   "program ran on into a subroutine because there was no `END` before it.\n\n"
   + prog('10 PRINT "MAIN PROGRAM"\n100 PRINT "SUBROUTINE"\n110 RETURN', fails=True)
   + "\n\nPut `END` on a line between the main program and the first subroutine.")

# ---------------------------------------------------------------- arrays, data

qa("How do arrays work in BASIC?",
   "`DIM A(5)` makes an array with elements `A(0)` to `A(5)`. The subscript can be "
   "any expression, so a `FOR` loop can visit every element.\n\n"
   + prog('10 DIM S(5)\n20 FOR I = 1 TO 5: S(I) = I * 10: NEXT I\n30 T = 0\n'
          '40 FOR I = 1 TO 5: T = T + S(I): NEXT I\n50 PRINT "TOTAL:"; T'))

qa("What happens if I use an array without DIM in BASIC?",
   "It gets subscripts 0 to 10. Using a larger subscript stops the program.\n\n"
   + prog('10 B(10) = 1\n20 PRINT B(10)\n30 B(11) = 1', fails=True))

qa("How do I make a two-dimensional array in BASIC?",
   "Give `DIM` two sizes, and use two subscripts.\n\n"
   + prog('10 DIM G(3, 3)\n20 FOR R = 1 TO 3: G(R, R) = 1: NEXT R\n30 FOR R = 1 TO 3\n'
          '40   FOR C = 1 TO 3: PRINT G(R, C);: NEXT C\n50   PRINT\n60 NEXT R'))

qa("How do READ and DATA work in BASIC?",
   "`DATA` lines hold values inside the program. Each `READ` takes the next values "
   "in order, wherever the `DATA` lines are. `RESTORE` goes back to the first "
   "item.\n\n"
   + prog('10 FOR I = 1 TO 3\n20   READ P$, D\n30   PRINT P$; TAB(10); D\n40 NEXT I\n'
          '50 DATA MERCURY, 58, VENUS, 108, EARTH, 150'))

qa("What does Out of DATA mean in BASIC?",
   "A `READ` was run when every `DATA` item had already been read.\n\n"
   + prog('10 FOR I = 1 TO 4\n20   READ X\n30   PRINT X;\n40 NEXT I\n50 DATA 10, 20, 30', fails=True)
   + "\n\nCount the items, put the number of items first in the data, or end the "
     "data with a marker value and stop reading when it turns up.")

qa("How do I find the largest number in a list in BASIC?",
   "Start with the first value and replace it whenever a larger one turns up.\n\n"
   + prog('10 READ N, M\n20 FOR I = 2 TO N\n30   READ X\n40   IF X > M THEN M = X\n50 NEXT I\n'
          '60 PRINT "LARGEST:"; M\n70 DATA 5, 12, 7, 31, 9, 24'))

# ---------------------------------------------------------------- strings

qa("What string functions does BASIC have?",
   "`LEN` gives the length. `LEFT$`, `RIGHT$` and `MID$` take pieces. `INSTR` "
   "searches. `ASC` and `CHR$` convert between a character and its code. `VAL` "
   "and `STR$` convert between a string and a number. `SPACE$` and `STRING$` make "
   "runs of characters.\n\n"
   + prog('10 A$ = "DARTMOUTH BASIC"\n20 PRINT LEN(A$)\n30 PRINT LEFT$(A$, 9)\n40 PRINT RIGHT$(A$, 5)\n'
          '50 PRINT MID$(A$, 6, 5)\n60 PRINT INSTR(A$, "BASIC")\n70 PRINT ASC("A"); CHR$(66)\n'
          '80 PRINT VAL("1964") + 1; STR$(64) + "!"'))

qa("How does MID$ work in BASIC?",
   "`MID$(A$, P, N)` gives N characters of A$ starting at position P, where the "
   "first character is position 1. Leave N out to get everything from position P "
   "to the end.\n\n" + prog('10 A$ = "PROGRAM"\n20 PRINT MID$(A$, 4, 2)\n30 PRINT MID$(A$, 4)'))

qa("How do I find one string inside another in BASIC?",
   "`INSTR(A$, B$)` gives the position where B$ first appears in A$, or 0 if it "
   "does not. `INSTR(P, A$, B$)` starts looking at position P.\n\n"
   + prog('10 A$ = "THE CAT SAT ON THE MAT"\n20 PRINT INSTR(A$, "CAT")\n30 PRINT INSTR(A$, "DOG")\n40 PRINT INSTR(2, A$, "THE")'))

qa("How do I reverse a string in BASIC?",
   "Build a new string from the characters taken last to first.\n\n"
   + prog('10 A$ = "DARTMOUTH"\n20 R$ = ""\n30 FOR I = LEN(A$) TO 1 STEP -1\n40   R$ = R$ + MID$(A$, I, 1)\n50 NEXT I\n60 PRINT R$'))

qa("How do I convert a string to upper case in BASIC?",
   "Small letters have codes 97 to 122, which is 32 more than the capitals. Go "
   "through the string and subtract 32 from the code of each small letter.\n\n"
   + prog('10 A$ = "Hello, World"\n20 U$ = ""\n30 FOR I = 1 TO LEN(A$)\n40   C = ASC(MID$(A$, I, 1))\n'
          '50   IF C >= 97 AND C <= 122 THEN C = C - 32\n60   U$ = U$ + CHR$(C)\n70 NEXT I\n80 PRINT U$'))

qa("How do I convert between numbers and strings in BASIC?",
   "`VAL` reads a number from the start of a string, giving 0 if there is none. "
   "`STR$` writes a number as a string, with a leading space where the sign would "
   "be. `MID$(STR$(N), 2)` drops that space.\n\n"
   + prog('10 PRINT VAL("42") + 1\n20 PRINT VAL("ABC")\n30 PRINT "[" + STR$(42) + "]"\n'
          '40 N = 3\n50 PRINT "YOU HAVE " + MID$(STR$(N), 2) + " LIVES LEFT."'))

qa("How do I split a sentence into words in BASIC?",
   "Use `INSTR` to find each space, starting each search just after the last "
   "space found.\n\n"
   + prog('10 A$ = "TAKE THE BRASS LAMP"\n20 P = 1\n30 S = INSTR(P, A$, " ")\n'
          '40 IF S = 0 THEN PRINT MID$(A$, P): END\n50 PRINT MID$(A$, P, S - P)\n60 P = S + 1\n70 GOTO 30'))

qa("How do I repeat a character in BASIC?",
   "`STRING$(N, A$)` gives the first character of A$ repeated N times, and "
   "`SPACE$(N)` gives N spaces.\n\n" + prog('10 PRINT STRING$(20, "-")\n20 PRINT "A"; SPACE$(18); "B"'))

# ---------------------------------------------------------------- functions

qa("What numeric functions does BASIC have?",
   "`ABS`, `INT`, `FIX`, `SGN`, `SQR`, `EXP`, `LOG`, `SIN`, `COS`, `TAN`, `ATN` and "
   "`RND`. `LOG` is the natural logarithm and the trigonometric functions work in "
   "radians.\n\n"
   + prog('10 PRINT INT(3.7); ABS(-5); SGN(-5); SQR(81)\n20 PRINT EXP(1); LOG(EXP(2))\n30 PRINT SIN(0); COS(0)'))

qa("How do I get the value of pi in BASIC?",
   "There is no constant for it. Four times the arctangent of 1 is pi.\n\n"
   + prog('10 P = 4 * ATN(1)\n20 PRINT P\n30 PRINT "SINE OF 30 DEGREES:"; SIN(30 * P / 180)'))

qa("How do I calculate a logarithm to base 10 in BASIC?",
   "`LOG` is the natural logarithm. Divide by `LOG(10)`.\n\n" + prog('10 PRINT LOG(1000) / LOG(10)'))

qa("How do I define my own function in BASIC?",
   "Use `DEF`. The name must begin with `FN`. The parameters belong to the "
   "function, so calling it does not change a variable of the same name "
   "elsewhere.\n\n"
   + prog('10 DEF FNR(X) = INT(X * 100 + .5) / 100\n20 DEF FNH(A, B) = SQR(A * A + B * B)\n'
          '30 PRINT FNR(3.14159); FNR(2 / 3)\n40 PRINT FNH(3, 4)'))

qa("How do I write a function of several lines in BASIC?",
   "In this BASIC, as in later Dartmouth BASICs, leave the `=` off the `DEF` line, "
   "write the body on the following lines, and finish with `FNEND`. Assign the "
   "result to the function's own name. The microcomputer BASICs of the 1970s had "
   "only the one-line form.\n\n"
   + prog('10 DEF FNM(A, B)\n20   FNM = A\n30   IF B > A THEN FNM = B\n40 FNEND\n50 PRINT FNM(3, 9); FNM(8, 2)'))

qa("Can a BASIC function call itself?",
   "A multi-line function defined with `DEF` ... `FNEND` can, and each call has "
   "its own parameters.\n\n"
   + prog('10 DEF FNG(A, B)\n20   IF B = 0 THEN FNG = A ELSE FNG = FNG(B, A - INT(A / B) * B)\n30 FNEND\n'
          '40 PRINT FNG(48, 18); FNG(1071, 462)'))

qa("What does Illegal function call mean in BASIC?",
   "A function was given a value it has no answer for, such as the square root of "
   "a negative number or the logarithm of zero.\n\n"
   + prog('10 PRINT SQR(16)\n20 PRINT SQR(-16)', fails=True))

# ---------------------------------------------------------------- PRINT USING

qa("What is PRINT USING in BASIC?",
   "`PRINT USING` prints values according to a picture. Each `#` is a digit "
   "position and `.` marks the decimal point. The number is rounded to fit and "
   "pushed to the right of its field, so columns line up.\n\n"
   + prog('10 PRINT USING "###.##"; 3.14159\n20 PRINT USING "###.##"; 42\n30 PRINT USING "###.##"; 123.456'))

qa("How do I print money with commas and a dollar sign in BASIC?",
   "In a `PRINT USING` picture, `$$` at the start puts a dollar sign just before "
   "the number, and a comma before the decimal point puts commas between the "
   "thousands.\n\n"
   + prog('10 FOR I = 1 TO 3\n20   READ A\n30   PRINT USING "$$###,###.##"; A\n40 NEXT I\n50 DATA 5, 1234.5, 250000'))

qa("How do I print a table with aligned columns in BASIC?",
   "Use `PRINT USING` with a string field, written as two backslashes with spaces "
   "between, and number fields.\n\n"
   + prog('10 FOR I = 1 TO 3\n20   READ N$, Q, P\n30   PRINT USING "\\        \\  ###  ####.##"; N$, Q, P\n40 NEXT I\n'
          '50 DATA WIDGET, 12, 4.5, SPROCKET, 150, 0.75, FLANGE, 3, 1299.5'))

qa("What does a percent sign in PRINT USING output mean in BASIC?",
   "The number was too big for its field. It is printed in full with a `%` in "
   "front, so the column is pushed out of line but no digits are lost.\n\n"
   + prog('10 PRINT USING "##.#"; 5\n20 PRINT USING "##.#"; 12345'))

# ---------------------------------------------------------------- random

qa("How do I generate a random number in BASIC?",
   "`RND(1)` gives a number that is at least 0 and less than 1. For a whole number "
   "from 1 to N, use `INT(RND(1) * N) + 1`. The output below was produced with a "
   "fixed seed; an ordinary run gives different numbers each time.\n\n"
   + prog('10 FOR I = 1 TO 8\n20   PRINT INT(RND(1) * 6) + 1;\n30 NEXT I\n40 PRINT'))

qa("How do I get a random number between two values in BASIC?",
   "For a whole number from A to B, use `INT(RND(1) * (B - A + 1)) + A`. The "
   "output below was produced with a fixed seed.\n\n"
   + prog('10 A = 10: B = 20\n20 FOR I = 1 TO 8: PRINT INT(RND(1) * (B - A + 1)) + A;: NEXT I\n30 PRINT'))

qa("How do I get the same random numbers every time in BASIC?",
   "Seed the generator. `RANDOMIZE 7` in the program does it, and so does running "
   "chuk-basic with `--seed 7`. The same seed gives the same numbers.\n\n"
   + prog('10 RANDOMIZE 7\n20 PRINT RND(1); RND(1)\n30 RANDOMIZE 7\n40 PRINT RND(1); RND(1)'))

qa("How do I shuffle a deck of cards in BASIC?",
   "Put the numbers in an array, then exchange each element with another chosen "
   "at random from those not yet fixed. The output below was produced with a fixed "
   "seed.\n\n"
   + prog('10 N = 10\n20 DIM D(N)\n30 FOR I = 1 TO N: D(I) = I: NEXT I\n40 FOR I = N TO 2 STEP -1\n'
          '50   J = INT(RND(1) * I) + 1\n60   T = D(I): D(I) = D(J): D(J) = T\n70 NEXT I\n'
          '80 FOR I = 1 TO N: PRINT D(I);: NEXT I\n90 PRINT'))

# ---------------------------------------------------------------- MAT

qa("What are the MAT statements in BASIC?",
   "They are Dartmouth BASIC's statements for working on whole arrays. `MAT READ` "
   "fills an array from `DATA`, `MAT PRINT` prints it, and `MAT C = A + B`, "
   "`A - B`, `A * B`, `(K) * A`, `TRN(A)` and `INV(A)` do matrix arithmetic. They "
   "use subscripts from 1. Most microcomputer BASICs left them out.\n\n"
   + prog('10 DIM A(2, 2), B(2, 2)\n20 MAT READ A, B\n30 DATA 1, 2, 3, 4\n40 DATA 10, 20, 30, 40\n'
          '50 MAT C = A + B\n60 MAT PRINT C;\n70 MAT C = A * B\n80 MAT PRINT C;'))

qa("How do I solve simultaneous equations in BASIC?",
   "With the `MAT` statements: put the coefficients in a matrix, invert it with "
   "`INV`, and multiply by the right-hand sides. This solves 2x + y = 5 and "
   "x + y = 3.\n\n"
   + prog('10 DIM A(2, 2), B(2)\n20 MAT READ A, B\n30 DATA 2, 1, 1, 1\n40 DATA 5, 3\n'
          '50 MAT I = INV(A)\n60 MAT X = I * B\n70 MAT PRINT X;')
   + "\n\nSo x is 2 and y is 1.")

qa("How do I make an identity matrix in BASIC?",
   "`MAT I = IDN(3, 3)` makes a 3 by 3 identity matrix. `ZER` and `CON` make "
   "matrices of zeros and of ones in the same way.\n\n" + prog('10 MAT I = IDN(3, 3)\n20 MAT PRINT I;'))

# ---------------------------------------------------------------- errors, old code

qa("What is the difference between a syntax error and a runtime error in BASIC?",
   "A syntax error is a line that is written wrongly. In chuk-basic the whole "
   "program is checked before it starts, so nothing runs. A runtime error is "
   "something impossible that happens while the program runs, such as dividing by "
   "zero. Output printed before the error still appears.\n\n"
   + prog('10 PRINT "ABOUT TO DIVIDE"\n20 X = 0\n30 PRINT 10 / X', fails=True)
   + "\n\nAnd a syntax error:\n\n" + prog('10 X = 1\n20 IF X PRINT 1', fails=True))

qa("Why does my BASIC variable have the value 0?",
   "A variable that has never been assigned is 0, so a misspelt name is a new "
   "variable and nothing reports it.\n\n" + prog('10 COUNT = 10\n20 PRINT "COUNT IS"; CUONT'))

qa("Why did my BASIC loop only run once when it calls a subroutine?",
   "All variables are shared. If the subroutine uses the same loop variable as "
   "the caller, it changes the caller's loop.\n\n"
   + prog('10 FOR I = 1 TO 3\n20   GOSUB 100\n30 NEXT I\n40 END\n'
          '100 FOR I = 1 TO 5: PRINT "*";: NEXT I\n110 PRINT\n120 RETURN')
   + "\n\nThe subroutine left I at 6, so the outer loop thought it had finished. Use "
     "a different variable in the subroutine.")

qa("Can BASIC programs be written without spaces?",
   "Many 1970s listings were, to save memory. chuk-basic reads a line the ordinary "
   "way first, and if that makes no sense reads it again finding keywords "
   "wherever they occur.\n\n" + prog('10 FORI=1TO3:PRINTI;:NEXTI\n20 PRINT\n30 A$="Y":IFA$="Y"THENPRINT"YES"'))

qa("What does PRINT \"X IS\" X mean in BASIC, with no semicolon?",
   "Items in a `PRINT` with nothing between them are printed as if a semicolon "
   "separated them. The first BASIC manual writes it that way.\n\n"
   + prog('10 X = 7\n20 PRINT "X IS" X "AND TWICE X IS" 2 * X'))

# ---------------------------------------------------------------- 1964, tools

qa("What was the first BASIC program in print?",
   "The preface of the first BASIC manual, Dartmouth College, May 1964, shows this "
   "program, and page 4 gives it line numbers.\n\n"
   + prog((SAMPLES / "dartmouth" / "preface.bas").read_text()))

qa("Why does SQR of a negative number work in 1964 Dartmouth BASIC?",
   "The 1964 manual says of `SQR` that \"the square-root is always taken of the "
   "absolute value\". Later BASICs treat the square root of a negative number as "
   "an error. chuk-basic follows the later rule by default:\n\n"
   + prog('10 PRINT SQR(-16)', fails=True)
   + "\n\nRun with `--dialect dartmouth`, the same program follows the manual:\n\n"
   + f"```\n{output('10 PRINT SQR(-16)', info='dialect=dartmouth')}\n```")

qa("What does --dialect dartmouth change in chuk-basic?",
   "Two things, both stated in the 1964 manual: `SQR` takes the absolute value of "
   "its argument, and a power takes the absolute value of the number being "
   "raised. Run with `--dialect dartmouth`, this program\n\n"
   "```basic\n10 PRINT SQR(-16)\n20 PRINT (-2) ^ 3\n```\n\nprints:\n\n"
   + f"```\n{output('10 PRINT SQR(-16)' + chr(10) + '20 PRINT (-2) ^ 3', info='dialect=dartmouth')}\n```"
   + "\n\nWith the default dialect line 10 is an error and line 20 gives -8.")

qa("How do I run a BASIC program with chuk-basic?",
   "Give the interpreter the name of the file: `chuk-basic program.bas`. With no "
   "file name it starts an interactive prompt where you type numbered lines and "
   "then `RUN`. Useful options are `--seed N` for repeatable random numbers, "
   "`--echo` to copy typed answers into the output, and `--max-steps N` to stop a "
   "program that loops for ever. For example, this program in a file\n\n"
   + prog('10 PRINT "HELLO, WORLD!"\n20 PRINT "THIS IS CHUK-BASIC."\n30 END'))

qa("How do I compile a BASIC program to WebAssembly?",
   "`chuk-basic --compile program.bas` writes `program.wasm`, a self-contained "
   "WASI module that runs on a runtime such as wasmtime: `wasmtime program.wasm`. "
   "The module takes `--seed`, `--echo`, `--max-steps` and `--dialect` after its "
   "name. A compiled program prints exactly what the interpreter prints. For "
   "example, compiled or interpreted, this program\n\n"
   + prog('10 FOR I = 1 TO 5\n20   PRINT I; "CUBED IS"; I ^ 3\n30 NEXT I'))

# ---------------------------------------------------------------- whole programs

TASKS = {
    "classic/fibonacci": "Write a BASIC program that prints Fibonacci numbers.",
    "classic/factorial": "Write a BASIC program that prints factorials.",
    "classic/primes": "Write a BASIC program that finds prime numbers.",
    "classic/gcd": "Write a BASIC program that finds a greatest common divisor.",
    "classic/collatz": "Write a BASIC program that follows the Collatz sequence.",
    "classic/bubblesort": "Write a bubble sort in BASIC.",
    "classic/binary": "Write a BASIC program that converts numbers to binary.",
    "classic/roman": "Write a BASIC program that converts numbers to Roman numerals.",
    "classic/temperature": "Write a BASIC program that prints a temperature conversion table.",
    "classic/timestable": "Write a BASIC program that prints a multiplication table.",
    "classic/hanoi": "Write the Towers of Hanoi in BASIC.",
    "classic/weekday": "Write a BASIC program that works out the day of the week for a date.",
    "classic/bottles": "Write a BASIC program that prints the bottles of beer song.",
    "classic/strings": "Show me the BASIC string functions in one program.",
    "classic/matrix": "Write a BASIC program that works with matrices.",
    "classic/pi": "Write a BASIC program that calculates pi.",
    "classic/average": "Write a BASIC program that averages numbers typed at the keyboard.",
    "classic/guess": "Write a number guessing game in BASIC.",
    "tutorial/13_game": "Write a higher-or-lower card game in BASIC.",
}
LIMIT = 60  # lines of output shown before an answer is left out as too long
for name, question in TASKS.items():
    base = SAMPLES / name
    src = base.with_suffix(".bas").read_text()
    stdin = base.with_suffix(".in").read_text() if base.with_suffix(".in").exists() else ""
    if output(src.strip("\n"), stdin).count("\n") > LIMIT:
        print(f"left out {name}: more than {LIMIT} lines of output")
        continue
    qa(question, prog(src, stdin))

out = HERE / "dataset" / "basic.jsonl"
out.parent.mkdir(exist_ok=True)
with out.open("w") as f:
    for row in rows:
        f.write(json.dumps(row, ensure_ascii=False) + "\n")
print(f"wrote {len(rows)} examples to {out}")
