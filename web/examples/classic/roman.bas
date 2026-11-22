10 REM Roman numerals
20 DIM V(13), R$(13)
30 FOR I = 1 TO 13: READ V(I), R$(I): NEXT I
40 DATA 1000, M, 900, CM, 500, D, 400, CD, 100, C, 90, XC, 50, L
50 DATA 40, XL, 10, X, 9, IX, 5, V, 4, IV, 1, I
60 FOR K = 1 TO 6
70   READ N
80   M = N: A$ = ""
90   FOR I = 1 TO 13
100     IF M >= V(I) THEN A$ = A$ + R$(I): M = M - V(I): GOTO 100
110   NEXT I
120   PRINT N; "= "; A$
130 NEXT K
140 DATA 4, 9, 14, 1964, 1999, 3888
150 END
