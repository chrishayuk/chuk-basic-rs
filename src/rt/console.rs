//! The terminal: PRINT's cursor, print zones, and INPUT.

use std::io::{BufRead, Write};

use super::dialect::Dialect;
use super::fail;
use super::format::parse_number;
use super::value::Value;
use crate::error::{Error, Result};

/// Width of the print zones that a comma in PRINT moves between.
pub const ZONE_WIDTH: usize = 14;

/// In 1964, "the teletype line is divided into five zones of fifteen spaces
/// each" (October 1964 manual, p. 31).
const DARTMOUTH_ZONE: usize = 15;
const DARTMOUTH_LINE: usize = 5 * DARTMOUTH_ZONE;

/// Widest gap a single TAB or SPC may produce.
const MAX_SPACING: f64 = 255.0;

pub struct Console<'a> {
    input: &'a mut dyn BufRead,
    output: &'a mut dyn Write,
    /// Current output column, for TAB and print zones.
    col: usize,
    /// Copy each line read from the keyboard to the output.
    echo: bool,
    /// Whose way of printing numbers to follow.
    dialect: Dialect,
    /// Where the last number printed began, and how many characters it
    /// took without its sign: for a packed `;` in 1964.
    last_number: Option<(usize, usize)>,
    /// The cursor was put at the start of a zone by a comma, and nothing
    /// has been printed since: a second comma moves on a whole zone.
    at_zone_start: bool,
}

impl<'a> Console<'a> {
    pub fn new(input: &'a mut dyn BufRead, output: &'a mut dyn Write) -> Self {
        Console {
            input,
            output,
            col: 0,
            echo: false,
            dialect: Dialect::default(),
            last_number: None,
            at_zone_start: false,
        }
    }

    /// Print numbers the way another BASIC does (see [`Dialect`]).
    pub fn set_dialect(&mut self, dialect: Dialect) {
        self.dialect = dialect;
    }

    /// Copy every line that is read to the output. A terminal already shows
    /// what is typed; this makes a transcript read the same way when input
    /// comes from a file or a pipe.
    pub fn set_echo(&mut self, echo: bool) {
        self.echo = echo;
    }

    /// The column the cursor is in (0 at the start of a line).
    pub fn column(&self) -> usize {
        self.col
    }

    pub fn flush(&mut self) -> Result<()> {
        self.output.flush().map_err(|e| Error::io(&e))
    }

    // ---- output ------------------------------------------------------------

    pub fn write(&mut self, text: &str) -> Result<()> {
        self.at_zone_start = false;
        self.output
            .write_all(text.as_bytes())
            .map_err(|e| Error::io(&e))?;
        match text.rfind('\n') {
            Some(at) => self.col = text[at + 1..].chars().count(),
            None => self.col += text.chars().count(),
        }
        Ok(())
    }

    pub fn newline(&mut self) -> Result<()> {
        self.write("\n")
    }

    /// Print a value the way PRINT does: a number gets a sign column in
    /// front and a space after.
    pub fn print_value(&mut self, value: &Value) -> Result<()> {
        match value {
            Value::Num(n) => {
                let text = self.dialect.number(*n);
                if self.dialect == Dialect::Dartmouth {
                    // A number starts on a multiple of three columns: after a
                    // label, "FIRST NØ. =  2.3" and "N =    1" (pp. 32-33).
                    self.pad_to(self.col.div_ceil(3) * 3)?;
                    // "BASIC checks to see if there are 12 or more spaces at
                    // the end of a line before printing a number there" (p. 34)
                    if self.col > 0 && self.col + 12 > DARTMOUTH_LINE {
                        self.newline()?;
                    }
                    self.last_number = Some((self.col, text.trim_start_matches('-').len()));
                }
                let sign = if text.starts_with('-') { "" } else { " " };
                self.write(&format!("{sign}{text} "))
            }
            Value::Str(s) => {
                self.last_number = None;
                self.write(s)
            }
        }
    }

    fn pad_to(&mut self, col: usize) -> Result<()> {
        if col > self.col {
            self.write(&" ".repeat(col - self.col))?;
        }
        Ok(())
    }

    /// A comma in PRINT: move to the start of the next zone. In 1964, a
    /// comma in the fifth zone moves to the first zone of the next line
    /// (October 1964 manual, p. 31).
    pub fn next_zone(&mut self) -> Result<()> {
        let number = self.last_number.take();
        if self.dialect == Dialect::Dartmouth {
            // "Each number occupies one zone. Each label occupies a whole
            // number of zones" (p. 31): the next zone is the first boundary at
            // or after the end of the label, or a zone's width after the
            // start of the number.
            let next = if self.at_zone_start {
                self.col + DARTMOUTH_ZONE
            } else {
                let end = match number {
                    Some((start, _)) => self.col.max(start + DARTMOUTH_ZONE),
                    None => self.col,
                };
                end.div_ceil(DARTMOUTH_ZONE).max(1) * DARTMOUTH_ZONE
            };
            if next >= DARTMOUTH_LINE {
                return self.newline();
            }
            self.pad_to(next)?;
            self.at_zone_start = true;
            return Ok(());
        }
        self.pad_to((self.col / ZONE_WIDTH + 1) * ZONE_WIDTH)
    }

    /// A semicolon in PRINT. Later BASICs print the next item straight
    /// after. In 1964 it packs numbers: "move to the beginning of the next
    /// multiple of three characters for the next answer ... with six
    /// characters being the minimum" (October 1964 manual, p. 34). The page
    /// it describes (p. 35), and its counts of 11 three-digit, 8 six-digit
    /// and 6 nine-digit numbers to a line, fit a field of the number's
    /// digits and three more, rounded up to a multiple of three.
    pub fn packed(&mut self) -> Result<()> {
        if self.dialect != Dialect::Dartmouth {
            return Ok(());
        }
        if let Some((start, digits)) = self.last_number.take() {
            let width = (digits + 3).div_ceil(3) * 3;
            self.pad_to(start + width.max(6))?;
        }
        Ok(())
    }

    /// `TAB(n)`: move right to column `n` (counting from 0); no effect if
    /// the cursor is already there or beyond. Old programs compute these
    /// freely, so out-of-range values are clamped rather than errors.
    pub fn tab(&mut self, n: f64) -> Result<()> {
        self.pad_to(n.round().clamp(0.0, MAX_SPACING) as usize)
    }

    /// `SPC(n)`: print `n` spaces.
    pub fn spc(&mut self, n: f64) -> Result<()> {
        self.write(&" ".repeat(n.round().clamp(0.0, MAX_SPACING) as usize))
    }

    // ---- input -------------------------------------------------------------

    /// Read one line from the keyboard, without its line ending.
    pub fn read_line(&mut self) -> Result<String> {
        self.flush()?;
        let mut line = String::new();
        let n = self.input.read_line(&mut line).map_err(|e| Error::io(&e))?;
        if n == 0 {
            return fail("Out of input");
        }
        let line = line.trim_end_matches(['\r', '\n']).to_string();
        if self.echo {
            self.write(&line)?;
            self.newline()?;
        }
        self.col = 0; // the user's RETURN moved the cursor to a new line
        Ok(line)
    }

    /// `INPUT`: prompt, then read one value for each entry of `strings`
    /// (true where a string is wanted, false for a number). Asks again
    /// until the answers make sense.
    pub fn input(
        &mut self,
        prompt: Option<&str>,
        question_mark: bool,
        strings: &[bool],
    ) -> Result<Vec<Value>> {
        'retry: loop {
            if let Some(prompt) = prompt {
                self.write(prompt)?;
            }
            if question_mark {
                self.write("? ")?;
            }
            let line = self.read_line()?;

            // a lone string variable takes the whole line, commas and all
            let mut fields = match strings {
                [true] => vec![unquote(&line)],
                _ => split_fields(&line),
            };
            while fields.len() < strings.len() {
                self.write("?? ")?;
                let more = self.read_line()?;
                fields.extend(split_fields(&more));
            }
            if fields.len() > strings.len() {
                self.write("?EXTRA IGNORED\n")?;
            }

            let mut values = Vec::with_capacity(strings.len());
            for (&string, field) in strings.iter().zip(fields) {
                if string {
                    values.push(Value::Str(field));
                } else if field.is_empty() {
                    values.push(Value::Num(0.0));
                } else if let Some(n) = parse_number(&field) {
                    values.push(Value::Num(n));
                } else {
                    self.write("?REDO FROM START\n")?;
                    continue 'retry;
                }
            }
            return Ok(values);
        }
    }

    /// `LINE INPUT`: prompt (with no question mark) and read a whole line
    /// exactly as typed.
    pub fn line_input(&mut self, prompt: Option<&str>) -> Result<String> {
        if let Some(prompt) = prompt {
            self.write(prompt)?;
        }
        self.read_line()
    }
}

fn unquote(field: &str) -> String {
    let field = field.trim();
    match field.strip_prefix('"') {
        Some(rest) => rest.strip_suffix('"').unwrap_or(rest).to_string(),
        None => field.to_string(),
    }
}

/// Split a line typed at INPUT into comma-separated fields; commas inside
/// double quotes do not split.
fn split_fields(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for c in line.chars() {
        match c {
            '"' => {
                quoted = !quoted;
                current.push(c);
            }
            ',' if !quoted => fields.push(unquote(&std::mem::take(&mut current))),
            _ => current.push(c),
        }
    }
    fields.push(unquote(&current));
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_fields() {
        assert_eq!(split_fields("1, 2 ,three"), vec!["1", "2", "three"]);
        assert_eq!(split_fields("\"a,b\",c"), vec!["a,b", "c"]);
        assert_eq!(split_fields(""), vec![""]);
    }

    #[test]
    fn zones_and_tabs_follow_the_cursor() {
        let mut out = Vec::new();
        let mut input = std::io::empty();
        let mut console = Console::new(&mut input, &mut out);
        console.write("AB").unwrap();
        console.next_zone().unwrap();
        console.write("C").unwrap();
        console.tab(20.0).unwrap();
        console.tab(3.0).unwrap(); // already past: no effect
        console.print_value(&Value::Num(-1.5)).unwrap();
        console.spc(2.0).unwrap();
        console.print_value(&Value::Num(2.0)).unwrap();
        console.newline().unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "AB            C     -1.5    2 \n"
        );
    }
}
