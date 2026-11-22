//! End-to-end tests of language behaviour: source in, printed output out.

mod common;

use chuk_basic::ErrorKind;
use common::{run, run_err, run_opts, run_with, Options};

// ---- PRINT ------------------------------------------------------------------

#[test]
fn print_numbers_and_strings() {
    assert_eq!(run("10 PRINT \"HELLO\""), "HELLO\n");
    assert_eq!(run("10 PRINT 5"), " 5 \n");
    assert_eq!(run("10 PRINT -5"), "-5 \n");
    assert_eq!(run("10 PRINT 1/4; .5; 1E10"), " .25  .5  10000000000 \n");
    assert_eq!(run("10 PRINT"), "\n");
    assert_eq!(run("10 ? \"A\""), "A\n");
}

#[test]
fn print_separators() {
    assert_eq!(run("10 PRINT \"A\";\"B\""), "AB\n");
    assert_eq!(run("10 PRINT \"A\",\"B\""), "A             B\n");
    assert_eq!(run("10 PRINT \"A\";\n20 PRINT \"B\""), "AB\n");
    assert_eq!(run("10 PRINT \"X=\"5"), "X= 5 \n");
}

#[test]
fn print_tab_and_spc() {
    assert_eq!(run("10 PRINT TAB(5);\"A\""), "     A\n");
    assert_eq!(run("10 PRINT \"AB\";TAB(5);\"C\""), "AB   C\n");
    assert_eq!(run("10 PRINT \"A\";SPC(3);\"B\""), "A   B\n");
    // TAB never moves backwards
    assert_eq!(run("10 PRINT \"ABCDEF\";TAB(2);\"G\""), "ABCDEFG\n");
}

// ---- expressions ------------------------------------------------------------

#[test]
fn arithmetic_and_precedence() {
    assert_eq!(run("10 PRINT 1+2*3"), " 7 \n");
    assert_eq!(run("10 PRINT (1+2)*3"), " 9 \n");
    assert_eq!(run("10 PRINT 2^3^2"), " 64 \n");
    assert_eq!(run("10 PRINT -2^2"), "-4 \n");
    assert_eq!(run("10 PRINT 2^-1"), " .5 \n");
    assert_eq!(run("10 PRINT 7/2"), " 3.5 \n");
    assert_eq!(run("10 PRINT 10-4-3"), " 3 \n");
}

#[test]
fn comparisons_and_logic() {
    assert_eq!(run("10 PRINT 1<2; 2<1; 1=1; 1<>1"), "-1  0 -1  0 \n");
    assert_eq!(
        run("10 PRINT 1<2 AND 2<3; 1>2 OR 2>3; NOT 0"),
        "-1  0 -1 \n"
    );
    assert_eq!(run("10 PRINT 6 AND 3; 6 OR 3"), " 2  7 \n");
    assert_eq!(run("10 PRINT \"A\"<\"B\"; \"A\"=\"A\""), "-1 -1 \n");
}

#[test]
fn strings() {
    assert_eq!(
        run("10 A$=\"HELLO\"\n20 PRINT A$+\", WORLD\""),
        "HELLO, WORLD\n"
    );
    assert_eq!(
        run("10 PRINT LEFT$(\"HELLO\",2);RIGHT$(\"HELLO\",2);MID$(\"HELLO\",2,3)"),
        "HELOELL\n"
    );
    assert_eq!(run("10 PRINT MID$(\"HELLO\",4)"), "LO\n");
    assert_eq!(
        run("10 PRINT LEN(\"HELLO\"); ASC(\"A\"); CHR$(66)"),
        " 5  65 B\n"
    );
    assert_eq!(
        run("10 PRINT STR$(12);STR$(-3);VAL(\"4.5\")+1"),
        " 12-3 5.5 \n"
    );
}

#[test]
fn numeric_functions() {
    assert_eq!(
        run("10 PRINT INT(2.7); INT(-2.7); FIX(-2.7); ABS(-3); SGN(-9); SGN(0)"),
        " 2 -3 -2  3 -1  0 \n"
    );
    assert_eq!(
        run("10 PRINT SQR(16); EXP(0); LOG(1); SIN(0); COS(0); ATN(0); TAN(0)"),
        " 4  1  0  0  1  0  0 \n"
    );
}

#[test]
fn rnd_is_in_range_and_repeatable() {
    let source = "10 FOR I=1 TO 200\n20 R=RND(1)\n30 IF R<0 OR R>=1 THEN PRINT \"BAD\"\n40 NEXT\n50 PRINT RND(1)=RND(0)";
    assert_eq!(run(source), "-1 \n");
    let dice = "10 FOR I=1 TO 5\n20 PRINT INT(6*RND(1))+1;\n30 NEXT";
    assert_eq!(run(dice), run(dice)); // same seed, same rolls
}

// ---- variables and arrays ---------------------------------------------------

#[test]
fn variables_default_and_are_case_insensitive() {
    assert_eq!(run("10 PRINT X; \"[\"; A$; \"]\""), " 0 []\n");
    assert_eq!(run("10 let total = 3\n20 PRINT TOTAL"), " 3 \n");
}

#[test]
fn arrays() {
    assert_eq!(run("10 A(3)=7\n20 PRINT A(3); A(10)"), " 7  0 \n");
    assert_eq!(
        run("10 DIM B(2,3), N$(5)\n20 B(2,3)=9: N$(5)=\"X\"\n30 PRINT B(2,3); N$(5)"),
        " 9 X\n"
    );
    assert_eq!(run("10 X=5\n20 X(1)=6\n30 PRINT X; X(1)"), " 5  6 \n");
    assert!(run_err("10 A(11)=1").message.contains("Subscript"));
    assert!(run_err("10 DIM A(5)\n20 DIM A(5)")
        .message
        .contains("already dimensioned"));
}

// ---- control flow -----------------------------------------------------------

#[test]
fn goto_and_if() {
    assert_eq!(
        run("10 GOTO 30\n20 PRINT \"NO\"\n30 PRINT \"YES\""),
        "YES\n"
    );
    assert_eq!(
        run("10 GO TO 30\n20 PRINT \"NO\"\n30 PRINT \"YES\""),
        "YES\n"
    );
    assert_eq!(
        run("10 IF 1 THEN 30\n20 PRINT \"NO\"\n30 PRINT \"YES\""),
        "YES\n"
    );
    assert_eq!(run("10 IF 0 THEN PRINT \"A\" ELSE PRINT \"B\""), "B\n");
    assert_eq!(run("10 IF 1 THEN PRINT \"A\" ELSE PRINT \"B\""), "A\n");
    assert_eq!(
        run("10 IF 0 THEN 99 ELSE 30\n20 PRINT \"NO\"\n30 PRINT \"YES\""),
        "YES\n"
    );
    assert_eq!(
        run("10 IF 1 GOTO 30\n20 PRINT \"NO\"\n30 PRINT \"YES\""),
        "YES\n"
    );
}

#[test]
fn a_false_if_skips_the_rest_of_the_line() {
    assert_eq!(
        run("10 IF 0 THEN PRINT \"A\": PRINT \"B\"\n20 PRINT \"C\""),
        "C\n"
    );
    assert_eq!(
        run("10 IF 1 THEN PRINT \"A\": PRINT \"B\"\n20 PRINT \"C\""),
        "A\nB\nC\n"
    );
    assert_eq!(run("10 X=1: IF X=1 THEN X=2: PRINT X"), " 2 \n");
}

#[test]
fn for_loops() {
    assert_eq!(run("10 FOR I=1 TO 3\n20 PRINT I;\n30 NEXT I"), " 1  2  3 ");
    assert_eq!(run("10 FOR I=3 TO 1 STEP -1: PRINT I;: NEXT"), " 3  2  1 ");
    assert_eq!(run("10 FOR I=0 TO 1 STEP .5: PRINT I;: NEXT"), " 0  .5  1 ");
    // the loop variable ends one step past the limit
    assert_eq!(run("10 FOR I=1 TO 3: NEXT\n20 PRINT I"), " 4 \n");
    // an empty range runs zero times
    assert_eq!(
        run("10 FOR I=5 TO 1: PRINT \"NO\": NEXT\n20 PRINT \"DONE\""),
        "DONE\n"
    );
}

#[test]
fn nested_for_loops() {
    assert_eq!(
        run("10 FOR I=1 TO 2: FOR J=1 TO 2: PRINT I*10+J;: NEXT J: NEXT I"),
        " 11  12  21  22 "
    );
    assert_eq!(
        run("10 FOR I=1 TO 2: FOR J=1 TO 2: PRINT I*10+J;: NEXT J,I"),
        " 11  12  21  22 "
    );
    // skipping an empty outer loop skips the inner one too
    assert_eq!(
        run("10 FOR I=1 TO 0: FOR J=1 TO 2: PRINT \"NO\": NEXT J: NEXT I\n20 PRINT \"OK\""),
        "OK\n"
    );
    assert!(run_err("10 NEXT").message.contains("NEXT without FOR"));
}

#[test]
fn gosub_and_return() {
    let source = "10 GOSUB 100\n20 GOSUB 100\n30 END\n100 N=N+1: PRINT N;\n110 RETURN";
    assert_eq!(run(source), " 1  2 ");
    assert!(run_err("10 RETURN")
        .message
        .contains("RETURN without GOSUB"));
    // RETURN from inside a loop abandons the loop
    let source = "10 GOSUB 100\n20 PRINT \"BACK\"\n30 END\n100 FOR I=1 TO 5\n110 IF I=2 THEN RETURN\n120 NEXT";
    assert_eq!(run(source), "BACK\n");
}

#[test]
fn on_goto_and_gosub() {
    let source = "10 FOR X=0 TO 3\n20 ON X GOTO 100,200\n30 PRINT \"NONE\";\n40 GOTO 300\n\
                  100 PRINT \"ONE\";: GOTO 300\n200 PRINT \"TWO\";\n300 NEXT";
    assert_eq!(run(source), "NONEONETWONONE");
    assert_eq!(
        run("10 ON 2 GOSUB 100,200\n20 END\n100 PRINT \"A\": RETURN\n200 PRINT \"B\": RETURN"),
        "B\n"
    );
}

#[test]
fn end_and_stop() {
    assert_eq!(run("10 PRINT \"A\"\n20 END\n30 PRINT \"B\""), "A\n");
    assert_eq!(run("10 PRINT \"A\"\n20 STOP\n30 PRINT \"B\""), "A\n");
}

#[test]
fn lines_run_in_line_number_order_and_may_be_unnumbered() {
    assert_eq!(run("20 PRINT \"B\"\n10 PRINT \"A\""), "A\nB\n");
    assert_eq!(
        run("PRINT \"A\"\nGOTO 100\nPRINT \"NO\"\n100 PRINT \"B\""),
        "A\nB\n"
    );
}

#[test]
fn rem_is_ignored() {
    assert_eq!(run("10 REM PRINT \"NO\"\n20 PRINT \"A\" ' trailing comment\n30 PRINT \"B\": REM : PRINT \"NO\""), "A\nB\n");
}

// ---- functions --------------------------------------------------------------

#[test]
fn single_line_functions() {
    assert_eq!(run("10 DEF FNS(X)=X*X\n20 PRINT FNS(4)"), " 16 \n");
    assert_eq!(
        run("10 DEF FNA(X,Y)=X+Y+Z\n20 Z=100\n30 PRINT FNA(1,2)"),
        " 103 \n"
    );
    assert_eq!(run("10 DEF FN D(X)=X*2\n20 PRINT FN D(4)"), " 8 \n");
    // the parameter does not clobber a global of the same name
    assert_eq!(
        run("10 X=7\n20 DEF FNS(X)=X*X\n30 PRINT FNS(2); X"),
        " 4  7 \n"
    );
    assert!(run_err("10 PRINT FNQ(1)")
        .message
        .contains("Undefined function"));
}

#[test]
fn multi_line_functions() {
    let source =
        "10 DEF FNM(A,B)\n20 FNM=A\n30 IF B>A THEN FNM=B\n40 FNEND\n50 PRINT FNM(3,9); FNM(8,2)";
    assert_eq!(run(source), " 9  8 \n");
    let factorial =
        "10 DEF FNF(N)\n20 IF N<=1 THEN FNF=1 ELSE FNF=N*FNF(N-1)\n30 FNEND\n40 PRINT FNF(5)";
    assert_eq!(run(factorial), " 120 \n");
    assert_eq!(run_err("10 DEF FNA(X)\n20 FNA=X").kind, ErrorKind::Syntax);
    assert!(
        run_err("10 DEF FNR(N)\n20 FNR=FNR(N)\n30 FNEND\n40 PRINT FNR(1)")
            .message
            .contains("nested too deeply")
    );
}

// ---- READ / DATA ------------------------------------------------------------

#[test]
fn read_data_restore() {
    assert_eq!(
        run("10 READ A,B$,C$\n20 PRINT A;B$;C$\n30 DATA 1.5, HELLO, \"A,B\""),
        " 1.5 HELLOA,B\n"
    );
    assert_eq!(
        run("10 READ A\n20 RESTORE\n30 READ B\n40 PRINT A;B\n50 DATA 7,8"),
        " 7  7 \n"
    );
    assert_eq!(
        run("10 DATA 1\n20 DATA 2\n30 RESTORE 20\n40 READ A\n50 PRINT A"),
        " 2 \n"
    );
    // an empty item is zero, or the empty string
    assert_eq!(
        run("10 READ A,B,C$,D$\n20 PRINT A;B;\"[\";C$;\"]\";D$\n30 DATA 1,,,X"),
        " 1  0 []X\n"
    );
    assert!(run_err("10 READ A").message.contains("Out of DATA"));
    assert!(run_err("10 READ A\n20 DATA X")
        .message
        .contains("Bad number"));
}

// ---- INPUT ------------------------------------------------------------------

#[test]
fn input_numbers_and_strings() {
    assert_eq!(run_with("10 INPUT A\n20 PRINT A*2", "21\n").0, "?  42 \n");
    assert_eq!(
        run_with("10 INPUT \"NAME\";N$\n20 PRINT \"HI \";N$", "Chris\n").0,
        "NAME? HI Chris\n"
    );
    assert_eq!(
        run_with("10 INPUT \"N: \",A\n20 PRINT A", "3\n").0,
        "N:  3 \n"
    );
    assert_eq!(
        run_with("10 INPUT A,B$\n20 PRINT A;B$", "1,two\n").0,
        "?  1 two\n"
    );
    // a lone string variable keeps its commas
    assert_eq!(run_with("10 INPUT A$\n20 PRINT A$", "a, b\n").0, "? a, b\n");
}

#[test]
fn input_reprompts() {
    assert_eq!(
        run_with("10 INPUT A\n20 PRINT A", "abc\n5\n").0,
        "? ?REDO FROM START\n?  5 \n"
    );
    assert_eq!(
        run_with("10 INPUT A,B\n20 PRINT A+B", "1\n2\n").0,
        "? ??  3 \n"
    );
    assert_eq!(
        run_with("10 INPUT A\n20 PRINT A", "1,2\n").0,
        "? ?EXTRA IGNORED\n 1 \n"
    );
    let (_, result) = run_with("10 INPUT A", "");
    assert!(result.unwrap_err().message.contains("Out of input"));
}

// ---- old listings ----------------------------------------------------------

#[test]
fn keywords_glued_to_values() {
    assert_eq!(run("10 FOR I=1TO3:PRINT I;:NEXT"), " 1  2  3 ");
    assert_eq!(run("10 FOR I=1 TO 5STEP2:PRINT I;:NEXT"), " 1  3  5 ");
    assert_eq!(
        run("10 A$=\"Y\":IF A$=\"Y\"THEN30\n20 PRINT \"NO\"\n30 PRINT \"YES\""),
        "YES\n"
    );
    assert_eq!(run("10 X=5:IF X>=1ANDX<9 THEN PRINT \"IN\""), "IN\n");
    assert_eq!(run("10 GOTO30\n20 PRINT \"NO\"\n30 PRINT \"YES\""), "YES\n");
    assert_eq!(
        run("10 IF 1 THEN PRINT \"A\":GOTO30\n20 PRINT \"NO\"\n30 END"),
        "A\n"
    );
}

#[test]
fn names_that_merely_contain_keywords_are_variables() {
    assert_eq!(
        run("10 TOTAL=3:FORT=4:SCORE=5:BAND=6\n20 PRINT TOTAL;FORT;SCORE;BAND"),
        " 3  4  5  6 \n"
    );
    assert_eq!(
        run("10 PRINT \"A\";TAB(3);TOTAL"),
        "A   0 
"
    );
}

#[test]
fn any_word_starting_with_rem_is_a_comment() {
    assert_eq!(
        run("10 REMARKABLE PROGRAM\n20 REMEMBER: PRINT \"NO\"\n30 PRINT \"A\""),
        "A\n"
    );
    // ... unless it is clearly a variable
    assert_eq!(run("10 REMAINDER = 7\n20 PRINT REMAINDER"), " 7 \n");
}

#[test]
fn print_is_forgiving() {
    // a minus after a string starts a new item rather than subtracting
    assert_eq!(
        run("10 C=-5\n20 PRINT \"OWED $\"-C\" EXTRA\""),
        "OWED $ 5  EXTRA\n"
    );
    // TAB and SPC clamp instead of failing
    assert_eq!(run("10 PRINT TAB(-3);\"A\";SPC(-1);\"B\""), "AB\n");
    // the closing quote may be left off at the end of a line
    assert_eq!(run("10 PRINT \"NO END QUOTE"), "NO END QUOTE\n");
}

// ---- later dialects ---------------------------------------------------------

#[test]
fn while_and_wend() {
    assert_eq!(
        run("10 I=1\n20 WHILE I<4\n30 PRINT I;\n40 I=I+1\n50 WEND"),
        " 1  2  3 "
    );
    assert_eq!(
        run("10 WHILE 0: PRINT \"NO\": WEND\n20 PRINT \"OK\""),
        "OK\n"
    );
    assert_eq!(
        run_err("10 WEND").to_string(),
        "Syntax error in line 10: WEND without WHILE"
    );
    assert_eq!(run_err("10 WHILE 1").message, "WHILE without WEND");
}

#[test]
fn print_using() {
    assert_eq!(run("10 PRINT USING \"###.##\";3.14159"), "  3.14\n");
    assert_eq!(run("10 PRINT USING \"$$#,###.##\";1234.5"), " $1,234.50\n");
    assert_eq!(
        run("10 PRINT USING \"& IS ## YEARS\";\"ADA\";36"),
        "ADA IS 36 YEARS\n"
    );
    assert_eq!(run("10 PRINT USING \"##\";123"), "%123\n");
    // a half rounds up, as in GW-BASIC (checked with PC-BASIC)
    assert_eq!(run("10 PRINT USING \"###.##\";.125"), "  0.13\n");
    assert_eq!(run("10 PRINT USING \"###.#\";.25"), "  0.3\n");
    assert_eq!(run("10 PRINT USING \"##.##\";2.675"), " 2.68\n");
    assert_eq!(run("10 PRINT USING \"##.##\";1.004;9.999"), " 1.0010.00\n");
    assert_eq!(
        run("10 PRINT USING \"## \";1;2;3;\n20 PRINT \"|\""),
        " 1  2  3 |\n"
    );
    assert!(run_err("10 PRINT USING \"###\";\"A\"")
        .message
        .contains("Type mismatch"));
    assert!(run_err("10 PRINT USING \"ABC\";1")
        .message
        .contains("Illegal function call"));
}

#[test]
fn line_input_takes_the_whole_line() {
    assert_eq!(
        run_with("10 LINE INPUT \"> \";A$\n20 PRINT A$", "a, \"b\"  c\n").0,
        "> a, \"b\"  c\n"
    );
    assert!(run_err("10 LINE INPUT A")
        .message
        .contains("string variable"));
}

#[test]
fn source_without_spaces() {
    assert_eq!(run("10 FORI=1TO3:PRINTI;:NEXTI"), " 1  2  3 ");
    assert_eq!(
        run("10 A=2:IFA>1THENPRINT\"BIG\"ELSEPRINT\"SMALL\""),
        "BIG\n"
    );
    assert_eq!(run("10 GOSUB100:END\n100 PRINT\"SUB\":RETURN"), "SUB\n");
    // each line is read on its own: spaced lines keep their long names
    assert_eq!(
        run("10 TOTAL = 5\n20 FORI=1TO2:T=T+I:NEXTI\n30 PRINT TOTAL; T"),
        " 5  3 \n"
    );
    // the error reported for a hopeless line is the ordinary one
    assert_eq!(
        run_err("10 FOO BAR").to_string(),
        "Syntax error in line 10: Expected '=', found 'BAR'"
    );
}

#[test]
fn matrices() {
    let source = "10 DIM A(2,2)\n20 MAT READ A\n30 DATA 1,2,3,4\n40 MAT B=TRN(A)\n50 MAT C=A*B\n60 MAT PRINT C;";
    assert_eq!(run(source), " 5  11 \n 11  25 \n\n");
    assert_eq!(
        run("10 MAT I=IDN(2,2)\n20 PRINT I(1,1);I(1,2);I(0,0)"),
        " 1  0  0 \n"
    );
    assert!(run_err("10 DIM A(2,2)\n20 MAT B=INV(A)")
        .message
        .contains("singular"));
    assert!(run_err("10 DIM A(2,2),B(3,3)\n20 MAT C=A+B")
        .message
        .contains("do not match"));
    assert!(run_err("10 MAT B=A")
        .message
        .contains("not been dimensioned"));
}

// ---- embedding --------------------------------------------------------------

#[test]
fn step_limit_stops_a_runaway_program() {
    let options = Options {
        step_limit: Some(1000),
        ..Options::default()
    };
    let (output, result) = run_opts("10 PRINT \"A\";\n20 GOTO 10", &options);
    let err = result.unwrap_err();
    assert_eq!(err.message, "Step limit exceeded");
    assert!(output.starts_with("AAAA") && output.len() < 1000);
    // a limit that is not reached changes nothing
    assert!(run_opts("10 FOR I=1 TO 10:NEXT", &options).1.is_ok());
}

#[test]
fn input_can_be_echoed() {
    let options = Options {
        input: "ADA\n",
        echo: true,
        ..Options::default()
    };
    let source = "10 INPUT \"NAME\";N$\n20 PRINT TAB(2);N$";
    assert_eq!(run_opts(source, &options).0, "NAME? ADA\n  ADA\n");
}

#[test]
fn output_errors_are_reported_not_panicked() {
    struct Full;
    impl std::io::Write for Full {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::ErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let err = chuk_basic::run("10 PRINT 1", &mut std::io::empty(), &mut Full, None).unwrap_err();
    assert_eq!(err.kind, ErrorKind::Io(std::io::ErrorKind::BrokenPipe));
}

// ---- errors -----------------------------------------------------------------

#[test]
fn runtime_errors_name_the_line() {
    let err = run_err("10 X=1\n20 PRINT X/0");
    assert_eq!((err.kind, err.line), (ErrorKind::Runtime, Some(20)));
    assert_eq!(
        err.to_string(),
        "Runtime error in line 20: Division by zero"
    );
    assert!(run_err("10 GOTO 99").message.contains("Undefined line 99"));
    assert!(run_err("10 A$=5").message.contains("Type mismatch"));
    assert!(run_err("10 PRINT \"A\"+1")
        .message
        .contains("Type mismatch"));
    assert!(run_err("10 PRINT SQR(-1)")
        .message
        .contains("Illegal function call"));
}

#[test]
fn syntax_errors_are_reported_before_anything_runs() {
    let (output, result) = run_with("10 PRINT \"A\"\n20 PRINT )", "");
    let err = result.unwrap_err();
    assert_eq!((err.kind, err.line), (ErrorKind::Syntax, Some(20)));
    assert_eq!(output, "");
}

/// GW-BASIC stops with "Illegal function call" here (checked with PC-BASIC).
#[test]
fn a_negative_number_to_a_fractional_power_is_an_error() {
    assert_eq!(run("10 PRINT (-8)^3;(-8)^-1;0^.5"), "-512 -.125  0 \n");
    let error = run_err("10 PRINT (-8)^(1/3)");
    assert_eq!(
        error.to_string(),
        "Runtime error in line 10: Illegal function call: ^"
    );
}
