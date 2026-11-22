//! What an editor wants to know about a program without running it:
//! mistakes, where a `GOTO` goes, what a word means, and how to renumber.
//!
//! Everything here works on the source text and gives positions as a line
//! and a column, which the parser does not keep. `scan` finds the words and
//! where they are; the parser is still what decides whether a line is valid.

mod diagnostics;
mod docs;
mod navigate;
mod original;
mod renumber;
mod scan;

pub use diagnostics::{diagnostics, Diagnostic, Severity};
pub use docs::hover;
pub use navigate::{definition, references, symbols, Symbol, SymbolKind};
pub use renumber::{renumber, renumbered, Edit};
pub use scan::{scan, Kind, Line, Span, Tok};

/// A place in the text: a 0-based line, and columns counted in UTF-16 code
/// units, which is how the language server protocol counts them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub line: usize,
    pub span: Span,
}

/// Which BASIC to hold a program to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lint {
    /// Anything this interpreter runs.
    #[default]
    Later,
    /// Also point out what the BASIC of 1964 did not have.
    Dartmouth1964,
}

/// Statement words after which a line number is a place to go.
const JUMPS: [&str; 5] = ["GOTO", "GOSUB", "THEN", "ELSE", "RESTORE"];

/// One mention of a line number as a target, as in `GOTO 100`.
#[derive(Debug, Clone, PartialEq)]
pub struct LineRef {
    pub target: u32,
    pub at: Location,
}

/// Every line number that the program's jumps name, with where it is written.
pub fn line_refs(lines: &[Line]) -> Vec<LineRef> {
    let mut refs = Vec::new();
    for line in lines {
        let toks = &line.toks;
        let mut i = 0;
        while i < toks.len() {
            let word = toks[i].word();
            let jump =
                JUMPS.contains(&word) || (word == "TO" && i > 0 && toks[i - 1].word() == "GO");
            i += 1;
            if !jump {
                continue;
            }
            // one number, or a list of them after ON ... GOTO
            while i < toks.len() && toks[i].kind == Kind::Num {
                if let Ok(target) = toks[i].text.parse::<u32>() {
                    refs.push(LineRef {
                        target,
                        at: Location {
                            line: line.index,
                            span: toks[i].span,
                        },
                    });
                }
                i += 1;
                if i < toks.len() && toks[i].text == "," {
                    i += 1;
                } else {
                    break;
                }
            }
        }
    }
    refs
}

/// The source line that carries BASIC line `number`.
pub(crate) fn line_numbered(lines: &[Line], number: u32) -> Option<&Line> {
    lines
        .iter()
        .find(|line| line.number.map(|(n, _)| n) == Some(number))
}

/// The token under a column, if any (the end of a word counts as on it).
pub(crate) fn token_at(line: &Line, column: u32) -> Option<&Tok> {
    line.toks
        .iter()
        .find(|tok| tok.span.start <= column && column <= tok.span.end)
}
