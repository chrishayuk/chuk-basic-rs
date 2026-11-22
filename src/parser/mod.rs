//! Recursive-descent parser: tokens -> [`Program`].

mod data;
mod expr;
mod stmt;

#[cfg(test)]
mod tests;

use crate::ast::{Line, Program, Stmt};
use crate::error::{Error, Result};
use crate::lexer::{tokenize, Tok, Token};

/// Tokenize and parse a whole program.
pub fn parse(source: &str) -> Result<Program> {
    Parser {
        toks: tokenize(source)?,
        pos: 0,
        line: None,
    }
    .program()
}

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

    fn program(mut self) -> Result<Program> {
        let mut lines: Vec<Line> = Vec::new();
        while self.pos < self.toks.len() {
            let src_line = self.toks[self.pos].src_line;
            self.line = None;
            if let Tok::LineNo(n) = *self.peek() {
                self.pos += 1;
                self.line = Some(n);
            }
            let stmts = self.stmts(false)?;
            if !self.eat(&Tok::Eol) {
                return Err(self.err(format!("Unexpected {}", describe(self.peek()))));
            }
            if self.line.is_some() || !stmts.is_empty() {
                lines.push(Line {
                    number: self.line,
                    src_line,
                    stmts,
                });
            }
        }

        // Execution order is line-number order; an unnumbered line stays
        // attached to the numbered line before it.
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
