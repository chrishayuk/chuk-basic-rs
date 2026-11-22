//! What the BASIC of 1964 had, for pointing out what it did not.
//!
//! The fifteen statements and ten functions are the ones Thomas Kurtz lists
//! for "the original BASIC" (dartmouth.edu/basicfifty/commands.html); the
//! May 1964 manual itself counts fourteen operations, END "and 13 other"
//! (p. 4), and does not name them all in one place. The rule for variable
//! names, numbers of at most nine digits and "the last instruction in any
//! BASIC program is END" are in the manual, pp. 4-5; the form `IF F1 rel F2
//! THEN N` on pp. 11-12; and a defined function of one argument on p. 13.
//!
//! These are checks of vocabulary and of a few statement forms, not a
//! grammar of 1964 BASIC: a program they pass may still not be one that the
//! 1964 system would have accepted.

use super::scan::{Kind, Line, Span, Tok};
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
        forms(line, &mut found);
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

/// The statement forms of 1964 that a list of words cannot show: `IF` with a
/// relation and a line number after `THEN`, `DEF` with one argument, and
/// numbers of at most nine digits.
fn forms(line: &Line, found: &mut Vec<(usize, Span, String)>) {
    let mut say = |span: Span, why: &str| found.push((line.index, span, why.to_string()));
    for statement in line
        .toks
        .split(|tok| tok.kind == Kind::Punct && tok.text == ":")
    {
        match statement.first().map(Tok::word) {
            Some("IF") => {
                let then = statement.iter().position(|tok| tok.word() == "THEN");
                let condition = &statement[1..then.unwrap_or(statement.len())];
                let relation = condition
                    .iter()
                    .any(|tok| tok.kind == Kind::Punct && "=<>".contains(tok.text.as_str()));
                if !relation {
                    say(
                        statement[0].span,
                        "In 1964 BASIC an IF compares two formulas: IF F1 < F2 THEN N (manual, pp. 11-12)",
                    );
                }
                if let Some(then) = then {
                    // (an ELSE after the number is marked as a later word)
                    let after = statement.get(then + 1);
                    if !after.is_some_and(|tok| tok.kind == Kind::Num) {
                        say(
                            statement[then].span,
                            "In 1964 BASIC THEN is followed by an instruction number (manual, pp. 11-12)",
                        );
                    }
                }
            }
            Some("DEF") => {
                let open = statement.iter().position(|tok| tok.text == "(");
                if let Some(open) = open {
                    let mut depth = 0;
                    let mut commas = 0;
                    for tok in &statement[open..] {
                        match tok.text.as_str() {
                            "(" => depth += 1,
                            ")" => depth -= 1,
                            "," if depth == 1 => commas += 1,
                            _ => {}
                        }
                        if depth == 0 {
                            break;
                        }
                    }
                    if commas > 0 {
                        say(
                            statement[0].span,
                            "A function of 1964 BASIC has one argument: DEF FNC(Z) = ... (manual, p. 13)",
                        );
                    }
                }
            }
            _ => {}
        }
    }
    for tok in &line.toks {
        let numbers: Vec<&str> = match tok.kind {
            Kind::Num => vec![tok.text.as_str()],
            Kind::Data => tok.text.split(',').map(str::trim).collect(),
            _ => continue,
        };
        if numbers.iter().any(|number| too_many_digits(number)) {
            say(
                tok.span,
                "A number in 1964 BASIC has at most 9 digits (manual, p. 5)",
            );
        }
    }
}

/// More than nine digits before any exponent, not counting leading zeros.
fn too_many_digits(number: &str) -> bool {
    let mantissa = number
        .trim_start_matches(['+', '-'])
        .split(['E', 'e'])
        .next()
        .unwrap_or("");
    if mantissa.is_empty() || !mantissa.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return false;
    }
    let digits = mantissa.trim_start_matches(['0', '.']);
    digits.chars().filter(char::is_ascii_digit).count() > 9
}

fn is_variable(word: &str) -> bool {
    let mut chars = word.chars();
    let letter = chars.next().is_some_and(|c| c.is_ascii_alphabetic());
    let digit = chars.next().is_none_or(|c| c.is_ascii_digit());
    letter && digit && chars.next().is_none()
}
