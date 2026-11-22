//! Renumbering: lines in tens, and every jump changed to match.

use std::collections::HashMap;

use super::line_refs;
use super::scan::{scan, Span};
use crate::ast::{Program, Stmt};

/// Replace the text at a place with something else.
#[derive(Debug, Clone, PartialEq)]
pub struct Edit {
    pub line: usize,
    pub span: Span,
    pub text: String,
}

/// The edits that renumber a program from `start` in steps of `step`.
///
/// Refuses, with a reason, when it cannot be sure it has found every jump:
/// a program that does not parse, or one written without spaces
/// (`IFA=1THEN100`), where the numbers are not where this looks for them.
pub fn renumber(source: &str, start: u32, step: u32) -> Result<Vec<Edit>, String> {
    let program = crate::parse(source).map_err(|e| format!("Not renumbered: {e}"))?;
    let lines = scan(source);
    let refs = line_refs(&lines);

    // the parser's idea of the jumps must be the same as ours
    let mut expected = targets(&program);
    let mut found: Vec<u32> = refs.iter().map(|r| r.target).collect();
    expected.sort_unstable();
    found.sort_unstable();
    if expected != found {
        return Err("Not renumbered: some jumps in this program are written in a way that cannot be changed safely (lines without spaces, perhaps)".to_string());
    }

    let mut numbers: Vec<u32> = lines
        .iter()
        .filter_map(|l| l.number.map(|(n, _)| n))
        .collect();
    numbers.sort_unstable();
    if numbers.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err("Not renumbered: a line number is used twice".to_string());
    }
    let new: HashMap<u32, u32> = numbers
        .iter()
        .enumerate()
        .map(|(i, &old)| (old, start + step * i as u32))
        .collect();

    let mut edits = Vec::new();
    for line in &lines {
        if let Some((old, span)) = line.number {
            edits.push(Edit {
                line: line.index,
                span,
                text: new[&old].to_string(),
            });
        }
    }
    for reference in refs {
        // a jump to a line that is not there is left as it is
        if let Some(to) = new.get(&reference.target) {
            edits.push(Edit {
                line: reference.at.line,
                span: reference.at.span,
                text: to.to_string(),
            });
        }
    }
    edits.retain(|edit| {
        let line = &lines[edit.line].text;
        let old: String = line
            .encode_utf16()
            .skip(edit.span.start as usize)
            .take((edit.span.end - edit.span.start) as usize)
            .map(|u| u as u8 as char)
            .collect();
        old != edit.text
    });
    edits.sort_by_key(|e| (e.line, e.span.start));
    Ok(edits)
}

/// The whole program, renumbered.
pub fn renumbered(source: &str, start: u32, step: u32) -> Result<String, String> {
    let edits = renumber(source, start, step)?;
    let mut out = String::new();
    for (index, text) in source.lines().enumerate() {
        let units: Vec<u16> = text.encode_utf16().collect();
        let mut at = 0usize;
        let mut line = Vec::new();
        for edit in edits.iter().filter(|e| e.line == index) {
            line.extend_from_slice(&units[at..edit.span.start as usize]);
            line.extend(edit.text.encode_utf16());
            at = edit.span.end as usize;
        }
        line.extend_from_slice(&units[at..]);
        out.push_str(&String::from_utf16_lossy(&line));
        out.push('\n');
    }
    Ok(out)
}

/// Every line number the program jumps to, as the parser understands it.
fn targets(program: &Program) -> Vec<u32> {
    fn walk(stmts: &[Stmt], out: &mut Vec<u32>) {
        for stmt in stmts {
            match stmt {
                Stmt::Goto(n) | Stmt::Gosub(n) | Stmt::Restore(Some(n)) => out.push(*n),
                Stmt::On { targets, .. } => out.extend(targets),
                Stmt::If {
                    then, otherwise, ..
                } => {
                    walk(then, out);
                    if let Some(otherwise) = otherwise {
                        walk(otherwise, out);
                    }
                }
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    for line in &program.lines {
        walk(&line.stmts, &mut out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_and_jumps_change_together() {
        let source = "5 PRINT \"GOTO 7\"\n7 IF X > 1 THEN 20 ELSE 5\n20 ON X GOSUB 7, 100\n100 GO TO 5: REM GOTO 7\n";
        assert_eq!(
            renumbered(source, 10, 10).unwrap(),
            "10 PRINT \"GOTO 7\"\n20 IF X > 1 THEN 30 ELSE 10\n30 ON X GOSUB 20, 40\n40 GO TO 10: REM GOTO 7\n"
        );
    }

    #[test]
    fn a_renumbered_program_runs_the_same() {
        for name in [
            "classic/hanoi",
            "classic/guess",
            "dartmouth/example2",
            "tutorial/07_subroutines",
        ] {
            let path = format!("samples/{name}.bas");
            let source = std::fs::read_to_string(path).unwrap();
            let again = renumbered(&source, 1000, 5).unwrap();
            assert_ne!(source, again);
            let run = |text: &str| {
                let mut output = Vec::new();
                let mut input: &[u8] = b"50\n25\n37\n43\n40\n41\n42\n";
                let result =
                    crate::run(text, &mut input, &mut output, Some(42)).map_err(|e| e.message);
                (String::from_utf8(output).unwrap(), result)
            };
            assert_eq!(run(&source), run(&again), "{name}");
        }
    }

    #[test]
    fn refuses_what_it_cannot_do_safely() {
        assert!(renumber("10 PRINT (\n", 10, 10)
            .unwrap_err()
            .contains("Expected"));
        // a number stuck to THEN is found; one buried in a run of letters is not
        assert_eq!(
            renumbered("10 IFA=1THEN30\n30 END\n", 100, 10).unwrap(),
            "100 IFA=1THEN110\n110 END\n"
        );
        assert!(renumber("10 ONXGOTO30,10\n30 END\n", 100, 10)
            .unwrap_err()
            .contains("cannot be changed safely"));
        assert!(renumber("10 END\n10 END\n", 100, 10).is_err());
        assert_eq!(renumber("10 GOTO 20\n20 END\n", 10, 10).unwrap(), []);
    }
}
