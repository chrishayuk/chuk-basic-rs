# Changelog

## 0.1.2

- Completion: after `GOTO`, `GOSUB` and `THEN`, the program's lines; otherwise words that begin as typed.
- The language server now sends semantic tokens.
- Run Program no longer prints its own command line in the terminal before the program's output.

## 0.1.1

- The hover on `SQR` names the later BASIC that was tested (GW-BASIC) instead of saying "later BASICs".

## 0.1.0

- Syntax highlighting for line-numbered BASIC (`.bas`), including listings
  typed without spaces.
- Nothing else to install: the interpreter and language server are carried
  as WebAssembly. `chukBasic.path` uses a `chuk-basic` program instead.
- Language server (`chuk-basic --lsp`): errors as you type, go to a line
  from `GOTO`/`GOSUB`/`THEN`, find the lines that jump to a line, hover help,
  an outline of functions and subroutines.
- Run Program runs in a terminal of its own, where `INPUT` is typed.
- Commands: Run Program, Renumber Lines, Switch Dialect, Restart Language
  Server.
- A 1964 Dartmouth setting, which warns about what the first manual's BASIC
  did not have and runs programs with its arithmetic and printing.
