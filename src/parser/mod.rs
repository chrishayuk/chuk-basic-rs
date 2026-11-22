//! Recursive-descent parser: tokens -> [`Program`].

mod data;
mod expr;
mod mat;
mod stmt;

#[cfg(test)]
mod tests;

use crate::ast::{Line, Program, Stmt};
use crate::error::{Error, Result};
use crate::lexer::{tokenize_line, Spacing, Tok, Token};

/// Tokenize and parse a whole program.
pub fn parse(source: &str) -> Result<Program> {
    let mut lines = Vec::new();
    for (i, raw) in source.lines().enumerate() {
        // Most programs put spaces around their keywords. A line that makes
        // no sense that way is tried again as "crunched" source, in which
        // keywords are found anywhere (`FORI=1TO10`).
        let line = match parse_line(raw, i + 1, Spacing::Spaced) {
            Ok(line) => line,
            Err(error) => parse_line(raw, i + 1, Spacing::Crunched).map_err(|_| error)?,
        };
        lines.extend(line);
    }
    in_execution_order(lines)
}

/// Parse one source line; a blank line gives `None`.
fn parse_line(raw: &str, src_line: usize, spacing: Spacing) -> Result<Option<Line>> {
    let toks = tokenize_line(raw, src_line, spacing)?;
    if toks.is_empty() {
        return Ok(None);
    }
    let mut parser = Parser {
        toks,
        pos: 0,
        line: None,
    };
    if let Tok::LineNo(n) = *parser.peek() {
        parser.pos += 1;
        parser.line = Some(n);
    }
    let stmts = parser.stmts(false)?;
    if !parser.eat(&Tok::Eol) {
        return Err(parser.err(format!("Unexpected {}", describe(parser.peek()))));
    }
    let numbered_or_not_empty = parser.line.is_some() || !stmts.is_empty();
    Ok(numbered_or_not_empty.then_some(Line {
        number: parser.line,
        src_line,
        stmts,
    }))
}

/// Execution order is line-number order; an unnumbered line stays attached
/// to the numbered line before it.
fn in_execution_order(lines: Vec<Line>) -> Result<Program> {
    let mut key = 0;
    let mut keyed: Vec<(u32, Line)> = lines
        .into_iter()
        .map(|line| {
            key = line.number.unwrap_or(key);
            (key, line)
        })
        .collect();
    keyed.sort_by_key(|(key, _)| *key);
    let lines: Vec<Line> = keyed.into_iter().map(|(_, line)| line).collect();

    let mut last: Option<u32> = None;
    for line in lines.iter().filter(|l| l.number.is_some()) {
        if line.number == last {
            return Err(Error::syntax(
                "Duplicate line number",
                line.number,
                Some(line.src_line),
            ));
        }
        last = line.number;
    }
    Ok(Program { lines })
}

/// Parser state for one source line.
struct Parser {
    toks: Vec<Token>,
    pos: usize,
    /// BASIC line number of the line being parsed.
    line: Option<u32>,
}

impl Parser {
    fn peek(&self) -> &Tok {
        self.toks.get(self.pos).map_or(&Tok::Eol, |t| &t.tok)
    }

    fn peek2(&self) -> &Tok {
        self.toks.get(self.pos + 1).map_or(&Tok::Eol, |t| &t.tok)
    }

    fn next(&mut self) -> Tok {
        let tok = self.peek().clone();
        self.pos += 1;
        tok
    }

    fn eat(&mut self, tok: &Tok) -> bool {
        let hit = self.peek() == tok;
        if hit {
            self.pos += 1;
        }
        hit
    }

    fn err(&self, message: impl Into<String>) -> Error {
        let idx = self.pos.min(self.toks.len().saturating_sub(1));
        Error::syntax(message, self.line, self.toks.get(idx).map(|t| t.src_line))
    }

    fn expect(&mut self, tok: Tok, what: &str) -> Result<()> {
        if self.eat(&tok) {
            Ok(())
        } else {
            Err(self.err(format!("Expected {what}, found {}", describe(self.peek()))))
        }
    }

    fn at_stmt_end(&self) -> bool {
        // a comment may follow a statement directly: PRINT X ' note
        matches!(self.peek(), Tok::Colon | Tok::Eol | Tok::Else | Tok::Rem(_))
    }

    fn ident(&mut self, what: &str) -> Result<String> {
        match self.next() {
            Tok::Ident(name) => Ok(name),
            other => {
                self.pos -= 1;
                Err(self.err(format!("Expected {what}, found {}", describe(&other))))
            }
        }
    }

    fn line_target(&mut self) -> Result<u32> {
        match self.next() {
            Tok::Num(n) if n.fract() == 0.0 && (0.0..=u32::MAX as f64).contains(&n) => Ok(n as u32),
            other => {
                self.pos -= 1;
                Err(self.err(format!(
                    "Expected a line number, found {}",
                    describe(&other)
                )))
            }
        }
    }

    /// Colon-separated statements up to the end of the line (or up to an
    /// `ELSE` when inside an `IF`).
    fn stmts(&mut self, in_if: bool) -> Result<Vec<Stmt>> {
        let mut out = Vec::new();
        loop {
            while self.eat(&Tok::Colon) {}
            if *self.peek() == Tok::Eol || (in_if && *self.peek() == Tok::Else) {
                return Ok(out);
            }
            out.push(self.stmt()?);
            if !self.at_stmt_end() {
                return Err(self.err(format!("Unexpected {}", describe(self.peek()))));
            }
        }
    }
}

/// A token as it should read in an error message.
fn describe(tok: &Tok) -> String {
    match tok {
        Tok::Eol => "end of line".to_string(),
        Tok::Num(n) => format!("number {n}"),
        Tok::Str(s) => format!("string \"{s}\""),
        Tok::Ident(name) => format!("'{name}'"),
        other => format!("{other:?}").to_uppercase(),
    }
}
