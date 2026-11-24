# Dartmouth BASIC manual, 1 October 1964

The programs printed in Dartmouth's second BASIC manual, *BASIC* (Dartmouth
College Computation Center, 1 October 1964, 58 numbered pages), typed in from
a scan of the manual. Pages are cited by the manual's own page numbers (the
scan's PDF page is that number plus six).

Each file is typed as the page prints it: line numbers, spacing between
tokens and the manual's own slips are kept. Like the May 1964 samples in
`samples/dartmouth`, the files carry no `REM` line giving their source; the
table below gives it. Where a character had to be read rather than copied, it
is listed under [Readings](#readings).

The manual prints its programs in two ways: typeset on a typewriter in the
text (pp. 3, 12-13, 22, 42, 46-47) and as teletype sessions (pp. 19, 23-25,
32-33, 35, 37, 40, 44), where the results are printed below the program. Only
the teletype sessions have printed results.

## Files

| File | Page | What it is |
| --- | --- | --- |
| `linear.bas` | 3 (listing), 19 (run) | Two linear equations, several right-hand sides |
| `sqrt-table.bas` | 12 | Table of square roots with `LET` and `IF` |
| `sqrt-table-for.bas` | 13 | The same with `FOR` and `NEXT` |
| `sqrt-table-even.bas` | 13 | The same for the even numbers, `STEP 2` |
| `maxsin-draft.bas` | 22 | The maximum of the sine curve, "as first written down on paper" |
| `maxsin-run1.bas` ... `maxsin-run5.bas` | 23-24 | The program as it stood at each `RUN` of the debugging session (see below) |
| `maxsin.bas` | 25 | The corrected program, as the final `LIST` prints it |
| `printer.bas` | 32 | A label followed by a number without a comma |
| `printer2.bas` | 32 | `PRINT I,` - five zones to a line |
| `powers.bas` | 33 | Powers `I↑N` for N from 1 to 7 |
| `cubes.bas` | 35 | Cubes 1 to 100, packed with `;` |
| `sales.bas` | 37 | Lists and tables: sales for five salesmen |
| `random.bas` | 40 | Twenty random digits (run twice on the page) |
| `random115.bas` | 40 | The same after discarding 115 random numbers |
| `sine-def.bas` | 42 | Table of sines in degrees, with `DEF` |
| `sine.bas` | 42 | The same without `DEF` |
| `gcd3no.bas` | 44 | Greatest common divisor of three numbers, with `GOSUB` |
| `roundoff.bas` | 46 | Summing multiples of .1: the round-off example |
| `roundoff-integers.bas` | 47 | The same, counting with integers |
| `roundoff-for.bas` | 47 | The same with `FOR` |

`linear.bas` is typed from p. 3. The teletype listing on p. 19 has the same
characters with different spacing.

The debugging session of pp. 23-25 types a program, runs it, corrects lines
and runs it again, six times. Pages 23-24 never list the program between runs,
so `maxsin-run1.bas` to `maxsin-run5.bas` are put together from the lines typed
on those pages, in the order typed: a retyped line replaces the earlier line
with its number, a line number alone deletes the line, and each backwards
arrow (`←`) deletes the character before it, as p. 16 describes. So
`STR←EP` is typed `STEP`, `SINE←(X)` is `SIN(X)`, `LOO←←←100` is `100`, and
`"SINE2←"` is `"SINE"`. The first line 20, `LWR XO = 0`, is replaced by the
retyped `20 LET X0 = 0` before the first `RUN`. Putting them together this way
is ours; every line in them is printed on the page. The sixth state is the
listing on p. 25, `maxsin.bas`, and is checked against it: the edits do
produce that listing.

Not transcribed, because they are neither whole programs nor followed by
printed results: single statements given as examples (pp. 5, 12, 13, 27-30,
41-42, 56-57), the `REM` lines of p. 45 and the `GOTO 999`/`STOP` fragment
on the same page, the `FNX` fragment of p. 41 (it has no `PRINT`; the text
says `S1` "would end up having the value 50"), and the `GOSUB` skeleton of
pp. 42-43. The `MAT` statements of pp. 51-52 and the card decks of Card
BASIC (pp. 50-52) are described but no program using them is printed, so
nothing here needs `MAT` or card input.

## Results

Run as `chuk-basic --dialect dartmouth FILE` (with `--seed 1` for the random
programs and `--max-steps` for the loops the session stops by hand), and,
for comparison, with the default dialect. "Agree" means the same figures and
labels. The dialect's printing follows this manual's rules (pp. 31 and 34;
see the main README), so the columns are compared too; the `USER NO.` and
`TIME:` lines are not reproduced.

| Program | Page | The page prints | `--dialect dartmouth` prints | Agree? |
| --- | --- | --- | --- | --- |
| `linear.bas` | 19 | `4  -5.5` / `.666667  .166667` / `-3.66667  3.83333` | the same | Yes |
| `sqrt-table.bas` | 12 | (no results) | 101 lines, `1  1` to `101  10.0499` | - |
| `sqrt-table-for.bas` | 13 | (no results) | `1  1` to `100  10` | - |
| `sqrt-table-even.bas` | 13 | (no results) | `2  1.41421` to `100  10` | - |
| `maxsin-draft.bas` | 22 | (no results) | nothing | - |
| `maxsin-run1.bas` | 23 | `ILLEGAL FØRMULA IN 70` / `NØT MATCHED WITH FØR IN 80` / `FØR WITHØUT NEXT` | nothing; exits normally | No (disagreement 4) |
| `maxsin-run2.bas` | 23 | nothing; the user types `STØP` | `.1 .1 .1` ... `1.6 1.6 .1`, then loops until stopped | No (disagreement 5) |
| `maxsin-run3.bas` | 24 | `0 0 .1` / `.1 .1 .1` / `.2 .2 .1` / `.3 .3 .1` / `.4`, then `STØP.` | `0  0  .1` / `.1  .1  .1` / ... until stopped | Yes |
| `maxsin-run4.bas` | 24 | `1.6  .999574  .1` twice, then `1.6  .99`, then `STØP.` | `1.6  .999574  .1` until stopped | Yes |
| `maxsin-run5.bas` | 24 | `INCØRRECT FØRMAT IN 5` | runs | No (disagreement 4) |
| `maxsin.bas` | 25 | `X VALUE  SINE  RESØLUTIØN` / `1.6  .999574  .1` / `1.57  1.  .01` / `1.571  1.  .001` | the same, `1.` included | Yes |
| `printer.bas` | 32 | `FIRST NØ. =  2.3   SECØND NØ. =-3.17` | the same, the second label in column 31 | Yes, columns too |
| `printer2.bas` | 32 | `1 2 3 4 5` / `6 7 8 9 10` / `11 12` | the same, five zones to a line | Yes, columns too |
| `powers.bas` | 33 | `N =    1`; `1  4.`; `1  8.  27.`; `1  16.  81.  256`; ... `46656.` and `279936.  823543.` on a second line | `N =    1`; `1  4`; `1  8  27`; `1  16  81  256`; ... the same line breaks | Figures and columns yes; the page's points after inexact powers, no (disagreement 1) |
| `cubes.bas` | 35 | 13 lines: 11 on the first, then 8 to a line, `1000000` alone | the same 13 lines | Yes, line breaks too |
| `sales.bas` | 37 | `TØTAL SALES FØR SALESMAN    1    $    180.5` ... | the same, the `$` in column 46 | Yes, columns too |
| `random.bas` | 40 | `9 0 4 1 6 4 9 5 8 3 8` / `6 6 9 3 4 0 7 7 7`, the same in both runs | other digits, eleven to a line | No (disagreement 8) |
| `random115.bas` | 40 | `7 9 9 0 9 4 5 9 9 7 5` / `2 2 4 2 0 0 0 3 3` | other digits | No (as above) |
| `sine-def.bas` | 42 | (no results) | `Syntax error in line 20: Unexpected number 14159265` | - (disagreement 6) |
| `sine.bas` | 42 | (no results) | the same error | - |
| `gcd3no.bas` | 44 | `A  B  C  GCD` / `60  90  120  30` / `38456  64872  98765  1` / `32  384  72  8.` | the same, `8` without the point | Yes (see the reading of `8.`) |
| `roundoff.bas` | 46 | (in the text) "the program will produce 23. 1 as the answer"; "The correct answer is 21" | `Syntax error in line 40: Unexpected character: .` | No (disagreements 6, 7) |
| `roundoff-integers.bas` | 47 | (implied) 21 | `21.` | Figures yes |
| `roundoff-for.bas` | 47 | (implied) 21 | `21.` | Figures yes |

`21.` has a point because a sum of tenths in binary is not exactly 21: the
point marks an inexact figure, as rule 4 implies.

The default dialect prints the same programs with up to nine significant
digits (`.666666667`, `.999573603`), in 14-column zones, and stops
with `Out of DATA` where the 1964 system ends quietly (`linear`, `maxsin`,
`gcd3no`). It prints `0` for every digit of `random.bas` and `random115.bas`,
because there `RND(X)` with `X` zero repeats the last number; with
`--dialect dartmouth` the argument is ignored, as p. 39 says it is.

With the spaces inside numbers closed up (`3.14159265`, `.1`; files not kept
here), `sine-def.bas` and `sine.bas` print 91 lines, `0. 0.` to `90. 1.`, and
`roundoff.bas` prints `21.`, not the manual's 23.1.

### Layout

Measured on the page, and now followed by the dialect:

- Zones are 15 columns: labels start at columns 1, 16, 31 and 46 (pp. 25,
  37, 44), as pp. 28 and 31 say, and a comma in the fifth zone starts a new
  line (pp. 32, 33).
- A number that follows a label with no comma (`"FIRST NØ. ="A`,
  `"N = "N`, `"... SALESMAN "J`) starts at the next multiple of three
  columns, its sign position included: `FIRST NØ. =  2.3`, `N =    1`.
- A number takes up the rest of its zone, so the next comma moves on a whole
  zone: after `SALESMAN` and `1` the `$` is in column 46, and after
  `FIRST NØ. =  2.3` the second label is in column 31. p. 32 says why: "the
  label and the number (with its unprinted non-significant zeros) occupies
  more than one zone". The same rule puts the May manual's `-1.` and
  `IMAGINARY PART` under the third coefficient (May p. 22).
- With `;`, the cubes of p. 35 are eleven to the first line (six columns
  apart) and eight to every later line (nine columns apart), and the random
  digits of p. 40 eleven to a line.

## Disagreements and their causes

1. **The point after an inexact power** (`powers`). *Interpreter
   (arithmetic).* The dialect now follows rule 4 (p. 34): an exact integer
   has no point, and a figure that only rounds to a whole number has one
   (`1.` for the sine at 1.57, p. 25). On p. 33 the 1964 `↑` gives `4.`,
   `8.`, `27.` but `256`, `1024`, `4096`: its power routine was exact for
   some arguments and not others. Here `↑` is exact for all of them, so
   every power prints without a point. Matching the page would need the
   1964 power routine, which is not documented.
2. **Line width, zones and packing.** Fixed: see Layout above. (The
   October rules were adopted in the dialect after this comparison was
   first run, when it showed a point after every whole number, 14-column
   zones and no line width.)
3. **Packing.** Fixed, with the field width an interpretation: see the main
   README.
4. **Errors of form not found** (`maxsin-run1`, `maxsin-run5`, and the draft).
   *Interpreter.* In 1964 `XØ` (the letter O) is not a variable, so line 70
   is an `ILLEGAL FØRMULA`; `NEXT X0` with `FOR X` is `NØT MATCHED WITH FØR`;
   and an unbalanced quote is `INCØRRECT FØRMAT`. The interpreter accepts
   `XO` and `RESOLUTION` as variables (`--check` warns about both, as
   warnings), does not match `NEXT` to `FOR` before running, and runs the
   program. `maxsin-run1` then prints nothing, because line 40 jumps to the
   `DATA` line at once and falls through to `END`.
5. **Variables not set** (`maxsin-run2`). *Interpreter, or undefined in 1964.*
   The session's comment 6 (p. 26) explains the page's silence: "This could
   happen only if M were so large that the IF- THEN statement was always
   satisfied. We then observe that we forgot to initialize M". So in October
   1964 a variable never assigned was not zero. The interpreter starts it at
   zero, and the program prints. `roundoff-for.bas` (p. 47) uses `S` without
   setting it (its line 10 sets `X`), and the text expects it to work; the
   interpreter's zero makes it print 21. The manual is inconsistent here, or
   p. 47 is a slip for `LET S = 0`.
6. **Spaces inside numbers** (`sine-def`, `sine`, `roundoff`). *Page or
   interpreter: undecided.* The typewriter that set these pages leaves a space
   after every decimal point, in the text too ("0. 1", "23. 1"), so
   `3. 14159265` and `X + . 1` may be typography rather than what was typed
   on the teletype. They are typed here as printed. The manual says (p. 4)
   "spaces have no significance in BASIC (except in messages to be printed
   out ...)", and under that rule they are legal; the interpreter refuses
   them in a `LET` but accepts them in `DATA` (`DATA . 1, . 01` reads .1 and
   .01).
7. **Round-off** (`roundoff`). *Interpreter (arithmetic).* With the spaces
   closed up the program prints 21; the manual says it "will produce 23.1"
   because ".1 in the computer is a number very slightly less than .1"
   (p. 46). In the interpreter's double precision .1 is slightly more than
   .1, the loop stops one pass earlier, and the sum is the correct 21. This
   is the 30-bit arithmetic the main README says is not reproduced.
8. **Random numbers** (`random`, `random115`). *Interpreter (undocumented).*
   The digits cannot agree: the 1964 generator is not described. Beyond the
   digits, the page shows the same sequence on every `RUN` ("the next time
   the program is run, the same sequence is obtained", p. 39); the
   interpreter gives a new sequence on each run unless `--seed` is given.
9. **`maxsin-run2` to `run4` never end.** Not a disagreement: they loop on the
   page too, until the user types `STØP`. They are run here with
   `--max-steps`, which ends them with `Step limit exceeded`.

One more test, of a statement rather than a program: p. 5 says line 15 of
the linear equations "could have been typed as `15 LETD=A1*A4-A2*A3`, a fully
equivalent though less readable form". In the interpreter it is not
equivalent: `LETD` is read as a variable (an implied `LET`), `D` stays zero,
and the program prints `NO UNIQUE SOLUTION`. *Interpreter.* (`FORI=1TO3` and
`PRINTI` are read correctly.)

The manual's own slip, typed as printed: `sqrt-table.bas` (p. 12) is
introduced as a table of "the first hundred whole numbers", but
`IF X <=100 THEN 20` after the `PRINT` prints 101 lines.

## Readings

| Where | What | Reading |
| --- | --- | --- |
| Throughout | The letter O is printed with a slash, `Ø`, to tell it from zero, as p. 4 explains ("the letter "Oh" is distinguished from the numeral "Zero" by having a diagonal slash"). | Typed as `O`. In the teletype listings zero is an unslashed `0`, so `X0` (zero) and `XØ` (letter) are told apart by the slash: p. 23's line 70 `PRINT XØ, X, D` is typed `PRINT XO, X, D`, and it is the error the session corrects. |
| Throughout | The teletype prints the digit one like a small `l`. | Typed `1` (`A1`, `B1`, `X1`). |
| p. 22, line 60 | `SIN(XO)`: the character after `X` is round, wider than the zeros of `X0` in lines 20, 50, 70 and 80, and has no slash. | Typed as the letter `O`, `SIN(XO)`, as its shape shows; zero would be the variable the program means. The draft prints nothing, so nothing depends on it. |
| pp. 22, 42, 46 | Spaces after the decimal point: `. 1`, `. 01`, `. 001`, `3. 14159265`, `X + . 1`. | Typed as printed; see disagreement 6. |
| p. 23 | `STR←EP`, `SINE←(X)`, `LOO←←←100` and `LWR XO = 0`; p. 24 `"SINE2←"`. | The backwards arrow erases the character before it (p. 16), so they are typed `STEP`, `SIN(X)`, `100` and `"SINE"`; the `LWR` line is replaced by the retyped line 20. |
| p. 24, `run5` line 5 | `RESØLUTIØN"` has no opening quote. | Typed as printed: it is the error the session corrects (comment 10). |
| p. 32 | `10 RA DELETED` before `10 READ A, B`. | A line abandoned with ALT MODE, which the computer answers `DELETED` (p. 16). Not typed. |
| p. 33, line 10 | `FØR N` is faint. | Read `FOR N = 1 TO 7`, from what is visible and from `NEXT N` on line 70. |
| p. 33, results | `256` with a damaged middle digit (the OCR read `236`). | `256`, which is 4↑4. |
| p. 37, results | A speck before `166.55`: `$ .166.55`. | `166.55` (1.25×29 + 4.30×21 + 2.50×16). |
| p. 40, first run | The eighth digit of the second line shows only its top stroke. | `7`: the second run, said to repeat the first, prints `7` there. |
| p. 44, results | `8.` for the last GCD, a dot close to the baseline after the `8`. | Recorded as `8.`; most likely a speck. The other GCDs print `30` and `1` without a point, and the GCD is computed exactly from whole numbers, so rule 4 (p. 34) would print `8`. |
| p. 45 (not transcribed) | `A4*X3 = B2` in the `REM` lines, for `A4*X2`. | A slip in the remark. |
