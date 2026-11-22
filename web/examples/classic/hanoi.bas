10 REM Towers of Hanoi for 3 discs, using an explicit stack
20 REM so that the subroutine can call itself
30 DIM N(20), F(20), T(20), V(20)
40 S = 0: M = 0
50 N = 3: F = 1: T = 3: V = 2
60 GOSUB 100
70 PRINT M; "MOVES"
80 END
100 REM Move N discs from peg F to peg T via peg V
110 IF N = 0 THEN RETURN
120 S = S + 1: N(S) = N: F(S) = F: T(S) = T: V(S) = V
130 N = N - 1: X = T: T = V: V = X
140 GOSUB 100
150 N = N(S): F = F(S): T = T(S): V = V(S)
160 M = M + 1
170 PRINT "MOVE DISC"; N; "FROM PEG"; F; "TO PEG"; T
180 N = N - 1: X = F: F = V: V = X
190 GOSUB 100
200 N = N(S): F = F(S): T = T(S): V = V(S): S = S - 1
210 RETURN
