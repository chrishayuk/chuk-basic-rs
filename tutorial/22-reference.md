# Reference

## Programs

A program is a list of lines. Each line has a number and one or more
statements separated by colons. Lines run in number order. Keywords and
names may be in capitals or small letters.

```basic
10 REM A LINE WITH ONE STATEMENT
20 A = 1: B = 2: PRINT A + B
```

```output
 3 
```

## Statements

| Statement | Example | Chapter |
| --- | --- | --- |
| `LET` (the word is optional) | `LET X = 5` &nbsp; `N$ = "ADA"` &nbsp; `A(I, J) = 0` | 3 |
| `PRINT`, or `?` | `PRINT "TOTAL:"; T, TAB(30); N$` | 2 |
| `PRINT USING` | `PRINT USING "###.##"; X` | 12 |
| `INPUT` | `INPUT "YOUR NAME"; N$` &nbsp; `INPUT A, B` | 4 |
| `LINE INPUT` | `LINE INPUT "ADDRESS: "; A$` | 4 |
| `IF` ... `THEN` ... `ELSE` | `IF X > 0 THEN PRINT "POS" ELSE PRINT "NOT"` &nbsp; `IF X = 0 THEN 200` | 5 |
| `GOTO`, or `GO TO` | `GOTO 100` | 5 |
| `ON` ... `GOTO` | `ON N GOTO 100, 200, 300` | 5 |
| `FOR` ... `TO` ... `STEP`, `NEXT` | `FOR I = 10 TO 1 STEP -1` ... `NEXT I` | 6 |
| `WHILE` ... `WEND` | `WHILE N > 1` ... `WEND` | 6 |
| `GOSUB`, or `GO SUB`; `RETURN` | `GOSUB 500` ... `RETURN` | 7 |
| `ON` ... `GOSUB` | `ON N GOSUB 100, 200, 300` | 7 |
| `DIM` | `DIM A(20), B$(5), G(8, 8)` | 8 |
| `READ`, `DATA`, `RESTORE` | `READ N, N$` &nbsp; `DATA 3, "A, B", C` &nbsp; `RESTORE 500` | 9 |
| `DEF` | `DEF FNA(X, Y) = X * Y` | 11 |
| `DEF` ... `FNEND` | a function of several lines | 11 |
| `RANDOMIZE` | `RANDOMIZE` &nbsp; `RANDOMIZE 42` | 13 |
| `MAT` | `MAT READ A` &nbsp; `MAT C = A * B` &nbsp; `MAT PRINT C;` | 14 |
| `REM`, or `'` | `REM A REMARK` | 7 |
| `END`, `STOP` | stop the program | 7 |

## Values and names

- Numbers are 64-bit floating point. They print with up to nine significant
  digits, a leading space or minus sign, and a trailing space.
- Number constants: `42`, `-7`, `3.5`, `.5`, `1E6`, `2.5E-3`.
- Strings are written in double quotes and may be any length.
- A name starts with a letter and may contain letters and digits. A name
  ending in `$` holds a string.
- A variable and an array may have the same name. They are unrelated.
- A variable never assigned is 0 or the empty string.

```basic
10 PRINT 1E6; 2.5E-3; .5; 1 / 3; 1E20
20 PRINT "["; U$; "]"; U
```

```output
 1000000  .0025  .5  .333333333  1E+20 
[] 0 
```

## Operators

Highest first. Operators on the same row are done from left to right.

| Operators | |
| --- | --- |
| `^` | power; `↑` is accepted as well |
| `-` | unary minus |
| `*` `/` | multiply, divide |
| `+` `-` | add, subtract; `+` also joins strings |
| `=` `<>` `<` `<=` `>` `>=` | comparisons, giving -1 for true and 0 for false |
| `NOT` | |
| `AND` | |
| `OR` | |

`AND`, `OR` and `NOT` work bit by bit on whole numbers, which is why true is
-1: every bit set.

```basic
10 PRINT 2 + 3 * 4 ^ 2; -2 ^ 2; 2 ^ 3 ^ 2
20 PRINT 5 > 3; 5 < 3; NOT 0; 6 AND 3; 6 OR 3
30 PRINT 1 < 2 AND 2 < 3 OR 1 > 2
```

```output
 50 -4  64 
-1  0 -1  2  7 
-1 
```

## Numeric functions

| Function | Gives |
| --- | --- |
| `ABS(X)` | absolute value |
| `ATN(X)` | arctangent, in radians |
| `COS(X)` | cosine of X radians |
| `EXP(X)` | e to the power X |
| `FIX(X)` | X with the fraction removed |
| `INT(X)` | largest whole number not above X |
| `LOG(X)` | natural logarithm; X must be positive |
| `RND(X)` | X > 0: next random number, 0 ≤ r < 1. X = 0: the last one again. X < 0: reseed |
| `SGN(X)` | -1, 0 or 1 |
| `SIN(X)` | sine of X radians |
| `SQR(X)` | square root; X must not be negative |
| `TAN(X)` | tangent of X radians |

## String functions

| Function | Gives |
| --- | --- |
| `ASC(A$)` | code of the first character |
| `CHR$(N)` | the character with code N |
| `INSTR(A$, B$)` | position of B$ in A$, or 0 |
| `INSTR(P, A$, B$)` | the same, searching from position P |
| `LEFT$(A$, N)` | first N characters |
| `LEN(A$)` | number of characters |
| `MID$(A$, P)` | from position P to the end |
| `MID$(A$, P, N)` | N characters from position P |
| `RIGHT$(A$, N)` | last N characters |
| `SPACE$(N)` | N spaces |
| `STR$(X)` | X as `PRINT` would write it, without the trailing space |
| `STRING$(N, A$)` | the first character of A$, N times |
| `VAL(A$)` | the number at the start of A$, or 0 |

In `PRINT` only: `TAB(N)` moves to column N, counting from 0, and `SPC(N)`
prints N spaces.

## PRINT

| | |
| --- | --- |
| `;` between items | no space between them |
| `,` between items | move to the next zone; zones are 14 columns wide |
| `;` or `,` at the end | do not start a new line |
| nothing between items | the same as `;` |

## PRINT USING

| In the picture | Prints |
| --- | --- |
| `#` | a digit position |
| `.` | the decimal point |
| `,` before the point | commas between thousands |
| `+` at the start or end | the sign, always |
| `-` at the end | a trailing minus |
| `**` | leading asterisks |
| `$$` | a dollar sign before the number |
| `**$` | both |
| `^^^^` | exponent form |
| `!` | first character of a string |
| `\  \` | a string field as wide as the backslashes and what is between them |
| `&` | the whole string |
| `_x` | the character x itself |

A number too big for its field is printed in full after a `%`.

## MAT

Subscripts start at 1. A one-dimensional array is a column.

| | |
| --- | --- |
| `MAT READ A, B(2, 3)` | fill from `DATA`, row by row |
| `MAT INPUT A` | one line of numbers for each row |
| `MAT PRINT A` &nbsp; `MAT PRINT A;` | print in zones, or packed |
| `MAT C = A` | copy |
| `MAT C = A + B` &nbsp; `A - B` | add, subtract |
| `MAT C = A * B` | matrix product |
| `MAT C = (K) * A` | multiply by a number |
| `MAT C = ZER` &nbsp; `CON` &nbsp; `IDN` | zeros, ones, identity; dimensions may follow in brackets |
| `MAT C = TRN(A)` | transpose |
| `MAT C = INV(A)` | inverse |

## Details that differ between BASICs

Where the old dialects disagree, this one does the following.

- **`FOR`**: a loop whose range is empty is not run at all.
- **`IF`**: when the condition is false, the rest of the line is skipped, up
  to an `ELSE` if there is one.
- **Arrays** start at subscript 0. Without a `DIM` they go up to 10.
  Subscripts are rounded to the nearest whole number.
- **`TAB`** counts columns from 0 and does nothing if the line is already
  past that column.
- **Output** is not wrapped at any width.
- **Division by zero** stops the program.
- **`INPUT`** asks again with `?REDO FROM START` after a bad number, asks for
  more with `??`, and discards spare values with `?EXTRA IGNORED`.
- **`STOP`** ends the program without a message.
- **`DEF`** may come after the function's first use.
- **`INPUT`** into a single string variable takes the whole line, commas
  included.
- **Lines without numbers** are allowed, and stay where they are written.

## Error messages

Syntax errors are found before the program starts, and nothing is run.
Runtime errors stop the program where it is. Both name the line.

| Message | Usual cause |
| --- | --- |
| `Division by zero` | a divisor that turned out to be 0 |
| `Subscript out of range` | an array index below 0 or above the `DIM` |
| `Type mismatch` | text where a number was needed, or the reverse |
| `Undefined line 500` | a `GOTO` or `GOSUB` to a line that is not there |
| `NEXT without FOR` | a `NEXT` reached by jumping into a loop |
| `RETURN without GOSUB` | the program ran on into a subroutine |
| `Out of DATA` | more `READ`s than `DATA` items |
| `Out of input` | `INPUT` was reading from a file that ran out |
| `Illegal function call: SQR` | `SQR` of a negative number, `LOG` of zero, and the like |
| `Undefined function FNA` | an `FN` name with no `DEF` |
| `Bad number in DATA` | `READ` into a number variable found text |
| `Matrix dimensions do not match` | `MAT` arithmetic on arrays of the wrong shape |
| `Matrix is singular` | `MAT ... INV` of a matrix that has no inverse |

Chapter 16 shows each of them happening.

## The command line

```text
chuk-basic [OPTIONS] [FILE.bas]
```

With no file, the interactive prompt starts.

| Option | Effect |
| --- | --- |
| `--seed N` | seed the random numbers, for repeatable runs |
| `--echo` | copy answers to `INPUT` into the output |
| `--max-steps N` | stop with an error after N operations |
| `--dialect NAME` | `microsoft` (the default) or `dartmouth` |
| `--compile` | write a WebAssembly module in place of running |
| `-o FILE` | where `--compile` writes it |
| `--tokens` | print the tokens in place of running |
| `--ast` | print the syntax tree as JSON in place of running |
| `-h`, `--help` | a summary of these |
| `-V`, `--version` | the version |

## The interactive prompt

| | |
| --- | --- |
| `10 PRINT "HI"` | store line 10 |
| `10` | delete line 10 |
| `LIST`, `LIST 10-50` | show the program, or part of it |
| `RUN` | run it |
| `NEW` | forget it |
| `LOAD "file"`, `SAVE "file"` | read or write a program |
| `HELP` | a summary |
| `BYE` | leave |
| anything else | run at once |

## The Dartmouth dialect

`--dialect dartmouth` follows the first manual of May 1964 in two respects:
`SQR(X)` is the square root of the absolute value of X, and `A ^ B` is the
absolute value of A to the power B. Chapter 17 has the details.

## Not supported

- Statements from later, machine-specific BASICs: files, graphics, sound,
  `PEEK` and `POKE`, `ON ERROR`, `DEFINT` and the like.
- `MAT` extras from later Dartmouth editions: `DET`, and string matrices.
- At the interactive prompt, variables do not outlive a `RUN`.
