//! What each word of a program is, for colouring it, and what could be
//! typed next.

use super::docs::WORDS;
use super::scan::{scan, Kind, Line, Span, Tok};
use super::{line_refs, original, Lint, JUMPS};
use crate::ast::Builtin;

/// The kinds of token a program is coloured by. The order is the order of
/// the language server's legend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Comment,
    String,
    Number,
    Variable,
    Function,
    Keyword,
    Operator,
    /// A line number, at the start of its line or as the target of a jump.
    Label,
}

impl TokenType {
    /// Names for the legend: the protocol's own where it has one.
    pub const NAMES: [&'static str; 8] = [
        "comment", "string", "number", "variable", "function", "keyword", "operator", "label",
    ];
}

/// Is this word one of the language's own, and not a name?
pub(super) fn is_keyword(word: &str) -> bool {
    matches!(word, "GO" | "LINE" | "REM")
        || crate::tokenize_line(word, 1, crate::Spacing::Spaced).is_ok_and(|toks| {
            !matches!(
                toks.first().map(|t| &t.tok),
                Some(crate::lexer::Tok::Ident(_)) | None
            )
        })
}

fn is_function(word: &str) -> bool {
    Builtin::from_name(word).is_some()
        || matches!(word, "TAB" | "SPC")
        || (word.starts_with("FN") && word.len() > 2)
}

fn classify(tok: &Tok, target: bool) -> Option<TokenType> {
    Some(match tok.kind {
        Kind::Comment => TokenType::Comment,
        Kind::Str => TokenType::String,
        Kind::Num if target => TokenType::Label,
        Kind::Num => TokenType::Number,
        Kind::Data => return None,
        Kind::Punct if "+-*/^↑=<>".contains(tok.text.as_str()) => TokenType::Operator,
        Kind::Punct => return None,
        Kind::Word if tok.text == "REM" => TokenType::Comment,
        Kind::Word if is_keyword(&tok.text) => TokenType::Keyword,
        Kind::Word if is_function(&tok.text) => TokenType::Function,
        Kind::Word => TokenType::Variable,
    })
}

/// Every token worth a colour, in order: (line, columns, what it is).
pub fn tokens(source: &str) -> Vec<(usize, Span, TokenType)> {
    let lines = scan(source);
    let targets: Vec<(usize, u32)> = line_refs(&lines)
        .iter()
        .map(|r| (r.at.line, r.at.span.start))
        .collect();
    let mut found = Vec::new();
    for line in &lines {
        if let Some((_, span)) = line.number {
            found.push((line.index, span, TokenType::Label));
        }
        for tok in &line.toks {
            let target = targets.contains(&(line.index, tok.span.start));
            if let Some(kind) = classify(tok, target) {
                found.push((line.index, tok.span, kind));
            }
        }
    }
    found
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Keyword,
    Function,
    Variable,
    /// A line of the program, offered after `GOTO` and its kind.
    Line,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Completion {
    pub label: String,
    pub kind: CompletionKind,
    /// How it is written, or the line it names.
    pub detail: String,
    pub documentation: String,
}

/// What could go at a place: after a jump, the program's line numbers;
/// anywhere else, the words that begin with what has been typed. Held to
/// 1964, only what 1964 had.
pub fn complete(source: &str, line: usize, column: u32, lint: Lint) -> Vec<Completion> {
    let lines = scan(source);
    let Some(here) = lines.get(line) else {
        return Vec::new();
    };
    // what has been typed of the word, and the token before that
    let typing = here
        .toks
        .iter()
        .find(|t| t.span.start < column && column <= t.span.end);
    let before = here
        .toks
        .iter()
        .rev()
        .find(|t| t.span.end <= typing.map_or(column, |t| t.span.start));
    let after_jump = before
        .is_some_and(|t| JUMPS.contains(&t.word()) || t.word() == "TO" || t.text == ",")
        && typing.is_none_or(|t| t.kind == Kind::Num)
        && here
            .toks
            .iter()
            .any(|t| JUMPS.contains(&t.word()) || t.word() == "GO");
    let is_for = here.toks.iter().any(|t| t.word() == "FOR");
    if after_jump && !(is_for && before.is_some_and(|t| t.word() == "TO")) {
        let prefix = typing.map_or("", |t| t.text.as_str());
        return lines
            .iter()
            .filter_map(|l: &Line| {
                let (number, _) = l.number?;
                let label = number.to_string();
                label.starts_with(prefix).then(|| Completion {
                    label,
                    kind: CompletionKind::Line,
                    // the line without its number, which is the label
                    detail: l
                        .text
                        .trim_start()
                        .trim_start_matches(|c: char| c.is_ascii_digit())
                        .trim()
                        .to_string(),
                    documentation: String::new(),
                })
            })
            .collect();
    }

    let prefix = match typing {
        Some(tok) if tok.kind == Kind::Word => tok.text.as_str(),
        Some(_) => return Vec::new(),
        None => "",
    };
    let allowed = |word: &str| lint == Lint::Later || original::is_original(word);
    let mut found: Vec<Completion> = WORDS
        .iter()
        .filter(|(word, ..)| word.starts_with(prefix) && *word != prefix && allowed(word))
        .map(|(word, form, meaning)| Completion {
            label: word.to_string(),
            kind: if is_function(word) {
                CompletionKind::Function
            } else {
                CompletionKind::Keyword
            },
            detail: form.to_string(),
            documentation: meaning.to_string(),
        })
        .collect();
    // the program's own names
    let mut names: Vec<&str> = lines
        .iter()
        .flat_map(|l| l.toks.iter())
        .filter(|t| {
            t.kind == Kind::Word && !is_keyword(&t.text) && Builtin::from_name(&t.text).is_none()
        })
        .filter(|t| typing.is_none_or(|typed| !std::ptr::eq(*t, typed)))
        .map(|t| t.text.as_str())
        .filter(|name| {
            name.starts_with(prefix) && *name != prefix && !matches!(*name, "TAB" | "SPC")
        })
        .collect();
    names.sort_unstable();
    names.dedup();
    found.extend(names.into_iter().map(|name| Completion {
        label: name.to_string(),
        kind: if name.starts_with("FN") && name.len() > 2 {
            CompletionKind::Function
        } else {
            CompletionKind::Variable
        },
        detail: "in this program".to_string(),
        documentation: String::new(),
    }));
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use TokenType::{Comment, Function, Keyword, Label, Number, Operator, Variable};

    #[test]
    fn a_line_in_colours() {
        let found: Vec<_> = tokens("40 IF T1 >= SQR(X↑2) THEN 90 ' far\n")
            .into_iter()
            .map(|(_, span, kind)| (span.start, kind))
            .collect();
        assert_eq!(
            found,
            [
                (0, Label),
                (3, Keyword),
                (6, Variable),
                (9, Operator),
                (10, Operator),
                (12, Function),
                (16, Variable),
                (17, Operator),
                (18, Number),
                (21, Keyword),
                (26, Label),
                (29, Comment),
            ]
        );
        let kinds = |text: &str| {
            tokens(text)
                .into_iter()
                .map(|(_, _, kind)| kind)
                .collect::<Vec<_>>()
        };
        assert_eq!(kinds("10 REM NOTE"), [Label, Comment, Comment]);
        assert_eq!(
            kinds("10 PRINT \"HI\"; N$; FNA(2)"),
            [
                Label,
                Keyword,
                TokenType::String,
                Variable,
                Function,
                Number
            ]
        );
        assert_eq!(kinds("10 GO TO 30"), [Label, Keyword, Keyword, Label]);
        assert_eq!(kinds("10 DATA 1, 2"), [Label, Keyword]);
    }

    fn labels(source: &str, line: usize, column: u32, lint: Lint) -> Vec<String> {
        complete(source, line, column, lint)
            .into_iter()
            .map(|c| c.label)
            .collect()
    }

    #[test]
    fn after_a_jump_the_lines_of_the_program() {
        let source = "10 LET X = 1\n20 PRINT X\n100 GOTO \n110 IF X THEN 1\n120 FOR I = 1 TO \n";
        assert_eq!(
            labels(source, 2, 9, Lint::Later),
            ["10", "20", "100", "110", "120"]
        );
        assert_eq!(
            labels(source, 3, 15, Lint::Later),
            ["10", "100", "110", "120"]
        );
        let first = &complete(source, 2, 9, Lint::Later)[0];
        assert_eq!(
            (first.kind, first.detail.as_str()),
            (CompletionKind::Line, "LET X = 1")
        );
        // the TO of a FOR is not a jump
        assert!(!labels(source, 4, 17, Lint::Later).contains(&"10".to_string()));
    }

    #[test]
    fn words_that_begin_as_typed() {
        let source = "10 LET TOTAL = 0\n20 DEF FNA(X) = X\n30 PR\n40 LET Y = S\n50 LET Z = T\n";
        assert_eq!(labels(source, 2, 5, Lint::Later), ["PRINT"]);
        assert_eq!(
            labels(source, 3, 12, Lint::Later),
            ["STEP", "STOP", "SPC", "SIN", "SQR", "SGN"]
        );
        // in 1964 there was no SGN and no SPC
        assert_eq!(
            labels(source, 3, 12, Lint::Dartmouth1964),
            ["STEP", "STOP", "SIN", "SQR"]
        );
        // the program's own names come after the language's
        assert_eq!(
            labels(source, 4, 12, Lint::Later),
            ["THEN", "TO", "TAB", "TAN", "TOTAL"]
        );
        let sqr = complete(source, 3, 12, Lint::Later)
            .into_iter()
            .find(|c| c.label == "SQR")
            .unwrap();
        assert_eq!(
            (sqr.kind, sqr.detail.as_str()),
            (CompletionKind::Function, "SQR(X)")
        );
    }
}
