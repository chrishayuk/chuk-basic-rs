10 REM Conway's Game of Life: a glider crossing an 8 x 8 board
20 DIM A(9, 9), B(9, 9)
30 FOR I = 1 TO 5: READ R, C: A(R, C) = 1: NEXT I
40 DATA 1, 2, 2, 3, 3, 1, 3, 2, 3, 3
50 FOR G = 0 TO 4
60   PRINT "GENERATION"; G
70   FOR R = 1 TO 8
80     FOR C = 1 TO 8
90       IF A(R, C) = 1 THEN PRINT "O"; ELSE PRINT ".";
100     NEXT C
110     PRINT
120   NEXT R
130   FOR R = 1 TO 8
140     FOR C = 1 TO 8
150       N = A(R-1, C-1) + A(R-1, C) + A(R-1, C+1) + A(R, C-1)
160       N = N + A(R, C+1) + A(R+1, C-1) + A(R+1, C) + A(R+1, C+1)
170       B(R, C) = 0
180       IF N = 3 OR (N = 2 AND A(R, C) = 1) THEN B(R, C) = 1
190     NEXT C
200   NEXT R
210   FOR R = 1 TO 8: FOR C = 1 TO 8: A(R, C) = B(R, C): NEXT C: NEXT R
220 NEXT G
230 END
