10 REM A multiplication table laid out with TAB
20 PRINT "  X";
30 FOR J = 1 TO 9: PRINT TAB(J * 4 + 1); J;: NEXT J
40 PRINT
50 FOR I = 1 TO 9
60   PRINT I;
70   FOR J = 1 TO 9
80     PRINT TAB(J * 4 + 1); I * J;
90   NEXT J
100   PRINT
110 NEXT I
120 END
