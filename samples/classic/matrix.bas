10 REM Multiply two matrices held in two-dimensional arrays
20 DIM A(2, 3), B(3, 2), C(2, 2)
30 FOR I = 1 TO 2: FOR J = 1 TO 3: READ A(I, J): NEXT J: NEXT I
40 FOR I = 1 TO 3: FOR J = 1 TO 2: READ B(I, J): NEXT J: NEXT I
50 FOR I = 1 TO 2
60   FOR J = 1 TO 2
70     FOR K = 1 TO 3
80       C(I, J) = C(I, J) + A(I, K) * B(K, J)
90     NEXT K
100     PRINT C(I, J),
110   NEXT J
120   PRINT
130 NEXT I
140 DATA 1, 2, 3, 4, 5, 6
150 DATA 7, 8, 9, 10, 11, 12
160 END
