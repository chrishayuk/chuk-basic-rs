//! Golden tests: runs example programs and compares what they print with
//! the transcript saved beside them.
//!
//! For `name.bas`, `name.in` (optional) is typed at the keyboard, `name.out`
//! is the expected output and `name.err` (optional) is the error the run
//! must end with. Runs use a fixed seed, and input is echoed so that the
//! transcripts read like a terminal session.
//!
//! After a deliberate change in behaviour, regenerate the transcripts with
//! `UPDATE_GOLDEN=1 cargo test --test golden` and review the diff.

mod common;

use std::path::{Path, PathBuf};

use common::{programs, read, run_opts, Options};

/// Check one program against the expectation files `<expected>.out` etc.
/// Returns a description of the mismatch, if any.
fn check(program: &Path, expected: &Path) -> Option<String> {
    let exists = |ext: &str| expected.with_extension(ext).exists();
    let input = if exists("in") {
        read(expected.with_extension("in"))
    } else {
        String::new()
    };
    let (output, result) = run_opts(
        &read(program),
        &Options {
            input: &input,
            echo: true,
            ..Options::default()
        },
    );
    let error = result.err().map(|e| format!("{e}\n"));

    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(expected.with_extension("out"), &output).unwrap();
        match &error {
            Some(error) => std::fs::write(expected.with_extension("err"), error).unwrap(),
            None => drop(std::fs::remove_file(expected.with_extension("err"))),
        }
        return None;
    }

    if !exists("out") {
        return Some(format!("{}: no expected output file", program.display()));
    }
    let want_output = read(expected.with_extension("out"));
    let want_error = exists("err").then(|| read(expected.with_extension("err")));
    if output != want_output {
        return Some(format!(
            "{}: output differs\n--- expected\n{want_output}\n--- actual\n{output}",
            program.display()
        ));
    }
    if error != want_error {
        return Some(format!(
            "{}: expected error {want_error:?}, got {error:?}",
            program.display()
        ));
    }
    None
}

fn check_all(cases: impl IntoIterator<Item = (PathBuf, PathBuf)>) {
    let failures: Vec<String> = cases
        .into_iter()
        .filter_map(|(program, expected)| check(&program, &expected))
        .collect();
    assert!(
        failures.is_empty(),
        "{} golden test(s) failed:\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

/// Programs whose transcripts sit beside them.
fn beside(dir: &str) -> Vec<(PathBuf, PathBuf)> {
    programs(dir)
        .into_iter()
        .map(|program| (program.clone(), program))
        .collect()
}

#[test]
fn classic_programs() {
    check_all(beside("samples/classic"));
}

#[test]
fn tutorial_programs() {
    check_all(beside("samples/tutorial"));
}

/// Small programs that probe one area of the language each. Their saved
/// output was confirmed byte for byte against PC-BASIC, an independent
/// emulation of Microsoft GW-BASIC (see `scripts/pcbasic-diff.py`), so these
/// pin down behaviour that two implementations agree on.
#[test]
fn conformance_programs() {
    check_all(beside("tests/conformance"));
}

/// Games from the downloaded collection whose output does not depend on
/// chance (or on more than a line of input).
#[test]
fn deterministic_games() {
    let games = [
        "3dplot", "bunny", "calendar", "sinewave", "diamond", "love", "lunar", "banner",
    ];
    check_all(games.map(|name| {
        (
            format!("samples/games/{name}.bas").into(),
            format!("tests/expected/games/{name}").into(),
        )
    }));
}
