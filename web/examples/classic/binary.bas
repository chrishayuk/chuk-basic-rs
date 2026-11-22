10 REM Decimal to binary and hexadecimal
20 H$ = "0123456789ABCDEF"
30 FOR I = 1 TO 6
40   READ N
50   B$ = "": X$ = "": M = N
60   B$ = CHR$(48 + M - 2 * INT(M / 2)) + B$
70   M = INT(M / 2): IF M > 0 THEN 60
80   M = N
90   X$ = MID$(H$, M - 16 * INT(M / 16) + 1, 1) + X$
100   M = INT(M / 16): IF M > 0 THEN 90
110   PRINT N, B$, X$
120 NEXT I
130 DATA 0, 5, 10, 255, 1024, 48879
140 END
