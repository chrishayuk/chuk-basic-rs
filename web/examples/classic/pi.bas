10 REM Three ways to approximate pi
20 PRINT "4*ATN(1)      ="; 4 * ATN(1)
30 REM Leibniz: 4 * (1 - 1/3 + 1/5 - 1/7 ...)
40 S = 0: G = 1
50 FOR K = 0 TO 9999
60   S = S + G / (2 * K + 1): G = -G
70 NEXT K
80 PRINT "LEIBNIZ 10000 ="; 4 * S
90 REM Wallis: 2 * (2/1 * 2/3 * 4/3 * 4/5 ...)
100 P = 1
110 FOR K = 1 TO 1000
120   P = P * (4 * K * K) / (4 * K * K - 1)
130 NEXT K
140 PRINT "WALLIS 1000   ="; 2 * P
150 END
