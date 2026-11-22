# Worked examples

Seven complete programs, each a little longer than those in the earlier
chapters, with notes on how they work.

## The sieve of Eratosthenes

To find the primes up to 100, write down every number, then cross out the
multiples of 2, of 3, and of each number that has not itself been crossed
out. What is left is prime.

```basic
10 N = 100
20 DIM C(N)
30 FOR I = 2 TO SQR(N)
40   IF C(I) = 1 THEN 80
50   FOR J = I * I TO N STEP I
60     C(J) = 1
70   NEXT J
80 NEXT I
90 K = 0
100 FOR I = 2 TO N
110   IF C(I) = 0 THEN PRINT I;: K = K + 1
120 NEXT I
130 PRINT
140 PRINT K; "PRIMES"
```

```output
 2  3  5  7  11  13  17  19  23  29  31  37  41  43  47  53  59  61  67  71  73  79  83  89  97 
 25 PRIMES
```

- `C(I)` is 1 when I has been crossed out. A new array is all zeros, so
  nothing needs setting up.
- The outer loop stops at the square root of N. A number below N that is not
  prime has a factor no bigger than that.
- Crossing out starts at `I * I`, since smaller multiples of I have a
  smaller factor and are already crossed out.
- The `STEP I` on line 50 visits the multiples directly. No division is
  done anywhere.

## Any number base

To write a number in base B, divide by B again and again. The remainders are
the digits, last digit first.

```basic
10 D$ = "0123456789ABCDEF"
20 FOR T = 1 TO 4
30   READ N, B
40   X = N: R$ = ""
50   Q = INT(X / B)
60   R$ = MID$(D$, X - Q * B + 1, 1) + R$
70   X = Q
80   IF X > 0 THEN 50
90   PRINT N; "IN BASE"; B; "IS "; R$
100 NEXT T
110 DATA 10, 2, 255, 2, 255, 16, 1964, 8
```

```output
 10 IN BASE 2 IS 1010
 255 IN BASE 2 IS 11111111
 255 IN BASE 16 IS FF
 1964 IN BASE 8 IS 3654
```

- Line 60 picks the digit out of a string of all sixteen digits. `MID$`
  counts from 1, so a remainder of 0 needs position 1.
- Each new digit is put on the *front* of `R$`, which reverses the order in
  which they were found.
- Lines 50 to 80 are a loop that always runs at least once, so that 0 would
  print as `0` and not as nothing.

## A loan

A table of repayments: each month interest is added to what is owed and a
fixed payment is taken off.

```basic
10 P = 1000: R = 12: M = 6
20 I = R / 1200
30 A = P * I / (1 - (1 + I) ^ -M)
40 PRINT USING "MONTHLY PAYMENT: ####.##"; A
50 PRINT
60 PRINT "MONTH  INTEREST  REPAID   OWING"
70 FOR K = 1 TO M
80   N = P * I
90   P = P + N - A
100   PRINT USING "  ##    ###.##  ###.##  ####.##"; K, N, A - N, ABS(P)
110 NEXT K
```

```output
MONTHLY PAYMENT:  172.55

MONTH  INTEREST  REPAID   OWING
   1     10.00  162.55   837.45
   2      8.37  164.17   673.28
   3      6.73  165.82   507.46
   4      5.07  167.47   339.99
   5      3.40  169.15   170.84
   6      1.71  170.84     0.00
```

- Line 20 turns a yearly percentage into a monthly fraction.
- Line 30 is the standard formula for the payment that clears a loan in M
  months.
- `ABS(P)` on line 100 is there for the last row. After the final payment P
  is a tiny number either side of zero, and without `ABS` it could print as
  `-0.00`.

## Day of the week

Zeller's rule gives the day of the week for any date. January and February
are counted as months 13 and 14 of the year before.

```basic
10 DIM N$(6)
20 FOR I = 0 TO 6: READ N$(I): NEXT I
30 DATA SATURDAY, SUNDAY, MONDAY, TUESDAY, WEDNESDAY, THURSDAY, FRIDAY
40 FOR T = 1 TO 3
50   READ D, M, Y
60   PRINT D; "/"; M; "/"; Y; "WAS A ";
70   IF M < 3 THEN M = M + 12: Y = Y - 1
80   K = Y - INT(Y / 100) * 100
90   J = INT(Y / 100)
100   H = D + INT(13 * (M + 1) / 5) + K + INT(K / 4) + INT(J / 4) + 5 * J
110   H = H - INT(H / 7) * 7
120   PRINT N$(H)
130 NEXT T
140 DATA 1, 5, 1964, 20, 7, 1969, 1, 1, 2000
```

```output
 1 / 5 / 1964 WAS A FRIDAY
 20 / 7 / 1969 WAS A SUNDAY
 1 / 1 / 2000 WAS A SATURDAY
```

- Lines 80 and 90 split the year into its last two digits and its century.
- Every division is wrapped in `INT`. The rule depends on throwing the
  fractions away.
- Line 110 is the remainder on dividing by 7, which is the subscript of the
  day's name.

BASIC first ran on 1 May 1964, which the first line of output says was a
Friday.

## The Towers of Hanoi

Move a pile of discs from one peg to another, one disc at a time, never
putting a larger disc on a smaller. To move N discs, move N - 1 out of the
way, move the largest, then move the N - 1 back on top.

That description uses itself, so the subroutine calls itself. BASIC's
variables are all shared, so the values each call needs afterwards are kept
in arrays, with `P` counting how deep the calls have gone.

```basic
10 DIM N(10), F(10), T(10), V(10)
20 P = 1: N(1) = 3: F(1) = 1: T(1) = 3: V(1) = 2: M = 0
30 GOSUB 100
40 PRINT M; "MOVES"
50 END
100 REM MOVE N(P) DISCS FROM PEG F(P) TO PEG T(P) USING PEG V(P)
110 IF N(P) = 0 THEN RETURN
120 N(P + 1) = N(P) - 1: F(P + 1) = F(P): T(P + 1) = V(P): V(P + 1) = T(P)
130 P = P + 1: GOSUB 100: P = P - 1
140 PRINT "DISC"; N(P); "FROM"; F(P); "TO"; T(P)
150 M = M + 1
160 N(P + 1) = N(P) - 1: F(P + 1) = V(P): T(P + 1) = T(P): V(P + 1) = F(P)
170 P = P + 1: GOSUB 100: P = P - 1
180 RETURN
```

```output
DISC 1 FROM 1 TO 3 
DISC 2 FROM 1 TO 2 
DISC 1 FROM 3 TO 2 
DISC 3 FROM 1 TO 3 
DISC 1 FROM 2 TO 1 
DISC 2 FROM 2 TO 3 
DISC 1 FROM 1 TO 3 
 7 MOVES
```

- Each level of calls has its own row in the arrays: how many discs, from
  which peg, to which, and using which as the spare.
- Lines 120 and 160 fill in the row for the call about to be made.
- `P = P + 1: GOSUB 100: P = P - 1` is the whole of the call. Nothing a
  deeper call does can disturb row P.

Three discs take 7 moves. N discs take 2 to the power N, less 1.

## A histogram of letters

Count how often each letter appears in a piece of text and draw the counts
as bars.

```basic
10 DIM C(26)
20 A$ = "THE QUICK BROWN FOX JUMPS OVER THE LAZY DOG AND THEN THE DOG SLEEPS"
30 FOR I = 1 TO LEN(A$)
40   K = ASC(MID$(A$, I, 1)) - 64
50   IF K >= 1 AND K <= 26 THEN C(K) = C(K) + 1
60 NEXT I
70 FOR K = 1 TO 26
80   IF C(K) >= 3 THEN PRINT CHR$(K + 64); " "; STRING$(C(K), "*"); C(K)
90 NEXT K
```

```output
D *** 3 
E ******* 7 
H **** 4 
N *** 3 
O ***** 5 
S *** 3 
T **** 4 
```

- Line 40 turns a letter into a number from 1 to 26 by subtracting 64 from
  its code. `A` is 65.
- Line 50 ignores spaces and anything else that is not a capital letter.
- Line 80 shows only the letters that appear three times or more, and turns
  the number back into its letter.

## A text adventure in thirty lines

Three rooms, a key and a locked door. The program reads a command of one or
two words and acts on the first letter of each.

```basic
10 DIM R$(3), N(3), S(3)
20 FOR I = 1 TO 3: READ R$(I), N(I), S(I): NEXT I
30 DATA A DUSTY HALL, 2, 0, A LIBRARY, 3, 1, THE TREASURE ROOM, 0, 2
40 L = 1: K = 2: H = 0
50 PRINT: PRINT "YOU ARE IN "; R$(L); "."
60 IF K = L THEN PRINT "THERE IS A KEY HERE."
70 IF L = 3 THEN PRINT "GOLD EVERYWHERE. YOU WIN.": END
80 LINE INPUT "> "; C$
90 V$ = LEFT$(C$, 1)
100 S = INSTR(C$, " ")
110 O$ = "": IF S > 0 THEN O$ = MID$(C$, S + 1, 1)
120 IF V$ = "N" THEN D = N(L): GOTO 200
130 IF V$ = "S" THEN D = S(L): GOTO 200
140 IF V$ = "T" AND O$ = "K" AND K = L THEN K = 0: H = 1: PRINT "TAKEN.": GOTO 80
150 IF V$ = "Q" THEN PRINT "GOODBYE.": END
160 PRINT "I DO NOT UNDERSTAND.": GOTO 80
200 IF D = 0 THEN PRINT "YOU CANNOT GO THAT WAY.": GOTO 80
210 IF D = 3 AND H = 0 THEN PRINT "THE DOOR IS LOCKED.": GOTO 80
220 L = D: GOTO 50
```

```input
SOUTH
NORTH
NORTH
DANCE
TAKE KEY
NORTH
```

```output

YOU ARE IN A DUSTY HALL.
> SOUTH
YOU CANNOT GO THAT WAY.
> NORTH

YOU ARE IN A LIBRARY.
THERE IS A KEY HERE.
> NORTH
THE DOOR IS LOCKED.
> DANCE
I DO NOT UNDERSTAND.
> TAKE KEY
TAKEN.
> NORTH

YOU ARE IN THE TREASURE ROOM.
GOLD EVERYWHERE. YOU WIN.
```

- The map is in `DATA`: for each room its name, the room to the north and
  the room to the south, with 0 for "no way through".
- `L` is the room the player is in. `K` is the room the key is in, or 0 once
  it has been taken. `H` is 1 when the player holds it.
- Lines 90 to 110 split the command. `V$` is the first letter of the first
  word and `O$` the first letter of the second, if there is one.
- Lines 200 to 220 are shared by every direction. Adding east and west would
  take two more arrays and two more lines.

The full-size adventures of the 1970s are this program with more rooms, more
objects and a longer list of verbs.
