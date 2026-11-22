10 REM A conversion table built with user-defined functions
20 DEF FNF(C) = C * 9 / 5 + 32
30 DEF FNK(C) = C + 273.15
40 PRINT "CELSIUS", "FAHRENHEIT", "KELVIN"
50 FOR C = -40 TO 100 STEP 20
60   PRINT C, FNF(C), FNK(C)
70 NEXT C
80 END
