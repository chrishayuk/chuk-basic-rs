# BASIC for VS Code

Line-numbered BASIC - Dartmouth's of 1964, and the Microsoft-style BASICs of
the 1970s that the type-in games were written for. Part of
[chuk-basic-rs](https://github.com/chrishayuk/chuk-basic-rs).

```basic
10 PRINT "QUADRATIC EQUATION"
20 READ A, B, C
30 LET D = B * B - 4 * A * C
40 LET T = SQR(D) / (2 * A)
50 IF D < 0 THEN 200
```

## What it does

**Highlighting**, with no other program needed:

- line numbers, and the line numbers that `GOTO`, `GOSUB`, `THEN`, `ELSE`,
  `RESTORE` and `ON ... GOTO` point at, coloured alike so a jump and its
  target look related;
- keywords, the built-in functions, `FN` functions, string variables,
  numbers, strings, `REM` and `'` comments, `DATA` lines;
- the upwards arrow `↑` that the 1964 manual uses for powers, as well as `^`;
- listings typed without spaces, as most of the 1978 games were:
  `FORI=1TO10:PRINTI:NEXTI`.

**With the `chuk-basic` program installed**, a language server adds:

- **errors as you type**: syntax errors, a `GOTO` to a line that is not
  there, two lines with the same number;
- **line numbers as links**: go to the line a `GOTO 300` means (F12), and
  find every line that jumps to this one (Shift+F12);
- **hover**: a sentence about each keyword and function;
- **an outline** of the functions a program defines and the subroutines it
  calls;
- **Renumber Lines**: renumber in tens and fix every reference;
- **Run Program** (F5, or the ▷ button): run the file in a terminal.

**A 1964 setting.** Set `chukBasic.dialect` to `dartmouth`, or click
`BASIC: later` in the status bar, and the editor warns about anything the
first BASIC manual (Dartmouth, May 1964) did not have - `INPUT`, variables
for words, names longer than a letter and a digit, a missing `LET` - and Run
Program uses 1964's arithmetic and printing (`--dialect dartmouth`).

## Install

The extension is not on the Marketplace. Build the `.vsix` and install it:

```bash
cd editors/vscode
npm install
npx @vscode/vsce package          # writes chuk-basic-0.1.0.vsix
code --install-extension chuk-basic-0.1.0.vsix
```

For everything beyond highlighting, install the interpreter, which is also
the language server:

```bash
cargo install --path .            # in the chuk-basic-rs repository; installs chuk-basic
chuk-basic --help                 # should list --lsp
```

If `chuk-basic` is not on your `PATH`, set `chukBasic.path` to where it is.
Without it you get a single message saying so, and highlighting carries on.

## Settings

| Setting | Default | |
| --- | --- | --- |
| `chukBasic.path` | `chuk-basic` | The program: a name on `PATH` or a full path. Started as `chuk-basic --lsp`. |
| `chukBasic.dialect` | `later` | `later` for the BASICs of the 1970s and after; `dartmouth` for BASIC as the 1964 manual describes it. |

## Commands

| Command | |
| --- | --- |
| BASIC: Run Program | Saves the file and runs it in a terminal called BASIC. F5. |
| BASIC: Renumber Lines | Renumbers in tens and rewrites every `GOTO`, `GOSUB`, `THEN` and `RESTORE`. |
| BASIC: Switch Dialect | Between `later` and `dartmouth`. Also the status-bar item. |
| BASIC: Restart Language Server | After installing or rebuilding `chuk-basic`. |

## `.bas` files and Visual Basic

VS Code's built-in Visual Basic support also claims `.bas`. With this
extension installed, `.bas` files open as BASIC: an installed extension takes
precedence over a built-in one, and the extension also sets
`"files.associations": { "*.bas": "basic" }` as a default. The end-to-end
test checks it in a real VS Code. To get Visual Basic back for a project, add
`"files.associations": { "*.bas": "vb" }` to its settings.

## What the highlighting gets wrong

The interpreter reads a line the ordinary way first and, if that fails,
again with the words run together. A grammar cannot try twice, so it accepts
run-together keywords only where that is unlikely to cut a name in half:

- A statement keyword stuck to what follows is recognised (`PRINTA`,
  `NEXTI`, `GOTO100`, `DIMA(10)`), unless an `=` follows the word, which
  makes it a name (`PRINTER=1`). So `PRINTA=B`, printing a comparison with no
  spaces, is coloured as an assignment to `PRINTA`, and `PRINTER(3)=5`,
  an assignment to an array, as a `PRINT`.
- `IF` and `ON` are keywords only if the line goes on to `THEN` or `GOTO`.
- `FORI=1TO10` is recognised when the loop variable is a letter, or a letter
  and a digit. In such a line `TO` and `STEP` are found anywhere, so a name
  containing them would be split.
- `THEN` stuck to the name before it is recognised when a line number or a
  statement keyword follows (`IFA=BTHEN100`, `IFA=BTHENPRINT`) but not before
  an assignment (`IFA=BTHENC=1`), so that `OTHER = 5` stays one name.
- `AND` and `OR` with no spaces are recognised after a digit, a bracket or a
  quote (`IFA=1ANDB=2`), not between two names (`IFAANDB`).
- `GOTO` and `GOSUB` are recognised anywhere, so a name containing them
  would be split.
- A function name stuck to a keyword is recognised after `IF`, `TO`, `OR`,
  `AND`, `NOT`, `THEN`, `ELSE`, `STEP` and `PRINT` (`IFINT(X)=1`), not
  elsewhere.

Measured on the sample programs: in the 103 listings of 1978 (11,600 lines,
mostly without spaces) every line number is recognised, no word is left
without a colour, and only one name longer than two characters is found - a
real one. `npm test` checks this.

Syntax errors are not marked by the grammar; the language server does that.

## Developing

```bash
npm install
npm run grammar       # scripts/build-grammar.js writes syntaxes/basic.tmLanguage.json
npm test              # grammar tests, in node, no VS Code needed
npm run test:vscode   # opens a real VS Code with its own empty profile and checks the lot
```

The grammar is generated: edit `scripts/build-grammar.js`, not the JSON.
`test:vscode` uses the VS Code installed on the machine if there is one, and
the `chuk-basic` in `target/release`; `CHUK_BASIC=none` runs it without a
server. Press F5 in VS Code with this folder open to try the extension in a
development window.
