//! Runs the original sample programs in `samples/` end to end.

mod common;

use common::{run, run_with};

fn sample(name: &str) -> String {
    common::read(format!("samples/{name}"))
}

#[test]
fn every_sample_parses() {
    for dir in [
        "samples",
        "samples/classic",
        "samples/tutorial",
        "samples/dartmouth",
    ] {
        for path in common::programs(dir) {
            chuk_basic::parse(&common::read(&path))
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        }
    }
}

#[test]
fn test1() {
    assert_eq!(
        run(&sample("test1.bas")),
        "Hello, World!\nx is greater than 5\n 1 \n 2 \n 3 \n 4 \n 5 \n 6 \n 7 \n 8 \n 9 \n 10 \n"
    );
}

#[test]
fn complex() {
    let (output, result) = run_with(&sample("complex.bas"), "Chris\n");
    result.unwrap();
    assert_eq!(
        output,
        "Hello, world!\nPositive\n 1 \n 2 \n 3 \n 4 \n 5 \nEnter your name: ? Hello, Chris\n"
    );
}

/// The first program in the 1964 Dartmouth manual: solves pairs of linear
/// equations until it runs out of DATA.
#[test]
fn dartmouth_first() {
    let (output, result) = run_with(&sample("dartmouth_first.bas"), "");
    assert_eq!(
        output,
        " 4            -5.5 \n .666666667    .166666667 \n-3.66666667    3.83333333 \n"
    );
    let err = result.unwrap_err();
    assert_eq!((err.line, err.message.as_str()), (Some(30), "Out of DATA"));
}

#[test]
fn on_fun() {
    for (choice, expected) in [
        ("1", "You entered 1"),
        ("2", "You entered 2"),
        ("3", "You entered 3"),
    ] {
        let (output, result) = run_with(&sample("on_fun.bas"), &format!("{choice}\n"));
        result.unwrap();
        assert_eq!(
            output,
            format!(
                "Enter a number between 1 and 3:\n? {expected}\nProgram execution continues...\n"
            )
        );
    }
    let (output, _) = run_with(&sample("on_fun.bas"), "9\n");
    assert!(!output.contains("You entered"));
}

#[test]
fn acey_ducey_plays_until_the_money_runs_out() {
    // bet 100 every hand; "NO" fails the bet prompt (redo) and then declines
    // the try-again prompt once the money is gone
    let input = "100\nNO\n".repeat(500);
    let (output, _) = run_with(&sample("aceyducey.bas"), &input);
    assert!(output.starts_with("                          ACEY DUCEY CARD GAME\n"));
    assert!(output.contains("YOU NOW HAVE  100  DOLLARS."));
    assert!(output.contains("HERE ARE YOUR NEXT TWO CARDS: "));
    assert!(output.contains("SORRY, FRIEND, BUT YOU BLEW YOUR WAD."));
    assert!(output.ends_with("O.K., HOPE YOU HAD FUN!\n"));
}

#[test]
fn acey_ducey_chickens_out() {
    let (output, result) = run_with(&sample("aceyducey.bas"), "0\n0\n");
    assert_eq!(output.matches("CHICKEN!!").count(), 2);
    assert!(result.unwrap_err().message.contains("Out of input"));
}

#[test]
fn math_dice_checks_answers() {
    // answer 7 to everything: sometimes right, sometimes corrected
    let (output, result) = run_with(&sample("mathdice.bas"), "7\n7\n7\n7\n");
    assert!(output.contains("MATH DICE"));
    assert!(output.contains(" ----- \nI"));
    assert!(output.contains("RIGHT!") || output.contains("NO, THE ANSWER IS"));
    assert!(output.contains("THE DICE ROLL AGAIN..."));
    assert!(result.unwrap_err().message.contains("Out of input"));
}

#[test]
fn basketball_plays_a_full_game() {
    // defense 6, opponent name, then always take shot 1 (long jump shot)
    let input = "6\nYALE\n".to_string() + &"1\n".repeat(2000);
    let (output, result) = run_with(&sample("basketball.bas"), &input);
    assert!(output.contains("CENTER JUMP"));
    assert!(output.contains("***** END OF FIRST HALF *****"));
    assert!(output.contains("SCORE: "));
    assert!(
        output.contains("***** END OF GAME *****"),
        "game did not finish: {result:?}"
    );
    result.unwrap();
}

// ---- the first BASIC instruction manual, Dartmouth, May 1964 ----------------

fn dartmouth(name: &str, dialect: chuk_basic::Dialect) -> (String, Result<(), chuk_basic::Error>) {
    let mut output = Vec::new();
    let result = chuk_basic::parse(&sample(&format!("dartmouth/{name}.bas"))).and_then(|program| {
        let mut input = std::io::empty();
        let mut interpreter = chuk_basic::Interpreter::new(program, &mut input, &mut output, None)?;
        interpreter.set_dialect(dialect);
        interpreter.run()
    });
    (String::from_utf8(output).unwrap(), result)
}

/// The manual prints the results of its example programs (pp. 21-23). Ours
/// must agree to the six digits the 1964 system printed.
#[test]
fn the_1964_manuals_examples_print_what_the_manual_prints() {
    use chuk_basic::Dialect::Microsoft;
    assert_eq!(dartmouth("preface", Microsoft).0, " 5 \n");
    // "AVERAGE OF 100 LOGS / 1.5797"
    assert_eq!(
        dartmouth("example1", Microsoft).0,
        "AVERAGE OF 100 LOGS\n 1.57970004 \n"
    );
    // "ROOT = .837585", and with the function changed, "ROOT = .693176"
    assert!(dartmouth("example3", Microsoft)
        .0
        .ends_with("ROOT = .837585449 \n"));
    assert!(dartmouth("example3b", Microsoft)
        .0
        .ends_with("ROOT = .69317627 \n"));

    // asked for 1964's printing, the figures are the page's
    use chuk_basic::Dialect::Dartmouth;
    assert_eq!(dartmouth("preface", Dartmouth).0, " 5. \n");
    assert_eq!(
        dartmouth("example1", Dartmouth).0,
        "AVERAGE OF 100 LOGS\n 1.5797 \n"
    );
    assert!(dartmouth("example3", Dartmouth)
        .0
        .ends_with("ROOT = .837585 \n"));
    assert!(dartmouth("example3b", Dartmouth)
        .0
        .ends_with("ROOT = .693176 \n"));
}

/// Example 2 takes the square root of a negative number at line 40 and
/// expects the 1964 rule: "the square-root is always taken of the absolute
/// value" (p. 5). Later BASICs call that an error.
#[test]
fn the_1964_quadratic_example_needs_dartmouth_arithmetic() {
    use chuk_basic::Dialect::{Dartmouth, Microsoft};
    let (output, result) = dartmouth("example2", Microsoft);
    assert_eq!(
        result.unwrap_err().to_string(),
        "Runtime error in line 40: Illegal function call: SQR"
    );
    assert!(output.contains("ONE ROOT") && !output.contains("COMPLEX ROOTS"));

    // the manual's page: "ROOTS = 3.  -1." / "ROOT = 2." / "IMAGINARY PART = 1.41421"
    let (output, result) = dartmouth("example2", Dartmouth);
    assert!(
        output.contains("TWO REAL ROOTS\nROOTS = 3.    -1. \n"),
        "{output}"
    );
    assert!(output.contains("ONE ROOT\nROOT = 2. \n"));
    assert!(
        output.contains("COMPLEX ROOTS\nREAL PART= 0.               IMAGINARY PART= 1.41421 \n")
    );
    // it loops back for more DATA until there is none
    assert_eq!(result.unwrap_err().message, "Out of DATA");
}

#[test]
fn dartmouth_powers_take_the_absolute_value() {
    let run = |dialect| {
        let mut output = Vec::new();
        let program = chuk_basic::parse("10 PRINT (-2)↑3; SQR(-9)").unwrap();
        let mut input = std::io::empty();
        let mut interpreter =
            chuk_basic::Interpreter::new(program, &mut input, &mut output, None).unwrap();
        interpreter.set_dialect(dialect);
        let result = interpreter.run();
        (String::from_utf8(output).unwrap(), result.is_ok())
    };
    assert_eq!(
        run(chuk_basic::Dialect::Dartmouth),
        (" 8.  3. \n".to_string(), true)
    );
    assert!(!run(chuk_basic::Dialect::Microsoft).1);
}
