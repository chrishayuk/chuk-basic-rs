10 REM Greatest common divisor by Euclid's algorithm
20 READ N
30 FOR I = 1 TO N
40   READ A, B
50   X = A: Y = B
60   IF Y = 0 THEN 100
70   T = X - INT(X / Y) * Y
80   X = Y: Y = T
90   GOTO 60
100   PRINT "GCD("; A; ","; B; ") ="; X
110 NEXT I
120 DATA 4
130 DATA 48, 18, 1071, 462, 17, 5, 100, 75
140 END
