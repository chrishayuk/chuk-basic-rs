//! Where a jump goes, who jumps here, and the outline of a program.

use super::scan::{scan, Kind, Line};
use super::{line_numbered, line_refs, token_at, Location};

fn located(line: &Line, tok: &super::Tok) -> Location {
    Location {
        line: line.index,
        span: tok.span,
    }
}

/// The BASIC line number that the cursor is on: the line's own, or a target.
fn number_at(lines: &[Line], line: usize, column: u32) -> Option<u32> {
    let here = lines.get(line)?;
    if let Some((number, span)) = here.number {
        if span.start <= column && column <= span.end {
            return Some(number);
        }
    }
    line_refs(lines)
        .into_iter()
        .find(|r| r.at.line == line && r.at.span.start <= column && column <= r.at.span.end)
        .map(|r| r.target)
}

/// A name under the cursor that is worth following: `FNA`, `X`, `N$`.
fn name_at(lines: &[Line], line: usize, column: u32) -> Option<String> {
    let tok = token_at(lines.get(line)?, column)?;
    let word = tok.word();
    let name = !word.is_empty()
        && !super::original::is_original(word)
        && crate::ast::Builtin::from_name(word).is_none()
        && crate::tokenize_line(word, 1, crate::Spacing::Spaced).is_ok_and(|toks| {
            matches!(
                toks.first().map(|t| &t.tok),
                Some(crate::lexer::Tok::Ident(_))
            )
        });
    name.then(|| word.to_string())
}

/// Where the thing under the cursor is defined: the line a jump goes to, or
/// the `DEF` of a function.
pub fn definition(source: &str, line: usize, column: u32) -> Option<Location> {
    let lines = scan(source);
    if let Some(number) = number_at(&lines, line, column) {
        let target = line_numbered(&lines, number)?;
        return Some(Location {
            line: target.index,
            span: target.number?.1,
        });
    }
    let name = name_at(&lines, line, column)?;
    if !name.starts_with("FN") {
        return None;
    }
    lines.iter().find_map(|l| {
        let at = l.toks.iter().position(|t| t.word() == "DEF")?;
        let tok = l.toks.get(at + 1).filter(|t| t.text == name)?;
        Some(located(l, tok))
    })
}

/// Every place that mentions the thing under the cursor: for a line number,
/// the jumps to it (and the line itself); for a name, each use.
pub fn references(source: &str, line: usize, column: u32, declaration: bool) -> Vec<Location> {
    let lines = scan(source);
    if let Some(number) = number_at(&lines, line, column) {
        let mut found: Vec<Location> = line_refs(&lines)
            .into_iter()
            .filter(|r| r.target == number)
            .map(|r| r.at)
            .collect();
        if declaration {
            if let Some((target, Some((_, span)))) =
                line_numbered(&lines, number).map(|l| (l, l.number))
            {
                found.insert(
                    0,
                    Location {
                        line: target.index,
                        span,
                    },
                );
            }
        }
        return found;
    }
    let Some(name) = name_at(&lines, line, column) else {
        return Vec::new();
    };
    lines
        .iter()
        .flat_map(|l| {
            l.toks
                .iter()
                .filter(|t| t.text == name && t.kind == Kind::Word)
                .map(move |t| located(l, t))
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// A line that a `GOSUB` goes to.
    Subroutine,
    /// A `DEF FN`.
    Function,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub at: Location,
}

/// The outline: subroutines, named by the REM at their head if there is
/// one, and the program's own functions.
pub fn symbols(source: &str) -> Vec<Symbol> {
    let lines = scan(source);
    let mut found = Vec::new();
    let mut seen = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        for (at, tok) in line.toks.iter().enumerate() {
            if tok.word() == "DEF" {
                if let Some(name) = line.toks.get(at + 1).filter(|t| t.kind == Kind::Word) {
                    found.push(Symbol {
                        name: name.text.clone(),
                        kind: SymbolKind::Function,
                        at: located(line, name),
                    });
                }
            }
            if tok.word() != "GOSUB" {
                continue;
            }
            for reference in line_refs(&lines[i..=i]) {
                // only the numbers that follow a GOSUB on this line
                let after_gosub = line.toks[..]
                    .iter()
                    .rev()
                    .find(|t| {
                        t.span.end <= reference.at.span.start && super::JUMPS.contains(&t.word())
                    })
                    .is_some_and(|t| t.word() == "GOSUB");
                if !after_gosub || seen.contains(&reference.target) {
                    continue;
                }
                seen.push(reference.target);
                let Some(target) = line_numbered(&lines, reference.target) else {
                    continue;
                };
                let remark = |l: &Line| {
                    l.toks
                        .iter()
                        .find(|t| t.kind == Kind::Comment)
                        .map(|t| t.text.trim_start_matches('\'').trim().to_string())
                        .filter(|text| !text.is_empty())
                };
                let title = remark(target).or_else(|| {
                    target
                        .index
                        .checked_sub(1)
                        .and_then(|above| lines.get(above))
                        .and_then(|l| {
                            (l.toks.first().is_some_and(|t| t.word() == "REM"))
                                .then(|| remark(l))
                                .flatten()
                        })
                });
                found.push(Symbol {
                    name: match title {
                        Some(title) => format!("{} {title}", reference.target),
                        None => format!("{} (subroutine)", reference.target),
                    },
                    kind: SymbolKind::Subroutine,
                    at: Location {
                        line: target.index,
                        span: target.number.map_or(reference.at.span, |(_, span)| span),
                    },
                });
            }
        }
    }
    found.sort_by_key(|s| (s.at.line, s.at.span.start));
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROGRAM: &str = "10 DEF FNA(X) = X * 2\n20 GOSUB 100\n30 PRINT FNA(3)\n40 IF X THEN 100\n50 END\n90 REM DOUBLE X\n100 LET X = X + X\n110 RETURN\n";

    #[test]
    fn a_jump_leads_to_its_line_and_back() {
        // the 100 of GOSUB 100
        let target = definition(PROGRAM, 1, 10).unwrap();
        assert_eq!((target.line, target.span.start, target.span.end), (6, 0, 3));
        // from line 100: both jumps, and the line itself when asked
        let uses = references(PROGRAM, 6, 1, false);
        assert_eq!(uses.iter().map(|l| l.line).collect::<Vec<_>>(), [1, 3]);
        assert_eq!(references(PROGRAM, 6, 1, true).len(), 3);
        assert_eq!(definition(PROGRAM, 4, 4), None);
    }

    #[test]
    fn names() {
        let def = definition(PROGRAM, 2, 10).unwrap();
        assert_eq!((def.line, def.span.start), (0, 7));
        let uses = references(PROGRAM, 6, 8, true);
        assert_eq!(uses.len(), 6, "{uses:?}");
        assert!(
            references(PROGRAM, 2, 4, true).is_empty(),
            "PRINT is not a name"
        );
    }

    #[test]
    fn outline() {
        let found = symbols(PROGRAM);
        let names: Vec<_> = found
            .iter()
            .map(|s| (s.name.as_str(), s.kind, s.at.line))
            .collect();
        assert_eq!(
            names,
            [
                ("FNA", SymbolKind::Function, 0),
                ("100 DOUBLE X", SymbolKind::Subroutine, 6)
            ]
        );
    }
}
