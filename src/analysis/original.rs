//! What the BASIC of 1964 had, for pointing out what it did not.
//!
//! The fifteen statements and ten functions are the ones Thomas Kurtz lists
//! for "the original BASIC" (dartmouth.edu/basicfifty/commands.html); the
//! rule for variable names and "the last instruction in any BASIC program
//! is END" are in the May 1964 manual, pp. 4-5.

use super::scan::{Kind, Line, Span};
use crate::ast::Builtin;

pub(super) const STATEMENTS: [&str; 15] = [
    "LET", "PRINT", "END", "READ", "DATA", "GOTO", "IF", "FOR", "NEXT", "GOSUB", "RETURN", "DEF",
    "DIM", "REM", "STOP",
];

pub(super) const FUNCTIONS: [&str; 10] = [
    "ABS", "ATN", "COS", "EXP", "INT", "LOG", "RND", "SIN", "SQR", "TAN",
];

/// Statement words of later BASICs that this interpreter knows.
const LATER_STATEMENTS: [&str; 11] = [
    "INPUT",
    "LINE",
    "ON",
    "WHILE",
    "WEND",
    "RANDOMIZE",
    "RESTORE",
    "MAT",
    "FNEND",
    "ELSE",
    "USING",
];

/// Words that are part of a 1964 statement without starting one.
const PARTS: [&str; 4] = ["TO", "STEP", "THEN", "GO"];

/// Words of later BASICs that are neither statements nor names.
const LATER_PARTS: [&str; 3] = ["AND", "OR", "NOT"];

pub(super) fn is_original(word: &str) -> bool {
    STATEMENTS.contains(&word) || FUNCTIONS.contains(&word) || PARTS.contains(&word)
}

/// What a 1964 Dartmouth BASIC would not have accepted, as (where, why).
pub(super) fn check(lines: &[Line]) -> Vec<(usize, Span, String)> {
    let mut found = Vec::new();
    for line in lines {
        if line.toks.is_empty() && line.number.is_none() {
            continue;
        }
        let whole = Span {
            start: 0,
            end: line.len,
        };
        if line.number.is_none() {
            found.push((
                line.index,
                whole,
                "Every instruction needs a number in 1964 BASIC".to_string(),
            ));
        }
        let mut start = true;
        for tok in &line.toks {
            let mut say = |why: String| found.push((line.index, tok.span, why));
            let at_start = start;
            start = false;
            match tok.kind {
                Kind::Comment if tok.text.starts_with('\'') => {
                    say("A comment is written REM in 1964 BASIC".to_string())
                }
                Kind::Punct if tok.text == ":" => {
                    say("One instruction to a line in 1964 BASIC".to_string());
                    start = true;
                }
                Kind::Word => {
                    let word = tok.text.as_str();
                    if word == "THEN" {
                        start = true;
                    }
                    if is_original(word) {
                        continue;
                    }
                    if LATER_STATEMENTS.contains(&word) {
                        say(format!(
                            "{word} is not one of the fifteen statements of 1964 BASIC"
                        ));
                    } else if LATER_PARTS.contains(&word) || word == "TAB" || word == "SPC" {
                        say(format!("{word} is not in 1964 BASIC"));
                    } else if Builtin::from_name(word).is_some() {
                        say(format!(
                            "{word} is not one of the ten functions of 1964 BASIC"
                        ));
                    } else if word.ends_with('$') {
                        say("1964 BASIC has no variables for text, only for numbers".to_string());
                    } else if word.starts_with("FN") && word.len() == 3 {
                        // a function of the program's own
                    } else if !is_variable(word) {
                        say(format!(
                            "{word}: in 1964 a variable is a single letter, or a letter followed by one digit"
                        ));
                    } else if at_start {
                        say("LET is required in 1964 BASIC".to_string());
                    }
                }
                _ => {}
            }
        }
    }
    // the last line with anything on it
    if let Some(line) = lines.iter().rev().find(|l| !l.toks.is_empty()) {
        if line.toks[0].word() != "END" {
            let span = Span {
                start: 0,
                end: line.len,
            };
            found.push((
                line.index,
                span,
                "The last instruction in any BASIC program is END (1964 manual, p. 4)".to_string(),
            ));
        }
    }
    found
}

fn is_variable(word: &str) -> bool {
    let mut chars = word.chars();
    let letter = chars.next().is_some_and(|c| c.is_ascii_alphabetic());
    let digit = chars.next().is_none_or(|c| c.is_ascii_digit());
    letter && digit && chars.next().is_none()
}
