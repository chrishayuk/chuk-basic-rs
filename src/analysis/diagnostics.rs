//! Mistakes that can be found without running the program.

use std::collections::HashMap;

use super::scan::{scan, Line, Span};
use super::{line_refs, original, Lint, Location};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub at: Location,
    pub severity: Severity,
    pub message: String,
    /// "1964" for what only the Dartmouth lint minds.
    pub source: &'static str,
}

/// The statements of a line, without its number or the space before them.
fn body(line: &Line) -> Span {
    let start = line.toks.first().map_or(0, |tok| tok.span.start);
    Span {
        start,
        end: line.len.max(start),
    }
}

pub fn diagnostics(source: &str, lint: Lint) -> Vec<Diagnostic> {
    let lines = scan(source);
    let mut found = Vec::new();
    let mut error = |line: &Line, span: Span, message: String| {
        found.push(Diagnostic {
            at: Location {
                line: line.index,
                span,
            },
            severity: Severity::Error,
            message,
            source: "basic",
        })
    };

    // 1. each line on its own, so that one mistake does not hide the rest
    let mut syntax_ok = true;
    for line in &lines {
        if line.text.trim().is_empty() {
            continue;
        }
        if let Err(e) = crate::parse(&line.text) {
            syntax_ok = false;
            error(line, body(line), e.message);
        }
    }

    // 2. a number used twice: the later line replaces the earlier one
    let mut first_use: HashMap<u32, usize> = HashMap::new();
    for line in &lines {
        if let Some((number, span)) = line.number {
            if let Some(&earlier) = first_use.get(&number) {
                error(
                    line,
                    span,
                    format!(
                        "Line {number} is already used on line {} of the file",
                        earlier + 1
                    ),
                );
            } else {
                first_use.insert(number, line.index);
            }
        }
    }

    // 3. jumps to lines that are not there
    for reference in line_refs(&lines) {
        if !first_use.contains_key(&reference.target) {
            error(
                &lines[reference.at.line],
                reference.at.span,
                format!("There is no line {}", reference.target),
            );
        }
    }

    // 4. what only shows when the lines are put together: WEND without
    //    WHILE, FNEND without DEF
    if syntax_ok {
        let structure = crate::parse(source).and_then(|program| {
            let mut input = std::io::empty();
            let mut output = std::io::sink();
            crate::Interpreter::new(program, &mut input, &mut output, None).map(|_| ())
        });
        if let Err(e) = structure {
            let line = e
                .line
                .and_then(|number| first_use.get(&number))
                .or(e.src_line.map(|n| n - 1).as_ref())
                .and_then(|&index| lines.get(index));
            // two of these are already reported above, at the exact place
            let reported =
                e.message.starts_with("Undefined line") || e.message.starts_with("Duplicate line");
            if let (Some(line), false) = (line, reported) {
                error(line, body(line), e.message);
            }
        }
    }

    if lint == Lint::Dartmouth1964 {
        for (line, span, message) in original::check(&lines) {
            found.push(Diagnostic {
                at: Location { line, span },
                severity: Severity::Warning,
                message,
                source: "1964",
            });
        }
    }
    found.sort_by_key(|d| (d.at.line, d.at.span.start));
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn messages(source: &str, lint: Lint) -> Vec<(usize, String)> {
        diagnostics(source, lint)
            .into_iter()
            .map(|d| (d.at.line, d.message))
            .collect()
    }

    #[test]
    fn a_good_program_has_none() {
        let source = "10 FOR I = 1 TO 3\n20 PRINT I\n30 NEXT I\n40 GOSUB 60\n50 END\n60 RETURN\n";
        assert_eq!(messages(source, Lint::Later), []);
    }

    #[test]
    fn every_bad_line_is_reported_not_only_the_first() {
        let found = messages("10 PRINT (1 +\n20 PRINT 2\n30 LET = 4\n", Lint::Later);
        assert_eq!(found.len(), 2);
        assert_eq!((found[0].0, found[1].0), (0, 2));
        assert!(found[0].1.contains("Expected an expression"));
    }

    #[test]
    fn jumps_to_missing_lines_point_at_the_number() {
        let found = diagnostics(
            "10 IF X > 1 THEN 90\n20 ON X GOTO 10, 70\n30 GO TO 55\n",
            Lint::Later,
        );
        let got: Vec<_> = found
            .iter()
            .map(|d| (d.at.line, d.at.span.start, d.message.as_str()))
            .collect();
        assert_eq!(
            got,
            [
                (0, 17, "There is no line 90"),
                (1, 17, "There is no line 70"),
                (2, 9, "There is no line 55")
            ]
        );
    }

    #[test]
    fn duplicates_and_structure() {
        assert_eq!(
            messages("10 PRINT 1\n10 PRINT 2\n", Lint::Later),
            [(
                1,
                "Line 10 is already used on line 1 of the file".to_string()
            )]
        );
        // a NEXT with no FOR is only a mistake if the program gets there
        assert_eq!(messages("10 PRINT 1\n20 NEXT I\n", Lint::Later), []);
        assert_eq!(
            messages("10 WEND\n", Lint::Later),
            [(0, "WEND without WHILE".to_string())]
        );
    }

    #[test]
    fn the_manuals_own_programs_are_1964_basic() {
        for name in ["preface", "example1", "example2", "example3", "example3b"] {
            let path = format!("samples/dartmouth/{name}.bas");
            let source = std::fs::read_to_string(path).unwrap();
            assert_eq!(messages(&source, Lint::Dartmouth1964), [], "{name}");
        }
    }

    #[test]
    fn what_1964_did_not_have() {
        let source = "10 INPUT N\n20 X = 2: PRINT X\n30 LET TOTAL = FIX(N)\n40 LET A$ = \"HI\" ' note\n50 IF N > 1 AND X > 1 THEN 10 ELSE 20\n";
        let found: Vec<String> = messages(source, Lint::Dartmouth1964)
            .into_iter()
            .map(|(_, m)| m)
            .collect();
        assert_eq!(
            found,
            [
                "INPUT is not one of the fifteen statements of 1964 BASIC",
                "LET is required in 1964 BASIC",
                "One instruction to a line in 1964 BASIC",
                "TOTAL: in 1964 a variable is a single letter, or a letter followed by one digit",
                "FIX is not one of the ten functions of 1964 BASIC",
                "1964 BASIC has no variables for text, only for numbers",
                "A comment is written REM in 1964 BASIC",
                "The last instruction in any BASIC program is END (1964 manual, p. 4)",
                "AND is not in 1964 BASIC",
                "ELSE is not one of the fifteen statements of 1964 BASIC",
            ]
        );
        // and none of it is an error in a later BASIC
        assert_eq!(messages(source, Lint::Later), []);
    }
}
