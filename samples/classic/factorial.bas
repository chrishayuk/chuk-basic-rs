10 REM Factorials, with a loop and with a recursive function
20 DEF FNF(N)
30   IF N <= 1 THEN FNF = 1 ELSE FNF = N * FNF(N - 1)
40 FNEND
50 F = 1
60 FOR N = 1 TO 12
70   F = F * N
80   PRINT N; "! ="; F
90   IF F <> FNF(N) THEN PRINT "MISMATCH": STOP
100 NEXT N
110 PRINT "20! ="; FNF(20)
120 END
