# Scans of the original source

Page images from the **first BASIC instruction manual** - Dartmouth College
Computation Center, May 1964, "First Draft" - taken from the commemorative
reprint that Dartmouth hosts at
<https://www.dartmouth.edu/basicfifty/basicmanual_1964.pdf>, and reduced in
size. They are here so that the interpreter can be checked against what was
actually printed.

| File in `dartmouth-1964/` | What it shows |
| --- | --- |
| `cover.jpg`, `title-page.jpg` | The reprint's cover, and the 1964 title page |
| `p01-preface.jpg` | The four requirements, and the preface's example program |
| `p04-instruction-format.jpg` | Line numbers; "There are only 13 other operations" |
| `p05-formulas.jpg` | Numbers, variables, the arrow for powers, the ten functions |
| `p21-examples.jpg`, `p22-results.jpg` | Example programs 1 and 2, with their printed results |
| `p23-example-3.jpg` | Example program 3, with two results |
| `detail-*.jpg` | Full-resolution crops of the passages discussed below |

Where to look:

- The preface's program, `LET X = (7+8)/3`, `PRINT X`, `END`: p. 1. It is
  `samples/dartmouth/preface.bas`, with the line numbers that p. 4 gives it.
- The three example programs: pp. 21-23. They are typed in as
  `samples/dartmouth/example1.bas`, `example2.bas` and `example3.bas`
  (`example3b.bas` is the variant the manual describes), and the tests
  compare what they print with the results on the page.
- `SQR` "always taken of the absolute value": p. 5, and
  `detail-sqr-absolute-value.jpg`. Example 2 relies on it at line 40
  (`detail-program-2-line-40.jpg`); run it with `--dialect dartmouth`.

Two characters in the samples are our readings of damaged print, and
nothing else in them is changed:

- Line 50 of example 2 is smudged. It is typed as `IF D = 0`. Of the six
  relations the manual allows (p. 12), only `=` gives the results printed
  on p. 22, but the scan itself does not show which was printed.
- The relation on line 160 of example 3 is damaged and looks like ≤. It is
  typed as `<=`. Typed as `<` the program prints the same for both
  functions, so the results do not decide it.

The printed results are not reproduced exactly either. Page 22 prints
`READ PART` where the program on pages 21-22 says `"REAL PART="` (the tests
follow the program); it has spaces before `=` that the program's strings
do not have, columns that follow no single zone width, and `TIME:` lines
that the 1964 system added. The tests compare the figures and labels, not
the layout. The README's "Readings and corrections" lists these by kind.

The manual is the work of Dartmouth College; these pages are reproduced for
reference and research, with their source. Check that you are comfortable
with that before publishing this folder.
