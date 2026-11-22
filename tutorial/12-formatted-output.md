# Formatted output: PRINT USING

`PRINT` chooses how a number looks. `PRINT USING` lets you choose: how many
digits, where the decimal point goes, whether there are commas or a currency
sign. It takes a *picture* of the output, followed by the values to fit into
it.

```basic
10 PRINT USING "###.##"; 3.14159
20 PRINT USING "###.##"; 42
30 PRINT USING "###.##"; 123.456
40 PRINT USING "TOTAL: ##### ITEMS"; 1250
```

```output
  3.14
 42.00
123.46
TOTAL:  1250 ITEMS
```

Each `#` is a place for a digit. The number is rounded to the decimal places
shown and pushed to the right of its field, so a column of figures lines up
on the decimal point. Anything in the picture that is not part of a field is
printed as it is.

## A table

```basic
10 PRINT "ITEM        QTY     PRICE"
20 FOR I = 1 TO 3
30   READ N$, Q, P
40   PRINT USING "\        \  ###  ####.##"; N$, Q, P
50 NEXT I
60 DATA WIDGET, 12, 4.5, SPROCKET, 150, 0.75, FLANGE, 3, 1299.999
```

```output
ITEM        QTY     PRICE
WIDGET       12     4.50
SPROCKET    150     0.75
FLANGE        3  1300.00
```

Two backslashes mark a string field as wide as the backslashes and
everything between them. The string is cut or padded to fit.

## Number fields

| In the picture | Prints |
| --- | --- |
| `#` | a digit position |
| `.` | the decimal point |
| `,` before the point | commas between thousands |
| `+` at the start or end | the sign, always |
| `-` at the end | a trailing minus for negative numbers |
| `**` at the start | asterisks in place of leading spaces |
| `$$` at the start | a dollar sign just before the number |
| `**$` at the start | both |
| `^^^^` at the end | exponent form |

```basic
10 PRINT USING "##,###,###.##"; 1234567.891
20 PRINT USING "+###.#"; 12.34
30 PRINT USING "+###.#"; -12.34
40 PRINT USING "###.#-"; -12.34
50 PRINT USING "**####.##"; 12.5
60 PRINT USING "$$####.##"; 12.5
70 PRINT USING "**$###.##"; 12.5
80 PRINT USING "#.###^^^^"; 123456
```

```output
 1,234,567.89
 +12.3
 -12.3
 12.3-
****12.50
   $12.50
***$12.50
0.123E+06
```

A negative number needs a place for its sign. Without a `+` or `-` in the
picture, the sign takes one of the digit positions.

```basic
10 PRINT USING "###"; -5
20 PRINT USING "###"; -50
30 PRINT USING "###"; -500
```

```output
 -5
-50
%-500
```

Line 30 did not fit. A number too big for its field is printed in full, with
a `%` in front to show that the column has been pushed out of line.

```basic
10 PRINT USING "##.#"; 5
20 PRINT USING "##.#"; 12345
```

```output
 5.0
%12345.0
```

## String fields

| In the picture | Prints |
| --- | --- |
| `!` | the first character of the string |
| `\  \` | as many characters as the field is wide, backslashes included |
| `&` | the whole string |

```basic
10 A$ = "LOVELACE"
20 PRINT USING "!"; A$
30 PRINT USING "\  \"; A$
40 PRINT USING "\      \|"; "ADA"
50 PRINT USING "& AND &"; "KEMENY", "KURTZ"
60 PRINT USING "!. !. &"; "JOHN", "GEORGE", "KEMENY"
```

```output
L
LOVE
ADA     |
KEMENY AND KURTZ
J. G. KEMENY
```

## Reusing the picture

If there are more values than fields, the picture is used again from the
start. A semicolon at the end keeps the line open, as in `PRINT`.

```basic
10 PRINT USING "###"; 1, 20, 300
20 FOR I = 1 TO 5
30   PRINT USING "#### "; I * I * I;
40 NEXT I
50 PRINT
```

```output
  1 20300
   1    8   27   64  125 
```

The picture can be a string variable, which lets one line of the program
print every row of a report:

```basic
10 F$ = "\      \ ###.# C  ###.# F"
20 FOR I = 1 TO 3
30   READ T$, C
40   PRINT USING F$; T$, C, C * 9 / 5 + 32
50 NEXT I
60 DATA FREEZING, 0, BODY, 37, BOILING, 100
```

```output
FREEZING   0.0 C   32.0 F
BODY      37.0 C   98.6 F
BOILING  100.0 C  212.0 F
```

## Printing the special characters

An underscore makes the next character ordinary, so `_#` prints a `#`.

```basic
10 PRINT USING "ITEM _###"; 7
20 PRINT USING "###_%"; 85
```

```output
ITEM # 7
 85%
```

## Money

Rounding to pennies, with commas and a currency sign:

```basic
10 FOR I = 1 TO 4
20   READ A
30   PRINT USING "$$###,###.##"; A
40 NEXT I
50 DATA 5, 1234.5, 99.999, 250000
```

```output
       $5.00
   $1,234.50
     $100.00
 $250,000.00
```

## TAB and SPC

`TAB` and `SPC`, from chapter 2, are still the right tools when the layout is
worked out by the program. This centres each line of text on a 40-column
page:

```basic
10 FOR I = 1 TO 3
20   READ T$
30   PRINT TAB((40 - LEN(T$)) / 2); T$
40 NEXT I
50 DATA BASIC, A MANUAL FOR BASIC, DARTMOUTH COLLEGE
```

```output
                  BASIC
           A MANUAL FOR BASIC
            DARTMOUTH COLLEGE
```

And this draws a bar chart, with `TAB` placing the figures after bars of
different lengths:

```basic
10 FOR I = 1 TO 4
20   READ N$, V
30   PRINT N$; TAB(8); STRING$(V / 5, "#"); TAB(30); V
40 NEXT I
50 DATA NORTH, 85, SOUTH, 40, EAST, 100, WEST, 65
```

```output
NORTH   #################      85 
SOUTH   ########               40 
EAST    ####################   100 
WEST    #############          65 
```

`PRINT USING` came with the Microsoft BASICs of the late 1970s. The listings
in *BASIC Computer Games* were written before it and do their formatting
with `TAB` and `INT`.

**Try it:** print a multiplication table from 1 to 9 with every column four
characters wide.
