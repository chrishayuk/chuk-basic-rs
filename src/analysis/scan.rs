//! Finds the words of a program and where each one is.
//!
//! This is deliberately simpler than the lexer: it does not decide what is
//! valid, and it does not take apart listings written without spaces beyond
//! `GOTO100` and `THEN100`. It only has to say what is under the cursor.

/// Columns on one line, in UTF-16 code units, `end` exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A keyword or a name, upper-cased, with its `$` if it has one.
    Word,
    Num,
    Str,
    /// What follows `REM` or `'`.
    Comment,
    /// What follows `DATA`.
    Data,
    Punct,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tok {
    pub kind: Kind,
    pub text: String,
    pub span: Span,
}

impl Tok {
    /// The word, or "" for anything that is not one.
    pub fn word(&self) -> &str {
        if self.kind == Kind::Word {
            &self.text
        } else {
            ""
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    /// 0-based line in the source text.
    pub index: usize,
    /// The BASIC line number and where it is written.
    pub number: Option<(u32, Span)>,
    pub toks: Vec<Tok>,
    pub text: String,
    /// Length of the line in UTF-16 code units.
    pub len: u32,
}

/// A word that is a jump when a number is stuck to it: `GOTO100`.
const STUCK: [&str; 5] = ["RESTORE", "GOSUB", "GOTO", "THEN", "ELSE"];

pub fn scan(source: &str) -> Vec<Line> {
    source
        .lines()
        .enumerate()
        .map(|(index, text)| scan_line(index, text))
        .collect()
}

fn scan_line(index: usize, text: &str) -> Line {
    // (character, column in UTF-16 units)
    let mut chars = Vec::new();
    let mut column = 0u32;
    for c in text.chars() {
        chars.push((c, column));
        column += c.len_utf16() as u32;
    }
    let len = column;
    let col = |i: usize| chars.get(i).map_or(len, |&(_, at)| at);
    let text_of =
        |from: usize, to: usize| chars[from..to].iter().map(|&(c, _)| c).collect::<String>();
    let mut toks = Vec::new();
    let mut push = |kind, from: usize, to: usize, text: String| {
        toks.push(Tok {
            kind,
            text,
            span: Span {
                start: col(from),
                end: col(to),
            },
        })
    };

    let mut i = 0;
    while i < chars.len() && chars[i].0.is_whitespace() {
        i += 1;
    }
    let mut number = None;
    let digits_from = i;
    while i < chars.len() && chars[i].0.is_ascii_digit() {
        i += 1;
    }
    if i > digits_from {
        if let Ok(n) = text_of(digits_from, i).parse::<u32>() {
            number = Some((
                n,
                Span {
                    start: col(digits_from),
                    end: col(i),
                },
            ));
        }
    }

    while i < chars.len() {
        let c = chars[i].0;
        let from = i;
        if c.is_whitespace() {
            i += 1;
        } else if c == '"' {
            i += 1;
            while i < chars.len() && chars[i].0 != '"' {
                i += 1;
            }
            i = (i + 1).min(chars.len());
            push(Kind::Str, from, i, text_of(from, i));
        } else if c == '\'' {
            i = chars.len();
            push(Kind::Comment, from, i, text_of(from, i));
        } else if c.is_ascii_digit()
            || (c == '.' && chars.get(i + 1).is_some_and(|n| n.0.is_ascii_digit()))
        {
            while i < chars.len() && chars[i].0.is_ascii_digit() {
                i += 1;
            }
            if i < chars.len() && chars[i].0 == '.' {
                i += 1;
                while i < chars.len() && chars[i].0.is_ascii_digit() {
                    i += 1;
                }
            }
            // an exponent, but not the E of `1 ELSE`
            if i < chars.len() && chars[i].0.eq_ignore_ascii_case(&'e') {
                let mut j = i + 1;
                if j < chars.len() && (chars[j].0 == '+' || chars[j].0 == '-') {
                    j += 1;
                }
                if j < chars.len() && chars[j].0.is_ascii_digit() {
                    while j < chars.len() && chars[j].0.is_ascii_digit() {
                        j += 1;
                    }
                    i = j;
                }
            }
            push(Kind::Num, from, i, text_of(from, i));
        } else if c.is_ascii_alphabetic() {
            while i < chars.len() && chars[i].0.is_ascii_alphabetic() {
                i += 1;
            }
            let letters = text_of(from, i).to_ascii_uppercase();
            // REM, and REMARK, and the REMARKABLE of old listings
            if letters.starts_with("REM") {
                push(Kind::Word, from, from + 3, "REM".to_string());
                i = chars.len();
                push(Kind::Comment, from + 3, i, text_of(from + 3, i));
                continue;
            }
            let digits_at = i;
            while i < chars.len() && chars[i].0.is_ascii_digit() {
                i += 1;
            }
            if i > digits_at && STUCK.contains(&letters.as_str()) {
                // GOTO100: the word, and leave the number for the next turn
                i = digits_at;
                push(Kind::Word, from, i, letters);
                continue;
            }
            if i < chars.len() && chars[i].0 == '$' {
                i += 1;
            }
            let word = text_of(from, i).to_ascii_uppercase();
            let data = word == "DATA";
            push(Kind::Word, from, i, word);
            if data {
                let rest = i;
                i = chars.len();
                push(Kind::Data, rest, i, text_of(rest, i));
            }
        } else {
            i += 1;
            push(Kind::Punct, from, i, c.to_string());
        }
    }
    Line {
        index,
        number,
        toks,
        text: text.to_string(),
        len,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(text: &str) -> Vec<String> {
        scan(text)[0].toks.iter().map(|t| t.text.clone()).collect()
    }

    #[test]
    fn finds_words_numbers_and_strings() {
        let line = &scan("  40 LET t1 = SQR(D) / 2.5E-3")[0];
        assert_eq!(line.number, Some((40, Span { start: 2, end: 4 })));
        assert_eq!(
            words("40 LET t1 = SQR(D) / 2.5E-3"),
            ["LET", "T1", "=", "SQR", "(", "D", ")", "/", "2.5E-3"]
        );
        assert_eq!(
            words("10 PRINT \"A: B\";N$"),
            ["PRINT", "\"A: B\"", ";", "N$"]
        );
        assert_eq!(line.toks[1].span, Span { start: 9, end: 11 });
    }

    #[test]
    fn comments_data_and_stuck_numbers() {
        assert_eq!(words("10 REMARKABLE PROGRAM"), ["REM", "ARKABLE PROGRAM"]);
        assert_eq!(words("10 X=1 ' note"), ["X", "=", "1", "' note"]);
        assert_eq!(words("10 DATA 1, TWO, 3"), ["DATA", " 1, TWO, 3"]);
        assert_eq!(
            words("10 IF A THEN100 ELSE 20"),
            ["IF", "A", "THEN", "100", "ELSE", "20"]
        );
        assert_eq!(words("10 GOTO100"), ["GOTO", "100"]);
    }

    #[test]
    fn columns_count_utf16_units() {
        // the arrow is one unit, three bytes
        let line = &scan("10 LET Y = X↑2 + Z")[0];
        let z = line.toks.last().unwrap();
        assert_eq!((z.text.as_str(), z.span.start), ("Z", 17));
        assert_eq!(line.len, 18);
    }
}
