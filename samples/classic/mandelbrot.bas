10 REM The Mandelbrot set in ASCII
20 FOR Y = -12 TO 12
30   FOR X = -39 TO 39
40     C = X * .0458 - .5: D = Y * .08333
50     A = 0: B = 0
60     FOR I = 0 TO 15
70       T = A * A - B * B + C
80       B = 2 * A * B + D: A = T
90       IF A * A + B * B > 4 THEN 110
100     NEXT I
110     PRINT MID$(" .:-=+*#%@@@@@@@@", I + 1, 1);
120   NEXT X
130   PRINT
140 NEXT Y
150 END
