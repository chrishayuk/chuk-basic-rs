# PRINT

`PRINT` is the statement you will write most often. It shows text, numbers
and the results of calculations, and it has a small language of its own for
laying them out.

**`samples/tutorial/02_print.bas`**

```basic
10 PRINT "A SEMICOLON JOINS: "; "LIKE"; "THIS"
20 PRINT "A COMMA", "MOVES TO", "THE NEXT ZONE"
30 PRINT "NUMBERS:"; 42; -7; 3.5
40 PRINT TAB(10); "TEN SPACES IN"
50 PRINT "NO NEW LINE ";
60 PRINT "AFTER A SEMICOLON"
70 PRINT
80 ? "A QUESTION MARK IS SHORT FOR PRINT"
```

```output
A SEMICOLON JOINS: LIKETHIS
A COMMA       MOVES TO      THE NEXT ZONE
NUMBERS: 42 -7  3.5 
          TEN SPACES IN
NO NEW LINE AFTER A SEMICOLON

A QUESTION MARK IS SHORT FOR PRINT
```

- A **semicolon** between items prints them right next to each other.
- A **comma** moves to the next *print zone*. Zones are 14 characters wide,
  which makes quick columns.
- **Numbers** are printed with a space in front, where a minus sign would
  go, and a space after. That is why `-7` and `3.5` do not run together.
- `TAB(n)` moves to column `n`, counting from 0.
- Ending a `PRINT` with a semicolon keeps the cursor on the same line.
- `PRINT` on its own prints a blank line.
- `?` is short for `PRINT`.

## Text

Everything between the double quotes is printed exactly, spaces included.

```basic
10 PRINT "  TWO SPACES FIRST"
20 PRINT "lower case stays lower case"
30 PRINT "SYMBOLS: $ % & ' ( ) * + , - . / : ; < = > ? @"
```

```output
  TWO SPACES FIRST
lower case stays lower case
SYMBOLS: $ % & ' ( ) * + , - . / : ; < = > ? @
```

A string cannot contain a double quote, because that ends it. Chapter 10
shows how to print one with `CHR$(34)`.

## Numbers

A number is printed with a sign position in front and a space behind.

```basic
10 PRINT 42
20 PRINT -42
30 PRINT 42; 42; 42
40 PRINT "["; 42; "]"
```

```output
 42 
-42 
 42  42  42 
[ 42 ]
```

Whole numbers print as whole numbers. Fractions print with up to nine
significant digits, and without a zero before the decimal point.

```basic
10 PRINT 1 / 4
20 PRINT 1 / 3
30 PRINT 2 / 3
40 PRINT 100 / 3
50 PRINT 10 / 5
```

```output
 .25 
 .333333333 
 .666666667 
 33.3333333 
 2 
```

Very large and very small numbers are printed in exponent form. `1E+20`
means 1 followed by twenty zeros, and `1E-10` is a decimal point, nine zeros
and a 1. Numbers can be written that way in a program too.

```basic
10 PRINT 1234567890
20 PRINT 2 ^ 40
30 PRINT 1E20
40 PRINT 2 ^ 70
50 PRINT .000001
60 PRINT 1E-10
70 PRINT 1 / 3E15
```

```output
 1234567890 
 1099511627776 
 1E+20 
 1.18059162E+21 
 .000001 
 1E-10 
 3.33333333E-16 
```

## Calculations

Anything that works out to a value can be printed. No variable is needed.

```basic
10 PRINT 7 + 5
20 PRINT 7 - 5
30 PRINT 7 * 5
40 PRINT 7 / 5
50 PRINT "SEVEN SQUARED IS"; 7 ^ 2
```

```output
 12 
 2 
 35 
 1.4 
SEVEN SQUARED IS 49 
```

## Commas and zones

The line is divided into zones 14 columns wide. A comma moves to the start
of the next zone, so items line up whatever their length.

```basic
10 PRINT "NAME", "AGE", "TOWN"
20 PRINT "ADA", 36, "LONDON"
30 PRINT "GRACE", 85, "NEW YORK"
40 PRINT 1, 2, 3, 4, 5
```

```output
NAME          AGE           TOWN
ADA            36           LONDON
GRACE          85           NEW YORK
 1             2             3             4             5 
```

A number in a zone still has its leading space, so numbers sit one column to
the right of text. Two commas together skip a zone:

```basic
10 PRINT "A", , "C"
20 PRINT , "B"
```

```output
A                           C
              B
```

## Staying on the line

A semicolon or comma at the end of a `PRINT` stops it from starting a new
line. The next `PRINT` carries on where this one left off.

```basic
10 PRINT "ONE ";
20 PRINT "TWO ";
30 PRINT "THREE"
40 PRINT "A",
50 PRINT "B"
```

```output
ONE TWO THREE
A             B
```

## TAB and SPC

`TAB(n)` moves to column `n`. The first column is 0. If the line is already
past column `n`, `TAB` does nothing.

`SPC(n)` prints `n` spaces from wherever the line has got to.

```basic
10 PRINT "0123456789012345678901234567890"
20 PRINT TAB(5); "FIVE"; TAB(20); "TWENTY"
30 PRINT "ABC"; SPC(5); "DEF"
40 PRINT "LONG TEXT HERE"; TAB(5); "TAB DID NOTHING"
```

```output
0123456789012345678901234567890
     FIVE           TWENTY
ABC     DEF
LONG TEXT HERETAB DID NOTHING
```

`TAB` can take a calculation, which is how the old programs drew pictures:

```basic
10 FOR I = 0 TO 4
20   PRINT TAB(I * 3); "*"
30 NEXT I
```

```output
*
   *
      *
         *
            *
```

Chapter 12 covers `PRINT USING`, which lines up columns of figures and money.

**Try it:** print your initials in large letters made of asterisks, five lines
high.
