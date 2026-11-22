//! Runs the 103 programs of *BASIC Computer Games* in `samples/games`.
//!
//! These are real programs from the 1970s, written by many hands for
//! several dialects, which makes them a good test of compatibility.

mod common;

use chuk_basic::ErrorKind;
use common::{play, programs, read, run_opts, Options, SEED};

fn game(name: &str) -> String {
    read(format!("samples/games/{name}.bas"))
}

#[test]
fn the_whole_collection_is_present() {
    assert_eq!(programs("samples/games").len(), 103);
}

#[test]
fn every_game_parses() {
    for path in programs("samples/games") {
        chuk_basic::parse(&read(&path)).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    }
}

/// Things a player might plausibly type.
const ANSWERS: [&str; 30] = [
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "0", "YES", "NO", "Y", "N", "1,1", "2,3",
    "4,4", "50", "100", "25", "3,3,3", "A", "HEADS", "1,2,3", "12", "20", "-1", ".5", "7,7",
];

/// 400 lines of keyboard mashing, different for each `seed`.
fn mash(seed: u64) -> String {
    let mut state = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let mut lines = String::new();
    for _ in 0..400 {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        lines.push_str(ANSWERS[(state >> 33) as usize % ANSWERS.len()]);
        lines.push('\n');
    }
    lines
}

/// Every game is played three times by a player typing at random. A game
/// may end, run out of input, loop until the step limit, or trip over the
/// nonsense it was fed (a subscript of -1, a division by zero) - but it must
/// never fail in a way that would mean the interpreter misread the program.
#[test]
fn every_game_survives_a_random_player() {
    const INTERPRETER_AT_FAULT: [&str; 10] = [
        "Undefined line",
        "Undefined function",
        "RETURN without GOSUB",
        "FOR without NEXT",
        "FNEND without DEF",
        "Type mismatch",
        "Out of DATA",
        "Bad number in DATA",
        "Wrong number of arguments",
        "already dimensioned",
    ];
    let mut failures = Vec::new();
    for path in programs("samples/games") {
        let source = read(&path);
        for seed in 0..3 {
            let options = Options {
                input: &mash(seed),
                seed,
                echo: false,
                step_limit: Some(1_000_000),
            };
            let (output, result) = run_opts(&source, &options);
            if output.trim().is_empty() {
                failures.push(format!("{} (seed {seed}): printed nothing", path.display()));
            }
            if let Err(e) = result {
                let at_fault = e.kind != ErrorKind::Runtime
                    || INTERPRETER_AT_FAULT.iter().any(|m| e.message.contains(m));
                if at_fault {
                    failures.push(format!("{} (seed {seed}): {e}", path.display()));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

// ---- games played properly ---------------------------------------------------

/// Hi-Lo picks a number from 1 to 100 and allows six guesses. A binary
/// search driven by the game's own "too high" / "too low" replies must win
/// whenever the number can be reached in six guesses - which checks RND,
/// INPUT and the comparisons against each other.
#[test]
fn hi_lo_is_won_by_binary_search() {
    let mut wins = 0;
    for seed in 0..20 {
        let (mut low, mut high, mut guess) = (1, 100, 0);
        let (transcript, result) = play(&game("hi-lo"), seed, |seen| {
            if seen.ends_with("PLAY AGAIN (YES OR NO)? ") {
                return Some("NO".to_string());
            }
            if seen.ends_with("YOUR GUESS IS TOO HIGH.\n\nYOUR GUESS? ") {
                high = guess - 1;
            } else if seen.ends_with("YOUR GUESS IS TOO LOW.\n\nYOUR GUESS? ") {
                low = guess + 1;
            }
            guess = (low + high) / 2;
            Some(guess.to_string())
        });
        result.unwrap();
        assert!(
            transcript.ends_with("SO LONG.  HOPE YOU ENJOYED YOURSELF!!!\n"),
            "{transcript}"
        );
        if transcript.contains("GOT IT!!!!!!!!!!") {
            wins += 1;
        } else {
            // a loss names the number: it must be consistent with every hint
            let number: i32 = transcript
                .split("THE NUMBER WAS")
                .nth(1)
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap()
                .parse()
                .unwrap();
            assert!(
                (low..=high).contains(&number),
                "seed {seed}: {number} outside {low}..={high}"
            );
        }
    }
    // six guesses reach 63 of the 100 numbers
    assert!((5..=19).contains(&wins), "won {wins} of 20");
}

/// The same idea for our own guessing game, which has no guess limit: the
/// search must always succeed, in at most seven tries.
#[test]
fn guess_is_always_won_by_binary_search() {
    let source = read("samples/classic/guess.bas");
    for seed in 0..25 {
        let (mut low, mut high, mut guess) = (1, 100, 0);
        let (transcript, result) = play(&source, seed, |seen| {
            if seen.ends_with("TOO LOW.\nYOUR GUESS? ") {
                low = guess + 1;
            } else if seen.ends_with("TOO HIGH.\nYOUR GUESS? ") {
                high = guess - 1;
            }
            guess = (low + high) / 2;
            Some(guess.to_string())
        });
        result.unwrap();
        let tries = transcript.matches("YOUR GUESS? ").count();
        assert!(
            transcript.ends_with(&format!("YOU GOT IT IN {tries} TRIES!\n")),
            "{transcript}"
        );
        assert!(tries <= 7, "seed {seed} took {tries} tries");
    }
}

/// Free fall in Lunar Lander is plain physics, so the answer can be checked
/// by hand: falling 120 miles from 1 mile/s under 0.001 miles/s^2 takes
/// (sqrt(1.24) - 1) / 0.001 = 113.553 s and ends at 1.113553 miles/s, which
/// is 4008.79 mph.
#[test]
fn lunar_lander_free_fall_matches_the_physics() {
    let (output, _) = run_opts(
        &game("lunar"),
        &Options {
            input: &"0\n".repeat(12),
            ..Options::default()
        },
    );
    assert!(
        output.contains("ON MOON AT 113.552873 SECONDS - IMPACT VELOCITY 4008.79034 MPH"),
        "{output}"
    );
    assert!(output.contains("SORRY THERE WERE NO SURVIVORS. YOU BLEW IT!"));
}

/// Feeding nobody in Hammurabi gets the ruler thrown out after one year.
#[test]
fn hammurabi_starving_the_city_ends_the_game() {
    // buy 0 acres, sell 0, feed 0 bushels, plant 0
    let (output, result) = run_opts(
        &game("hammurabi"),
        &Options {
            input: "0\n0\n0\n0\n",
            ..Options::default()
        },
    );
    result.unwrap();
    assert!(
        output.contains("YOU STARVED 100 PEOPLE IN ONE YEAR!!!"),
        "{output}"
    );
    assert!(
        output.ends_with("SO LONG FOR NOW.\n\n") || output.ends_with("SO LONG FOR NOW.\n"),
        "{output}"
    );
}

/// In 23 Matches whoever takes the last match loses; always taking three
/// leaves the last one to the player or the computer, never to nobody.
#[test]
fn twenty_three_matches_plays_to_the_end() {
    for seed in 0..10 {
        let (output, _) = run_opts(
            &game("23matches"),
            &Options {
                input: &"3\n".repeat(12),
                seed,
                ..Options::default()
            },
        );
        assert!(
            output.contains("YOU POOR BOOB! YOU TOOK THE LAST MATCH!")
                || output.contains("YOU WON, FLOPPY EARS !"),
            "seed {seed}:\n{output}"
        );
    }
}

/// Super Star Trek is the largest program in the collection (425 lines).
#[test]
fn super_star_trek_boots_and_obeys_commands() {
    let options = Options {
        input: "SRS\nLRS\nCOM\n0\nZZZ\nXXX\n",
        echo: true,
        ..Options::default()
    };
    let (output, result) = run_opts(&game("superstartrek"), &options);
    assert!(output.contains("THE USS ENTERPRISE --- NCC-1701"));
    assert!(output.contains("YOUR MISSION BEGINS WITH YOUR STARSHIP LOCATED"));
    assert!(output.contains("COMMAND? LRS\nLONG RANGE SCAN FOR QUADRANT"));
    assert!(output.contains("COMPUTER RECORD OF GALAXY FOR QUADRANT"));
    // the short range scan shows the Enterprise exactly once
    let scan = output
        .split("COMMAND? SRS\n")
        .nth(1)
        .unwrap()
        .split("COMMAND?")
        .next()
        .unwrap();
    assert_eq!(scan.matches("<*>").count(), 1, "{scan}");
    // ZZZ is not a command, so the game lists the real ones and asks again
    assert!(output.contains("COMMAND? ZZZ\nENTER ONE OF THE FOLLOWING:"));
    // XXX resigns
    assert!(output.contains("COMMAND? XXX\nTHERE WERE 21 KLINGON BATTLE CRUISERS LEFT AT"));
    assert!(result.unwrap_err().message.contains("Out of input"));
}

/// Running the same game twice with the same seed gives the same game.
#[test]
fn games_are_repeatable_with_a_seed() {
    for name in ["aceyducey", "craps", "slots", "war"] {
        let options = Options {
            input: &mash(1),
            seed: SEED,
            ..Options::default()
        };
        let first = run_opts(&game(name), &options).0;
        assert_eq!(first, run_opts(&game(name), &options).0, "{name}");
        let other = run_opts(
            &game(name),
            &Options {
                seed: SEED + 1,
                ..options
            },
        )
        .0;
        assert_ne!(
            first, other,
            "{name}: a different seed should give a different game"
        );
    }
}
