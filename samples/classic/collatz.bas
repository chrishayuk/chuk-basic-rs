10 REM Collatz sequences: halve if even, triple and add one if odd
20 FOR S = 1 TO 4
30   READ N
40   PRINT N;
50   C = 0
60   IF N = 1 THEN 110
70   IF N / 2 = INT(N / 2) THEN N = N / 2 ELSE N = 3 * N + 1
80   PRINT N;
90   C = C + 1
100   GOTO 60
110   PRINT: PRINT "  ("; C; "STEPS )"
120 NEXT S
130 DATA 6, 7, 27, 1
140 END
