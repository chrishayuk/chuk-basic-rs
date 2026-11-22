# Independent review

On 22 November 2026 this repository, at commit `10669b1`, was reviewed by
OpenAI Codex CLI (codex-cli 0.166.0): an independent second model, not a
person. It was asked to build and run everything the README describes, to
compare passages of the repository with the scanned pages, and to check
every correction and every historical claim.

It ran the builds and tests (all passed, except the PC-BASIC comparison and
the VS Code extension tests, which could not run without network access),
and checked 20 places where the repository transcribes or quotes the 1964
manual against the scans. 15 matched. The other five were the two damaged
relations that the repository had already said it reads (`=` on line 50 of
example 2, `<=` on line 160 of example 3) and three places where the tests
do not hold the page's printed result: `READ PART` for `REAL PART`, the
space before `=` in `IMAGINARY PART =`, and the column of the second root.
It could not check the 1978 book.

Each finding was then checked again here, against the code, by running
it, and against the scans and the full reprint of the manual
(dartmouth.edu/basicfifty/basicmanual_1964.pdf, which holds Part I only).

## Findings and what was done

**1. The Dartmouth dialect left out documented 1964 behaviour.** Accepted.
The May 1964 manual does not describe `INT` or `RND` (it defers them to a
Part II that the reprint does not include) or how a leading minus binds,
but Kurtz's account, which the repository already cites, does. The dialect
now truncates `INT` toward 0, ignores `RND`'s argument and reads `-X^2` as
`(-X)^2`, in the interpreter and in compiled programs (commit `1e5f591`;
tests `dartmouth_int_rnd_and_leading_minus_follow_kurtz` in
`tests/samples.rs`, the dialect probe in `tests/compiled.rs`, and unit tests
in `src/rt/dialect.rs` and `src/parser/tests.rs`). Chapter 17's "runs here
unchanged" now claims only what three programs show. Taking Kurtz's 2005
account as describing the system of May 1964 is our interpretation, and is
labelled so.

**2. The playground could change numbers already shown after
`RANDOMIZE`.** Accepted; a real defect. A bare `RANDOMIZE` reseeded from
the clock, so the playground's replay at each `INPUT` could differ from the
run before it - and a native run with `--seed` did not repeat either. In a
seeded run it now reseeds from the generator (commit `b473655`; tests
`a_seeded_run_repeats_even_after_a_bare_randomize` in `src/rt/rng.rs` and
"a bare RANDOMIZE does not change what was shown before an INPUT" in
`web/test.mjs`).

**3. "Unmodified" games are not established as the 1978 book.** Accepted.
All 103 files were compared here with the coding-horror repository (commit
`a81c331`): all are byte-identical. But that collection says its listings
were normalised and tweaked for Vintage BASIC, and the printed book was
not checked (the one online scan that was tried asks for a human
verification). The README and `samples/README.md` now say exactly that,
and SPLAT's missing line 540 is attributed to the collection's listing,
not to the book (commit `5477c65`).

**4. The tests reproduce the figures, not the printed transcript.**
Accepted, and one difference fixed. The page's `READ PART` (against the
program's `"REAL PART="`), its spaces before `=`, its columns and its
`TIME:` lines are now listed in the README, `docs/scans/README.md` and
chapter 17, by kind. The columns were measured on the scan: the numbers
stand near columns 0, 15 and 29 and the words at 42, which fits no single
zone width, so the 14-column zones are kept. Looking at this also showed
that our Dartmouth run ended with `Out of DATA`, which the page does not
show; page 15 says that with no data left "the computer stops". With
`--dialect dartmouth` running out of data now ends the program normally
(commit `1e5f591`; `the_1964_quadratic_example_needs_dartmouth_arithmetic`
in `tests/samples.rs`, and `web/test.mjs`).

**5. "Byte-for-byte" against PC-BASIC was more than the checker showed.**
Accepted in part. PC-BASIC 2.0.8 installed with pip, and the comparison
was run. The 22 conformance programs are byte-identical on standard output
once PC-BASIC's CRLF line endings, its 0xFF bytes and its closing `Ok`
prompt are removed, so the claim held for them; but the script's "exact"
verdict ignored trailing spaces, and neither side's error messages or exit
status were compared. "exact" is now strict, with a separate
"trailing-space" verdict, and the README and `tests/golden.rs` say what is
normalised and what is not compared (commit `79f4fbf`). Of the 35 classic
and tutorial samples, 20 are exact, 1 differs only in trailing spaces, 4 in
spacing, 3 in wrapping at 80 columns and 7 otherwise (for example random
numbers, precision, `DEF`...`FNEND`, error handling); "most" is now replaced by
those counts.

**6. The 1964 checks accepted forms the manual does not have.** Accepted.
`IF 1 THEN PRINT 2`, `DEF FNA(X,Y)` and a ten-digit number now get
warnings, citing pp. 11-12, 13 and 5 (commit `5fefc72`; test
`the_1964_lint_checks_the_manuals_statement_forms` in `tests/analysis.rs`).
The README now says these are checks of vocabulary and a few forms, not a
grammar of 1964 BASIC.

**7. Smaller claims.** Accepted.
- Six significant digits: whole numbers were printed in full. That is kept
  (the pages print no whole number longer than one digit) and documented
  as our choice, with a test for `1234567.`.
- Fourteen operations (p. 4) against Kurtz's fifteen statements: now
  stated as a conflict between the sources, in the README, chapter 17 and
  `src/analysis/original.rs`.
- The dataset said the dialect changes "two things": corrected and the
  dataset rebuilt.
- "The first BASIC program in print": withdrawn; it is "the preface's
  program" in `docs/scans/README.md`, chapter 17 and the dataset.

**8. Classification of the corrections.** Accepted as given, with one
strengthening: of the six relations the manual allows, only `=` on line 50
of example 2 reproduces the printed results (checked by running all six),
which supports the reading but does not prove what was printed. The table
"Readings and corrections" in the README now labels every reading by kind:
slip, conflict, interpretation or ours.

**9. Transcription spot-checks.** No discrepancy beyond those in 4 and 8.
The `EXP(X)` in example 3's variant is as printed and is kept.

**10. Build and test results.** Confirmed. After the changes, everything
was run again here: `cargo build --release`; `cargo test --release` and
`cargo test` (all suites pass); `cargo test --target wasm32-wasip1`;
`scripts/wasi-check.sh` (150 programs, none differ); `cargo fmt --check`
and `cargo clippy --all-targets -- -D warnings`;
`python3 tutorial/check_tutorial.py` (232 programs, 0 failed) and with
`--wasm` (221 compiled); `python3 tutorial/build_dataset.py` (113
examples); `web/build.sh` and `node web/test.mjs` (all passed);
`scripts/pcbasic-diff.py tests/conformance/*.bas` (22 exact); and in
`editors/vscode`, `npm install`, `npm test` (16 passed) and
`npm run test:vscode` (passed).

Nothing in the review was disputed outright. Where we accepted only part
of a finding, the reason is given above.
