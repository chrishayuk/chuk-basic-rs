//! The editor's analysis, held against every sample program: nothing that
//! runs is called a mistake, and renumbering changes nothing a program does.

mod common;

use chuk_basic::analysis::{diagnostics, renumbered, Lint, Severity};
use common::{programs, read, run_opts, Options};

const FOLDERS: [&str; 5] = [
    "samples",
    "samples/classic",
    "samples/tutorial",
    "samples/dartmouth",
    "samples/games",
];

fn every_sample() -> Vec<std::path::PathBuf> {
    FOLDERS.iter().flat_map(|folder| programs(folder)).collect()
}

/// 150 programs that run, from many hands: the checker must not cry wolf.
/// It finds one real fault, a jump in a 1978 listing to a line that is not
/// in it.
#[test]
fn no_sample_has_an_error_but_the_one_that_does() {
    let mut found = Vec::new();
    let samples = every_sample();
    assert!(samples.len() >= 150);
    for path in samples {
        for d in diagnostics(&read(&path), Lint::Later) {
            assert_eq!(d.severity, Severity::Error);
            found.push(format!(
                "{}:{}: {}",
                path.display(),
                d.at.line + 1,
                d.message
            ));
        }
    }
    assert_eq!(found, ["samples/games/splat.bas:75: There is no line 540"]);
}

/// Old listings fed the answers a player might type, before and after.
#[test]
fn every_sample_prints_the_same_after_renumbering() {
    let answers = [
        "1", "2", "5", "10", "0", "YES", "NO", "Y", "N", "1,1", "50", "25", "A", ".5", "7,7",
    ];
    let input: String = (0..300)
        .map(|i| format!("{}\n", answers[(i * 7 + i / 5) % answers.len()]))
        .collect();
    let options = Options {
        input: &input,
        step_limit: Some(200_000),
        ..Options::default()
    };
    let mut changed = 0;
    for path in every_sample() {
        let source = read(&path);
        let again =
            renumbered(&source, 10, 10).unwrap_or_else(|why| panic!("{}: {why}", path.display()));
        changed += usize::from(again.replace("\r\n", "\n") != source.replace("\r\n", "\n"));
        // an error names a line number, which has changed; the output must not
        let (before, _) = run_opts(&source, &options);
        let (after, _) = run_opts(&again, &options);
        assert_eq!(before, after, "{}", path.display());
    }
    assert!(
        changed > 100,
        "most listings are not already numbered in tens: {changed}"
    );
}

/// The manual's programs are the BASIC of 1964, and almost nothing else is.
#[test]
fn the_1964_lint_knows_the_manual_from_the_rest() {
    for path in programs("samples/dartmouth") {
        assert_eq!(
            diagnostics(&read(&path), Lint::Dartmouth1964),
            [],
            "{}",
            path.display()
        );
    }
    let later = programs("samples/games")
        .iter()
        .filter(|path| !diagnostics(&read(path), Lint::Dartmouth1964).is_empty())
        .count();
    assert_eq!(
        later, 103,
        "every 1978 listing uses something 1964 did not have"
    );
}
