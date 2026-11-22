// Tests for syntaxes/basic.tmLanguage.json, run with `npm test`. No VS Code
// needed: tokens.js drives the grammar with the engine VS Code uses.

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");

const { shape, tokenize } = require("./tokens.js");

const samples = path.join(__dirname, "..", "..", "..", "samples");

/** Each case is a line of BASIC and the tokens it should give. */
async function cases(table) {
  for (const [line, expected] of table) assert.equal(await shape(line), expected, line);
}

test("line numbers, and the lines they point at", () =>
  cases([
    ['10 PRINT "HELLO"', '10:lineno PRINT:keyword "HELLO":string'],
    ["  20 END", "20:lineno END:control"],
    ["310 GOTO 10", "310:lineno GOTO:control 10:target"],
    ["90 GO TO 300", "90:lineno GO TO:control 300:target"],
    ["95 GOSUB 500", "95:lineno GOSUB:control 500:target"],
    ["50 IF D = 0 THEN 100", "50:lineno IF:control D:var =:op 0:number THEN:control 100:target"],
    [
      '90 IF X THEN PRINT "A" ELSE 120',
      '90:lineno IF:control X:var THEN:control PRINT:keyword "A":string ELSE:control 120:target',
    ],
    ["100 RESTORE 400", "100:lineno RESTORE:keyword 400:target"],
    [
      "20 ON X GOSUB 100, 200,300: PRINT",
      "20:lineno ON:control X:var GOSUB:control 100:target ,:punct 200:target ,:punct 300:target ::punct PRINT:keyword",
    ],
    ["30 ON K GO TO 10,20", "30:lineno ON:control K:var GO TO:control 10:target ,:punct 20:target"],
    // a number that is not a line number stays a number
    ["40 X = 100", "40:lineno X:var =:op 100:number"],
    // a line with no number (typed at the prompt)
    ["PRINT 2+2", "PRINT:keyword 2:number +:op 2:number"],
  ]));

test("the 1964 manual's programs", () =>
  cases([
    ["10 LET X = (7+8)/3", "10:lineno LET:keyword X:var =:op (:plain 7:number +:op 8:number ):plain /:op 3:number"],
    [
      "40 LET T = SQR(D) / (2 * A)",
      "40:lineno LET:keyword T:var =:op SQR:function (:plain D:var ):plain /:op (:plain 2:number *:op A:var ):plain",
    ],
    // the upwards arrow for powers
    [
      "30 DEF FNF(Z) = Z↑5 +Z↑3 -1",
      "30:lineno DEF:keyword FNF:fn (:plain Z:var ):plain =:op Z:var ↑:op 5:number +:op Z:var ↑:op 3:number -:op 1:number",
    ],
    // lower-case `to`, as the page prints it
    ["30  FOR N = 1 to 100", "30:lineno FOR:control N:var =:op 1:number to:control 100:number"],
    ['80 PRINT "ROOTS =" S + T, S - T', '80:lineno PRINT:keyword "ROOTS =":string S:var +:op T:var ,:punct S:var -:op T:var'],
    [
      "400 DATA 1, -2, -3, 1, -4, 4",
      "400:lineno DATA:keyword 1:number ,:punct -2:number ,:punct -3:number ,:punct 1:number ,:punct -4:number ,:punct 4:number",
    ],
  ]));

test("keywords, functions, names, numbers and operators", () =>
  cases([
    ["40 FOR I = 1 TO TOTAL STEP 2", "40:lineno FOR:control I:var =:op 1:number TO:control TOTAL:var STEP:control 2:number"],
    ["45 NEXT I", "45:lineno NEXT:control I:var"],
    ["140 WHILE X<10: X=X+1: WEND", "140:lineno WHILE:control X:var <:op 10:number ::punct X:var =:op X:var +:op 1:number ::punct WEND:control"],
    ["50 DIM A(10), B$(5)", "50:lineno DIM:keyword A:var (:plain 10:number ):plain ,:punct B$:strvar (:plain 5:number ):plain"],
    ["55 READ A, B: RANDOMIZE: RETURN", "55:lineno READ:keyword A:var ,:punct B:var ::punct RANDOMIZE:keyword ::punct RETURN:control"],
    ['60 INPUT "NAME"; N$', '60:lineno INPUT:keyword "NAME":string ;:punct N$:strvar'],
    ['80 LINE INPUT "NAME"; N$', '80:lineno LINE INPUT:keyword "NAME":string ;:punct N$:strvar'],
    ['70 PRINT USING "###.##"; X', '70:lineno PRINT:keyword USING:keyword "###.##":string ;:punct X:var'],
    ["60 MAT A = ZER(3,3)", "60:lineno MAT:keyword A:var =:op ZER:function (:plain 3:number ,:punct 3:number ):plain"],
    ["61 MAT PRINT A;", "61:lineno MAT:keyword PRINT:keyword A:var ;:punct"],
    ["62 MAT C = INV(A)", "62:lineno MAT:keyword C:var =:op INV:function (:plain A:var ):plain"],
    ["65 DEF FNA(X): FNEND", "65:lineno DEF:keyword FNA:fn (:plain X:var ):plain ::punct FNEND:keyword"],
    [
      "160 PRINT TAB(5);FNA(3);CHR$(65)",
      "160:lineno PRINT:keyword TAB:function (:plain 5:number ):plain ;:punct FNA:fn (:plain 3:number ):plain ;:punct CHR$:function (:plain 65:number ):plain",
    ],
    ["165 X = RND + LEFT$(A$, 2)", "165:lineno X:var =:op RND:function +:op LEFT$:function (:plain A$:strvar ,:punct 2:number ):plain"],
    [
      "110 B=1.5E-3:C=.5:D=2E10",
      "110:lineno B:var =:op 1.5E-3:number ::punct C:var =:op .5:number ::punct D:var =:op 2E10:number",
    ],
    ["115 IF A <> B AND NOT C OR D >= 1 THEN 10", "115:lineno IF:control A:var <>:op B:var AND:logical NOT:logical C:var OR:logical D:var >=:op 1:number THEN:control 10:target"],
    ["116 X = 2 ^ 3", "116:lineno X:var =:op 2:number ^:op 3:number"],
    // any case
    ['120 print "a": goto 10', '120:lineno print:keyword "a":string ::punct goto:control 10:target'],
    ['125 ? "A"', '125:lineno ?:keyword "A":string'],
    ["170 STOP", "170:lineno STOP:control"],
  ]));

test("comments and DATA", () =>
  cases([
    ["10 REM A COMMENT WITH PRINT AND \"QUOTES\"", '10:lineno REM A COMMENT WITH PRINT AND "QUOTES":comment'],
    ["36 X=1: REM note", "36:lineno X:var =:op 1:number ::punct REM note:comment"],
    ["37 PRINT X ' note", "37:lineno PRINT:keyword X:var ' note:comment"],
    // an apostrophe inside a string is not a comment
    ['38 PRINT "IT\'S"', '38:lineno PRINT:keyword "IT\'S":string'],
    // DATA runs to an unquoted colon; bare words in it are text
    [
      '400 DATA 1, "A:B", HELLO: PRINT X',
      '400:lineno DATA:keyword 1:number ,:punct "A:B":string ,:punct HELLO:data ::punct PRINT:keyword X:var',
    ],
    ["410 DATA PRINT, 12ABC", "410:lineno DATA:keyword PRINT:data ,:punct 12ABC:data"],
  ]));

test("listings typed without spaces", () =>
  cases([
    [
      '10 FORI=1TO10:PRINTI;"A":NEXTI',
      '10:lineno FOR:control I:var =:op 1:number TO:control 10:number ::punct PRINT:keyword I:var ;:punct "A":string ::punct NEXT:control I:var',
    ],
    ["15 FORI=Q1-1TOQ1+1STEP2", "15:lineno FOR:control I:var =:op Q1:var -:op 1:number TO:control Q1:var +:op 1:number STEP:control 2:number"],
    ["16 FORJ=ATOB", "16:lineno FOR:control J:var =:op A:var TO:control B:var"],
    ["41 FOR I=1TON", "41:lineno FOR:control I:var =:op 1:number TO:control N:var"],
    ["20 IFA=BTHEN100", "20:lineno IF:control A:var =:op B:var THEN:control 100:target"],
    [
      '25 IFA=1ANDB$="X"THENPRINT"Y":GOTO50',
      '25:lineno IF:control A:var =:op 1:number AND:logical B$:strvar =:op "X":string THEN:control PRINT:keyword "Y":string ::punct GOTO:control 50:target',
    ],
    ["26 IFA=1THENX=2", "26:lineno IF:control A:var =:op 1:number THEN:control X:var =:op 2:number"],
    ["27 ONXGOTO10,20", "27:lineno ON:control X:var GOTO:control 10:target ,:punct 20:target"],
    ["28 LETA=5:GOSUB900:RETURN", "28:lineno LET:keyword A:var =:op 5:number ::punct GOSUB:control 900:target ::punct RETURN:control"],
    ["29 A=INT(RND(1)*6)+1", "29:lineno A:var =:op INT:function (:plain RND:function (:plain 1:number ):plain *:op 6:number ):plain +:op 1:number"],
    ["150 DEFFNA(X)=X*2", "150:lineno DEF:keyword FNA:fn (:plain X:var ):plain =:op X:var *:op 2:number"],
    ["155 DIMA(10):READA,B", "155:lineno DIM:keyword A:var (:plain 10:number ):plain ::punct READ:keyword A:var ,:punct B:var"],
  ]));

test("names that begin with, or contain, a keyword", () =>
  cases([
    // comments, as in Microsoft BASIC
    ["30 REMARKABLE PROGRAM", "30:lineno REMARKABLE PROGRAM:comment"],
    ["31 REMEMBER TO SAVE", "31:lineno REMEMBER TO SAVE:comment"],
    // but an assignment is an assignment
    ["35 REMAINDER = 5", "35:lineno REMAINDER:var =:op 5:number"],
    ["50 TOTAL = TOTAL + LEN(A$)", "50:lineno TOTAL:var =:op TOTAL:var +:op LEN:function (:plain A$:strvar ):plain"],
    ["42 FOR J = START TO STOP", "42:lineno FOR:control J:var =:op START:var TO:control STOP:var"],
    ["120 PRINTER=1", "120:lineno PRINTER:var =:op 1:number"],
    ["121 ENDING$ = \"X\"", '121:lineno ENDING$:strvar =:op "X":string'],
    ["122 LETTER = 5", "122:lineno LETTER:var =:op 5:number"],
    ["123 IFLAG = 1", "123:lineno IFLAG:var =:op 1:number"],
    ["124 ONCE = 1", "124:lineno ONCE:var =:op 1:number"],
    ["125 FORCE = 1", "125:lineno FORCE:var =:op 1:number"],
    ["126 DATA1 = 2", "126:lineno DATA1:var =:op 2:number"],
    ["130 OTHER = 5", "130:lineno OTHER:var =:op 5:number"],
    ["131 IF OTHER = 5 THEN 10", "131:lineno IF:control OTHER:var =:op 5:number THEN:control 10:target"],
    ["132 INTEREST = SCORE + FORMAT", "132:lineno INTEREST:var =:op SCORE:var +:op FORMAT:var"],
    ["133 PRINT NOTE; ANDY; ORDER", "133:lineno PRINT:keyword NOTE:var ;:punct ANDY:var ;:punct ORDER:var"],
  ]));

// Every sample program: the 1964 manual's, the tutorial's, the classics and
// the 103 listings of 1978, most of which have no spaces.
for (const folder of ["dartmouth", "tutorial", "classic", "games"]) {
  test(`samples/${folder}: every line is recognised`, async () => {
    const files = fs.readdirSync(path.join(samples, folder)).filter((name) => name.endsWith(".bas"));
    assert.ok(files.length > 0);
    let numbered = 0;
    for (const name of files) {
      const source = fs.readFileSync(path.join(samples, folder, name), "latin1");
      const lines = source.split(/\r?\n/);
      const tokens = await tokenize(source);
      const first = new Map();
      for (const token of tokens) if (!first.has(token.line)) first.set(token.line, token);
      lines.forEach((text, line) => {
        if (!/^\s*\d/.test(text)) return;
        numbered += 1;
        assert.equal(first.get(line).scope, "entity.name.label.line-number.basic", `${name}:${line + 1}: ${text}`);
      });
      // no word is left without a scope
      for (const token of tokens) {
        assert.ok(
          token.scope !== "source.basic" || !/[A-Za-z]/.test(token.text),
          `${name}:${token.line + 1}: "${token.text}" has no scope in: ${lines[token.line]}`,
        );
      }
      // these programs use one- and two-character names, so a longer "name"
      // is a keyword the grammar failed to split off
      if (folder === "games") {
        for (const token of tokens) {
          if (!token.scope.startsWith("variable.")) continue;
          const word = token.text.replace(/[$%]$/, "").toUpperCase();
          assert.ok(word.length <= 2 || word === "DELAY", `${name}:${token.line + 1}: "${token.text}" in: ${lines[token.line]}`);
        }
      }
    }
    assert.ok(numbered > 50, `${numbered} numbered lines`);
  });
}
