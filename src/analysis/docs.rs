//! What a word means, for showing when the pointer rests on it.

use super::scan::{scan, Kind};
use super::{line_numbered, line_refs, original, token_at};

/// (word, how it is written, what it does)
pub(super) const WORDS: [(&str, &str, &str); 52] = [
    ("LET", "LET X = formula", "Works out the formula and keeps the answer in the variable. Later BASICs let you leave the word out."),
    ("PRINT", "PRINT item, item; item", "Types its items. A comma moves to the next zone, a semicolon packs them together, and either at the end holds the line open."),
    ("END", "END", "Stops the program. In 1964 it had to be the last instruction."),
    ("READ", "READ A, B", "Takes the next values from the program's DATA lines."),
    ("DATA", "DATA 1, 2, 3", "Values for READ, taken in order from every DATA line in the program."),
    ("GOTO", "GOTO line", "Carries on from that line. Also written GO TO."),
    ("GO", "GO TO line", "Carries on from that line."),
    ("IF", "IF condition THEN line", "Goes to the line when the condition holds. Later BASICs allow statements after THEN, and ELSE."),
    ("THEN", "IF condition THEN line", "What to do when the condition holds."),
    ("FOR", "FOR I = first TO last STEP by", "Repeats the lines down to NEXT I, counting I from first to last."),
    ("TO", "FOR I = first TO last", "The last value of a FOR loop."),
    ("STEP", "FOR I = first TO last STEP by", "How much a FOR loop counts by; 1 if left out."),
    ("NEXT", "NEXT I", "The end of a FOR loop: counts on, and goes round again unless the loop is done."),
    ("GOSUB", "GOSUB line", "Goes to a subroutine, remembering where it came from for RETURN."),
    ("RETURN", "RETURN", "Goes back to the instruction after the GOSUB that came here."),
    ("DEF", "DEF FNA(X) = formula", "Gives a formula a name, FN and a letter, to be used like a function."),
    ("DIM", "DIM A(20), B(5, 5)", "Makes room for a list or table. Without it an array runs from 0 to 10."),
    ("REM", "REM anything", "A remark for the reader. The computer ignores the rest of the line."),
    ("STOP", "STOP", "Stops the program, like reaching END."),
    ("INPUT", "INPUT \"prompt\"; A, B$", "Asks the person at the keyboard for values."),
    ("LINE", "LINE INPUT \"prompt\"; A$", "Reads a whole line of typing, commas and all, into a string."),
    ("ELSE", "IF condition THEN ... ELSE ...", "What to do when the condition does not hold."),
    ("ON", "ON X GOTO 100, 200, 300", "Goes to the first, second or third line according to X. Also ON X GOSUB."),
    ("WHILE", "WHILE condition", "Repeats the lines down to WEND for as long as the condition holds."),
    ("WEND", "WEND", "The end of a WHILE loop."),
    ("RESTORE", "RESTORE", "Starts READ again from the first DATA item, or from the DATA at a given line."),
    ("RANDOMIZE", "RANDOMIZE", "Starts RND from a new place, or from a given seed."),
    ("MAT", "MAT C = A * B", "Works on a whole matrix: READ, INPUT, PRINT, sums, products, ZER, CON, IDN, TRN, INV."),
    ("FNEND", "FNEND", "The end of a function defined over several lines."),
    ("USING", "PRINT USING \"###.##\"; X", "Prints numbers to a picture: # for a digit, a point, commas, $$ and so on."),
    ("AND", "a AND b", "True when both are. On numbers it works bit by bit."),
    ("OR", "a OR b", "True when either is. On numbers it works bit by bit."),
    ("NOT", "NOT a", "True when a is false."),
    ("TAB", "TAB(n)", "In PRINT, moves to column n, counting from 0."),
    ("SPC", "SPC(n)", "In PRINT, leaves n spaces."),
    ("SIN", "SIN(X)", "The sine of X, with X in radians."),
    ("COS", "COS(X)", "The cosine of X, with X in radians."),
    ("TAN", "TAN(X)", "The tangent of X, with X in radians."),
    ("ATN", "ATN(X)", "The angle, in radians, whose tangent is X."),
    ("EXP", "EXP(X)", "e to the power X."),
    ("LOG", "LOG(X)", "The natural logarithm of X."),
    ("ABS", "ABS(X)", "X without its sign."),
    ("SQR", "SQR(X)", "The square root of X. In 1964 it was \"always taken of the absolute value\" (manual, p. 5); GW-BASIC, a later BASIC, calls a negative X an error."),
    ("INT", "INT(X)", "The whole number at or below X."),
    ("RND", "RND(X)", "The next random number, at least 0 and below 1."),
    ("FIX", "FIX(X)", "X with its fraction cut off."),
    ("SGN", "SGN(X)", "-1, 0 or 1 as X is negative, zero or positive."),
    ("LEN", "LEN(A$)", "How many characters are in the string."),
    ("VAL", "VAL(A$)", "The number written in the string."),
    ("ASC", "ASC(A$)", "The code of the string's first character."),
    ("INSTR", "INSTR(A$, B$)", "Where B$ first appears in A$, counting from 1, or 0."),
    ("MID$", "MID$(A$, start, length)", "Part of a string, from a starting place. LEFT$ and RIGHT$ take the ends; CHR$, STR$, SPACE$ and STRING$ make strings."),
];

/// Markdown for the word, number or line target at a place, and the span
/// it describes.
pub fn hover(source: &str, line: usize, column: u32) -> Option<(String, super::Span)> {
    let lines = scan(source);
    let here = lines.get(line)?;
    // a line number that something jumps to: show the line
    if let Some(reference) = line_refs(&lines)
        .into_iter()
        .find(|r| r.at.line == line && r.at.span.start <= column && column <= r.at.span.end)
    {
        let text = match line_numbered(&lines, reference.target) {
            Some(target) => format!("```basic\n{}\n```", target.text.trim()),
            None => format!("There is no line {}.", reference.target),
        };
        return Some((text, reference.at.span));
    }
    let tok = token_at(here, column).filter(|tok| tok.kind == Kind::Word)?;
    let word = match tok.text.as_str() {
        "LEFT$" | "RIGHT$" | "CHR$" | "STR$" | "SPACE$" | "STRING$" => "MID$",
        other => other,
    };
    let (_, form, meaning) = WORDS.iter().find(|(name, ..)| *name == word)?;
    let era = if original::is_original(word) {
        "In BASIC since 1964."
    } else {
        "Not in the BASIC of 1964; added by later BASICs."
    };
    Some((
        format!("```basic\n{form}\n```\n{meaning}\n\n*{era}*"),
        tok.span,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_and_targets() {
        let source = "10 LET T = SQR(D)\n20 INPUT N\n30 GOTO 10\n40 GOTO 99\n";
        let (text, span) = hover(source, 0, 12).unwrap();
        assert!(
            text.contains("SQR(X)")
                && text.contains("absolute value")
                && text.contains("since 1964")
        );
        assert_eq!((span.start, span.end), (11, 14));
        assert!(hover(source, 1, 4)
            .unwrap()
            .0
            .contains("Not in the BASIC of 1964"));
        assert_eq!(
            hover(source, 2, 9).unwrap().0,
            "```basic\n10 LET T = SQR(D)\n```"
        );
        assert_eq!(hover(source, 3, 9).unwrap().0, "There is no line 99.");
        assert_eq!(hover(source, 0, 7), None, "a variable has nothing to say");
    }

    #[test]
    fn every_statement_and_function_of_1964_is_described() {
        for word in original::STATEMENTS
            .iter()
            .chain(original::FUNCTIONS.iter())
        {
            assert!(WORDS.iter().any(|(name, ..)| name == word), "{word}");
        }
    }
}
