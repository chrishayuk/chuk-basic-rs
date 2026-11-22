10 A$="HELLO WORLD"
20 PRINT LEFT$(A$,0);"|";LEFT$(A$,3);"|";LEFT$(A$,99);"|"
30 PRINT RIGHT$(A$,0);"|";RIGHT$(A$,3);"|";RIGHT$(A$,99);"|"
40 PRINT MID$(A$,1,1);"|";MID$(A$,7);"|";MID$(A$,7,2);"|";MID$(A$,20);"|";MID$(A$,11,5);"|";MID$(A$,3,0);"|"
50 PRINT LEN(A$);LEN("");ASC("A");ASC("AB");CHR$(65);CHR$(33)
60 PRINT STR$(5);"|";STR$(-5);"|";STR$(0);"|";STR$(1.5);"|"
70 PRINT VAL("12");VAL(" 12");VAL("12ABC");VAL("ABC");VAL("-3.5");VAL("1E2");VAL("");VAL(".5");VAL("+7")
80 PRINT "A"+"B"+"C";LEN("A"+"BC")
90 B$=A$: A$="X": PRINT B$;A$
100 PRINT LEFT$(A$+"YZ",2);MID$("ABCDEF",2.6,2);LEFT$("ABCDEF",2.5)
110 PRINT VAL("1 2 3");VAL("1E");VAL("-");VAL("3-2")
