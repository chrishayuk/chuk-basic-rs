// Writes syntaxes/basic.tmLanguage.json.
//
//   node scripts/build-grammar.js          write the grammar
//   node scripts/build-grammar.js --check  fail if the file on disk is stale
//
// The grammar is generated because its patterns share long pieces (where a
// statement can start, the list of functions) and are far easier to read
// unescaped. Edit this file, not the JSON.
//
// Old listings are often typed with no spaces (`FORI=1TO10:PRINTI:NEXTI`).
// The interpreter reads a line the spaced way first and falls back to
// splitting words apart; a TextMate grammar cannot try twice, so the rules
// below accept run-together keywords only where that is unlikely to cut a
// name in half. The README lists what that leaves out.

const fs = require("node:fs");
const path = require("node:path");

const r = String.raw;

// Where a statement can start: after the line number (the \G of the line
// rule), after a colon, or after THEN or ELSE.
const STMT = r`(?:\G|(?<=:)|(?<=then|else))[ \t]*`;
// Not the start of an assignment to a variable whose name begins with the
// keyword: PRINTER=1, ENDING$="X", DATA1=2.
const NOT_ASSIGNED = r`(?![A-Za-z0-9]*[$%]?[ \t]*=)`;
const JUMP = r`GOTO|GOSUB|GO[ \t]*TO|GO[ \t]*SUB`;
const FUNCTIONS = r`LEFT\$|RIGHT\$|MID\$|CHR\$|STR\$|SPACE\$|STRING\$|INSTR|SIN|COS|TAN|ATN|EXP|ABS|LOG|SQR|INT|FIX|SGN|RND|VAL|LEN|ASC|TAB|SPC`;
// A statement keyword that may directly follow a run-together THEN.
const AFTER_THEN = r`PRINT|INPUT|GOTO|GOSUB|GO[ \t]*TO|LET|IF|FOR|NEXT|READ|RETURN|END|STOP|DIM|ON`;
const NUMBER = r`(?:\d+\.?\d*|\.\d+)(?:E[-+]?\d+)?`;
const STMT_END = r`(?=:)|$`;

const i = (pattern) => `(?i)${pattern}`;

const string = { name: "string.quoted.double.basic", match: r`"[^"]*"?` };
const number = { name: "constant.numeric.basic", match: i(r`(?<![A-Za-z0-9.])${NUMBER}`) };

const grammar = {
  $schema: "https://raw.githubusercontent.com/martinring/tmlanguage/master/tmlanguage.json",
  name: "BASIC",
  scopeName: "source.basic",
  patterns: [{ include: "#line" }],
  repository: {
    line: {
      begin: r`^[ \t]*(\d+)?[ \t]*`,
      beginCaptures: { 1: { name: "entity.name.label.line-number.basic" } },
      end: r`$`,
      patterns: [
        string,
        { include: "#rem" },
        { name: "comment.line.apostrophe.basic", match: r`'.*$` },
        { include: "#data" },
        { include: "#for" },
        { include: "#mat" },
        { include: "#jump" },
        { include: "#statement" },
        { include: "#expression" },
      ],
    },

    // REM, REMARK, REMEMBER TO... but not REMAINDER = 5
    rem: {
      match: i(r`${STMT}(REM(?![A-Za-z0-9]*[$%]?[ \t]*=).*)$`),
      captures: { 1: { name: "comment.line.rem.basic" } },
    },

    // DATA runs to the end of the line or an unquoted colon; unquoted words
    // in it are text, not names.
    data: {
      begin: i(r`${STMT}(DATA)${NOT_ASSIGNED}`),
      beginCaptures: { 1: { name: "keyword.other.data.basic" } },
      end: STMT_END,
      contentName: "meta.data.basic",
      patterns: [
        string,
        { name: "constant.numeric.basic", match: i(r`(?<![A-Za-z0-9.])[-+]?${NUMBER}(?![A-Za-z0-9.])`) },
        { name: "punctuation.separator.basic", match: "," },
      ],
    },

    for: {
      patterns: [
        {
          // FORI=1TO10 - no spaces, so TO and STEP are found anywhere
          begin: i(r`${STMT}(FOR)(?=[A-Za-z][0-9]?[ \t]*=)`),
          beginCaptures: { 1: { name: "keyword.control.basic" } },
          end: STMT_END,
          patterns: [
            string,
            { name: "keyword.control.basic", match: i(r`TO|STEP`) },
            { name: "constant.numeric.basic", match: i(r`(?:\d+\.?\d*|\.\d+)(?:E[-+]?\d+(?![A-Za-z]))?`) },
            { name: "support.function.basic", match: i(r`(?:${FUNCTIONS})(?=[ \t]*\()`) },
            { name: "entity.name.function.basic", match: i(r`FN[A-Za-z][0-9]?(?=[ \t]*\()`) },
            { name: "variable.other.basic", match: i(r`[A-Za-z][0-9]?[$%]?`) },
            { include: "#operators" },
          ],
        },
        {
          begin: i(r`${STMT}(FOR)(?![A-Za-z0-9$%])`),
          beginCaptures: { 1: { name: "keyword.control.basic" } },
          end: STMT_END,
          patterns: [
            string,
            { name: "keyword.control.basic", match: i(r`(?<![A-Za-z])(?:TO|STEP)(?![A-Za-z0-9$%])`) },
            // FOR I=1TO10 and FOR I=1TON
            { name: "keyword.control.basic", match: i(r`(?<=[0-9)])(?:TO|STEP)`) },
            { include: "#expression" },
          ],
        },
      ],
    },

    mat: {
      begin: i(r`${STMT}(MAT)(?![A-Za-z0-9$%])`),
      beginCaptures: { 1: { name: "keyword.other.basic" } },
      end: STMT_END,
      patterns: [
        string,
        { name: "keyword.other.basic", match: i(r`(?<![A-Za-z])(?:READ|PRINT|INPUT)(?![A-Za-z0-9$%])`) },
        { name: "support.function.matrix.basic", match: i(r`(?<![A-Za-z])(?:ZER|CON|IDN|TRN|INV)(?![A-Za-z0-9$%])`) },
        { include: "#expression" },
      ],
    },

    // GOTO 100, GO TO 100, ON X GOSUB 100,200,300, THEN 100, ELSE 100,
    // RESTORE 100: the numbers are line numbers, not quantities.
    jump: {
      patterns: [
        {
          begin: i(r`(${JUMP})[ \t]*(?=\d)`),
          beginCaptures: { 1: { name: "keyword.control.basic" } },
          end: r`(?![\d \t,])`,
          patterns: [
            { name: "entity.name.label.line-number.reference.basic", match: r`\d+` },
            { name: "punctuation.separator.basic", match: "," },
          ],
        },
        { name: "keyword.control.basic", match: i(JUMP) },
        {
          // ...BTHEN100 and ...BTHENPRINT: a THEN with no space before it
          match: i(r`(THEN)(?:[ \t]*(\d+)|(?=[ \t]*(?:${AFTER_THEN})))`),
          captures: {
            1: { name: "keyword.control.basic" },
            2: { name: "entity.name.label.line-number.reference.basic" },
          },
        },
        {
          // IFA=1THENX=2: after a digit, a bracket or a quote it cannot be part of a name
          match: i(r`(?<=[0-9)"$])(THEN)(?:[ \t]*(\d+))?`),
          captures: {
            1: { name: "keyword.control.basic" },
            2: { name: "entity.name.label.line-number.reference.basic" },
          },
        },
        {
          match: i(r`(?<![A-Za-z])(THEN|ELSE)(?![A-Za-z0-9$%])(?:[ \t]*(\d+))?`),
          captures: {
            1: { name: "keyword.control.basic" },
            2: { name: "entity.name.label.line-number.reference.basic" },
          },
        },
        {
          match: i(r`${STMT}(RESTORE)${NOT_ASSIGNED}(?:[ \t]*(\d+))?`),
          captures: {
            1: { name: "keyword.other.basic" },
            2: { name: "entity.name.label.line-number.reference.basic" },
          },
        },
      ],
    },

    statement: {
      patterns: [
        {
          // IFA=BTHEN100 - only an IF if the line goes on to say where to go
          match: i(r`${STMT}(IF)(?=.*(?:THEN|GOTO|GO[ \t]*TO))`),
          captures: { 1: { name: "keyword.control.basic" } },
        },
        {
          match: i(r`${STMT}(ON)(?=.*(?:${JUMP}))`),
          captures: { 1: { name: "keyword.control.basic" } },
        },
        {
          match: i(r`${STMT}(NEXT|RETURN|END|STOP|WHILE|WEND)${NOT_ASSIGNED}`),
          captures: { 1: { name: "keyword.control.basic" } },
        },
        {
          // LETA=5, but not LETTER=5
          match: i(r`${STMT}(LET)(?:(?![A-Za-z0-9$%])|(?=[A-Za-z][0-9]?[$%]?[ \t]*[=(]))`),
          captures: { 1: { name: "keyword.other.basic" } },
        },
        {
          match: i(r`${STMT}(LINE[ \t]*INPUT)${NOT_ASSIGNED}`),
          captures: { 1: { name: "keyword.other.basic" } },
        },
        {
          match: i(r`${STMT}(PRINT)[ \t]*(USING)(?![A-Za-z0-9$%])`),
          captures: { 1: { name: "keyword.other.basic" }, 2: { name: "keyword.other.basic" } },
        },
        {
          match: i(r`${STMT}(DEF)(?:(?![A-Za-z0-9$%])|(?=FN))`),
          captures: { 1: { name: "keyword.other.basic" } },
        },
        {
          match: i(r`${STMT}(PRINT|INPUT|READ|DIM|RANDOMIZE|FNEND)${NOT_ASSIGNED}`),
          captures: { 1: { name: "keyword.other.basic" } },
        },
        // PRINT can be written ?
        { match: i(r`${STMT}(\?)`), captures: { 1: { name: "keyword.other.basic" } } },
      ],
    },

    expression: {
      patterns: [
        string,
        { name: "keyword.operator.logical.basic", match: i(r`(?<![A-Za-z])(?:AND|OR|NOT)(?![A-Za-z0-9$%])`) },
        // IFA=1ANDB=2THEN...
        { name: "keyword.operator.logical.basic", match: i(r`(?<=[0-9)"$])(?:AND|OR)(?=[A-Za-z(]|[ \t]*NOT)`) },
        { name: "keyword.other.basic", match: i(r`(?<![A-Za-z])USING(?![A-Za-z0-9$%])`) },
        {
          name: "support.function.basic",
          match: i(r`(?:(?<![A-Za-z])|(?<=IF|OR|TO|AND|NOT|THEN|ELSE|STEP|PRINT))(?:${FUNCTIONS})(?:(?=[ \t]*\()|(?![A-Za-z0-9$%(]))`),
        },
        {
          name: "entity.name.function.basic",
          match: i(r`(?:(?<![A-Za-z])|(?<=DEF))FN[A-Za-z][A-Za-z0-9]*[$%]?`),
        },
        number,
        {
          // a name stops short of a run-together THEN or GOTO: BTHEN100
          name: "variable.other.string.basic",
          match: i(r`[A-Za-z](?:(?!THEN(?:[ \t]*\d|[ \t]*(?:${AFTER_THEN}))|GOTO|GOSUB)[A-Za-z0-9])*\$`),
        },
        {
          name: "variable.other.basic",
          match: i(r`[A-Za-z](?:(?!THEN(?:[ \t]*\d|[ \t]*(?:${AFTER_THEN}))|GOTO|GOSUB)[A-Za-z0-9])*%?`),
        },
        { include: "#operators" },
      ],
    },

    operators: {
      patterns: [
        { name: "keyword.operator.comparison.basic", match: r`<>|<=|>=|=<|=>|[<>]` },
        { name: "keyword.operator.assignment.basic", match: r`=` },
        { name: "keyword.operator.arithmetic.basic", match: r`[-+*/^↑]` },
        { name: "punctuation.separator.basic", match: r`[,;]` },
        { name: "punctuation.terminator.statement.basic", match: r`:` },
      ],
    },
  },
};

const file = path.join(__dirname, "..", "syntaxes", "basic.tmLanguage.json");
const text = JSON.stringify(grammar, null, 2) + "\n";
if (process.argv.includes("--check")) {
  if (!fs.existsSync(file) || fs.readFileSync(file, "utf8") !== text) {
    console.error("syntaxes/basic.tmLanguage.json is stale: run `npm run grammar`");
    process.exit(1);
  }
} else {
  fs.writeFileSync(file, text);
  console.log(`wrote ${path.relative(process.cwd(), file)}`);
}
