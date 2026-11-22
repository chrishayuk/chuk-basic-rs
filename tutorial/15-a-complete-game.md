# A complete game

Everything so far, in under forty lines. The computer shows a card and you
say whether the next will be higher or lower.

**`samples/tutorial/13_game.bas`**

```basic
10 REM HIGHER OR LOWER - a complete small game
20 PRINT "HIGHER OR LOWER": PRINT
30 S = 0
40 C = INT(RND(1) * 13) + 1
50 FOR T = 1 TO 5
60   GOSUB 300: PRINT "THE CARD IS "; C$
70   INPUT "WILL THE NEXT BE HIGHER OR LOWER (H/L)"; A$
80   A$ = LEFT$(A$, 1)
90   IF A$ <> "H" AND A$ <> "L" THEN PRINT "PLEASE ANSWER H OR L.": GOTO 70
100   N = INT(RND(1) * 13) + 1
110   IF N = C THEN 100
120   W = 0
130   IF A$ = "H" AND N > C THEN W = 1
140   IF A$ = "L" AND N < C THEN W = 1
150   C = N: GOSUB 300
160   IF W = 1 THEN S = S + 1: PRINT "IT IS "; C$; ". RIGHT!"
170   IF W = 0 THEN PRINT "IT IS "; C$; ". WRONG."
180   PRINT
190 NEXT T
200 PRINT "YOU SCORED"; S; "OUT OF 5."
210 IF S = 5 THEN PRINT "PERFECT!"
220 END
300 REM Set C$ to the name of card C
310 IF C = 1 THEN C$ = "AN ACE": RETURN
320 IF C = 11 THEN C$ = "A JACK": RETURN
330 IF C = 12 THEN C$ = "A QUEEN": RETURN
340 IF C = 13 THEN C$ = "A KING": RETURN
350 IF C = 8 THEN C$ = "AN 8": RETURN
360 C$ = "A" + STR$(C)
370 RETURN
```

```input
H
X
L
H
L
H
```

```output
HIGHER OR LOWER

THE CARD IS A 3
WILL THE NEXT BE HIGHER OR LOWER (H/L)? H
IT IS AN 8. RIGHT!

THE CARD IS AN 8
WILL THE NEXT BE HIGHER OR LOWER (H/L)? X
PLEASE ANSWER H OR L.
WILL THE NEXT BE HIGHER OR LOWER (H/L)? L
IT IS A 7. RIGHT!

THE CARD IS A 7
WILL THE NEXT BE HIGHER OR LOWER (H/L)? H
IT IS A 4. WRONG.

THE CARD IS A 4
WILL THE NEXT BE HIGHER OR LOWER (H/L)? L
IT IS A JACK. WRONG.

THE CARD IS A JACK
WILL THE NEXT BE HIGHER OR LOWER (H/L)? H
IT IS AN 8. WRONG.

YOU SCORED 2 OUT OF 5.
```

## How it is put together

- **Lines 20 to 40** set up: a title, a score of 0, and a first card from 1
  to 13.
- **Lines 50 to 190** are one turn, repeated five times by a `FOR` loop.
- **Lines 200 to 220** report the score and stop.
- **Lines 300 to 370** are a subroutine that turns a card number into a name.

Things to notice:

- The subroutine at line 300 is called from two places, lines 60 and 150.
  Its input is `C` and its output is `C$`.
- Line 80 keeps only the first letter of the answer, so `H`, `HIGHER` and
  `HIGH` all work.
- Line 90 checks the answer and asks again if it makes no sense. Programs
  that talk to people need a lot of this.
- Line 110 deals again if the new card is the same as the old one, so that
  there is always a right answer.
- `W` is a *flag*: 1 for a win, 0 for a loss, set by lines 130 and 140 and
  used by lines 160 and 170.
- Line 350 exists because of English: "AN 8", not "A 8". Line 360 uses
  `STR$`, which supplies the space between "A" and the number.

## Writing a game of your own

The games of the 1970s nearly all have the same shape, and it is a good one
to copy:

1. Print the title and the rules.
2. Set up the starting position.
3. Show the position.
4. Ask for a move. Check it. Ask again if it is not allowed.
5. Work out what happens, using `RND` for anything left to chance.
6. If the game is not over, go back to step 3.
7. Say who won, and offer another game.

Here is that shape as a second game. There are 21 matches. You and the
computer take turns to remove 1, 2 or 3, and whoever takes the last match
loses.

```basic
10 PRINT "TWENTY-ONE MATCHES": PRINT
20 M = 21
30 PRINT "THERE ARE"; M; "MATCHES."
40 INPUT "HOW MANY DO YOU TAKE (1-3)"; T
50 IF T < 1 OR T > 3 OR T <> INT(T) OR T > M THEN PRINT "YOU CANNOT DO THAT.": GOTO 40
60 M = M - T
70 IF M = 0 THEN PRINT "YOU TOOK THE LAST MATCH. I WIN.": END
80 C = M - 1 - INT((M - 1) / 4) * 4
90 IF C = 0 THEN C = INT(RND(1) * 3) + 1
100 IF C > M THEN C = M
110 PRINT "I TAKE"; C
120 M = M - C
130 IF M = 0 THEN PRINT "I TOOK THE LAST MATCH. YOU WIN.": END
140 GOTO 30
```

```input
2
5
3
1
3
2
1
```

```output
TWENTY-ONE MATCHES

THERE ARE 21 MATCHES.
HOW MANY DO YOU TAKE (1-3)? 2
I TAKE 2 
THERE ARE 17 MATCHES.
HOW MANY DO YOU TAKE (1-3)? 5
YOU CANNOT DO THAT.
HOW MANY DO YOU TAKE (1-3)? 3
I TAKE 1 
THERE ARE 13 MATCHES.
HOW MANY DO YOU TAKE (1-3)? 1
I TAKE 3 
THERE ARE 9 MATCHES.
HOW MANY DO YOU TAKE (1-3)? 3
I TAKE 1 
THERE ARE 5 MATCHES.
HOW MANY DO YOU TAKE (1-3)? 2
I TAKE 2 
THERE ARE 1 MATCHES.
HOW MANY DO YOU TAKE (1-3)? 1
YOU TOOK THE LAST MATCH. I WIN.
```

Line 80 is the computer's strategy. It wants to leave a number of matches
that is one more than a multiple of four: 17, 13, 9, 5, 1. `C` is the
number it must take to get there. If that is 0 there is no winning move, and
line 90 takes a random number of matches and hopes for a mistake.

Line 50 does all the checking in one condition: at least 1, at most 3, a
whole number, and no more than are left.

## Reading the old games

`samples/games` holds 103 games from *BASIC Computer Games*. They are the
best way to see what people built with the statements in this tutorial. Good
ones to start with:

| Program | What it is |
| --- | --- |
| `guess.bas` | guess the number, with a comment on how well you did |
| `hi-lo.bas` | guess the jackpot in six tries |
| `nim.bas`, `23matches.bas` | relatives of the game above |
| `hammurabi.bas` | rule a city for ten years: grain, land and plague |
| `lunar.bas` | land on the moon with the fuel you have |
| `superstartrek.bas` | the big one: 400 lines, a galaxy, and Klingons |

Run one with a seed to get the same game every time:

```bash
basic --seed 1 samples/games/hammurabi.bas
```

They were written for small memories, so they are terse: one-letter
variables, several statements to a line, few remarks. Chapter 21 lists the
habits you will meet in them.

**Try it:** keep a running total across several games of Higher or Lower, or
let the player bet points on each card.
