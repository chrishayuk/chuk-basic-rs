10 REM String handling: reversing, palindromes and a Caesar cipher
20 FOR W = 1 TO 3
30   READ A$
40   R$ = ""
50   FOR I = LEN(A$) TO 1 STEP -1
60     R$ = R$ + MID$(A$, I, 1)
70   NEXT I
80   PRINT A$; " REVERSED IS "; R$;
90   IF A$ = R$ THEN PRINT " (A PALINDROME)" ELSE PRINT
100 NEXT W
110 DATA BASIC, RACECAR, "HELLO, WORLD"
120 REM Shift each letter three places along the alphabet
130 P$ = "ATTACK AT DAWN": E$ = ""
140 FOR I = 1 TO LEN(P$)
150   C = ASC(MID$(P$, I, 1))
160   IF C >= 65 AND C <= 90 THEN C = 65 + (C - 65 + 3) - 26 * INT((C - 65 + 3) / 26)
170   E$ = E$ + CHR$(C)
180 NEXT I
190 PRINT P$; " -> "; E$
200 PRINT "LEFT$: "; LEFT$(P$, 6); "   RIGHT$: "; RIGHT$(P$, 4); "   LEN:"; LEN(P$)
210 PRINT "VAL OF '42' PLUS 1 ="; VAL("42") + 1; "  STR$ OF 3.5 = ["; STR$(3.5); "]"
220 END
