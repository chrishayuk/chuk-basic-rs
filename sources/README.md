# Sources

The documents are not kept in the repository (see `.gitignore`). Fetch
them into the directories named to check a reading. The PDF is a scan
with no text layer.

| Directory | Document | From | Used for |
| --- | --- | --- | --- |
| `dartmouth-1964/` | Dartmouth College Computation Center, *BASIC: Beginners' All-purpose Symbolic Instruction Code*, Instruction Manual, May 1964, First Draft (`basicmanual_1964.pdf`, a reprint, 14 sheets, pp. 1–23). Part I only | https://www.dartmouth.edu/basicfifty/basicmanual_1964.pdf | The `--dialect dartmouth` rules, the 1964 lint, and the page references in `samples/classic/tpk.bas`. The page images in `docs/scans/dartmouth-1964/` are from it |
| `dartmouth-oct1964/` | Dartmouth College Computation Center, *BASIC: A Manual for BASIC, the elementary algebraic language designed for use with the Dartmouth Time Sharing System*, 1 October 1964 (`BASIC_Oct64.pdf`, a scan, 64 pages, 2,146,200 bytes; the second edition) | another scan: https://alandix.com/wp-content/uploads/2021/07/BASIC_Oct64.pdf | The dialect's rules for `PRINT`, `INT`, `RND` and `LOG`, and `samples/dartmouth-oct64`. The page images in `docs/scans/dartmouth-oct1964/` are from it |

## Not found

- **Part II of the May 1964 manual.** Page 4: "There are only 13 other
  operations. Nine of these will be discussed on the following pages.
  Four more operations, available for the more experienced programmer,
  are discussed in Part II." Page 5 also leaves vectors to Part II.
  Without it, it can't be checked whether the first system had arrays
  with subscript 0, `DIM`, or `REM`. Kurtz's later list
  (dartmouth.edu/basicfifty/commands.html) has fifteen statements with
  `END`, one more than the manual's fourteen. Part I covers nine of them
  (`LET PRINT READ DATA GOTO IF FOR NEXT DEF`), so five (`GOSUB RETURN
  DIM REM STOP`) compete for Part II's four. One of those five was not in
  the May 1964 system, and it can't be identified from the reprint.

The October 1964 manual narrows this. Its Appendix C, "Summary of the 15
BASIC Statements" (pp. 56-57), lists `LET READ DATA PRINT GOTO IF-THEN FOR
NEXT END STOP DEF GOSUB RETURN DIM REM`: Kurtz's fifteen. So by October all
five were in the system, and one was added between May and October. The
October manual does not say which.
