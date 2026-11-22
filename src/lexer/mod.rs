//! Tokenizer: turns BASIC source text into a flat list of tokens.
//!
//! Keywords are case-insensitive and matched on word boundaries, so `TOTAL`
//! is an identifier rather than `TO` + `TAL`. Identifiers are upper-cased
//! and keep their `$` / `%` type suffix.

mod token;

#[cfg(test)]
mod tests;

pub use token::{Tok, Token};

use crate::ast::Builtin;
use crate::error::{Error, Result};
use token::{keyword, CRUNCHED_WORDS};

/// Tokenize a whole program.
pub fn tokenize(source: &str) -> Result<Vec<Token>> {
    let mut out = Vec::new();
    for (i, raw) in source.lines().enumerate() {
        out.extend(tokenize_line(raw, i + 1, Spacing::Spaced)?);
    }
    Ok(out)
}

/// How keywords are told apart from names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spacing {
    /// A keyword is a whole word, so `TOTAL` and `SCORE` are names. Keywords
    /// glued to a number, string or bracket are still split off.
    Spaced,
    /// Keywords are found anywhere, as the original Microsoft interpreters
    /// did: `FORI=1TO10` is `FOR I = 1 TO 10`, and `TOTAL` is `TO TAL`.
    Crunched,
}

/// Tokenize one source line. A blank line gives no tokens; any other ends
/// with [`Tok::Eol`].
pub fn tokenize_line(raw: &str, src_line: usize, spacing: Spacing) -> Result<Vec<Token>> {
    let mut out = Vec::new();
    Line {
        chars: raw.chars().collect(),
        pos: 0,
        src_line,
        line_no: None,
        at_stmt_start: true,
        spacing,
    }
    .tokenize(&mut out)?;
    Ok(out)
}

/// Scanner state for one source line.
struct Line {
    chars: Vec<char>,
    pos: usize,
    src_line: usize,
    line_no: Option<u32>,
    /// True when the next token begins a statement.
    at_stmt_start: bool,
    spacing: Spacing,
}

impl Line {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.pos += 1;
        }
    }

    fn rest(&mut self) -> String {
        let s: String = self.chars[self.pos..].iter().collect();
        self.pos = self.chars.len();
        s
    }

    fn err(&self, message: impl Into<String>) -> Error {
        Error::syntax(message, self.line_no, Some(self.src_line))
    }

    fn tokenize(mut self, out: &mut Vec<Token>) -> Result<()> {
        self.skip_ws();
        if self.peek().is_none() {
            return Ok(()); // blank line
        }
        let src_line = self.src_line;
        let mut push = |tok| out.push(Token { tok, src_line });

        // a leading run of digits is the line number
        if self.peek().is_some_and(|c| c.is_ascii_digit()) {
            let start = self.pos;
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.pos += 1;
            }
            let digits: String = self.chars[start..self.pos].iter().collect();
            let n = digits
                .parse::<u32>()
                .map_err(|_| self.err(format!("Line number {digits} is too large")))?;
            self.line_no = Some(n);
            push(Tok::LineNo(n));
        }

        loop {
            self.skip_ws();
            let Some(c) = self.peek() else { break };
            let tok = match c {
                '"' => self.string(),
                '\'' => {
                    self.pos += 1;
                    Tok::Rem(self.rest().trim().to_string())
                }
                '0'..='9' => self.number()?,
                '.' if self.peek_at(1).is_some_and(|c| c.is_ascii_digit()) => self.number()?,
                c if c.is_ascii_alphabetic() => self.word()?,
                _ => self.punct()?,
            };
            self.at_stmt_start = matches!(tok, Tok::LineNo(_) | Tok::Colon | Tok::Then | Tok::Else);
            push(tok);
        }
        push(Tok::Eol);
        Ok(())
    }

    fn string(&mut self) -> Tok {
        self.pos += 1; // opening quote
        let start = self.pos;
        while self.peek().is_some_and(|c| c != '"') {
            self.pos += 1;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        if self.peek().is_some() {
            self.pos += 1; // closing quote (optional at end of line, as in MS BASIC)
        }
        Tok::Str(s)
    }

    fn number(&mut self) -> Result<Tok> {
        let start = self.pos;
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.pos += 1;
        }
        if self.peek() == Some('.') {
            self.pos += 1;
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        // exponent, only when it is unambiguously one (1E5, 1E-5)
        if matches!(self.peek(), Some('E' | 'e')) {
            let sign = usize::from(matches!(self.peek_at(1), Some('+' | '-')));
            if self.peek_at(1 + sign).is_some_and(|c| c.is_ascii_digit()) {
                self.pos += 1 + sign;
                while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    self.pos += 1;
                }
            }
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        text.parse::<f64>()
            .map(Tok::Num)
            .map_err(|_| self.err(format!("Invalid number: {text}")))
    }

    fn read_word(&mut self) -> String {
        let start = self.pos;
        while self.peek().is_some_and(|c| c.is_ascii_alphanumeric()) {
            self.pos += 1;
        }
        self.chars[start..self.pos]
            .iter()
            .collect::<String>()
            .to_ascii_uppercase()
    }

    fn word(&mut self) -> Result<Tok> {
        let start = self.pos;
        let mut word = match self.spacing {
            Spacing::Spaced => self.read_word(),
            Spacing::Crunched => self.read_crunched_word(),
        };
        let plain_name =
            keyword(&word).is_none() && !matches!(word.as_str(), "REM" | "DATA" | "GO");
        if plain_name && self.spacing == Spacing::Spaced {
            // REMARK, REMEMBER ... are comments, as in MS BASIC - unless the
            // word is being assigned to or subscripted like a variable
            if word.starts_with("REM") && self.at_stmt_start && !self.assignment_follows() {
                self.pos = start + 3;
                return Ok(Tok::Rem(self.rest().trim().to_string()));
            }
            if let Some(len) = self.crunched_keyword(start, &word) {
                self.pos = start + len;
                word.truncate(len);
            }
        }
        if let Some(suffix @ ('$' | '%')) = self.peek() {
            self.pos += 1;
            word.push(suffix);
            return Ok(Tok::Ident(word));
        }
        match word.as_str() {
            "REM" => Ok(Tok::Rem(self.rest().trim().to_string())),
            "DATA" => Ok(Tok::Data(self.data())),
            "LINE" if self.next_word_is("INPUT") => {
                self.skip_ws();
                self.pos += "INPUT".len();
                Ok(Tok::LineInput)
            }
            // GO TO / GO SUB written as two words
            "GO" => {
                self.skip_ws();
                match self.read_word().as_str() {
                    "TO" => Ok(Tok::Goto),
                    "SUB" => Ok(Tok::Gosub),
                    _ => Err(self.err("Expected TO or SUB after GO")),
                }
            }
            _ => Ok(keyword(&word).unwrap_or(Tok::Ident(word))),
        }
    }

    /// Does `word` come next on the line (after any spaces)?
    fn next_word_is(&self, word: &str) -> bool {
        let rest = self.chars[self.pos..]
            .iter()
            .skip_while(|c| c.is_whitespace());
        rest.take(word.len())
            .map(char::to_ascii_uppercase)
            .eq(word.chars())
    }

    /// The reserved word or built-in function name that starts at `at`, if
    /// any; the longest wins (`GOSUB` over `GO`).
    fn reserved_at(&self, at: usize) -> Option<&'static str> {
        let matches = |word: &&&str| {
            let mut here = self.chars[at..].iter().map(char::to_ascii_uppercase);
            word.chars().all(|c| here.next() == Some(c))
        };
        CRUNCHED_WORDS
            .iter()
            .chain(Builtin::NAMES)
            .filter(matches)
            .max_by_key(|word| word.len())
            .copied()
    }

    /// In crunched source a word is a reserved word if one starts here;
    /// otherwise it is a name, which runs until the next reserved word.
    fn read_crunched_word(&mut self) -> String {
        let start = self.pos;
        if let Some(word) = self.reserved_at(start) {
            // a `$` suffix is picked up by the caller
            self.pos += word.trim_end_matches('$').len();
        } else {
            self.pos += 1;
            while let Some(c) = self.peek() {
                let more = c.is_ascii_digit()
                    || (c.is_ascii_alphabetic() && self.reserved_at(self.pos).is_none());
                if !more {
                    break;
                }
                self.pos += 1;
            }
        }
        self.chars[start..self.pos]
            .iter()
            .collect::<String>()
            .to_ascii_uppercase()
    }

    /// Is the next thing on the line `=` or `(`?
    fn assignment_follows(&self) -> bool {
        let next = self.chars[self.pos..].iter().find(|c| !c.is_whitespace());
        matches!(next, Some('=' | '('))
    }

    /// Old listings often leave out the spaces around a keyword that sits
    /// between two values: `FOR I=1TO8`, `IF A$="Y"THEN100`, `X>=1ANDY<9`.
    /// If `word` (which starts at `start` and is not itself a keyword) begins
    /// with such a keyword, returns the keyword's length.
    ///
    /// The split is only made where a variable name is not plausible: when
    /// the word is glued to a number before it, when a control keyword is
    /// glued to a `)` or `"` before it, or when what follows the keyword is
    /// a line number (`THEN100`).
    fn crunched_keyword(&self, start: usize, word: &str) -> Option<usize> {
        const AFTER_NUMBER: [&str; 8] =
            ["GOSUB", "GOTO", "THEN", "ELSE", "STEP", "AND", "TO", "OR"];
        const AFTER_VALUE: [&str; 4] = ["GOSUB", "GOTO", "THEN", "ELSE"];
        const BEFORE_NUMBER: [&str; 6] = ["GOSUB", "GOTO", "THEN", "ELSE", "STEP", "TO"];
        let before = start.checked_sub(1).map(|i| self.chars[i]);
        let glued: &[&str] = match before {
            Some('0'..='9') => &AFTER_NUMBER,
            Some(')' | '"') => &AFTER_VALUE,
            _ => &[],
        };
        let prefix = |kw: &&str| word.starts_with(*kw).then_some(kw.len());
        glued.iter().find_map(prefix).or_else(|| {
            BEFORE_NUMBER.iter().find_map(|kw| {
                let rest = word.strip_prefix(kw)?;
                let numeric = !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit());
                numeric.then_some(kw.len())
            })
        })
    }

    /// Raw DATA text, up to the end of the line or an unquoted colon.
    fn data(&mut self) -> String {
        let start = self.pos;
        let mut quoted = false;
        while let Some(c) = self.peek() {
            match c {
                '"' => quoted = !quoted,
                ':' if !quoted => break,
                _ => {}
            }
            self.pos += 1;
        }
        self.chars[start..self.pos]
            .iter()
            .collect::<String>()
            .trim()
            .to_string()
    }

    fn punct(&mut self) -> Result<Tok> {
        let c = self.chars[self.pos];
        let next = self.peek_at(1);
        let (tok, len) = match (c, next) {
            ('<', Some('>')) | ('>', Some('<')) => (Tok::Ne, 2),
            ('<', Some('=')) | ('=', Some('<')) => (Tok::Le, 2),
            ('>', Some('=')) | ('=', Some('>')) => (Tok::Ge, 2),
            ('<', _) => (Tok::Lt, 1),
            ('>', _) => (Tok::Gt, 1),
            ('=', _) => (Tok::Eq, 1),
            ('+', _) => (Tok::Plus, 1),
            ('-', _) => (Tok::Minus, 1),
            ('*', _) => (Tok::Star, 1),
            ('/', _) => (Tok::Slash, 1),
            ('^', _) => (Tok::Caret, 1),
            (',', _) => (Tok::Comma, 1),
            (';', _) => (Tok::Semi, 1),
            (':', _) => (Tok::Colon, 1),
            ('(', _) => (Tok::LParen, 1),
            (')', _) => (Tok::RParen, 1),
            ('?', _) => (Tok::Print, 1),
            _ => return Err(self.err(format!("Unexpected character: {c}"))),
        };
        self.pos += len;
        Ok(tok)
    }
}
