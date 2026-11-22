# Matrices: MAT

Dartmouth BASIC had a set of statements that work on a whole array at once.
They all begin with `MAT`. Most microcomputer BASICs left them out, so they
do not appear in the game listings, but they make short work of tables and
linear algebra.

```basic
10 DIM A(2, 3)
20 MAT READ A
30 DATA 1, 2, 3, 4, 5, 6
40 MAT PRINT A;
```

```output
 1  2  3 
 4  5  6 
```

`MAT READ` fills the array from `DATA`, row by row. `MAT PRINT` prints it,
one row to a line.

The `MAT` statements use subscripts from 1. Row 0 and column 0 are left
alone. An array with one dimension is treated as a column in arithmetic, and
printed along one line.

## Reading and printing

`MAT PRINT A` puts each value in its own print zone. `MAT PRINT A;` packs
them together.

```basic
10 DIM A(2, 2), V(3)
20 MAT READ A, V
30 DATA 1, 2, 3, 4, 10, 20, 30
40 MAT PRINT A
50 MAT PRINT V;
```

```output
 1             2 
 3             4 

 10  20  30 
```

The dimensions can be given in the `MAT READ` itself, with no `DIM`:

```basic
10 MAT READ B(3, 2)
20 DATA 1, 2, 3, 4, 5, 6
30 MAT PRINT B;
```

```output
 1  2 
 3  4 
 5  6 
```

`MAT INPUT` asks for one line of numbers for each row.

```basic
10 DIM A(2, 2)
20 MAT INPUT A
30 MAT PRINT A;
```

```input
1, 2
3, 4
```

```output
? 1, 2
? 3, 4
 1  2 
 3  4 
```

## ZER, CON and IDN

These fill an array with zeros, with ones, or with the identity matrix: ones
on the diagonal and zeros elsewhere. With dimensions in brackets they also
set the array's shape.

```basic
10 DIM A(2, 3)
20 MAT A = CON
30 MAT PRINT A;
40 MAT I = IDN(3, 3)
50 MAT PRINT I;
60 MAT A = ZER(2, 2)
70 MAT PRINT A;
```

```output
 1  1  1 
 1  1  1 

 1  0  0 
 0  1  0 
 0  0  1 

 0  0 
 0  0 
```

## Arithmetic

| | |
| --- | --- |
| `MAT C = A` | copy |
| `MAT C = A + B` | add, element by element |
| `MAT C = A - B` | subtract |
| `MAT C = (K) * A` | multiply every element by K |
| `MAT C = A * B` | the matrix product |
| `MAT C = TRN(A)` | transpose: rows become columns |
| `MAT C = INV(A)` | the inverse |

The array on the left takes the shape of the result, so it needs no `DIM`.

```basic
10 DIM A(2, 2), B(2, 2)
20 MAT READ A, B
30 DATA 1, 2, 3, 4
40 DATA 10, 20, 30, 40
50 MAT C = A + B
60 MAT PRINT C;
70 MAT C = B - A
80 MAT PRINT C;
90 MAT C = (3) * A
100 MAT PRINT C;
```

```output
 11  22 
 33  44 

 9  18 
 27  36 

 3  6 
 9  12 
```

The matrix product is not element by element. Each value in the result is a
row of the first times a column of the second, so the number of columns in A
has to equal the number of rows in B.

```basic
10 DIM A(2, 3), B(3, 2)
20 MAT READ A, B
30 DATA 1, 2, 3, 4, 5, 6
40 DATA 7, 8, 9, 10, 11, 12
50 MAT C = A * B
60 MAT PRINT C;
70 MAT T = TRN(A)
80 MAT PRINT T;
```

```output
 58  64 
 139  154 

 1  4 
 2  5 
 3  6 
```

Arrays of the wrong shape stop the program:

```basic
10 DIM A(2, 3), B(2, 3)
20 MAT A = CON
30 MAT B = CON
40 MAT C = A * B
```

```output
Runtime error in line 40: Matrix dimensions do not match
```

## A use: totals from a table

A shop sells three products in two branches. Multiplying the table of sales
by the column of prices gives the takings of each branch.

```basic
10 DIM S(2, 3), P(3)
20 MAT READ S, P
30 DATA 10, 4, 7
40 DATA 2, 12, 5
50 DATA 1.5, 3, 10
60 MAT T = S * P
70 MAT PRINT T;
```

```output
 97  89 
```

Without `MAT` that is two nested loops and a running total.

## Solving equations

`INV` gives the inverse of a square matrix, and the inverse solves
simultaneous equations. These two,

```text
2x + y = 5
 x + y = 3
```

are solved by multiplying the inverse of the coefficients by the right-hand
sides:

```basic
10 DIM A(2, 2), B(2)
20 MAT READ A, B
30 DATA 2, 1, 1, 1
40 DATA 5, 3
50 MAT I = INV(A)
60 MAT X = I * B
70 MAT PRINT X;
80 MAT C = A * I
90 MAT PRINT C;
```

```output
 2  1 

 1  0 
 0  1 
```

So x is 2 and y is 1. Lines 80 and 90 check the inverse: a matrix times its
inverse is the identity. A matrix with no inverse stops the program:

```basic
10 DIM A(2, 2)
20 MAT READ A
30 DATA 1, 2, 2, 4
40 MAT I = INV(A)
```

```output
Runtime error in line 40: Matrix is singular
```

**Try it:** read a 3 by 3 matrix and print its transpose, then check that
transposing twice gives the matrix you started with.
